# Production Plugin Architecture for Seabubble

## 1. Rust Plugin Architectures Evaluation

As a low-latency security firewall for LLM agents, Seabubble requires a plugin system that balances execution speed, strict security (sandboxing), and developer ergonomics. 

### A. WebAssembly (WASI / Wasmtime / Extism)
*   **Mechanism:** Plugins are compiled to `.wasm` and executed inside a lightweight JIT/AOT runtime (like `wasmtime` or a higher-level framework like `extism`) embedded in the Rust daemon.
*   **Pros:** 
    *   **Strict Security:** Complete sandboxing by default. Plugins cannot access the filesystem, network, or environment unless explicitly granted via WASI capabilities.
    *   **Performance:** Near-native execution speeds (typically within 10-20% of native Rust).
    *   **Language Agnostic:** Plugin authors can write scanners in Rust, Go, Zig, C++, or even JS/Python.
    *   **Fault Isolation:** A panic or memory leak in a plugin will not crash the Seabubble daemon.
*   **Cons:** Tooling for certain languages targeting Wasm can still be slightly immature; requires serializing strings/JSON across the host-guest boundary.

### B. Out-of-Process IPC (gRPC / ttrpc / Unix Sockets)
*   **Mechanism:** Plugins are standalone binary processes managed by the daemon. Communication happens over Unix domain sockets using protobufs or JSON.
*   **Pros:**
    *   **Ultimate Isolation:** Full OS-level process isolation.
    *   **Flexibility:** Any language can be used easily without Wasm build chains.
*   **Cons:**
    *   **Latency:** Serialization, deserialization, and context-switching over IPC add significant latency. For an *inline* security firewall analyzing streaming LLM tokens or large prompt chunks, this latency is often unacceptable.
    *   **Complexity:** Managing child process lifecycles (zombies, orphans, restarts) is highly complex.

### C. Dynamic Loading (`.dylib` / `libloading`)
*   **Mechanism:** Plugins are compiled as C-ABI dynamic libraries (`.so` or `.dylib`) and loaded directly into the daemon's memory space at runtime.
*   **Pros:**
    *   **Maximum Performance:** Zero-cost abstraction; calling a plugin is just a raw function pointer jump.
*   **Cons:**
    *   **Zero Safety:** A segfault, panic, or infinite loop in a plugin takes down the entire Seabubble daemon.
    *   **Security Risk:** Malicious plugins have full access to the daemon's memory and OS permissions.
    *   **ABI Instability:** Rust lacks a stable ABI, forcing all interfaces through `extern "C"`, which makes passing complex Rust types (like nested Enums or standard library collections) painful and error-prone.

## 2. Recommendation

**Top Recommendation: WebAssembly (WASI/Wasmtime)**
Given Seabubble's core identity as a *security* firewall with *low-latency* requirements, WebAssembly is the clear winner. Out-of-process IPC introduces too much latency for inline text scanning, and dynamic loading poses an unacceptable security and stability risk. 

Using a framework like **Extism** (which wraps Wasmtime) provides an excellent developer experience for plugin authors while keeping Seabubble fast, safe, and immune to plugin crashes.

## 3. Feature Toggling & Configuration

To support hot-reloading from the Swift UI without restarting the daemon:

*   **State Management (`ArcSwap`):** The active pipeline should be stored in an `ArcSwap` or similar lock-free construct (e.g., `arc-swap` crate). This allows the hot path (packet/LLM scanning) to maintain lock-free read access to the pipeline. When the pipeline is updated, a new version is built in the background and atomically swapped in.
*   **Trigger Mechanism:**
    1.  **Control Socket (Preferred):** Expose a lightweight Unix Domain Socket (UDS) API (e.g., a local REST or gRPC endpoint using `tonic` or `axum`). The Swift UI sends `POST /api/v1/plugins/toggle` or `POST /api/v1/config/reload`. This is more reliable than file watching.
    2.  **File Watcher (Fallback):** Use the `notify` crate to watch `~/.config/seabubble/config.toml`. When the UI modifies the config, the daemon detects the filesystem event, parses the new config, instantiates the enabled Wasm modules, and swaps the `ArcSwap` pointer.

## 4. Refactoring Strategy

To transition from the hardcoded `Pipeline` in `daemon/security-islandd/src/scanners/mod.rs` to this dynamic architecture:

1.  **Extract the API:** Create a new workspace crate (e.g., `seabubble-plugin-api`) defining the standard `Scanner` input/output models (Prompt, Response, Violation, Risk Score).
2.  **Update the `Scanner` Trait:** Refactor the trait to be object-safe or to represent a Wasm host wrapper:
    ```rust
    pub trait Scanner: Send + Sync {
        fn scan_prompt(&self, input: &str) -> Result<ScanResult, ScannerError>;
        fn name(&self) -> &str;
    }
    ```
3.  **Implement the Wasm Runner:** Create a `WasmScanner` struct that implements `Scanner` by invoking the underlying Extism/Wasmtime plugin.
4.  **Refactor the Pipeline:** 
    ```rust
    // From hardcoded structs:
    pub struct Pipeline {
        pii: PiiScanner,
        prompt_injection: PromptInjectionScanner,
    }

    // To dynamic, hot-swappable collection:
    pub struct Pipeline {
        scanners: Vec<Box<dyn Scanner>>,
    }
    
    // In the daemon state:
    pub struct DaemonState {
        pub active_pipeline: ArcSwap<Pipeline>,
    }
    ```
5.  **Build the Registry:** Create a local registry manager that handles fetching, verifying (checksums/signatures), and loading `.wasm` files from a dedicated plugins directory.
