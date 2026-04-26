# Surgical Plan: Multi-Scanner Architecture & Core Alignment

**Date:** April 26, 2026
**Target:** `~/git/seabubble`
**Context:** Synthesis of Deep Research identifying the need for a multi-layered scanner bus and critical corrections to socket alignment, identity hashing, and hardcoded variables.

## 1. Architectural Reality Check & Critical Fixes
The research caught a fundamental disconnect between our Phase 1 and Phase 2 implementations:
- **The Mismatch:** `security-islandd` binds strictly to `/tmp/cmux.sock`, but `agent-launcher` creates a unique `/tmp/security-island/<uid>/<agent_id>/cmux.sock` and injects it. The agent will connect to a dead socket.
- **The Fix:** The daemon must dynamically bind per-agent sockets, or the launcher must communicate with the daemon via the control socket to request a new listener before spawning the agent.
- **UID Hardcoding:** Hardcoded `UID 501` must be replaced with `libc::geteuid()`.
- **Nonce Security:** Storing the `session_nonce` as plaintext in `identity.json` is a risk. We must store a hash (`SHA-256` or HMAC) and the daemon must hash the presented nonce for comparison.

## 2. The Multi-Scanner Bus
Security Island cannot rely solely on Magika or an LLM. It needs a stratified, deterministic Rust-based scanner pipeline.

**Scanner Stages:**
1. `pre_forward_blocking`: High-speed, immediate drop (e.g., regex secrets, hard deny).
2. `pre_forward_fast`: Evaluated before forwarding (e.g., shell structure parsing).
3. `artifact_async`: Run on extracted file paths/artifacts (e.g., Magika, YARA).
4. `ci_only`: Supply chain/repo security (OSV, `cargo-deny`, Semgrep).

## 3. Swarm Chunks

### Chunk 1: Core Alignment & Identity Hardening
- **Agent:** Systems/Rust
- **Task:** 
  - Fix `security-islandd` to use `libc::geteuid()`.
  - Implement dynamic Unix Listener generation: When the daemon starts, it watches the `/tmp/security-island/` directory (or receives a command) to bind new per-agent sockets on the fly.
  - Update `agent-launcher` to hash the `session_nonce` before writing `identity.json`, and update the daemon to hash the incoming nonce for verification.

### Chunk 2: The Rust Scanner Pipeline
- **Agent:** Core/Rust
- **Task:**
  - Create `daemon/security-islandd/src/scanners/mod.rs`.
  - Define `trait Scanner`, `ScannerStage` enum, and `ScannerFinding` struct.
  - Wire the hot path: `CapabilityRequest` -> Deterministic Policy -> Fast Scanners -> Forward/Hold.

### Chunk 3: Secret & Shell Scanners (Hot Path)
- **Agent:** Security/Rust
- **Task:**
  - Build `secrets.rs`: Implement a fast regex/entropy-based secret scanner (emulating Gitleaks logic) that inspects `payload` text for `.env` reads, AWS keys, or SSH strings.
  - Build `shell_policy.rs`: Parse structural intent (e.g., pipeline chaining `curl | sh`) rather than flat strings.

### Chunk 4: Artifact Scanners (Magika)
- **Agent:** Security/Rust & UI/Swift
- **Task:**
  - Deprecate `MagikaScanner.swift` in the UI. Magika belongs in the Rust daemon.
  - Build `magika.rs` in Rust.
  - Update the Normalizer to extract `ArtifactRef` (file paths) from shell commands. Magika only scans the paths, never the raw command string.

### Chunk 5: CI/CD Supply Chain (Post-MVP)
- **Agent:** Devops
- **Task:**
  - Add `cargo-deny`, `cargo-vet`, and OSV-Scanner to GitHub Actions to secure the Security Island supply chain itself.