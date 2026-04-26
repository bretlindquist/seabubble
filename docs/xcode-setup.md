# Xcode Setup for SMAppService App Bundling (Track B)

To successfully bundle and run the Security Island native macOS app with its companion daemon (`security-islandd`), follow these manual steps in Xcode:

1. **Add Daemon Executable Build Phase:**
   - In your app's Xcode target, go to **Build Phases**.
   - Add a new **Copy Files Phase**.
   - Set the **Destination** to `Executable` (which maps to `Contents/MacOS`).
   - Add the compiled Rust binary `security-islandd` to this phase.
   
2. **Add LaunchAgent Plist Build Phase:**
   - Add another **Copy Files Phase**.
   - Set the **Destination** to `Wrapper`.
   - Specify the **Subpath** as `Contents/Library/LaunchAgents`.
   - Add `daemon/com.seabubble.daemon.plist` to this phase.

3. **Configure Hardened Runtime:**
   - Go to **Signing & Capabilities**.
   - Under the **Hardened Runtime** section, check **Disable Library Validation**. (This may be required if the daemon binary isn't signed identically to the main app during development).

By correctly bundling the plist into `Contents/Library/LaunchAgents` and the daemon into `Contents/MacOS`, `SMAppService.agent(plistName:)` will successfully locate and start the companion daemon when the main app launches.
