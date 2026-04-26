use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn ensure_cmux_config(uid: u32) -> Result<()> {
    let home_dir = std::env::var("HOME").context("Could not find HOME environment variable")?;
    let cmux_dir = PathBuf::from(&home_dir).join(".cmux");
    if !cmux_dir.exists() {
        fs::create_dir_all(&cmux_dir)?;
    }

    let config_path = cmux_dir.join("config.toml");
    let target_socket = format!("/tmp/security-island/{}/cmux.sock", uid);
    
    let needs_update = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        !content.contains(&target_socket)
    } else {
        true
    };

    if needs_update {
        let snippet = format!(
            "\n[broker]\nsocket_path = \"{}\"\n",
            target_socket
        );
        let mut content = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };
        content.push_str(&snippet);
        fs::write(&config_path, content)?;
    }

    Ok(())
}

pub fn ensure_shims(uid: u32) -> Result<()> {
    let home_dir = std::env::var("HOME").context("Could not find HOME environment variable")?;
    let bin_dir = PathBuf::from(&home_dir).join(".seabubble").join("bin");

    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)?;
    }

    let clis = vec!["claude", "codex"];
    let socket_path = format!("/tmp/security-island/{}/cmux.sock", uid);

    for cli in clis {
        let shim_path = bin_dir.join(cli);
        let script = format!(
            "#!/bin/bash\nexport CMUX_SOCKET_PATH=\"{}\"\nexec cmux run -- {} \"$@\"\n",
            socket_path, cli
        );

        fs::write(&shim_path, script)?;

        // Ensure executable
        let mut perms = fs::metadata(&shim_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_path, perms)?;
    }

    Ok(())
}
