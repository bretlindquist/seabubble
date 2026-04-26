mod scanners;
mod setup;

use anyhow::{bail, Context, Result};
use scanners::Pipeline;
use plugin_api::{PolicyDecision, IncidentTemplate};
use serde::{Deserialize, Serialize};
use shared::control::{AllowedAction, ControlMessage, DecisionAck, DecisionCommand};

use shared::{
    hash_session_nonce, ActorContext, CapabilityRequest, CmuxContext, FilterResults,
    IdentityRecord, Incident, IncidentState, RegistrationAck, RegistrationMessage,
    SecurityIdentifyHandshake, Severity,
};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, Mutex};

// PRODUCTION HARDENING NOTE:
// The real upstream cmux socket (`SECURITY_ISLAND_UPSTREAM_CMUX_SOCKET` or `/tmp/cmux.sock.real`)
// MUST be placed in a secure directory with 0700 permissions owned exclusively by the daemon's user/group.
// Otherwise, an attacker could bypass the Security Island daemon by writing directly to the upstream socket.

#[derive(Debug)]
pub struct ConnectionIdentity {
    pub peer_pid: i32,
    pub peer_uid: u32,
    pub peer_gid: u32,
    pub session_nonce_validated: bool,
    pub public_broker_mode: bool,
}

struct DaemonState {
    allowed_uid: u32,
    demo_mode: bool,
    audit_log_path: String,
    registered_agents: Mutex<HashMap<String, IdentityRecord>>,
    pending_incidents: Mutex<HashMap<String, Incident>>,
    ui_clients: Mutex<Vec<mpsc::UnboundedSender<ControlMessage>>>,
    incident_counter: AtomicU64,
    active_pipeline: arc_swap::ArcSwap<Pipeline>,
}

impl DaemonState {
    fn new(allowed_uid: u32, demo_mode: bool, audit_log_path: String) -> Self {
        Self {
            allowed_uid,
            demo_mode,
            audit_log_path,
            registered_agents: Mutex::new(HashMap::new()),
            pending_incidents: Mutex::new(HashMap::new()),
            ui_clients: Mutex::new(Vec::new()),
            incident_counter: AtomicU64::new(1),
            active_pipeline: arc_swap::ArcSwap::from_pointee(Pipeline::new()),
        }
    }

    #[allow(dead_code)]
    pub fn reload_pipeline(&self, new_pipeline: Pipeline) {
        self.active_pipeline.store(Arc::new(new_pipeline));
    }

    fn next_incident_id(&self) -> String {
        let next = self.incident_counter.fetch_add(1, Ordering::Relaxed);
        format!("SI-LIVE-{next:04}")
    }
}

#[derive(Debug, Deserialize)]
struct CmuxV2Frame {
    method: String,
    #[serde(default)]
    params: serde_json::Value,
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

    let control_socket = format!("{runtime_dir}/control.sock");
    let _ = std::fs::remove_file(&control_socket);
    let control_listener = UnixListener::bind(&control_socket)?;
    std::fs::set_permissions(&control_socket, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to secure control socket: {}", control_socket))?;

    let registration_socket = format!("{runtime_dir}/registration.sock");
    let _ = std::fs::remove_file(&registration_socket);
    let registration_listener = UnixListener::bind(&registration_socket)?;
    std::fs::set_permissions(&registration_socket, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to secure registration socket: {}", registration_socket))?;

    // Auto-configure Cmux and Shims
    if let Err(e) = setup::ensure_cmux_config(allowed_uid) {
        eprintln!("Warning: Failed to ensure cmux config: {}", e);
    }
    if let Err(e) = setup::ensure_shims(allowed_uid) {
        eprintln!("Warning: Failed to ensure shims: {}", e);
    }

    let public_cmux_socket = std::env::var("SECURITY_ISLAND_PUBLIC_CMUX_SOCKET")
        .unwrap_or_else(|_| format!("{runtime_dir}/cmux.sock"));
    let _ = std::fs::remove_file(&public_cmux_socket);
    let public_listener = UnixListener::bind(&public_cmux_socket)
        .with_context(|| format!("Failed to bind public cmux socket: {public_cmux_socket}"))?;
    std::fs::set_permissions(&public_cmux_socket, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to secure public cmux socket: {}", public_cmux_socket))?;

    let upstream_cmux_socket = std::env::var("SECURITY_ISLAND_UPSTREAM_CMUX_SOCKET")
        .unwrap_or_else(|_| "/tmp/cmux.sock.real".to_string());

    println!("🖥  Listening for UI decisions on {}", control_socket);
    println!("🧾 Listening for launcher registrations on {}", registration_socket);
    println!("🌉 Public cmux broker listening on {}", public_cmux_socket);
    println!("🔌 Upstream cmux target is {}", upstream_cmux_socket);
    if demo_mode {
        println!("🧪 Daemon demo mode enabled via SECURITY_ISLAND_DAEMON_DEMO=1");
    }

    loop {
        tokio::select! {
            Ok((stream, _)) = control_listener.accept() => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_ui_client(stream, state).await {
                        eprintln!("❌ UI control connection rejected: {}", e);
                    }
                });
            }
            Ok((stream, _)) = registration_listener.accept() => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_registration_client(stream, state).await {
                        eprintln!("❌ Launcher registration rejected: {}", e);
                    }
                });
            }
            Ok((stream, _)) = public_listener.accept() => {
                let state = Arc::clone(&state);
                let socket_path = public_cmux_socket.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_public_cmux_client(stream, state, socket_path).await {
                        eprintln!("❌ Public cmux connection rejected: {}", e);
                    }
                });
            }
        }
    }
}

async fn handle_agent_client(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    #[cfg(target_os = "macos")]
    let (peer_pid, peer_uid, peer_gid) = {
        let token = shared::darwin::get_audit_token(&stream)?;
        let uid = shared::darwin::get_uid_from_token(token);
        let pid = shared::darwin::get_pid_from_token(token);
        let gid = shared::darwin::get_gid_from_token(token);
        (pid, uid, gid)
    };

    #[cfg(not(target_os = "macos"))]
    let (peer_pid, peer_uid, peer_gid) = {
        let creds = stream.peer_cred()?;
        (creds.pid().unwrap_or(0), creds.uid(), creds.gid())
    };

    if peer_uid != state.allowed_uid {
        bail!(
            "Unauthorized UID: {}. Only UID {} is allowed.",
            peer_uid,
            state.allowed_uid
        );
    }

    let mut identity = ConnectionIdentity {
        peer_pid,
        peer_uid,
        peer_gid,
        session_nonce_validated: false,
        public_broker_mode: false,
    };

    println!(
        "✅ Verified peer UID: {} | PID: {} | GID: {}",
        identity.peer_uid, identity.peer_pid, identity.peer_gid
    );

    let mut lines = BufReader::new(stream).lines();
    let Some(line) = lines.next_line().await? else {
        bail!("Agent disconnected before sending security.identify handshake.");
    };

    let handshake: SecurityIdentifyHandshake = serde_json::from_str(&line)
        .context("Failed to decode security.identify handshake JSON")?;

    if handshake.method != "security.identify" {
        bail!("First packet must be security.identify, got: {}", handshake.method);
    }

    let expected = state
        .registered_agents
        .lock()
        .await
        .get(&handshake.params.agent_id)
        .cloned()
        .with_context(|| {
            format!(
                "Agent {} is not registered with daemon",
                handshake.params.agent_id
            )
        })?;

    if expected.uid != identity.peer_uid {
        bail!("Identity file UID mismatch for agent {}", expected.agent_id);
    }

    let presented_nonce_hash = hash_session_nonce(&handshake.params.session_nonce);
    if expected.session_nonce != presented_nonce_hash {
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
    audit(
        &state,
        "agent_authenticated",
        "Agent nonce hash and PID validated",
        None,
        Some(&expected.agent_id),
    );

    let upstream_socket = std::env::var("SECURITY_ISLAND_UPSTREAM_CMUX_SOCKET")
        .unwrap_or_else(|_| "/tmp/cmux.sock.real".to_string());
    
    let upstream_stream = match UnixStream::connect(&upstream_socket).await {
        Ok(s) => Some(s),
        Err(error) => {
            eprintln!("⚠️ Failed to connect to upstream cmux {upstream_socket}: {error}. Running in disconnected mode.");
            None
        }
    };
    
    let mut upstream_writer = match upstream_stream {
        Some(s) => {
            let (_, w) = s.into_split();
            Some(w)
        }
        None => None,
    };

    while let Some(line) = lines.next_line().await? {
        let frame = line.trim();
        if frame.is_empty() {
            continue;
        }

        let request: CapabilityRequest = match serde_json::from_str(frame) {
            Ok(request) => request,
            Err(error) => {
                eprintln!("⚠️ Failed to decode capability frame: {}", error);
                audit(
                    &state,
                    "capability_frame_invalid",
                    "Agent sent invalid capability frame",
                    None,
                    Some(&expected.agent_id),
                );
                continue;
            }
        };

        handle_capability_request(
            &state,
            &expected,
            &identity,
            request,
            &mut lines,
            frame,
            &mut upstream_writer,
        )
        .await?;
    }

    audit(
        &state,
        "agent_disconnected",
        "Authenticated agent disconnected",
        None,
        Some(&expected.agent_id),
    );
    Ok(())
}

async fn handle_capability_request<R: tokio::io::AsyncBufRead + Unpin>(
    state: &Arc<DaemonState>,
    identity_record: &IdentityRecord,
    connection_identity: &ConnectionIdentity,
    request: CapabilityRequest,
    _agent_stream: &mut tokio::io::Lines<R>,
    raw_frame: &str,
    upstream_writer: &mut Option<tokio::net::unix::OwnedWriteHalf>,
) -> Result<()> {
    let pipeline = state.active_pipeline.load();
    match pipeline.classify(&request) {
        PolicyDecision::Allow { reason } => {
            audit(
                state,
                "capability_allowed",
                reason,
                None,
                Some(&identity_record.agent_id),
            );
            if let Some(w) = upstream_writer.as_mut() {
                forward_to_upstream(w, raw_frame).await?;
            }
        }
        PolicyDecision::Watch(template) => {
            let incident = build_incident(
                state,
                identity_record,
                connection_identity,
                request,
                template,
            )?;

            state
                .pending_incidents
                .lock()
                .await
                .insert(incident.incident_id.clone(), incident.clone());
            audit(
                state,
                "incident_created",
                "Watch incident created from capability request",
                Some(&incident.incident_id),
                Some(&identity_record.agent_id),
            );
            broadcast_control_message(state, ControlMessage::Incident(Box::new(incident))).await;

            if let Some(w) = upstream_writer.as_mut() {
                forward_to_upstream(w, raw_frame).await?;
            }
        }
        PolicyDecision::Block(template) => {
            let incident = build_incident(
                state,
                identity_record,
                connection_identity,
                request,
                template,
            )?;

            let incident_id = incident.incident_id.clone();
            state
                .pending_incidents
                .lock()
                .await
                .insert(incident.incident_id.clone(), incident.clone());
            audit(
                state,
                "incident_created",
                "Blocking incident created from capability request",
                Some(&incident.incident_id),
                Some(&identity_record.agent_id),
            );

            let pgid = incident.pgid;
            let _ = signal_process_group(pgid, libc::SIGSTOP);
            audit(
                state,
                "process_paused",
                "Sent SIGSTOP to process group pending decision",
                Some(&incident_id),
                Some(&identity_record.agent_id),
            );

            broadcast_control_message(state, ControlMessage::Incident(Box::new(incident))).await;
        }
    }

    Ok(())
}

async fn forward_to_upstream(writer: &mut tokio::net::unix::OwnedWriteHalf, raw_frame: &str) -> Result<()> {
    writer.write_all(raw_frame.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    println!("⏩ Forwarded {} bytes to upstream cmux", raw_frame.len());
    Ok(())
}

fn normalize_cmux_v2_frame(frame: &CmuxV2Frame) -> CapabilityRequest {
    let capability = match frame.method.as_str() {
        "surface.send_text" => "terminal.send_text",
        "surface.send_key" => "terminal.send_key",
        "surface.read_text" => "terminal.read_text",
        other => other,
    }
    .to_string();

    let payload = match frame.method.as_str() {
        "surface.send_text" => frame.params.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        "surface.send_key" => frame.params.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        "browser.navigate" => frame.params.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        _ => serde_json::to_string(&frame.params).unwrap_or_default(),
    };
    let cwd = frame
        .params
        .get("cwd")
        .and_then(|v| v.as_str())
        .or_else(|| frame.params.get("workspace_id").and_then(|v| v.as_str()))
        .or_else(|| frame.params.get("surface_id").and_then(|v| v.as_str()))
        .unwrap_or("cmux")
        .to_string();

    CapabilityRequest { capability, payload, cwd }
}

async fn handle_public_cmux_client(stream: UnixStream, state: Arc<DaemonState>, socket_path: String) -> Result<()> {
    let creds = stream.peer_cred()?;

    if creds.uid() != state.allowed_uid {
        bail!(
            "Unauthorized public broker UID: {}. Only UID {} is allowed.",
            creds.uid(),
            state.allowed_uid
        );
    }

    let upstream_socket = std::env::var("SECURITY_ISLAND_UPSTREAM_CMUX_SOCKET")
        .unwrap_or_else(|_| "/tmp/cmux.sock.real".to_string());

    let mut upstream_stream = match UnixStream::connect(&upstream_socket).await {
        Ok(s) => Some(s),
        Err(error) => {
            eprintln!("⚠️ Failed to connect to upstream cmux {upstream_socket}: {error}. Running in disconnected mode.");
            None
        }
    };

    let identity_record = IdentityRecord {
        agent_id: format!("public-broker-pid-{}", creds.pid().unwrap_or(0)),
        session_nonce: "public-broker".to_string(),
        uid: creds.uid(),
        socket_dir: socket_path.clone(),
        socket_path: socket_path.clone(),
        pid: creds.pid().map(|pid| pid as u32),
        pgid: creds.pid().map(|pid| pid as u32),
    };

    let connection_identity = ConnectionIdentity {
        peer_pid: creds.pid().unwrap_or(0),
        peer_uid: creds.uid(),
        peer_gid: creds.gid(),
        session_nonce_validated: false,
        public_broker_mode: true,
    };

    println!(
        "🌉 Public cmux client connected | UID: {} | PID: {}",
        connection_identity.peer_uid, connection_identity.peer_pid
    );

    let (stream_reader, mut stream_writer) = stream.into_split();
    let mut lines = BufReader::new(stream_reader).lines();
    while let Some(line) = lines.next_line().await? {
        let frame = line.trim();
        if frame.is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<CmuxV2Frame>(frame) {
            Ok(cmux_frame) => normalize_cmux_v2_frame(&cmux_frame),
            Err(error) => {
                eprintln!("⚠️ Failed to decode public capability frame: {}", error);
                continue;
            }
        };

        if let Some(upstream) = upstream_stream.as_mut() {
            upstream.write_all(frame.as_bytes()).await?;
            upstream.write_all(b"\n").await?;
            upstream.flush().await?;
        }

        handle_capability_request(
            &state,
            &identity_record,
            &connection_identity,
            request,
            &mut lines,
            frame,
            &mut None,
        )
        .await?;

        if let Some(upstream) = upstream_stream.as_mut() {
            let mut response = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                let bytes_read = upstream.read(&mut byte).await?;
                if bytes_read == 0 {
                    break;
                }
                response.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }

            if !response.is_empty() {
                stream_writer.write_all(&response).await?;
                stream_writer.flush().await?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::Pipeline;

    fn dummy_request(capability: &str, payload: &str) -> CapabilityRequest {
        CapabilityRequest {
            capability: capability.to_string(),
            payload: payload.to_string(),
            cwd: "/tmp".to_string(),
        }
    }

    #[test]
    fn policy_blocks_browser_secrets() {
        let req = dummy_request("browser.eval", "console.log(document.cookie);");
        let pipeline = Pipeline::new();
        match pipeline.classify(&req) {
            PolicyDecision::Block(t) => {
                assert_eq!(t.rule_id, "SI-BROWSER-01");
                assert_eq!(t.severity, Severity::Critical);
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn policy_blocks_destructive_shell() {
        let req = dummy_request("terminal.exec", "rm -rf /");
        let pipeline = Pipeline::new();
        match pipeline.classify(&req) {
            PolicyDecision::Block(t) => {
                assert_eq!(t.rule_id, "SI-TERM-01");
                assert_eq!(t.severity, Severity::High);
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn policy_watches_sensitive_files() {
        let req = dummy_request("terminal.read_file", "cat ~/.ssh/id_rsa");
        let pipeline = Pipeline::new();
        match pipeline.classify(&req) {
            PolicyDecision::Watch(t) => {
                assert_eq!(t.rule_id, "SI-DATA-01");
                assert_eq!(t.severity, Severity::Medium);
            }
            _ => panic!("Expected watch"),
        }
    }

    #[test]
    fn policy_allows_safe_commands() {
        let req = dummy_request("terminal.exec", "ls -la");
        let pipeline = Pipeline::new();
        match pipeline.classify(&req) {
            PolicyDecision::Allow { .. } => {}
            _ => panic!("Expected allow"),
        }
    }
}

fn build_incident(
    state: &DaemonState,
    identity_record: &IdentityRecord,
    connection_identity: &ConnectionIdentity,
    request: CapabilityRequest,
    template: IncidentTemplate,
) -> Result<Incident> {
    let pid = connection_identity.peer_pid;
    let pgid = identity_record
        .pgid
        .map(|value| value as i32)
        .unwrap_or(connection_identity.peer_pid);

    let agent_prefix = if connection_identity.public_broker_mode {
        "public"
    } else {
        "agent"
    };

    Ok(Incident {
        incident_id: state.next_incident_id(),
        actor: ActorContext {
            uid: connection_identity.peer_uid,
            process: identity_record.socket_path.clone(),
            agent_id: identity_record.agent_id.clone(),
        },
        cmux: CmuxContext {
            workspace_id: format!("{agent_prefix}:{}", identity_record.agent_id),
            surface_id: format!("surface:{}", identity_record.agent_id),
            socket_path: identity_record.socket_path.clone(),
        },
        request,
        pid,
        pgid,
        state: template.state,
        risk: template.risk,
        severity: template.severity,
        reason: template.reason.to_string(),
        rule_id: Some(template.rule_id.to_string()),
        evidence: template.evidence,
        filter_results: FilterResults {
            regex: template.regex,
            bash_ast: template.bash_ast,
            magika: template.magika,
            llm: None,
        },
        created_at: now_rfc3339()?,
        allowed_actions: template.allowed_actions,
    })
}

fn now_rfc3339() -> Result<String> {
    let now = unsafe { libc::time(std::ptr::null_mut()) };
    let mut tm = std::mem::MaybeUninit::<libc::tm>::uninit();
    let tm_ptr = unsafe { libc::gmtime_r(&now, tm.as_mut_ptr()) };
    if tm_ptr.is_null() {
        bail!("Failed to compute UTC timestamp for incident");
    }

    let tm = unsafe { tm.assume_init() };
    Ok(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    ))
}

async fn handle_registration_client(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    let creds = stream.peer_cred()?;
    if creds.uid() != state.allowed_uid {
        bail!(
            "Unauthorized launcher UID: {}. Only UID {} is allowed.",
            creds.uid(),
            state.allowed_uid
        );
    }

    let mut lines = BufReader::new(stream).lines();
    let Some(line) = lines.next_line().await? else {
        bail!("Launcher disconnected before sending registration message.");
    };

    let message: RegistrationMessage = serde_json::from_str(&line)
        .context("Failed to decode launcher registration message")?;
    let ack = register_agent(Arc::clone(&state), message).await;

    let mut stream = lines.into_inner();
    let mut frame = serde_json::to_vec(&ack)?;
    frame.push(b'\n');
    stream.write_all(&frame).await?;
    stream.flush().await?;

    Ok(())
}

async fn register_agent(state: Arc<DaemonState>, message: RegistrationMessage) -> RegistrationAck {
    let is_initial_registration = matches!(message, RegistrationMessage::RegisterAgent(_));
    let record = match message {
        RegistrationMessage::RegisterAgent(record) | RegistrationMessage::AgentStarted(record) => {
            record
        }
    };

    if record.uid != state.allowed_uid {
        return RegistrationAck {
            accepted: false,
            message: Some(format!(
                "Registration UID {} does not match daemon UID {}",
                record.uid, state.allowed_uid
            )),
        };
    }

    state
        .registered_agents
        .lock()
        .await
        .insert(record.agent_id.clone(), record.clone());
    audit(
        &state,
        "agent_registered",
        "Launcher registered agent identity",
        None,
        Some(&record.agent_id),
    );

    if is_initial_registration {
        let agent_id = record.agent_id.clone();
        let socket_path = record.socket_path.clone();
        let state_for_task = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(error) = bind_agent_socket(socket_path, state_for_task).await {
                eprintln!("❌ Agent socket listener for {agent_id} failed: {error}");
            }
        });
    }

    RegistrationAck {
        accepted: true,
        message: None,
    }
}

async fn bind_agent_socket(socket_path: String, state: Arc<DaemonState>) -> Result<()> {
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind agent socket: {socket_path}"))?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to secure agent socket: {}", socket_path))?;
    println!("🎧 Listening for registered agent on {socket_path}");

    loop {
        let (stream, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_agent_client(stream, state).await {
                eprintln!("❌ Agent connection rejected: {}", e);
            }
        });
    }
}

async fn handle_ui_client(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    let creds = stream.peer_cred()?;
    if creds.uid() != state.allowed_uid {
        bail!(
            "Unauthorized UI process UID: {}. Only UID {} is allowed.",
            creds.uid(),
            state.allowed_uid
        );
    }

    println!("🖥  SwiftUI Dashboard connected.");
    audit(&state, "ui_connected", "SwiftUI dashboard connected", None, None);

    let (tx, mut rx) = mpsc::unbounded_channel();
    state.ui_clients.lock().await.push(tx.clone());

    if state.demo_mode {
        let incident = demo_incident(state.allowed_uid);
        state
            .pending_incidents
            .lock()
            .await
            .insert(incident.incident_id.clone(), incident.clone());

        audit(
            &state,
            "incident_created",
            "Daemon demo incident emitted",
            Some(&incident.incident_id),
            Some(&incident.actor.agent_id),
        );
        let _ = tx.send(ControlMessage::Incident(Box::new(incident)));
        let _ = tx.send(ControlMessage::Focus(shared::control::FocusEvent {
            surface_id: "surface:daemon-demo".to_string(),
        }));
    }

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    loop {
        tokio::select! {
            maybe_message = rx.recv() => {
                let Some(message) = maybe_message else {
                    break;
                };
                write_frame(&mut writer, &message).await?;
            }
            maybe_line = lines.next_line() => {
                let Some(line) = maybe_line? else {
                    break;
                };

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
        }
    }

    println!("🖥  SwiftUI Dashboard disconnected.");
    audit(&state, "ui_disconnected", "SwiftUI dashboard disconnected", None, None);
    Ok(())
}

async fn broadcast_control_message(state: &DaemonState, message: ControlMessage) {
    let mut clients = state.ui_clients.lock().await;
    clients.retain(|client| client.send(message.clone()).is_ok());
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
    audit(
        state,
        "decision_accepted",
        "UI decision accepted by daemon",
        Some(&incident.incident_id),
        Some(&incident.actor.agent_id),
    );

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
        Err(format!(
            "Failed to send signal {signal} to pgid {pgid}: {error}"
        ))
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
