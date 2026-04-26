# Surgical Plan: Nonce Handshake Validation (MVP-Cryptographic Binding)

**Date:** April 26, 2026
**Target:** `~/git/seabubble/daemon`
**Objective:** Add a real first-packet authentication handshake so `security-islandd` no longer trusts a same-user Unix socket connection on UID alone.

## 1. Agent Assignments
- **Agent 1 (Launcher):**
  - Persist a short-lived `identity.json` record into `/tmp/security-island/<uid>/<agent_id>/`.
  - Record `agent_id`, `session_nonce`, `uid`, and expected socket path metadata.
- **Agent 2 (Daemon):**
  - Read the first packet from the socket using `tokio::io::AsyncReadExt`.
  - Decode `security.identify` handshake JSON.
  - Resolve the matching identity record by `agent_id` and compare nonce + uid.
  - Reject immediately if invalid.
- **Reviewer Agent:**
  - Ensure no nonce is printed to stdout.
  - Ensure invalid JSON or missing identity files fail closed.
  - Run `cargo clippy -- -D warnings`.

## 2. Security Notes
- This is the MVP cryptographic bind.
- It is not as strong as `audit_token_t` + code-sign validation, but it is materially better than trusting same-user socket access alone.
- The daemon must fail closed on malformed or missing handshake packets.
