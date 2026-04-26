# Security Island

**Security Island** is a CLI/TUI security overlay designed to monitor multiple AI agent sessions, detect suspicious actions, pause the relevant agent, create a pending human decision, and let the user resolve it from the keyboard.

Built for the Safety & Security hackathon, it acts as a decision bus and forensic dashboard. It treats the LLM as the expensive court of appeal, relying on fast local filters to create a pending human decision first.

## Quick Start
*(Detailed instructions to be added by the development swarm)*

## Architecture
- **Decision Bus:** The core source of truth.
- **Process Control:** Safely pauses/resumes process groups using `SIGSTOP`/`SIGCONT`/`SIGKILL`.
- **TUI Dashboard:** Keyboard-driven forensic view of pending incidents.
- **Adapters (Optional):** cmux notification and Telegram stubs.
