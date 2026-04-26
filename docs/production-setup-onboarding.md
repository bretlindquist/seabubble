# Seabubble: Production Setup & Onboarding Proposal

## 1. Zero-Friction Onboarding UX

The goal is to transition from a manual, terminal-heavy setup to a seamless, one-click experience that instantly protects the user's workspace via `cmux`.

**The Ideal Flow:**
1. **Download & Install:** User downloads a packaged application (e.g., DMG for macOS, AppImage/DEB for Linux) or runs a single-line curl script.
2. **First Run (UI Launch):** The user opens the Seabubble UI. The UI detects that this is the first run.
3. **Automated Daemon Bootstrapping:** The UI checks if the Seabubble background daemon is running. If not, it requests standard permissions (if necessary) and automatically installs/starts the daemon as a user-level service (e.g., `launchd` on macOS, `systemd --user` on Linux).
4. **Shell Integration Prompt:** A single setup wizard screen asks: *"Secure your agent environments (Claude, Codex, OpenCode)?"* Clicking "Yes" automatically injects the necessary environment hooks into the user's `.zshrc` or `.bashrc`.
5. **Ready State:** The UI shows a "Protected" status. The user can immediately open a terminal and run their agent CLI. The daemon is already listening, and `cmux` is pre-configured to intercept and route the session.

## 2. Configuration Management: Daemon and UI Shared State

To decouple the UI from the daemon while maintaining real-time synchronization, we should use a shared configuration file acting as the source of truth, combined with file system watching.

**Architecture:**
*   **Location:** Use standard OS application data directories:
    *   macOS: `~/Library/Application Support/Seabubble/config.json`
    *   Linux: `~/.config/seabubble/config.json`
*   **Source of Truth:** The UI acts as the primary configuration editor. When a user updates a policy (e.g., "Allow read access to ~/projects"), the UI writes to `config.json`.
*   **Hot-Reloading Daemon:** The Seabubble daemon uses a file watcher (like `fsnotify`) to monitor `config.json`. Upon detecting a change, the daemon validates the JSON and hot-reloads its active policy rules in memory without restarting.
*   **Socket Communication (Optional for State):** For ephemeral state (like active session requests or dynamic approval prompts), the daemon and UI communicate via a local UNIX socket (`~/.seabubble/daemon.sock`) or WebSocket. But persistent configuration must remain in the file system.

## 3. Environment Hooking

The biggest friction point currently is forcing users to manually export `SECURITY_ISLAND_BIND_PATH` or `CMUX_SOCKET_PATH` for every agent session. We need a way to ensure agent CLIs automatically route through Seabubble.

**Proposed Mechanisms:**

*   **Approach A: The Shell Initializer (Recommended)**
    *   Seabubble provides a command: `seabubble init zsh`.
    *   During onboarding, the app appends `eval "$(seabubble init zsh)"` to the user's `.zshrc`.
    *   This dynamically exports `CMUX_SOCKET_PATH` and `SECURITY_ISLAND_BIND_PATH` pointing to the active daemon, globally across all new terminal sessions. Agent CLIs inherently pick up these standard variables.

*   **Approach B: Transparent CLI Shims**
    *   Seabubble creates a hidden binary directory: `~/.seabubble/bin`.
    *   It places wrapper scripts here named after common agents (`claude`, `codex`).
    *   The wrapper script injects the environment variables and pipes the standard input/output through the `cmux` interface before calling the *actual* binary.
    *   The setup process prepends `~/.seabubble/bin` to the user's `$PATH`.
    *   *Pros:* Ensures protection even if standard env vars are cleared. *Cons:* Requires maintaining a list of supported CLI tool names.

*   **Approach C: Global Aliases**
    *   Inject aliases into the shell config: `alias claude="SECURITY_ISLAND_BIND_PATH=... claude"`.

**Conclusion for Hooking:**
A hybrid approach is best. Use **Approach A (Shell Initializer)** as the primary mechanism to set global environment variables for the current session. For highly specific agent frameworks that might ignore environment variables, we can offer **Approach B (Shims)** as an advanced "Strict Mode" toggle in the UI.
