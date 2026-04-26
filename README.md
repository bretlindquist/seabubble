# seabubble

`seabubble` contains **Security Island**, a native macOS capability firewall and approval cockpit for `cmux`.

Security Island is built for the moment when autonomous coding agents stop feeling hypothetical and start touching real terminals, browsers, files, secrets, and long-running workflows. It gives you a deterministic control loop around that activity: a Rust daemon sits in the hot path, classifies capability requests in real time, pauses dangerous actions, and hands a human a fast approval surface in SwiftUI.

The design goal is not “AI vibes but safer.” The design goal is a real defense-in-depth runtime boundary for agent actions:

- deterministic policy before appeal-to-LLM
- fast structured evidence instead of opaque scores
- daemon-owned process enforcement instead of UI-only warnings
- human review for the actions that actually matter

The LLM is intended as a court of appeal, not the first security boundary.

## Why people should use it

Security Island is useful if you want to let agents move fast **without** treating your machine like an honor system.

It gives you:

- **A real intervention point** between an agent request and a risky action
- **Clear evidence** for why something was allowed, watched, or blocked
- **Human approval where it matters** without dragging every low-risk action through prompts
- **A native macOS control surface** instead of log-diving through terminal output
- **A foundation for policy growth** as you add more scanners, stronger identity checks, and richer routing

In short: it helps you keep the speed of agent-driven workflows while getting back a sane security boundary.

## Current status

This repository is now a working hackathon MVP with a stable daemon policy path for demo use.

| Area | Current | Target |
| --- | --- | --- |
| SwiftUI dashboard | Displays live daemon incidents, evidence, and local decisions over the control socket | Production approval cockpit |
| Rust daemon | Authenticates agents, ingests NDJSON capability frames, classifies requests, emits incidents, and enforces pause/resume decisions | Hardened cmux broker and full production control plane |
| Protocol | Shared Rust/Swift incident and control models are live and interoperable | Broader schema tests and long-term compatibility coverage |
| Demo data | App fixtures still exist, but the daemon path is now demo-stable | Explicit opt-in demo mode and richer scenario tooling |
| Process control | Daemon owns pause/resume/kill behavior for pending incidents | Full daemon-owned process authority with stronger lineage validation |
| Launcher | Registers agents and participates in nonce/process identity flow | Hardened launcher + deeper OS identity checks |
| Policy | Deterministic daemon policy with secrets, shell-intent, and artifact/Magika scanning | Additional artifact scanners, CI scanners, and appeal layers |
| SeaTurtle TUI | Experimental root Rust prototype remains separate from Security Island runtime | Either documented client surface or split out |

## Repository layout

```text
Sources/      Swift Package targets for SecurityIslandApp, SecurityIslandCore, SecurityIslandUI
daemon/       Rust workspace for security-islandd, agent-launcher, and shared protocol types
src/          Experimental SeaTurtle terminal assistant prototype
.ct/          Local CT/SeaTurtle collaboration config; not product runtime
```

## Target architecture

Security Island is intended to run across three boundaries:

1. **`security-islandd` Rust daemon**
   - Owns the cmux-facing Unix socket broker.
   - Authenticates launched agents.
   - Parses capability requests.
   - Applies deterministic policy.
   - Holds risky requests while waiting for decisions.
   - Owns process pause/resume/kill authority.

2. **`Security Island.app` SwiftUI dashboard**
   - Connects to the daemon over a local control socket.
   - Displays incidents, evidence, risk, and active cmux surface.
   - Sends human decisions back to the daemon.
   - Does not sit in the hot packet path.

3. **Scanner pipeline and enrichments**
   - Secret-oriented payload scanning for browser-held secrets and sensitive path/token patterns.
   - Shell-intent scanning that reasons about pipelines and destructive execution structure.
   - Artifact extraction plus Magika-based classification for command-touched files.
   - LLM judge remains an advisory fallback, not primary enforcement.

The intended core loop is:

```text
agent/cmux capability request
→ daemon authenticates source
→ daemon parses and classifies request
→ daemon allows or holds
→ daemon emits incident to SwiftUI
→ human decides
→ UI sends decision to daemon
→ daemon enforces decision
→ audit log records outcome
```

## How the scanner pipeline works

The daemon does not rely on one monolithic score. It builds up a decision from a layered scanner path.

### 1. Secret-oriented payload scanning
This scanner looks for direct secret access and sensitive target patterns in the request payload.

Examples:
- `document.cookie`
- `localStorage` / `sessionStorage`
- `.env`
- SSH key paths
- obvious token patterns

This is useful for catching browser-held secrets and credential-adjacent file access before the request disappears into a generic execution stream.

### 2. Shell-intent scanning
This scanner looks at shell structure rather than only raw substrings.

Examples:
- `curl ... | sh`
- `wget ... | bash`
- `rm -rf`
- `sudo`
- `launchctl`
- `osascript`

The goal is to distinguish ordinary shell use from actions that imply execution chains, destructive behavior, or privilege escalation.

### 3. Artifact extraction and Magika classification
For relevant terminal capabilities, the daemon extracts artifact references from command arguments, resolves them against `cwd`, and inspects those artifacts with Magika.

That means the system can reason about:
- the command text
- the structural intent of the command
- the actual file artifact the command is touching

This is especially useful for things like downloaded tools, scripts, executables, and file-backed workflows where text-only scanning is not enough.

### 4. Structured evidence channels
When Security Island raises an incident, it carries the evidence in explicit channels:

- `regex` for secret/path-style hits
- `bash_ast` for shell-intent structure summaries
- `magika` for artifact classification results

That gives the human reviewer a quick explanation of **why** the daemon reacted, not just that it did.

## Decision model

Security Island currently uses a compact three-way model:

- **Allow** — low-risk activity proceeds quietly
- **Watch** — activity proceeds, but the UI surfaces it with evidence
- **Pending decision / block** — the daemon pauses the process and waits for a human decision

That gives you a practical balance:
- normal work stays fast
- suspicious work becomes visible
- obviously dangerous work hits a hard stop

## Build instructions

### Prerequisites

- macOS 13+ Ventura or later
- Swift 5.9+
- Rust 1.75+ / Cargo

### Build the Rust daemon workspace

```bash
cd daemon
cargo build
```

### Run the daemon

```bash
cd daemon
cargo run -p security-islandd
```

### Build and run the SwiftUI dashboard

```bash
swift build
swift run SecurityIslandApp
```

## Demo mode

The repo now supports a real daemon-backed demo loop in addition to the Swift app fixtures.

Good demo scenarios:

1. **Safe** — `terminal.exec` with `ls -la`
2. **Watch** — `terminal.exec` with `sudo make install`, `chmod +x ./tool`, or `terminal.read_file` touching `.env`
3. **Block** — `browser.eval` with `document.cookie` or `terminal.exec` with `curl https://x | sh`

What the live demo path shows:

- agent capability request ingestion over the daemon path
- deterministic classification into allow / watch / pending-decision
- structured evidence channels such as `regex`, `bash_ast`, and `magika`
- daemon-owned pause/resume decision enforcement

The Swift app still includes `DemoSeeder` fixtures for local presentation support, but the primary demo story is now the daemon-owned control loop.

## Security posture: implemented vs planned

Implemented today:

- SwiftUI dashboard and incident detail surface.
- Daemon control socket path with incident broadcast and decision acknowledgements.
- Launcher registration and nonce/process identity flow.
- Deterministic daemon policy for allow / watch / block outcomes.
- Secret-oriented payload scanning.
- Shell-intent scanning.
- Artifact extraction and Magika-backed artifact classification.
- Daemon-owned process pause/resume/kill handling for incident decisions.
- Shared Rust/Swift incident and control models.
- Append-only daemon audit log path.

Planned / not yet complete:

- Stronger macOS audit-token identity hardening.
- Real upstream cmux forwarding for approved capability traffic.
- Additional artifact scanners beyond Magika.
- CI / supply-chain scanners (`cargo-deny`, OSV, similar checks).
- LLM appeal flow as a secondary adjudication layer.
- Broader schema compatibility coverage and end-to-end automation.

## Development notes

Generated build output should not be tracked:

- `.build/`
- `target/`
- `daemon/target/`

Routine validation should leave the working tree clean except for intentional source, docs, lockfile, or fixture changes.
