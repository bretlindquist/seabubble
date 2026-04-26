use anyhow::{bail, Context, Result};
use bytes::BytesMut;
use shared::control::{AllowedAction, ControlMessage, DecisionAck, DecisionCommand};
use shared::{
    ActorContext, CapabilityRequest, CmuxContext, FilterResults, IdentityRecord, Incident,
    IncidentState, SecurityIdentifyHandshake, Severity,
};
use std::collections::HashMap;
use serde::Serialize;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ConnectionIdentity {
    pub peer_pid: i32,
    pub peer_uid: u32,
    pub peer_gid: u32,
    pub session_nonce_validated: bool,
}

#[derive(Debug)]
struct DaemonState {
    allowed_uid: u32,
    demo_mode: bool,
    audit_log_path: String,
    pending_incidents: Mutex<HashMap<String, Incident>>,
}

impl DaemonState {
    fn new(allowed_uid: u32, demo_mode: bool, audit_log_path: String) -> Self {
        Self {
            allowed_uid,
            demo_mode,
            audit_log_path,
            pending_incidents: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Serialize)]
struct AuditEvent<'a> {
    event_type: &'a str,
    message: &'a str,
    uid: u32,
    incident_id: Option<&'a str>,
    agent_id: Option<&'a str>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏝 Security Island Daemon Starting...");

    let allowed_uid = unsafe { libc::getuid() };
    let demo_mode = std::env::var("SECURITY_ISLAND_DAEMON_DEMO").as_deref() == Ok("1");
    let runtime_dir = format!("/tmp/security-island/{allowed_uid}");
    std::fs::create_dir_all(&runtime_dir)
        .with_context(|| format!("Failed to create runtime directory: {runtime_dir}"))?;
    std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to secure runtime directory: {runtime_dir}"))?;
    let audit_log_path = format!("{runtime_dir}/audit.jsonl");
    let state = Arc::new(DaemonState::new(allowed_uid, demo_mode, audit_log_path));
    audit(&state, "daemon_started", "Security Island daemon started", None, None);

    // 1. Hot Path: Agent Capability Intercept
    let cmux_socket = "/tmp/cmux.sock";
    let _ = std::fs::remove_file(cmux_socket);
    let agent_listener = UnixListener::bind(cmux_socket)?;

    // 2. Control Path: SwiftUI Decision Bus
    let control_socket = format!("{runtime_dir}/control.sock");
    let _ = std::fs::remove_file(&control_socket);
    let control_listener = UnixListener::bind(&control_socket)?;

    println!("🎧 Listening for cmux capabilities on {}", cmux_socket);
    println!("🖥  Listening for UI decisions on {}", control_socket);
    if demo_mode {
        println!("🧪 Daemon demo mode enabled via SECURITY_ISLAND_DAEMON_DEMO=1");
    }

    loop {
        tokio::select! {
            Ok((stream, _)) = agent_listener.accept() => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_agent_client(stream, state).await {
                        eprintln!("❌ Agent connection rejected: {}", e);
                    }
                });
            }
            Ok((stream, _)) = control_listener.accept() => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_ui_client(stream, state).await {
                        eprintln!("❌ UI control connection rejected: {}", e);
                    }
                });
            }
        }
    }
}

async fn handle_agent_client(mut stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    // macOS `audit_token_t` extraction remains a production hardening step.
    // For the MVP, capture peer credentials at accept time, then require a nonce handshake.
    let creds = stream.peer_cred()?;

    if creds.uid() != state.allowed_uid {
        bail!("Unauthorized UID: {}. Only UID {} is allowed.", creds.uid(), state.allowed_uid);
    }

    let mut identity = ConnectionIdentity {
        peer_pid: creds.pid().unwrap_or(0),
        peer_uid: creds.uid(),
        peer_gid: creds.gid(),
        session_nonce_validated: false,
    };

    println!(
        "✅ Verified peer UID: {} | PID: {}",
        identity.peer_uid, identity.peer_pid
    );

    let mut buffer = BytesMut::with_capacity(4096);
    buffer.resize(4096, 0);
    let bytes_read = stream.read(&mut buffer).await?;

    if bytes_read == 0 {
        bail!("Agent disconnected before sending security.identify handshake.");
    }

    let handshake: SecurityIdentifyHandshake = serde_json::from_slice(&buffer[..bytes_read])
        .context("Failed to decode security.identify handshake JSON")?;

    if handshake.method != "security.identify" {
        bail!("First packet must be security.identify, got: {}", handshake.method);
    }

    let identity_path = format!(
        "/tmp/security-island/{}/agents/{}/identity.json",
        identity.peer_uid, handshake.params.agent_id
    );

    let identity_bytes = std::fs::read(&identity_path)
        .with_context(|| format!("Failed to load identity file: {}", identity_path))?;
    let expected: IdentityRecord = serde_json::from_slice(&identity_bytes)
        .with_context(|| format!("Failed to parse identity file: {}", identity_path))?;

    if expected.uid != identity.peer_uid {
        bail!("Identity file UID mismatch for agent {}", expected.agent_id);
    }

    if expected.session_nonce != handshake.params.session_nonce {
        bail!("Session nonce mismatch for agent {}", expected.agent_id);
    }

    if let Some(expected_pid) = expected.pid {
        if expected_pid as i32 != identity.peer_pid {
            bail!(
                "Identity PID mismatch for agent {}: expected {}, got {}",
                expected.agent_id,
                expected_pid,
                identity.peer_pid
            );
        }
    }

    identity.session_nonce_validated = true;
    println!(
        "🔐 Nonce handshake validated for agent {} (pid={}, validated={})",
        expected.agent_id, identity.peer_pid, identity.session_nonce_validated
    );
    audit(&state, "agent_authenticated", "Agent nonce and PID validated", None, Some(&expected.agent_id));

    // In prod: Continue reading capability JSON, evaluate policy, and optionally forward.
    Ok(())
}

async fn handle_ui_client(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    let creds = stream.peer_cred()?;
    if creds.uid() != state.allowed_uid {
        bail!("Unauthorized UI process UID: {}. Only UID {} is allowed.", creds.uid(), state.allowed_uid);
    }

    println!("🖥  SwiftUI Dashboard connected.");
    audit(&state, "ui_connected", "SwiftUI dashboard connected", None, None);

    let (reader, mut writer) = stream.into_split();
    if state.demo_mode {
        let incident = demo_incident(state.allowed_uid);
        state
            .pending_incidents
            .lock()
            .await
            .insert(incident.incident_id.clone(), incident.clone());

        audit(&state, "incident_created", "Daemon demo incident emitted", Some(&incident.incident_id), Some(&incident.actor.agent_id));
        write_frame(&mut writer, &ControlMessage::Incident(Box::new(incident)))
            .await
            .context("Failed to send demo incident to UI")?;
        write_frame(
            &mut writer,
            &ControlMessage::Focus(shared::control::FocusEvent {
                surface_id: "surface:daemon-demo".to_string(),
            }),
        )
        .await
        .context("Failed to send demo focus event to UI")?;
    }

    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<DecisionCommand>(&line) {
            Ok(command) => {
                println!(
                    "📨 UI decision received: {:?} for {}",
                    command.action, command.incident_id
                );
                let ack = acknowledge_decision(&state, command).await;
                write_frame(&mut writer, &ControlMessage::DecisionAck(ack)).await?;
            }
            Err(error) => {
                eprintln!("⚠️ Failed to decode UI decision frame: {}", error);
            }
        }
    }

    println!("🖥  SwiftUI Dashboard disconnected.");
    audit(&state, "ui_disconnected", "SwiftUI dashboard disconnected", None, None);
    Ok(())
}

async fn acknowledge_decision(state: &DaemonState, command: DecisionCommand) -> DecisionAck {
    let mut pending = state.pending_incidents.lock().await;
    let incident = pending.remove(&command.incident_id);
    let Some(incident) = incident else {
        return DecisionAck {
            incident_id: command.incident_id,
            action: command.action,
            accepted: false,
            message: Some("Incident is not pending or is unknown to the daemon.".to_string()),
        };
    };

    let action_message = apply_process_action(&incident, &command.action);
    audit(state, "decision_accepted", "UI decision accepted by daemon", Some(&incident.incident_id), Some(&incident.actor.agent_id));

    DecisionAck {
        incident_id: command.incident_id,
        action: command.action,
        accepted: true,
        message: action_message,
    }
}

fn apply_process_action(incident: &Incident, action: &AllowedAction) -> Option<String> {
    match action {
        AllowedAction::AllowOnce | AllowedAction::ContinueWatched => {
            signal_process_group(incident.pgid, libc::SIGCONT).err()
        }
        AllowedAction::Kill => signal_process_group(incident.pgid, libc::SIGTERM).err(),
        AllowedAction::LlmJudge => None,
    }
}

fn signal_process_group(pgid: i32, signal: libc::c_int) -> std::result::Result<(), String> {
    if pgid <= 1 {
        return Ok(());
    }

    let result = unsafe { libc::killpg(pgid, signal) };
    if result == 0 {
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        Err(format!("Failed to send signal {signal} to pgid {pgid}: {error}"))
    }
}

fn audit(
    state: &DaemonState,
    event_type: &str,
    message: &str,
    incident_id: Option<&str>,
    agent_id: Option<&str>,
) {
    let event = AuditEvent {
        event_type,
        message,
        uid: state.allowed_uid,
        incident_id,
        agent_id,
    };

    let Ok(mut line) = serde_json::to_string(&event) else {
        return;
    };
    line.push('\n');

    if let Err(error) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.audit_log_path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()))
    {
        eprintln!("⚠️ Failed to write audit event: {error}");
    }
}

async fn write_frame<T: serde::Serialize>(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    value: &T,
) -> Result<()> {
    let mut data = serde_json::to_vec(value)?;
    data.push(b'\n');
    writer.write_all(&data).await?;
    writer.flush().await?;
    Ok(())
}

fn demo_incident(uid: u32) -> Incident {
    Incident {
        incident_id: "SI-DAEMON-0001".to_string(),
        actor: ActorContext {
            uid,
            process: "security-islandd".to_string(),
            agent_id: "daemon-demo-agent".to_string(),
        },
        cmux: CmuxContext {
            workspace_id: "workspace:daemon-demo".to_string(),
            surface_id: "surface:daemon-demo".to_string(),
            socket_path: format!("/tmp/security-island/{uid}/control.sock"),
        },
        request: CapabilityRequest {
            capability: "browser.eval".to_string(),
            payload: "document.cookie".to_string(),
            cwd: "https://internal.admin.panel".to_string(),
        },
        pid: 0,
        pgid: 0,
        state: IncidentState::PendingDecision,
        risk: 98,
        severity: Severity::Critical,
        reason: "browser_cookie_exfiltration".to_string(),
        rule_id: Some("SI-BROWSER-01".to_string()),
        evidence: vec![
            "Daemon-originated NDJSON control message".to_string(),
            "Browser JS execution attempted document.cookie access".to_string(),
        ],
        filter_results: FilterResults {
            regex: Some("matched document.cookie".to_string()),
            bash_ast: None,
            magika: Some("not_applicable".to_string()),
            llm: None,
        },
        created_at: "2026-04-26T00:00:00Z".to_string(),
        allowed_actions: vec![
            AllowedAction::AllowOnce,
            AllowedAction::ContinueWatched,
            AllowedAction::Kill,
            AllowedAction::LlmJudge,
        ],
    }
}
