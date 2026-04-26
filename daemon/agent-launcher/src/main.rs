use anyhow::{Context, Result};
use shared::{hash_session_nonce, IdentityRecord, RegistrationAck, RegistrationMessage};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::Command;
use uuid::Uuid;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: security-island-launcher <agent_executable> [args...]");
    }

    let target_executable = &args[1];
    let target_args = &args[2..];

    println!("🚀 Security Island Agent Launcher starting: {}", target_executable);

    let agent_id = Uuid::new_v4();
    let session_nonce = Uuid::new_v4().to_string();
    let session_nonce_hash = hash_session_nonce(&session_nonce);
    let uid = unsafe { libc::getuid() };
    let socket_dir = format!("/tmp/security-island/{}/agents/{}", uid, agent_id);

    std::fs::create_dir_all(&socket_dir)
        .with_context(|| format!("Failed to create socket directory: {}", socket_dir))?;

    let mut perms = std::fs::metadata(&socket_dir)?.permissions();
    perms.set_mode(0o700);
    std::fs::set_permissions(&socket_dir, perms)?;

    let socket_path = format!("{}/cmux.sock", socket_dir);
    println!("🔒 Established secure boundary at: {}", socket_dir);

    let mut identity_record = IdentityRecord {
        agent_id: agent_id.to_string(),
        session_nonce: session_nonce_hash,
        uid,
        socket_dir: socket_dir.clone(),
        socket_path: socket_path.clone(),
        pid: None,
        pgid: None,
    };

    register_with_daemon(uid, RegistrationMessage::RegisterAgent(identity_record.clone()))?;

    let mut child = Command::new(target_executable)
        .args(target_args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin")
        .env("USER", env::var("USER").unwrap_or_default())
        .env("HOME", env::var("HOME").unwrap_or_default())
        .env("CMUX_SOCKET_PATH", &socket_path)
        .env("SECURITY_ISLAND_AGENT_ID", agent_id.to_string())
        .env("SECURITY_ISLAND_SESSION_NONCE", &session_nonce)
        .process_group(0)
        .spawn()
        .with_context(|| format!("Failed to spawn agent process: {}", target_executable))?;

    let child_pid = child.id();
    identity_record.pid = Some(child_pid);
    identity_record.pgid = Some(child_pid);
    register_with_daemon(uid, RegistrationMessage::AgentStarted(identity_record.clone()))?;

    println!("🛡️  Agent {} spawned successfully with PID/PGID: {}", agent_id, child_pid);

    let status = child.wait()?;
    println!("🛑 Agent {} exited with status: {}", agent_id, status);

    let _ = std::fs::remove_dir_all(&socket_dir);

    Ok(())
}

fn register_with_daemon(uid: u32, message: RegistrationMessage) -> Result<()> {
    let registration_socket = format!("/tmp/security-island/{}/registration.sock", uid);
    let mut stream = UnixStream::connect(&registration_socket)
        .with_context(|| format!("Failed to connect to daemon registration socket: {}", registration_socket))?;

    let mut frame = serde_json::to_vec(&message)?;
    frame.push(b'\n');
    stream.write_all(&frame)?;
    stream.flush()?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    let ack: RegistrationAck = serde_json::from_str(&response)
        .context("Failed to decode daemon registration acknowledgement")?;

    if !ack.accepted {
        anyhow::bail!(ack.message.unwrap_or_else(|| "Daemon rejected registration".to_string()));
    }

    Ok(())
}
