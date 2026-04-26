# Surgical Plan: XPC / Control Socket Decoupling

**Phase:** Separation of Concerns (Chunk 4 Implementation)
**Objective:** Decouple the native Swift UI (`Security Island.app`) from the Rust Daemon (`security-islandd`). The UI must be moved out of the hot packet path. It will communicate with the daemon via a dedicated UNIX Control Socket or XPC connection, receiving `Incident` broadcasts and returning `Decision` commands.

## 1. Agent Assignments
- **Agent 1 (Systems - Rust):** 
  - Update `security-islandd/src/main.rs`.
  - Spin up a secondary UNIX listener at `/tmp/security-island-control.sock`.
  - Accept UI connections, stream `Incident` payloads to them, and parse incoming `Decision` JSON commands (e.g., `{"action": "kill", "incident_id": "..."}`).
- **Agent 2 (Frontend - Swift):** 
  - Update `SecurityIslandCore/DecisionBus.swift`.
  - Remove all `.appendIncident` logic tied to the local `DemoSeeder` or internal `CmuxSocketBroker`.
  - Build an `NWConnection` client that dials `/tmp/security-island-control.sock`, decodes the streaming Rust JSON, and updates the `@Published` incidents array on the `@MainActor`.
- **Reviewer Agent:**
  - Verify that if the Rust daemon restarts, the Swift UI gracefully reconnects.
  - Run `cargo clippy` and `swift build` to ensure cross-language compatibility.

## 2. Architecture Details
- The UI becomes entirely passive until human intervention is required.
- The Rust daemon handles the `cmux` hot path. Only when a policy evaluates to "Ask" (or "High Risk") does it serialize the capability and blast it over the control socket to the SwiftUI layer.
- `SECURITY_ISLAND_CONTROL_SOCKET`: `/tmp/security-island-control.sock`.
