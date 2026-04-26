# Surgical Plan: cmux Pane-to-UI Focus Sync

**Date:** April 26, 2026
**Context:** Hackathon Demo UX Enhancement
**Objective:** When the human orchestrator switches tabs/panes inside the `cmux` terminal, the Security Island SwiftUI dashboard must instantly sync its focus (selection state) to match the active `cmux` surface. This demonstrates a fluid, tightly-coupled "command center" UX.

## 1. Research & Evidence Check
- **cmux Behavior:** `cmux` emits events or state updates regarding its active `workspace_id` and `surface_id`. To sync focus, the SwiftUI app needs to be aware of the active surface.
- **Apple HIG & SwiftUI Patterns:** 
  - The SwiftUI `NavigationSplitView` drives its detail view based on a `selection` binding (currently bound to an `Incident.id`).
  - To implement pane-syncing, we must update this selection programmatically.
  - *Observation:* We shouldn't blindly select an *incident* when a pane changes, because a pane might be safe (no incidents). Instead, the UI should reflect the *active agent/pane* state, or if we are strictly showing incidents, we filter/highlight incidents belonging to the newly focused `surfaceId`.
- **Implementation Strategy:**
  1. The Rust daemon must listen for `cmux` state-change notifications (or we poll/intercept them) and forward a `FocusEvent(surface_id)` over the control socket to the UI.
  2. The SwiftUI `DaemonControlClient` decodes the `FocusEvent` and updates a new `@Published var activeSurfaceId: String?` in the `DecisionBus`.
  3. The `SidebarView` observes `activeSurfaceId` and automatically scrolls to or highlights the relevant agent/incident using SwiftUI's `ScrollViewReader` or by updating the `selectedIncidentId` binding if an incident exists for that surface.

## 2. Agent Assignments
- **Agent 1 (Rust Daemon):** 
  - Add a `FocusEvent` payload to `shared/src/control.rs`.
  - Simulate emitting a `FocusEvent` from the daemon (since we don't have live `cmux` running in this sandbox, we will mock the event generation).
- **Agent 2 (SwiftUI Frontend):**
  - Update `DecisionBus` to hold `@Published var activeSurfaceId: String?`.
  - Update `DaemonControlClient` to decode `FocusEvent` and update the bus.
  - Update `SidebarView` to visually highlight incidents matching the `activeSurfaceId` and auto-select them.
- **Reviewer Agent:** Ensure `@MainActor` thread safety, smooth animations (`withAnimation`), and HIG compliance.

## 3. Strict Quality Criteria
- Must use SwiftUI's native data flow (`@Published`, `.onChange`).
- Must not crash if the UI receives a focus event for an unknown pane.
