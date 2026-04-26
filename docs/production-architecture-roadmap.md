# Security Island: Production Architecture & Phased Roadmap

**Date:** April 26, 2026
**Context:** Deep research synthesis into modern macOS security primitives (Endpoint Security, Unix Socket Identity, Memory Optimization) aligned with cmux and OWASP LLM risks.

## 1. Architectural Reality Check
Security Island cannot be "un-bypassable" solely via socket proxying. If an agent can spawn a shell, it can bypass the socket and run `curl` or `osascript` directly. 
The production architecture is a 4-part defense:
1. **Security Island UI:** Approval cockpit, live risk state, kill switch, audit viewer.
2. **Security Gateway:** cmux socket proxy, capability normalizer, deterministic policy engine.
3. **Agent Sandbox Launcher:** Launches agents with constrained filesystem/env/network boundaries.
4. **Endpoint Security Monitor:** Observes or blocks `exec`/`file`/`network` side effects (via Apple ESF).

## 2. The Hot Path (Zero-Copy-ish Parsing)
We must not rewrite `cmux` traffic into FlatBuffers unless `cmux` supports it natively. The goal is **Fast-Path JSON Classification**.
- **Strategy:** Read the UNIX socket into a reusable slab buffer. Find the newline boundary. Use a partial parser (like `simd-json`) to extract just the `method` and `params`.
- **Zero-Copy Forwarding:** If the capability is known-safe (e.g., `workspace.list`), forward the original raw bytes without full JSON deserialization.
- **Language Split:** Write the Core Gateway in **Rust** (using `tokio` and `simd-json`) for predictable memory allocation and backpressure. Write the UI in **SwiftUI**, communicating with the Rust core via XPC or a local control socket.

## 3. Cryptographic Identity & Spoofing Prevention
A shared `/tmp/cmux.sock` with `0600` permissions is vulnerable to same-user PID reuse attacks. We need a layered identity model:
- **Layer A (Unix Peer):** `getpeereid()` at accept time.
- **Layer B (Audit Token):** Use macOS `audit_token_t` to extract PID, UID, and GID safely.
- **Layer C (Session Nonce):** 
  - Security Island launches the agent, generating a short-lived secret nonce and unique socket path (e.g., `/tmp/security-island/501/agent-7F91/cmux.sock`).
  - The first message on the socket must be `{"method": "security.identify", "params": {"agent_id": "...", "session_nonce": "..."}}`.
- **Layer D (Code Signing):** Validate the executable path and code signature hash before trusting the connection.

## 4. Bypassing the Proxy: OS Containment
The socket proxy handles `cmux` actions, but OS containment stops direct shell abuse.
- **MVP (Seatbelt):** Dynamically generate a `sandbox-exec` profile that allows access only to the project repository and the `/tmp/security-island/.../cmux.sock`, while explicitly denying `~/.ssh`, Keychains, and Browser profile data.
- **Production (Endpoint Security):** Use Apple's ESF (`com.apple.developer.endpoint-security.client`). Hook `AUTH_EXEC` to block `osascript`/`launchctl`, and `AUTH_OPEN` to block access to secrets. (Requires a System Extension).

## 5. Phased Roadmap

### Phase 1: Robust cmux Gateway (The Advanced MVP)
- Swift App + Rust Daemon (or highly tuned Swift UnsafeRawBuffer).
- Unix socket proxy with `CMUX_SOCKET_PATH` rewriting.
- Partial JSON parsing (`CapabilityRequest` normalization).
- Deterministic YAML policy engine (Hard Deny, Ask, Allow).
- Intercepts terminal text, browser clicks, and navigation.

### Phase 2: Process Containment (The Sandbox)
- `security-island-agent-launcher` binary.
- Generates unique per-agent socket directories (`0700` permissions).
- Injects `SECURITY_ISLAND_SESSION_NONCE`.
- Applies baseline `sandbox-exec` profile denying global secrets.

### Phase 3: Production Security Product (EDR-Level Guardrail)
- Apple Endpoint Security entitlement.
- System Extension to block `AUTH_EXEC` (rogue processes) and `AUTH_OPEN` (secrets).
- Append-only audit logs.
- "Security Island: a cmux capability firewall and agent process governor for macOS."