# 🏝 Security Island: Hackathon Demo Script

## 🚀 Setup (Before the Judges Arrive)
1. Open the project in Xcode, or build it from the terminal:
   ```bash
   swift build
   swift run SecurityIslandApp
   ```
2. Have `cmux` running in the background (or visibly next to the app) to establish the context of multiple running AI agents.
3. Ensure the app is loaded. The `DemoSeeder` will automatically populate the UI with 3 incidents.

---

## 🎤 The Pitch (2-3 Minutes)

### 1. The Hook
*"Hi everyone, we built **Security Island**. When you're running powerful, parallel AI coding agents in a terminal like `cmux`, the standard approach is to have an LLM watch every single command. That's slow, expensive, and a massive privacy risk. We took a different approach."*

### 2. The Golden Line
*(Say this verbatim, make eye contact)*
**"Security Island does not make the LLM the firewall. It treats the LLM as the expensive court of appeal. Fast local filters create a pending human decision first."**

### 3. The UI Tour
*"We built this as a 100% native macOS app using Swift and SwiftUI to ensure deep OS integration. Let me show you the dashboard."*
- **Point to the Sidebar:** *"Here are our active agents. Notice the immediate visual triage: Green (Safe), Yellow (Hold), and Red (Critical)."*
- **Toggle the OS User Mask (Bottom/Top Toolbar):** *"Because we're integrated at the OS level, we need to know exactly which host user is executing these agents. To prevent accidental leaks during screen shares or screenshots, we built a privacy toggle. I click the lock, and it securely reveals my macOS identity."* *(Click the lock to unmask '••••••••' to your actual username, then click to hide).*

### 4. The Forensic Deep Dive
- **Select the Red Incident (`SI-000003`):** 
*"Let's look at this critical incident. Agent-deploy attempted a network fetch piped directly into a shell interpreter (`curl | sh`)."*
*"Before the agent could even execute this, Security Island intercepted the process group ID. Using native Darwin POSIX C APIs, we sent an instant `SIGSTOP` to freeze the entire process tree."*

- **Point to the Evidence & Plugins:**
*"Take a look at the Forensic Card. You can see the redacted command, our normalized intent graph, and the evidence list. We also built an extensible plugin architecture. Behind the scenes, our Magika plugin scanned the binary payload locally and added to the risk score, while our Telegram adapter silently pushed this exact forensic card to my phone as an actionable notification."*

### 5. The Resolution
*"The agent is frozen. It cannot proceed. As the human orchestrator, I have the final say. I can allow it, continue watching it, send it to the LLM for a deeper audit, or kill it."*
- **Action:** Press `Cmd + K` (or click KILL).
*"I'm pressing Command-K. Security Island just dispatched a `SIGTERM` followed by a `SIGKILL` directly to the Darwin kernel, wiping out the rogue process group entirely."*

### 6. The Close
*"Native macOS performance. Instant POSIX-level freezing. Extensible plugins for local Magika scanning and Telegram telemetry. Human-in-the-loop safety without the LLM latency. That's Security Island. Thank you."*

---

## ⚠️ Anticipated Judge Questions
- **"Why Swift instead of Python?"** 
  *Answer:* "To interact safely and instantly with macOS process groups (`killpg`) and to share the exact memory space with native macOS tools without relying on fragile inter-process communication."
- **"How does the Telegram bot work behind a firewall?"**
  *Answer:* "We built an async long-polling adapter using `URLSession`. It reaches out to Telegram rather than waiting for incoming webhooks, completely bypassing local NAT/firewall issues."
- **"What if the process group ID is spoofed?"**
  *Answer:* "Because Security Island is embedded locally alongside the parent orchestrator (`cmux`), the `pgid` is derived directly from the host OS kernel spawn, making it impossible for the guest agent to spoof its own ID."
