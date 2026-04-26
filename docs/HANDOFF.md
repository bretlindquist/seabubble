# Handoff State: Security Island (seabubble)

## 📍 Current Status
- **Repository:** `~/git/seabubble` (Pushed to `origin/main`)
- **Phase:** Finished **Chunk 2** of the multi-scanner architecture plan. The daemon is now actively enforcing policy on live agent capability traffic.

## ✅ What's Done
1. **Core Architecture Aligned:** Fixed socket topology mismatches and replaced hardcoded UIDs with dynamic host UID fetching.
2. **Identity Hardened:** End-to-end `SHA-256` hashing implemented for the `session_nonce`. Plaintext nonces are no longer stored on disk or broadcast over the control socket.
3. **Capability Loop Active:** The daemon now enters a persistent loop post-handshake, ingesting `CapabilityRequest` NDJSON frames.
4. **Deterministic MVP Policy Enforced:**
   - **Allow:** Safe actions are audited silently.
   - **Watch:** Suspicious actions (e.g., touching `.env`) emit an `Incident` to the UI but allow the process to continue.
   - **Block:** High-risk actions (e.g., `rm -rf`, scraping `document.cookie`) emit a `pending_decision` Incident and pause the agent, waiting for UI intervention.
5. **Control Protocol Verified:** The Swift `DaemonControlClient` correctly buffers and parses the NDJSON `ControlMessage` frames emitted by Rust.

## 🔜 What's Next (The Roadmap)

You are ready to start **Chunk 3**. Here is what is left in the pipeline:

### Chunk 3: Secret & Shell Scanners (Hot Path)
- **Goal:** Replace the MVP policy heuristics with structured scanners.
- **Tasks:**
  - Create `daemon/security-islandd/src/scanners/mod.rs` with a `trait Scanner` interface.
  - Implement `secrets.rs`: A fast regex/entropy-based scanner (emulating Gitleaks logic) on the hot path.
  - Implement `shell_policy.rs`: A structural parser that evaluates shell intent (e.g., pipeline chaining `curl | sh`) rather than just matching substrings.

### Chunk 4: Artifact Scanners (Magika Migration)
- **Goal:** Move file inspection out of the Swift UI and into the Rust daemon.
- **Tasks:**
  - Deprecate `MagikaScanner.swift`.
  - Build `magika.rs` in the daemon.
  - Ensure the normalizer extracts file paths from commands so Magika scans the *files*, not the raw command strings.

### Architectural Flaws (Identified during Chunk 3 audit)
- **Direct Socket Bypass**: If file permissions aren't properly isolated, the agent process could write directly to the real upstream cmux socket, completely bypassing the security island daemon.
- **Stateless Scanning**: The current scanner relies on regex and basic string matching. It cannot catch multi-step attacks (e.g., `download payload -> chmod +x -> execute`) because it lacks AST parsing and state tracking across multiple commands.

### Chunk 5: Hardening & Forwarding
- **Tasks:**
  - **The final cmux link:** Forward approved capability traffic to the *real* cmux upstream socket.
  - Implement Darwin `audit_token_t` FFI for un-spoofable process lineage.
  - CI/CD Supply chain scanning (`cargo-deny`, OSV).

## 🧹 Repository Note
I pushed the latest swarm commits to `origin/main`. 
*Note: Your working tree currently has some unstaged modifications in older Swift files and legacy `src/` artifacts (leftover from the older seaturtle mocks). We ignored them to keep the swarm commits surgical, but you may want to review or stash them when you open the repo.*