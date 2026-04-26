# Production App Architecture Proposal for Seabubble

## 1. macOS App Bundling (SwiftUI + Rust)
To distribute Seabubble as a single `.app` bundle containing both the SwiftUI frontend and the Rust backend, we will use the **Helper Executable** pattern within a standard Xcode project.

- **Compilation**: The Rust backend is compiled into a standalone executable (ideally a universal binary for Apple Silicon and Intel).
- **Bundling via Xcode**: In the Xcode project for the SwiftUI app, add a "Copy Files" build phase. Configure it to copy the compiled Rust binary into the `Contents/MacOS` or `Contents/Helpers` directory of the `.app` bundle.
- **Code Signing**: Ensure the Rust binary is code-signed during the build process with the same Apple Developer Team ID as the SwiftUI app. This prevents Gatekeeper from blocking the execution of the bundled daemon.

## 2. Daemon Lifecycle (`SMAppService`)
To replace the hacky Python launcher scripts and seamlessly manage the Rust daemon, we will leverage Apple's modern `SMAppService` API (introduced in macOS 13 Ventura).

- **LaunchAgent Plist**: Create a standard `launchd` `.plist` file (e.g., `com.seabubble.daemon.plist`). The `ProgramArguments` should point dynamically to the embedded Rust executable.
- **Embedding the Plist**: Place this `.plist` in the `Contents/Library/LaunchAgents/` directory of the app bundle.
- **App Lifecycle Integration**: When the SwiftUI frontend opens, it checks the daemon's status. If it's not running, it executes:
  ```swift
  let service = SMAppService.agent(plistName: "com.seabubble.daemon.plist")
  do {
      try service.register()
  } catch {
      // Handle registration error
  }
  ```
- **Benefits**: This registers the Rust backend as a user agent. It will start automatically, stay running in the background even if the UI is closed, and automatically launch on user login. No admin password prompt is required for user-level agents via `SMAppService`.

## 3. Seamless `cmux` Integration
Seabubble must seamlessly inject itself as the capability broker for `cmux` without forcing the user to manually edit configuration files.

- **Auto-Configuration**: On startup, the Rust daemon will look for the `~/.cmux/config.toml` file.
- **Dynamic Injection**: If the file doesn't exist, or the broker configuration doesn't point to Seabubble, the daemon will automatically safely back up the file and write the necessary TOML configuration to point the `[broker]` settings to Seabubble's dedicated Unix Domain Socket (e.g., `~/.seabubble/daemon.sock`).
- **Environment Fallback**: Alternatively, if `cmux` supports environment variable overrides (e.g., `CMUX_BROKER_SOCKET`), the Seabubble UI could install a lightweight shell hook (`~/.zshrc` / `~/.bashrc`) that exports this variable, ensuring terminal sessions are always routed to the Seabubble daemon.
- **Conflict Resolution**: If another broker is detected, the SwiftUI app can gracefully surface a prompt asking the user to confirm setting Seabubble as the default `cmux` broker.
