# Surgical Plan: Adapters & Security Plugins

**Phase:** External Telemetry & Extended Risk Evaluation
**Objective:** Implement the `SecurityPlugin` architecture using Native Swift to bridge the Decision Bus to the outside world (Telegram) and deep forensic tools (Magika).

## 1. Agent Assignments
- **Agent 1 (Integrations):** 
  - Build `TelegramAdapter.swift`. Connects to the Telegram Bot API. It will parse env vars (`SECURITY_ISLAND_TELEGRAM_BOT_TOKEN`), stub the URLSession for the hackathon MVP, and log formatted alert strings (emulating inline keyboards).
- **Agent 2 (Forensics):**
  - Build `MagikaScanner.swift`. A plugin that uses `Foundation.Process` to shell out to the `magika` CLI. For the demo, it will implement heuristic fallbacks when scanning mocked binary paths, modifying the risk score and halting the process dynamically.
- **Reviewer Agent:** 
  - Wire plugins into `SecurityIslandApp.swift`.
  - Compile the package.
  - Commit to the `main` branch.

## 2. Architecture Details
- Plugins implement `SecurityPlugin.evaluate(incident:) -> PluginEvaluation`.
- **Pipeline:** When `DemoSeeder` (or cmux) adds an incident, it passes through `MagikaScanner` (which increments risk by 20 if it detects a disguised executable), then hits `TelegramAdapter` (which fires an async task to send the notification payload).
- This strictly enforces the "LLM/Telegram is the court of appeal" principle while keeping local rules fast.
