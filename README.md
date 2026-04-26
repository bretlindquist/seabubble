# 🏝 Security Island

**Security Island** is a native macOS capability firewall and agent process governor for `cmux`. 

When running autonomous AI coding agents, standard security models rely on LLMs to evaluate command strings. Security Island rejects that approach. Instead, it acts as a high-performance **Unix Domain Socket Broker** written in Rust, intercepting deterministic `CapabilityRequests` and routing high-risk actions to a native macOS SwiftUI approval dashboard.

*The LLM is the expensive court of appeal. Fast local filters create a pending human decision first.*

## 🚀 Architecture

Security Island is a true defense-in-depth security product built across three boundaries:

1. **`security-islandd` (Rust Daemon):** A zero-copy `tokio` proxy. It binds to `/tmp/cmux.sock`, authenticates the connecting agent via XNU `getpeereid()` and macOS audit tokens, parses capabilities using `simd-json`, and enforces deterministic policy.
2. **`Security Island.app` (SwiftUI):** A decoupled, native Apple HIG dashboard. It connects to the Rust daemon via a secondary XPC control socket, ensuring the UI is never in the hot packet path.
3. **Plugins:** Extensible heuristic scanners (e.g., Google Magika) and remote telemetry (e.g., Telegram Bot API) evaluate incidents asynchronously via the `DecisionBus`.

## 🛠 Build Instructions

### Prerequisites
- macOS 13+ (Ventura or later)
- Swift 5.9+
- Rust 1.75+ (Cargo)

### 1. Build the Rust Daemon
The core proxy and shared schema live in the `daemon/` cargo workspace.
```bash
cd daemon
cargo build --release
```

### 2. Build the SwiftUI Dashboard
The native macOS app lives in the root directory and builds via Swift Package Manager.
```bash
swift build
swift run SecurityIslandApp
```

## 🎥 Hackathon Demo Mode
For the MVP presentation, the app includes a `DemoSeeder` that automatically injects three common OWASP LLM attack vectors:
1. **Safe:** `terminal.send_text` -> Log reading.
2. **Hold:** `terminal.send_text` -> Modifying executable permissions (`chmod`).
3. **Critical:** `browser.eval` -> Attempting to exfiltrate `document.cookie` via browser automation.

*Keyboard Shortcuts:* `Cmd+K` (Kill), `Cmd+A` (Allow), `Cmd+C` (Watch), `Cmd+L` (LLM Judge).

## 🛡 Security Posture
- **No Identity Spoofing:** Relies on XNU Kernel credentials (`getsockopt` / `uid: 501`) rather than trusting user-provided metadata.
- **Process Freezing:** Utilizes Darwin POSIX C APIs (`killpg`, `SIGSTOP`) to freeze entire process trees instantly.
- **Zero-Trust Network:** Employs a unique Session Nonce to cryptographically bind agent subprocesses to the socket.

---
*Built for the 2026 Safety & Security Hackathon.*
