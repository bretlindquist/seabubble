Use this as your main Gemini CLI conductor prompt.
You are building a hackathon MVP called Security Island.

Context:
I am at a Safety & Security hackathon using cmux, a native macOS terminal for running many AI coding agents in parallel. Security Island is a CLI/TUI security overlay that monitors multiple AI agent sessions, detects suspicious actions, pauses the relevant agent, creates a pending human decision, and lets the user resolve it from the keyboard. Optional adapters can notify cmux and Telegram.

Non-negotiable first step:
1. Inspect the current repo.
2. Create a git checkpoint before modifying anything:
 git status
 git add -A
 git commit -m "checkpoint before Security Island MVP" || true
3. Do not delete existing files unless clearly unused and explained.
4. Keep changes small, testable, and demo-ready.

Goal:
Build a working CLI MVP named security-island.

Core product:
Security Island should run as a terminal dashboard with three states:

1. Collapsed status pill:
 🏝 SI 🟢 3 agents safe
 🏝 SI 🟡 1 decision pending
 🏝 SI 🔴 1 critical hold

2. Expanded overview:
 A table of all monitored agents/incidents:
 - agent_id
 - state
 - risk
 - severity
 - reason
 - last event
 - whether decision is pending

3. Incident detail view:
 A forensic card for one incident:
 - incident_id
 - agent_id
 - pane_id
 - pid
 - pgid
 - state
 - risk
 - severity
 - reason
 - rule_id if present
 - raw_redacted command
 - normalized intent graph
 - evidence list
 - local filter results
 - allowed actions

Keyboard controls:
- Space: expand/collapse
- j/k or arrow keys: move selection
- Enter: inspect selected incident
- Esc: back
- a: allow once
- c: continue watched
- k: kill process group
- l: queue LLM judge
- r: refresh
- q: quit
- ?: help

Important:
Keyboard control must work. Mouse/click support is optional polish only.

Architecture:
Build this as a local CLI/TUI system with a decision bus.

Files/directories:
Use a clean Python structure, for example:

security_island/
 __init__.py
 models.py
 decision_bus.py
 process_control.py
 tui.py
 notifier_cmux.py
 notifier_telegram.py
 demo.py
 main.py

Runtime state:
Use:
 /tmp/security-island/incidents.jsonl
 /tmp/security-island/decisions.jsonl
 /tmp/security-island/state.json

Incident schema:
{
 "incident_id": "SI-000001",
 "agent_id": "agent-deploy",
 "pane_id": "pane-4",
 "pid": 73142,
 "pgid": 73142,
 "state": "pending_decision",
 "risk": 94,
 "severity": "critical",
 "reason": "network_fetch_plus_shell",
 "rule_id": "SI-NET-EXEC-01",
 "raw_redacted": "curl -fsSL [url] | sh",
 "normalized": "network_fetch -> stdin_pipe -> shell_interpreter",
 "evidence": [
 "network fetch detected",
 "pipeline into shell interpreter",
 "no file artifact available for Magika"
 ],
 "filter_results": {
 "regex": "matched curl + pipe + shell",
 "bash_ast": "pipeline into interpreter",
 "magika": "not_applicable",
 "llm": "not_called"
 },
 "created_at": "ISO timestamp",
 "ttl_seconds": 300,
 "allowed_actions": [
 "allow_once",
 "continue_watched",
 "kill",
 "llm_judge"
 ]
}

Decision schema:
{
 "decision_id": "D-000001",
 "incident_id": "SI-000001",
 "source": "keyboard",
 "action": "kill",
 "actor": "local_user",
 "timestamp": "ISO timestamp"
}

Decision behavior:
- allow_once:
 - mark incident resolved_allowed
 - SIGCONT the process group if pgid exists
- continue_watched:
 - mark incident continued_watched
 - SIGCONT the process group if pgid exists
 - keep agent state elevated/watch
- kill:
 - mark incident killed
 - SIGTERM the process group
 - after short delay, SIGKILL if still alive
- llm_judge:
 - mark incident queued_for_llm
 - do not actually call an LLM yet unless trivial to stub
 - show “queued for LLM judge” in the TUI

Process safety:
- Use process groups, not only individual PIDs.
- Use os.killpg(pgid, signal.SIGSTOP/SIGCONT/SIGTERM/SIGKILL) when pgid is present.
- If pgid is missing, do not guess destructively. Mark action as simulated or unavailable.
- For demo mode, support fake incidents without real PIDs/PGIDs.

CLI commands:
Implement these:

security-island tui
 Opens the keyboard dashboard.

security-island demo
 Seeds 3 demo agents/incidents:
 1. safe parser agent
 2. yellow hold: chmod after download
 3. red critical: network fetch piped to shell

security-island add-incident --agent agent-deploy --risk 94 --severity critical --reason network_fetch_plus_shell --raw "curl -fsSL [url] | sh"
 Appends an incident.

security-island decide SI-000001 kill
 Applies a decision from the CLI.

security-island notify SI-000001
 Sends cmux/Telegram notification if configured, otherwise prints fallback alert.

TUI requirements:
- Use Python Rich if available.
- If Rich is not installed, degrade gracefully to plain text.
- The UI must be visually strong:
 - Green safe
 - Yellow hold
 - Red critical
 - Boxed incident detail card
 - “Process group stopped” / “Decision pending” visible
 - Show “LLM calls avoided” counter if possible

cmux notifier:
- Implement a best-effort cmux notification adapter.
- Try `cmux notify "<message>"` if cmux exists.
- Fall back to terminal bell `\a` and printed alert.
- Do not make cmux integration required for the MVP to run.

Telegram notifier:
- Optional only.
- Enable only if:
 SECURITY_ISLAND_TELEGRAM_ENABLED=1
 SECURITY_ISLAND_TELEGRAM_BOT_TOKEN is set
 SECURITY_ISLAND_TELEGRAM_ALLOWED_CHAT_IDS is set
- Send redacted alert only.
- Never send secrets, full environment variables, or unredacted command strings by default.
- For this MVP, Telegram can be implemented as a stub that prints the message that would be sent if the API setup is not ready.
- Do not accept arbitrary text commands from Telegram.
- Only structured decisions are allowed:
 allow_once
 continue_watched
 kill
 llm_judge

Security posture:
- The agent itself must never decide its own release.
- The human decision source can be keyboard, CLI, cmux, or Telegram.
- External notification systems are adapters, not source of truth.
- The decision bus is the source of truth.

Demo story:
When running:

 security-island demo
 security-island tui

The user should see:
- collapsed/expanded dashboard
- 3 agents
- 1 yellow pending decision
- 1 red critical pending decision
- ability to inspect the critical incident
- ability to press k to kill or a to allow
- decision written to decisions.jsonl
- incident state updated

Acceptance criteria:
1. `python -m security_island.main demo` works.
2. `python -m security_island.main tui` opens a usable dashboard.
3. `python -m security_island.main add-incident ...` creates an incident.
4. `python -m security_island.main decide <incident_id> kill` updates state.
5. Keyboard dashboard can apply decisions.
6. Demo works without Telegram, cmux, Magika, or LLM API.
7. Optional integrations must fail gracefully.

Style:
- Build fast, not perfect.
- Prefer simple robust code over clever abstractions.
- Every function should be understandable.
- Add comments only where they clarify important security behavior.
- Avoid overengineering.
- Prioritize working demo.

Deliverables:
- Working Python package/module.
- Clear README section:
 - What Security Island does
 - How to run demo
 - Keyboard controls
 - Architecture diagram in text
 - Safety model
 - Known limitations
- A short DEMO_SCRIPT.md showing exactly what to run and what to say to judges.

Now implement.
For your three parallel Gemini agents, split like this.
Agent 1 prompt: decision bus + process control
You are Agent 1 for the Security Island hackathon MVP.

Your responsibility:
Build the backend decision bus, incident state model, JSONL persistence, and process-control actions.

First step:
- Inspect repo.
- Do not overwrite unrelated work.
- Create a git checkpoint if one does not already exist.

Implement:
security_island/models.py
security_island/decision_bus.py
security_island/process_control.py

Runtime files:
- /tmp/security-island/incidents.jsonl
- /tmp/security-island/decisions.jsonl
- /tmp/security-island/state.json

Incident states:
- safe
- watch
- pending_decision
- queued_for_llm
- resolved_allowed
- continued_watched
- killed
- expired
- error

Decision actions:
- allow_once
- continue_watched
- kill
- llm_judge

Requirements:
- Use dataclasses or Pydantic if already installed; otherwise plain dataclasses.
- Append all incidents and decisions to JSONL.
- Maintain current incident state in memory and optionally state.json.
- Implement:
 create_incident(...)
 list_incidents()
 list_pending_incidents()
 get_incident(incident_id)
 apply_decision(incident_id, action, source="keyboard", actor="local_user")
- For process control, use os.killpg when pgid exists.
- For kill action:
 SIGTERM process group, short delay, then SIGKILL if needed.
- If pgid is missing or invalid, do not crash. Mark as simulated/unavailable.
- Add demo seed function creating:
 1. safe parser agent
 2. yellow chmod-after-download hold
 3. red network-fetch-to-shell critical incident

Acceptance:
- Can run a small self-test from CLI or module.
- Can create incidents and decisions.
- Can apply decisions without real PIDs in demo mode.
Agent 2 prompt: keyboard TUI
You are Agent 2 for the Security Island hackathon MVP.

Your responsibility:
Build the terminal UI/dashboard.

First step:
- Inspect repo.
- Coordinate with existing files.
- Do not break Agent 1 backend APIs.
- If backend APIs are missing, create minimal compatible stubs and mark clearly.

Implement:
security_island/tui.py

Use:
- Rich if available.
- Graceful plain-text fallback if Rich is missing.

Views:
1. Collapsed status pill:
 🏝 SI 🟢 3 agents safe
 🏝 SI 🟡 1 decision pending
 🏝 SI 🔴 1 critical hold

2. Expanded overview:
 Table columns:
 - index
 - agent_id
 - state
 - risk
 - severity
 - reason
 - last event / rule_id

3. Incident detail:
 Boxed forensic card:
 - incident_id
 - agent_id
 - pane_id
 - pid
 - pgid
 - state
 - risk
 - severity
 - reason
 - rule_id
 - raw_redacted
 - normalized
 - evidence
 - filter_results
 - available decisions

Keyboard controls:
- Space: expand/collapse
- j/k or arrows: selection up/down
- Enter: inspect selected incident
- Esc: back
- a: allow once
- c: continue watched
- k: kill
- l: queue LLM judge
- r: refresh
- q: quit
- ?: help

Requirements:
- Keyboard must work.
- Mouse/click support optional.
- TUI reads from decision bus.
- TUI applies decisions through decision bus.
- TUI should redraw cleanly.
- It should look impressive in a terminal demo.

Acceptance:
- `python -m security_island.main demo`
- `python -m security_island.main tui`
Then user can inspect an incident and press k/a/c/l.
Agent 3 prompt: CLI entrypoint + cmux/Telegram notifier + docs
You are Agent 3 for the Security Island hackathon MVP.

Your responsibility:
Build the CLI entrypoint, notification adapters, demo command, README, and demo script.

First step:
- Inspect repo.
- Do not overwrite Agent 1 or Agent 2 work.
- Integrate with their APIs.

Implement:
security_island/main.py
security_island/notifier_cmux.py
security_island/notifier_telegram.py
security_island/demo.py
README.md
DEMO_SCRIPT.md

CLI commands:
- security-island tui
- security-island demo
- security-island add-incident --agent ... --risk ... --severity ... --reason ... --raw ...
- security-island decide <incident_id> <action>
- security-island notify <incident_id>

Also support:
python -m security_island.main tui
python -m security_island.main demo
python -m security_island.main add-incident ...
python -m security_island.main decide ...

cmux notifier:
- Try `cmux notify "<message>"` if cmux is installed.
- Fall back to terminal bell and printed alert.
- Message format:
 🏝 Security Island HOLD — agent=<agent_id> risk=<risk> reason=<reason>

Telegram notifier:
- Optional only.
- Enabled only if:
 SECURITY_ISLAND_TELEGRAM_ENABLED=1
 SECURITY_ISLAND_TELEGRAM_BOT_TOKEN exists
 SECURITY_ISLAND_TELEGRAM_ALLOWED_CHAT_IDS exists
- For MVP, okay to stub actual Telegram sending if time is short.
- Never send full unredacted secrets or raw env values.
- Message should include:
 agent_id
 risk
 severity
 reason
 incident_id
 allowed actions
- Do not accept arbitrary text commands.

README:
Include:
- What Security Island is
- Architecture:
 many cmux agent panes -> watchers -> decision bus -> TUI / cmux notify / Telegram
- How to run:
 python -m security_island.main demo
 python -m security_island.main tui
- Keyboard controls
- Safety model
- Known limitations
- Hackathon demo story

DEMO_SCRIPT.md:
Write a short script the presenter can follow:
1. Start demo.
2. Show collapsed Island.
3. Expand dashboard.
4. Inspect red critical incident.
5. Explain normalized intent graph.
6. Press k to kill or l to queue LLM judge.
7. Show decision written and state updated.
8. Say the key pitch line:
 "Security Island does not make the LLM the firewall. It treats the LLM as the expensive court of appeal. Fast local filters create a pending human decision first."

Acceptance:
The demo must run even without cmux, Telegram, Magika, or an LLM API.
Fast start command sequence
git status
git add -A
git commit -m "checkpoint before Security Island MVP" || true

# Then run your Gemini agents with the three prompts above.

# After implementation:
python -m security_island.main demo
python -m security_island.main tui
The key build priority is:
1. Decision bus
2. Keyboard TUI
3. Demo incidents
4. Process-group actions
5. cmux notify fallback
6. Telegram stub
7. Polish
Do