# Surgical Plan: Final UI Polish & Hackathon Wrapping

**Date:** April 26, 2026
**Context:** MVP Final Polish
**Objective:** Before wrapping up the codebase for the hackathon judging, we need to ensure the UI looks flawless, the keyboard shortcuts map perfectly without blocking text input, and the app gracefully handles empty states.

## 1. Agent Assignments
- **Agent 1 (Frontend Polish):** 
  - Ensure the "LLM Judge" queue state updates visually in the UI.
  - Refine the empty state of `MainDashboardView` with a sleek, HIG-compliant placeholder.
  - Double-check that SF Symbols scale properly with dynamic type.
- **Reviewer Agent:** 
  - Run the final `swift build` and `cargo check`.
  - Compile a checklist verifying all MVP requirements have been met.
  - Push the final code state to `main`.

## 2. Architecture Details
- The UI must look like a premium macOS utility (translucent materials, clean typography).
- Ensure the `DecisionBus` appropriately disables buttons once an incident state moves from `.watch` / `.pendingDecision` to a resolved state.
