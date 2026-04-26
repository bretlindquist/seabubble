# Surgical Plan: SwiftUI Dashboard & Demo Seeder

**Phase:** Frontend UI & Data Seeding
**Objective:** Implement the Apple HIG-compliant SwiftUI interface, hook it to the `DecisionBus`, and inject mock data so the app can be fully demonstrated.

## 1. Agent Assignments
- **Agent 2 (Frontend Architect):** 
  - Build `MainDashboardView`, `SidebarView`, and `ForensicDetailView` in `SecurityIslandUI`.
  - Connect the `SystemUserService` to a toolbar item for the privacy-toggled OS username display.
  - Implement the `App` lifecycle in `SecurityIslandApp.swift` (replacing the CLI stub).
- **Agent 3 (Data Engineer):** 
  - Implement `DemoSeeder.swift` in `SecurityIslandCore` to generate 3 realistic hackathon incidents (Safe, Hold, Critical).
- **Reviewer Agent:** 
  - Fix prior compilation warnings (`@discardableResult` in `ProcessController`).
  - Verify 100% clean compilation.
  - Ensure strict separation of concerns (Core vs UI).

## 2. Architecture Details
- **NavigationSplitView:** The root of the app. Sidebar lists incidents (colored by severity), Detail view shows the forensic card.
- **Forensic Detail View:** Shows raw redacted commands, the normalized intent graph, and action buttons mapped to `AllowedAction`.
- **Toolbar:** The `SystemUserService` will sit in the bottom/top toolbar. A click toggles the OS user from `••••••••` to `bretlindquist`.

## 3. Strict Linter/Build Gates
- All new methods returning unused values must be marked `@discardableResult`.
- UI components must strictly rely on `@EnvironmentObject` to observe the `DecisionBus`.
