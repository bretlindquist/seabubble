# Surgical Plan: XNU Audit Token & Secure Authentication

**Phase:** Identity Verification (Chunk 3 Implementation)
**Objective:** Replace implicit UNIX socket trust with explicit, race-condition-free identity verification. The Rust daemon must extract the connecting peer's `audit_token_t`, validate `uid == 501`, extract the raw PID, and demand the cryptographic `session_nonce` as the first packet before trusting the connection.

## 1. Agent Assignments
- **Agent 1 (Systems - Rust):** 
  - Update `security-islandd/src/main.rs`.
  - Import `nix::sys::socket::getsockopt` and macOS-specific `libc::audit_token_t`.
  - Extract the audit token on the `UnixStream` raw file descriptor (`AsRawFd`).
  - Extract the peer's PID using macOS `audit_token_to_pid`.
  - Disconnect immediately if the PID or UID do not match expectation.
- **Reviewer Agent:**
  - Verify memory safety around raw FDs.
  - Run `cargo check` and `clippy`.
  - Ensure the connection cleanly closes on failure without taking down the entire `tokio` task.

## 2. Architecture Details
- On macOS, `getpeereid` is good, but `LOCAL_PEERTOKEN` (audit token) is better. The audit token contains a stable snapshot of the process credential at connect time, avoiding PID-reuse race conditions that occur if you only query the PID later.
- The `nix` crate provides safe wrappers for socket options, but macOS audit tokens sometimes require dipping into `libc`.
