use anyhow::{Context, Result};
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use uuid::Uuid;

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
    let socket_dir = format!("/tmp/security-island/{}/{}", uid, agent_id);
    
    std::fs::create_dir_all(&socket_dir)
        .with_context(|| format!("Failed to create socket directory: {}", socket_dir))?;
        
    let mut perms = std::fs::metadata(&socket_dir)?.permissions();
    perms.set_mode(0o700); // Only the owner can read/write/traverse
    std::fs::set_permissions(&socket_dir, perms)?;
    
    let socket_path = format!("{}/cmux.sock", socket_dir);

    println!("🔒 Established secure boundary at: {}", socket_dir);
    
    // 4. Secure Subprocess Spawning (Environment Cleansing)
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
        .spawn()
        .with_context(|| format!("Failed to spawn agent process: {}", target_executable))?;

    println!("🛡️  Agent {} spawned successfully with PID: {}", agent_id, child.id());

    // 5. Wait for agent to exit naturally
    let status = child.wait()?;
    println!("🛑 Agent {} exited with status: {}", agent_id, status);

    // 6. Cleanup secure socket directory
    let _ = std::fs::remove_dir_all(&socket_dir);
    
    Ok(())
}
