# Security Island Hackathon Presentation

## Slide 1 — Title
- **Security Island**
- A native macOS capability firewall and approval cockpit for autonomous coding agents
- Built to sit in front of `cmux` and give humans a real security control loop
- Tagline: **let agents move fast without giving them silent root over your machine**

## Slide 2 — The Problem
- Autonomous coding agents are getting more powerful
- They can read files, run shell commands, evaluate browser scripts, and touch sensitive local state
- Today, a lot of trust is implicit: once the agent is running, the human often sees too little, too late
- Existing approval UX is usually buried in logs, terminal spam, or generic prompts
- We wanted a system that makes risky actions visible, understandable, and interruptible in real time

## Slide 3 — The Big Idea
- Put a **security checkpoint** between the agent and powerful capabilities
- Let a fast Rust daemon inspect capability requests before they execute
- Classify requests with deterministic policy
- Auto-allow safe actions
- Watch suspicious actions
- Pause and escalate high-risk actions to a human approval dashboard
- Treat the LLM as a **court of appeal**, not the first line of defense

## Slide 4 — What It Does
- Receives agent capability requests
- Verifies agent identity with **same-user checks, PID binding, and session nonce validation**
- Scans requests for risky patterns
- Creates structured incidents with severity, evidence, risk score, and allowed actions
- Broadcasts incidents to a native SwiftUI dashboard
- Lets a human choose: **Allow Once**, **Continue Watched**, **LLM Judge**, or **Kill**
- Records an audit trail of what happened

## Slide 5 — Why It’s Cool
- Native macOS security tooling feel, not a web dashboard bolted on later
- Human-in-the-loop without making every safe action annoying
- Real process control: high-risk requests can trigger a process-group pause while waiting for a decision
- Deterministic policy first, so the system is explainable and fast
- Designed for agent workflows instead of retrofitting older endpoint-security patterns

## Slide 6 — Standout Features
- **Capability firewall model** for agent actions
- **Identity hardening** with launcher registration, nonce hashing, and PID checks
- **Incident forensics UI** with capability, payload, cwd, evidence, state, agent id, workspace, and surface info
- **Multi-scanner policy pipeline** that chooses the strongest finding
- **Audit logging** to JSONL for traceability
- **Native decision cockpit** with one-keystroke response actions
- **Disconnected-mode resilience** when upstream `cmux` is unavailable

## Slide 7 — Scanner Highlights
- Secret scanner watches for:
  - `.env`
  - `~/.ssh/id_rsa`
  - token-like patterns
  - browser secret access like `document.cookie`
- Shell policy blocks obviously destructive actions such as `rm -rf /`
- Browser script execution is watched by default in the MVP policy
- Findings are ranked so the most important block/watch result wins
- Extra findings are preserved as additional evidence

## Slide 8 — Example Risk Decisions
- **Safe command**: `ls -la`
  - allowed automatically
- **Sensitive file read**: `cat ~/.ssh/id_rsa`
  - watched and surfaced with evidence
- **Browser secret access**: `console.log(document.cookie)`
  - blocked and escalated as critical
- **Destructive shell command**: `rm -rf /`
  - blocked before forward path

## Slide 9 — Human Approval Experience
- Dashboard shows live connection state to the daemon
- Sidebar + forensic detail workflow for fast triage
- Incident detail includes:
  - agent identity
  - process and process group
  - workspace and surface ids
  - requested capability
  - raw payload
  - working directory
  - evidence list
- Action buttons are explicit and fast:
  - Allow Once
  - Continue Watched
  - LLM Judge
  - Kill
- Keyboard shortcuts make it feel operational, not ceremonial

## Slide 10 — Architecture in One Slide
- **Rust daemon**
  - owns hot-path policy and enforcement
  - accepts control and registration sockets
  - emits incidents
- **SwiftUI app**
  - native approval cockpit for human review and decisions
- **Launcher / identity layer**
  - registers agent identity, process info, and nonce material
- **Shared protocol types**
  - align wire contract between Rust and Swift
- **Plugin/advisory layer**
  - supports future enrichment such as LLM judgment and file-type analysis

## Slide 11 — Demo Flow
- Launch daemon and dashboard
- Simulate or receive an agent capability request
- Show a safe request being auto-allowed
- Show a sensitive request generating a watched incident
- Show a critical request pausing the process and opening a human decision point
- Approve, continue watched, or kill from the UI
- Show the audit story: the system can explain what happened and why

## Slide 12 — What We Built During the Hackathon
- A working native SwiftUI dashboard
- A Rust daemon scaffold with Unix socket control flow
- Identity verification with same-user credential checks
- Agent registration and nonce-based authentication path
- Multi-scanner classification pipeline
- Structured incidents and decision actions
- Audit event emission
- A credible end-to-end prototype of an agent safety control loop

## Slide 13 — Why This Matters
- Agents are becoming real operators on developer machines
- Local capability security needs something between “fully trusted” and “fully blocked”
- Security Island makes agent behavior:
  - visible
  - reviewable
  - stoppable
  - attributable
- This creates a path toward safer autonomous coding without killing the speed people want

## Slide 14 — Future Direction
- Full daemon-backed live telemetry instead of seeded/demo incidents
- Stronger wire-contract tests between Swift and Rust
- Deeper `cmux` integration on the hot path
- Better policy coverage and richer scanners
- Append-only audit history and review workflows
- Plugin ecosystem for enrichment and external notifications
- Production-grade agent registry and stronger macOS identity guarantees

## Slide 15 — Closing / One-Liner
- **Security Island gives autonomous coding agents a native macOS safety layer: inspect, pause, explain, and decide before risky actions slip through.**
- The dream: fast agents, visible power, human control
