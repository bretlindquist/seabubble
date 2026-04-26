use anyhow::{Result, bail, Context};
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use bytes::BytesMut;
use shared::{IdentityRecord, SecurityIdentifyHandshake};

#[derive(Debug)]
pub struct ConnectionIdentity {
    pub peer_pid: i32,
    pub peer_uid: u32,
    pub peer_gid: u32,
    pub session_nonce_validated: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏝 Security Island Daemon Starting...");

    // 1. Hot Path: Agent Capability Intercept
    let cmux_socket = "/tmp/cmux.sock";
    let _ = std::fs::remove_file(cmux_socket);
    let agent_listener = UnixListener::bind(cmux_socket)?;
    
    // 2. Control Path: SwiftUI Decision Bus
    let control_socket = "/tmp/security-island-control.sock";
    let _ = std::fs::remove_file(control_socket);
    let control_listener = UnixListener::bind(control_socket)?;

    println!("🎧 Listening for cmux capabilities on {}", cmux_socket);
    println!("🖥  Listening for UI decisions on {}", control_socket);

    loop {
        tokio::select! {
            Ok((stream, _)) = agent_listener.accept() => {
                tokio::spawn(async move {
                    if let Err(e) = handle_agent_client(stream).await {
                        eprintln!("❌ Agent connection rejected: {}", e);
                    }
                });
            }
            Ok((stream, _)) = control_listener.accept() => {
                tokio::spawn(async move {
                    if let Err(e) = handle_ui_client(stream).await {
                        eprintln!("❌ UI control connection rejected: {}", e);
                    }
                });
            }
        }
    }
}

async fn handle_agent_client(mut stream: UnixStream) -> Result<()> {
    // macOS `audit_token_t` extraction remains a production hardening step.
    // For the MVP, capture peer credentials at accept time, then require a nonce handshake.
    let creds = stream.peer_cred()?;

    if creds.uid() != 501 {
        bail!("Unauthorized UID: {}. Only UID 501 is allowed.", creds.uid());
    }

    let mut identity = ConnectionIdentity {
        peer_pid: creds.pid().unwrap_or(0),
        peer_uid: creds.uid(),
        peer_gid: creds.gid(),
        session_nonce_validated: false,
    };

    println!("✅ Verified peer UID: {} | PID: {}", identity.peer_uid, identity.peer_pid);

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
        "/tmp/security-island/{}/{}/identity.json",
        identity.peer_uid,
        handshake.params.agent_id
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

    identity.session_nonce_validated = true;
    println!(
        "🔐 Nonce handshake validated for agent {} (pid={}, validated={})",
        expected.agent_id,
        identity.peer_pid,
        identity.session_nonce_validated
    );

    // In prod: Continue reading capability JSON, evaluate policy, and optionally forward.
    Ok(())
}

async fn handle_ui_client(stream: UnixStream) -> Result<()> {
    let creds = stream.peer_cred()?;
    if creds.uid() != 501 {
        bail!("Unauthorized UI process UID: {}.", creds.uid());
    }
    
    println!("🖥  SwiftUI Dashboard connected.");
    
    // In prod: Read from stream for `DecisionCommand` objects sent by Swift UI.
    // Forward decisions back to the blocked agent tasks.
    
    Ok(())
}
