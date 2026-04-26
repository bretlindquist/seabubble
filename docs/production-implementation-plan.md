# Surgical Implementation Plan: Security Island Production V1

**Date:** April 26, 2026
**Target:** macOS (Rust Core Daemon + Swift UI + ESF System Extension)
**Status:** Pre-Implementation Swarm Blueprint

This document breaks down the deep research from the production roadmap into highly detailed, surgically scoped chunks. The swarm will execute these in parallel, gated by strict `cargo clippy` and `swiftlint` checks.

---

## 1. Swarm Agent Assignments

- **Agent 1 (Systems & Network - Rust):** Builds the high-performance `security-islandd` daemon using `tokio`, `simd-json`, and UNIX domain sockets.
- **Agent 2 (Security & Identity - Rust/C):** Builds the `agent-launcher`, session nonce cryptography, and macOS `audit_token_t` process validation.
- **Agent 3 (macOS Frontend - Swift):** Re-architects `SecurityIslandApp` to drop `NWListener` and instead connect to the Rust daemon via local XPC/Control Socket, and scaffolds the Endpoint Security (ESF) System Extension.
- **Reviewer Agent:** Enforces 100% linter passing, zero-copy memory safety limits, and cross-language ABI compatibility.

---

## 2. Surgical Chunks & Implementation Steps

### Chunk 1: The Rust Core Daemon (`security-islandd`)
**Goal:** A memory-safe, non-blocking UNIX socket proxy with zero-copy forwarding for safe capabilities.
1. **Workspace Setup:** Initialize a Cargo workspace with `daemon`, `policy`, and `ipc` crates.
2. **Socket Proxy Loop:** Implement a `tokio` or `mio` UNIX domain socket server binding to a protected path.
3. **Slab Allocation & Framing:** Use `bytes::BytesMut` to read raw bytes. Use `memchr` to find newline boundaries for JSON-RPC framing without converting to strings.
4. **Partial Parsing:** Integrate `simd-json`. Extract *only* `method` and `params`.
5. **Deterministic Policy Engine:** Build a YAML-backed rule engine evaluating `CapabilityRequest`.
6. **Zero-Copy Hot Path:** If `method` matches a safe rule (e.g., `workspace.list`), directly write the `BytesMut` buffer to `cmux.sock.real`. If dangerous, serialize the incident and hold the buffer.

### Chunk 2: Cryptographic Agent Launcher (`agent-launcher`)
**Goal:** Prevent socket spoofing by ensuring the agent is launched in a cryptographically bound context.
1. **CLI Scaffolding:** Create a fast Rust or Swift CLI (`security-island run <agent_cmd>`).
2. **Secure Directory:** Generate a UUID. Create `/tmp/security-island/501/<uuid>/` with strict `0700` permissions. Create the `cmux.sock` inside it.
3. **Nonce Generation:** Generate a 256-bit cryptographically secure random `SECURITY_ISLAND_SESSION_NONCE`.
4. **Environment Injection:** Spawn the `<agent_cmd>` subprocess with stripped inherited secrets, injecting `CMUX_SOCKET_PATH`, `CMUX_WORKSPACE_ID`, and the session nonce.
5. **Initial Handshake Auth:** Update the Rust Daemon (Chunk 1) to reject any connection that does not send `security.identify` with the correct nonce as its first bytes.

### Chunk 3: Identity Validation & XNU Interrogation (The Broker)
**Goal:** Validate that the process connecting is exactly the process we spawned.
1. **Peer Credentials:** Call `getpeereid()` on accepted connections to ensure `uid == 501`.
2. **Audit Token Extraction:** Use macOS native calls (`getsockopt` with `LOCAL_PEERTOKEN`) to retrieve the `audit_token_t`.
3. **PID Validation:** Extract the `pid` and `process start time` from the audit token to protect against PID rollover races.
4. **Connection Pinning:** Create an immutable `ConnectionIdentity` struct bound to the socket lifecycle. Never re-authenticate after the initial handshake.

### Chunk 4: XPC & SwiftUI Decoupling
**Goal:** Move the Swift UI out of the packet path so it only subscribes to human-approval events.
1. **Control Socket / XPC:** Define a lightweight local protocol between `security-islandd` and `Security Island.app`.
2. **UI Refactor:** Rip out `CmuxSocketBroker.swift` from the App bundle. Replace it with a daemon client that listens for `PendingDecision` structs.
3. **Async Responses:** When the user clicks "Allow" or "Kill" (or hits `Cmd+K`), send a decision packet back to the Rust daemon, which either drops the held `BytesMut` buffer or forwards it to `.real`.

### Chunk 5: Endpoint Security Framework (ESF) Jail
**Goal:** Physically prevent the agent from bypassing the cmux socket using `osascript`, direct `curl`, or secret reads.
1. **System Extension Scaffolding:** Create a new target in Xcode for a macOS System Extension requiring `com.apple.developer.endpoint-security.client`.
2. **ESF Client Initialization:** Call `es_new_client()`.
3. **AUTH_EXEC Hook:** Subscribe to `ES_EVENT_TYPE_AUTH_EXEC`. Fast-path allow repository binaries. Block/Ask on `osascript`, `launchctl`, and unknown binaries executed by the agent's process tree.
4. **AUTH_OPEN Hook:** Subscribe to `ES_EVENT_TYPE_AUTH_OPEN`. Explicitly deny access to `~/.ssh`, `~/Library/Keychains`, and `~/Library/Application Support/Google/Chrome/Default/Cookies` for the agent's PID tree.
5. **LRU Cache:** Implement an LRU cache for ESF responses to avoid kernel panics or system stalls during high-volume filesystem walks by the agent.

---

## 3. Strict Quality Gates
- **Rust:** `cargo check`, `cargo clippy -- -D warnings`, and `cargo test`.
- **Swift:** `swiftlint strict` and `swift build`.
- **Cross-Boundary:** All JSON bridging between Rust and Swift must use strict, shared schemas. 
- **Performance:** ESF `AUTH` handlers must resolve in < 10ms.
