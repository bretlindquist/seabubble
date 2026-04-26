use anyhow::{Result, bail};
use tokio::net::{UnixListener, UnixStream};
use bytes::BytesMut;

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

async fn handle_agent_client(stream: UnixStream) -> Result<()> {
    let creds = stream.peer_cred()?;
    if creds.uid() != 501 {
        bail!("Unauthorized UID: {}. Only UID 501 is allowed.", creds.uid());
    }

    let _buffer = BytesMut::with_capacity(8192);
    // In prod: Read agent JSON, evaluate policy.
    // If blocked, send Incident to UI over control socket.
    
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
