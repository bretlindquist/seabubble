# Surgical Plan: UNIX Socket Broker & Model Convergence

**Phase:** Socket Interception & UI Reconciliation
**Objective:** Fix compilation errors caused by the model pivot, update the `DemoSeeder` to match the new `CapabilityRequest` schema, and stub the `UnixSocketBroker` using Apple's `Network` framework to act as the MITM proxy between AI agents and the real `cmux`.

## 1. Agent Assignments
- **Agent 1 (Core & Network):** 
  - Build `CmuxSocketBroker.swift` in `SecurityIslandCore`. It will use `NWListener` binding to `/tmp/cmux.sock` and `NWConnection` routing to `/tmp/cmux.sock.real`.
  - Fix `DemoSeeder.swift` to generate the new nested `ActorContext`, `CmuxContext`, and `CapabilityRequest` structures.
- **Agent 2 (Frontend & Plugins):** 
  - Update `TelegramAdapter.swift` and `MagikaScanner.swift` to read `incident.request.payload` instead of `rawRedacted`.
  - Update `SidebarView.swift` to display `incident.actor.agentId` and `incident.request.capability`.
  - Update `ForensicDetailView.swift` to render the Capability Request JSON beautifully.
- **Reviewer Agent:** 
  - Compile the package (`swift build`).
  - Commit to `main`.

## 2. Architecture Details
- **Network.framework:** Native, async/await compatible, and highly secure for UNIX domain sockets.
- **Broker Flow:** 
  - Client connects to `/tmp/cmux.sock`.
  - `NWListener` accepts.
  - Data parsed to `CapabilityRequest`.
  - Sent to `DecisionBus`.
  - If approved, a secondary `NWConnection` opens to `/tmp/cmux.sock.real` and pipes the data forward.
