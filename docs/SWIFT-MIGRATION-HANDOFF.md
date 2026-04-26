# Production Handoff: Seabubble Evolution

## 📍 Current State
The repository (`~/git/seabubble`) is fully up-to-date on the `main` branch. 
We have successfully transitioned the hackathon MVP into a highly robust, secure, and extensible architecture:
- **Daemon Backend (Rust):** Lock-free, hot-swappable Wasm plugin engine (via Extism), strict Unix socket permissions, and un-spoofable macOS `audit_token_t` process lineage tracking.
- **Frontend (SwiftUI):** Native macOS UI bundled with the Rust daemon via Xcode's Helper Executable pattern, utilizing `SMAppService` for zero-prompt background persistence.
- **Auto-Wiring:** Seamless `cmux` integration out of the box via dynamic config rewriting and strict-mode bash shims.

## 🔜 The Big Pivot: Native Swift Rewrite
You have decided to move the entire architecture to a **100% Native Swift** production environment, deprecating the Rust daemon backend.

This is a massive but logical step for a deeply integrated macOS security product. It will unify the codebase, simplify the build process, and allow you to leverage Apple's native Endpoint Security (ESF) APIs natively.

### 📋 Migration / Rewrite TODOs

#### Phase 1: Core Architecture Translation
- [ ] **Port the Daemon:** Translate `security-islandd/src/main.rs` into a headless Swift service or XPC service.
- [ ] **Port the Socket Logic:** Implement Unix Domain Socket listeners in Swift (using `Network.framework` or `DispatchSourceRead`) to replace the Rust `tokio` streams.
- [ ] **Port the Lineage Tracking:** Translate the `audit_token_t` FFI (`darwin.rs`) into Swift. Swift has native access to `bsm/libbsm.h` and `sys/socket.h`, making `getsockopt` with `LOCAL_PEERTOKEN` much cleaner.

#### Phase 2: The Scanner Pipeline
- [ ] **Port the Plugin Engine:** Decide on the plugin architecture for the Swift ecosystem. 
  - Will you continue using Wasm (via `Wasmtime` Swift bindings or `Extism` Swift SDK)?
  - Or will you pivot to native Swift frameworks/bundles loaded dynamically?
- [ ] **Port the AST Scanner:** Translate the custom shell tokenizer and `StateTracker` from `shell_policy.rs` into Swift.

#### Phase 3: System Integration & UI
- [ ] **Refactor `SMAppService`:** Ensure the new native Swift daemon registers correctly via `SMAppService`.
- [ ] **Unify the Control Bus:** Replace the JSON/NDJSON Unix socket communication between the UI and Daemon with a more robust native solution (e.g., Apple's XPC).
- [ ] **Clean Up the Repo:** Remove the `daemon/` Rust workspace, the `.cargo` files, and update the CI/CD workflows (`.github/workflows/ci.yml`) to exclusively run `xcodebuild` and Swiftformat.

***

*Godspeed on the Swift rewrite! The architectural blueprints and research docs in `docs/` remain highly relevant to the core concepts, even as the language changes.*