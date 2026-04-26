# Surgical Plan: Real Unix Socket IO & Decoder Integration

**Phase:** End-to-End Socket Implementation & Quality Gate
**Objective:** Evolve `CmuxSocketBroker.swift` from a printed stub to a functional `NWConnection` byte-stream parser that actually decodes JSON `CapabilityRequest` payloads from real TCP/UNIX streams and injects them into the `DecisionBus`.

## 1. Agent Assignments
- **Agent 1 (Socket Engineer):**
  - Refactor `CmuxSocketBroker.swift`.
  - Implement a recursive `receive(on:)` function to continuously read from the `NWConnection`.
  - Convert `Data` to UTF-8 `String`, then attempt JSON deserialization into an `Incident` (or intermediate `CapabilityRequest`).
  - If valid, inject into `DecisionBus`.
- **Reviewer Agent (Quality Gate):**
  - Verify `swift build` produces zero warnings.
  - Ensure memory safety (no retain cycles in the `receive` closure).
  - Enforce graceful failure (if invalid JSON is received, log and drop, do not crash the app).

## 2. Architecture Details
- **Data Flow:** `NWConnection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { data, _, isComplete, error in ... }`
- **Memory Safety:** Use `[weak self]` and `[weak connection]` in all Network.framework closures.
- **Dependency Injection:** `CmuxSocketBroker` must be initialized with a reference to `DecisionBus` so it can trigger `@Published` UI updates on the main thread via `Task { @MainActor in }`.

## 3. Strict Quality Criteria
- No force unwraps (`!`).
- All errors handled via `guard` or `catch`.
- Code must be production-level Swift, ready to ship.
