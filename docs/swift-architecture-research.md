# Security Island: Native Swift & cmux Integration Research

**Date:** April 26, 2026
**Target:** macOS Native (Swift/SwiftUI)
**Context:** Hackathon MVP, Safety & Security overlay embedded within or alongside `cmux`.

## 1. Executive Summary
To achieve a flawless, native macOS feel and integrate directly with `cmux`, Security Island must pivot from a standalone Python CLI to a **modular Swift Package** (or tightly coupled App Target) embedded within `cmux`. It relies on Darwin POSIX process controls, Apple HIG-compliant SwiftUI, a Telegram long-polling service for two-way human-in-the-loop decisions, and `Foundation.Process` to interface with Google's Magika.

## 2. cmux Integration Strategy
Since `cmux` is a native macOS terminal running multiple AI agents, integrating Security Island "flawlessly" requires a shared state mechanism. For a hackathon, XPC (Cross-Process Communication) is too complex and fragile.

**Recommendation:** Build Security Island as an embedded Swift framework (`SecurityIslandCore` + `SecurityIslandUI`).
- **State Sharing:** `cmux` and Security Island run in the same memory space. `cmux` pushes `Incident` objects directly into the `DecisionBus` (an `@Observable` or `ObservableObject` singleton).
- **Process Management:** `cmux` must track the Process Group ID (`pgid`) of each AI agent pane. When an incident occurs, `cmux` passes the `pgid` to Security Island.
- **Darwin POSIX Layer:** Swift can call native C APIs. We will use `killpg(pgid, SIGSTOP)` to freeze the entire tree of the rogue agent instantly, and `killpg(pgid, SIGCONT)` to resume if the human allows it.

## 3. Apple HIG & SwiftUI Dashboard
To follow Apple Human Interface Guidelines for professional macOS utility apps:
- **Layout:** Use `NavigationSplitView`. 
  - **Sidebar/List:** The collapsed view/table of monitored agents (`Table` or `List` of incidents).
  - **Detail View:** The "Forensic Card" displaying the redacted command, normalized intent, and Magika output.
- **Typography & Symbology:** Use SF Symbols (`shield.lefthalf.filled`, `exclamationmark.triangle.fill`, `checkmark.seal.fill`) to map to Safe (Green), Hold (Yellow), and Critical (Red) states. Use Monospaced fonts for terminal commands (`.font(.system(.body, design: .monospaced))`).
- **Keyboard Navigation:** Bind core decisions to native keyboard shortcuts using `.keyboardShortcut("k", modifiers: [.command])` for Kill, etc., ensuring power users never need the mouse.

## 4. Telegram Two-Way Integration
macOS apps behind a NAT/Firewall cannot easily receive Webhooks.
- **Mechanism:** Implement an `async/await` long-polling loop using `URLSession` hitting the `getUpdates` Telegram API endpoint.
- **Message Format:** When an incident triggers, send a summary message containing an **Inline Keyboard** with callback buttons: `[Allow Once]`, `[Continue Watched]`, `[Kill]`.
- **Callback Handling:** The long-poller intercepts the callback query, maps it to the `incident_id`, and triggers the local `DecisionBus`, which in turn fires `SIGCONT` or `SIGKILL` to the `pgid`.

## 5. Magika File Scanning Integration
Google's Magika is optimized for Python/CLI. We will wrap it in a Swift `Process` execution.
- **Mechanism:** `SecurityIslandCore` will invoke `/path/to/magika --json <filepath>` via `Process()`.
- **Parsing:** Use Swift's `Codable` to decode Magika's JSON output (e.g., determining if a file is a harmless text file or an executable binary) and attach the `magika_result` to the incident's evidence list.

## 6. Surgically Scoped Implementation Plan (The Swarm Chunks)

### Chunk 1: Core Models & Process Control (Agent 1)
- Define `Incident`, `Decision`, and `AgentState` models (`Codable`, `Identifiable`).
- Implement `ProcessController.swift` wrapping `Darwin.killpg` for `SIGSTOP`, `SIGCONT`, `SIGTERM`, `SIGKILL`.
- Create the `DecisionBus` (the central state manager).

### Chunk 2: External Adapters (Telegram & Magika) (Agent 2)
- Implement `TelegramAdapter.swift` (long-polling loop, parsing `getUpdates`, sending `sendMessage` with `inline_keyboard`).
- Implement `MagikaScanner.swift` using `Foundation.Process` to scan suspicious payloads and return structured metadata.

### Chunk 3: SwiftUI Dashboard (Agent 3)
- Build `SecurityIslandView.swift` (`NavigationSplitView`).
- Build `IncidentRowView.swift` (Status pill, agent ID).
- Build `ForensicCardView.swift` (Detail view, code highlighting, decision buttons, keyboard shortcuts).
- Build the `cmux` integration API (e.g., a `.sheet` modifier or a standalone NSWindow).

### Chunk 4: Swarm Review & Linter Gate (Reviewer Agent)
- Run `swiftlint` across all files. Ensure 0 warnings.
- Build a Mock Data provider for the hackathon demo so the UI can be showcased even if Telegram or Magika are offline.