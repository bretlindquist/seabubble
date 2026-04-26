# Security Island: The Capability Firewall (cmux Socket Broker)

**Date:** April 26, 2026
**Context:** Deep architectural cross-check against actual `cmux` protocol specifications and OWASP LLM top 10.

## 1. Core Architectural Pivot
Security Island is not just a command linter or a PID freezer; it is a **Same-User Socket Broker** acting as a **Capability Firewall**.

`cmux` automation operates via a JSON-RPC-like Unix socket (`/tmp/cmux.sock`). The safest integration path is a controlled MITM proxy:
1. **Real cmux:** Binds to `/tmp/cmux.sock.real` (with `CMUX_SOCKET_MODE=automation` or `password`).
2. **Security Island Gateway:** Binds to the default `/tmp/cmux.sock`.
3. **AI Agents:** Connect to `/tmp/cmux.sock`, believing it is the real cmux.
4. **Broker Logic:** Security Island normalizes the JSON-RPC into a `Capability Request`, runs deterministic policies, surfaces it to the human UI (DecisionBus), and only forwards it to `.real` if approved.

## 2. Enforcement Hierarchy
Do not make the LLM the security brain. The LLM is an optional secondary classifier ("why is this risky?"), never the root authority.
1. **Hard Deny Rules** (Deterministic)
2. **Path / Network / Secret Guards** (Deterministic)
3. **Rate Limits**
4. **Approval Rules** (Human-in-the-loop via TUI/SwiftUI)
5. **Audit Log**
6. **LLM Explanation** (Optional)

## 3. The Capability Request Schema
The object being judged is no longer a raw string. It is a structured capability request.
```json
{
  "actor": {
    "uid": 501,
    "process": "codex",
    "agent_id": "codex-worker-2"
  },
  "cmux": {
    "workspace_id": "workspace:2",
    "surface_id": "surface:4",
    "socket_path": "/tmp/cmux.sock"
  },
  "capability": "terminal.send_text",
  "payload": "git reset --hard",
  "cwd": "/Users/bret/git/project",
  "risk": "high"
}
```

## 4. Capability Scope
The firewall must monitor all cmux actions, not just terminal text:
- `terminal.send_text`, `terminal.send_key`
- `read_screen`
- `browser.navigate`, `browser.click`, `browser.fill`, `browser.eval`, `browser.extract`
- `workspace.create`, `surface.create`

## 5. Defense in Depth
The Island is the cockpit, not the jail. It must be paired with:
- Per-agent working directories
- Network/Repo allowlists
- No inherited broad secrets
- (Optional) macOS Sandbox/VM boundary
