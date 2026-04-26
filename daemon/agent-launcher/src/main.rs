use anyhow::{Context, Result};
use shared::IdentityRecord;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::Command;
use uuid::Uuid;
use sha2::{Sha256, Digest};

fn main() -> Result<()> {
    // 1. Parse CLI arguments for the target agent command
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: security-island-launcher <agent_executable> [args...]");
    }
    
    let target_executable = &args[1];
    let target_args = &args[2..];

    println!("🚀 Security Island Agent Launcher starting: {}", target_executable);
    
    // 2. Cryptographic Identity Generation
    let agent_id = Uuid::new_v4();
    let session_nonce = Uuid::new_v4(); // CSPRNG backing in v4
    
    // 3. Secure Directory Creation (0700)
    let uid = unsafe { libc::getuid() };
    let socket_dir = format!("/tmp/security-island/{}/agents/{}", uid, agent_id);
    
    std::fs::create_dir_all(&socket_dir)
        .with_context(|| format!("Failed to create socket directory: {}", socket_dir))?;
        
    let mut perms = std::fs::metadata(&socket_dir)?.permissions();
    perms.set_mode(0o700); // Only the owner can read/write/traverse
    std::fs::set_permissions(&socket_dir, perms)?;
    
    let socket_path = format!("{}/cmux.sock", socket_dir);

    println!("🔒 Established secure boundary at: {}", socket_dir);

    // 4. Hash the nonce before persisting to disk
    let mut hasher = Sha256::new();
    hasher.update(session_nonce.to_string().as_bytes());
    let nonce_hash = hex::encode(hasher.finalize());

    // 5. Persist short-lived identity record for daemon-side nonce validation
    let mut identity_record = IdentityRecord {
        agent_id: agent_id.to_string(),
        session_nonce: nonce_hash, // Store HASH only
        uid,
        socket_dir: socket_dir.clone(),
        socket_path: socket_path.clone(),
        pid: None,
        pgid: None,
    };

    let identity_path = format!("{}/identity.json", socket_dir);
    let identity_json = serde_json::to_vec_pretty(&identity_record)?;
    std::fs::write(&identity_path, identity_json)
        .with_context(|| format!("Failed to write identity file: {}", identity_path))?;
        
    // 6. Spawn the Daemon attached to this specific agent socket
    // In a full production daemon, this would be an RPC call to a master daemon.
    // For this architecture phase, the launcher spawns the daemon per-agent.
    let mut daemon_child = Command::new("security-islandd")
        .env("SECURITY_ISLAND_BIND_PATH", &socket_path)
        .spawn()
        .with_context(|| "Failed to spawn security-islandd. Ensure it is in PATH.")?;
    
    // Give the daemon a moment to bind the socket
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // 7. Secure Subprocess Spawning (Environment Cleansing)
    let mut child = Command::new(target_executable)
        .args(target_args)
        .env_clear() // Drop inherited secrets (like AWS keys or global PATH)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin") // Baseline safe PATH
        .env("USER", env::var("USER").unwrap_or_default())
        .env("HOME", env::var("HOME").unwrap_or_default())
        // Inject Security Island specific routing
        .env("CMUX_SOCKET_PATH", &socket_path)
        .env("SECURITY_ISLAND_AGENT_ID", agent_id.to_string())
        .env("SECURITY_ISLAND_SESSION_NONCE", session_nonce.to_string())
        .process_group(0)
        .spawn()
        .with_context(|| format!("Failed to spawn agent process: {}", target_executable))?;

    let child_pid = child.id();
    identity_record.pid = Some(child_pid);
    identity_record.pgid = Some(child_pid);
    let identity_json = serde_json::to_vec_pretty(&identity_record)?;
    std::fs::write(&identity_path, identity_json)
        .with_context(|| format!("Failed to update identity file: {}", identity_path))?;

    println!("🛡️  Agent {} spawned successfully with PID/PGID: {}", agent_id, child_pid);

    // 8. Wait for agent to exit naturally
    let status = child.wait()?;
    println!("🛑 Agent {} exited with status: {}", agent_id, status);

    // 9. Cleanup secure socket directory and kill daemon
    let _ = daemon_child.kill();
    let _ = std::fs::remove_dir_all(&socket_dir);
    
    Ok(())
}
