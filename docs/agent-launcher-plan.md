# Surgical Plan: Secure Agent Launcher (Phase 2)

**Date:** April 26, 2026
**Target:** `~/git/seabubble/daemon/agent-launcher`
**Objective:** Replace the stubbed `agent-launcher` with a fully functional, cryptographically secure agent spawner. This launcher must create a dedicated `0700` socket directory, securely inject the `session_nonce`, and launch the actual AI agent process.

## 1. Agent Assignments
- **Agent 1 (Systems/Rust):**
  - Implement `daemon/agent-launcher/src/main.rs` to accept an agent binary path as CLI arguments (e.g., `cargo run -- bin/codex`).
  - Use `std::os::unix::fs::PermissionsExt` to enforce `0700` on `/tmp/security-island/501/<uuid>`.
  - Use `std::process::Command` to spawn the agent, injecting `CMUX_SOCKET_PATH`, `SECURITY_ISLAND_AGENT_ID`, and `SECURITY_ISLAND_SESSION_NONCE` via `.env()`.
  - (MVP Sandbox) Explicitly clear unsafe inherited environment variables (`.env_clear()`) before launching to prevent global secret leakage.
- **Reviewer Agent:**
  - Verify that the `session_nonce` is generated via a cryptographically secure RNG (e.g., `uuid::Uuid::new_v4()` backing).
  - Verify `cargo clippy -- -D warnings` passes.

## 2. Architecture Details
- The launcher acts as the "Sandbox Master." By launching the agent as a subprocess, the OS records the parent-child PID lineage.
- The `session_nonce` must **never** be logged to stdout or disk. It exists purely in the environment memory space of the spawned subprocess and is sent over the UNIX socket during the first `security.identify` handshake.
