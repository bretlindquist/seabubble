# Swift Plugin Architecture Research: Wasm vs. Native Bundles

## 1. Swift Wasm Options

### Extism Swift SDK (`extism/swift-sdk`)
- **Maturity:** Actively maintained and maturing. Extism provides a unified SDK approach across many languages.
- **Integration:** It supports Swift Package Manager (SwiftPM). However, it relies on wrapping the core Extism Rust library (via C-bindings). This means your Swift app must link against the Extism dynamic/static library, which slightly complicates pure-Swift cross-compilation and distribution.
- **Pros:** Highly opinionated, easy-to-use API for passing strings/bytes in and out of plugins.
- **Cons:** Dependency on a non-Swift core library.

### Wasmtime Swift Bindings
- **Maturity:** The Bytecode Alliance provides Swift bindings for Wasmtime (via `wasmtime-swift` or similar wrappers).
- **Integration:** Also available via SwiftPM, wrapping the C API of Wasmtime.
- **Pros:** Extremely robust, industry-standard Wasm engine.
- **Cons:** Lower-level API than Extism. You have to manage memory, exports, and imports manually unless you build an abstraction layer.

## 2. Wasm vs. Native Swift Bundles

### Native Swift Bundles (`Bundle(url:).load()`)
Apple platforms support loading dynamic frameworks or loadable bundles at runtime.
- **Pros:** 
  - **Zero Overhead:** Direct memory access and native Swift execution speed.
  - **Ecosystem:** Deep integration with Apple frameworks (Foundation, Combine, etc.).
  - **Simplicity:** No need to embed a Wasm runtime; pure Swift tooling.
- **Cons:**
  - **Security:** **Zero sandboxing.** A malicious or buggy plugin can crash the host daemon, read sensitive memory, or execute arbitrary code on the system.
  - **Language Lock-in:** Plugins must be written in Swift (or C/Objective-C).
  - **Platform Lock-in:** `Bundle` loading is heavily tied to Darwin (macOS/iOS). Linux support for dynamic library loading in Swift exists but is less seamless.
  - **ABI Stability:** Plugins must be compiled with matching Swift compiler versions unless built with Library Evolution enabled.

### WebAssembly (Wasm) via Extism/Wasmtime
- **Pros:**
  - **Sandboxing:** Plugins run in a secure sandbox. They cannot access the host filesystem, network, or memory unless explicitly permitted.
  - **Language Agnostic:** Plugins can be written in Rust, Go, JavaScript, Python, Zig, or Swift (via SwiftWasm).
  - **Cross-Platform:** Wasm binaries run identically on macOS, Linux, and Windows.
- **Cons:**
  - **Overhead:** Serialization/deserialization cost when passing complex data across the host/plugin boundary.
  - **Tooling:** Requires developers to set up Wasm toolchains.

## 3. Recommendation for Seabubble

**Top Recommendation: Extism (Wasm)**

For a security-focused daemon like Seabubble, **sandboxing is non-negotiable**. While native Swift Bundles offer incredible performance and ease of use, giving plugins unrestricted access to the host daemon's memory space violates the principle of least privilege. A vulnerability or malicious code in a plugin could compromise the entire daemon.

By using **Extism**, Seabubble gains:
1. Strict, configurable security boundaries (memory and capability sandboxing).
2. The ability for users to write plugins in their language of choice (Rust, JS, Go, etc.), vastly expanding the ecosystem.
3. A relatively high-level API compared to raw Wasmtime.

*Trade-off:* The team will need to accept the overhead of integrating the Extism C-library via SwiftPM and the slight performance hit of Wasm serialization, but the security and flexibility gains far outweigh these costs for a system daemon.
