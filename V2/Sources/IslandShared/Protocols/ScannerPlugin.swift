import Foundation

/// The foundational protocol that all Security Island Scanners must conform to.
/// This allows us to hot-swap, enable, and disable scanners without modifying the core Daemon engine.
public protocol ScannerPlugin: Sendable {
    
    /// The unique identifier of the scanner (e.g., "com.island.scanner.secrets").
    var id: String { get }
    
    /// A human-readable name for the UI (e.g., "Secret Payload Scanner").
    var name: String { get }
    
    /// Evaluates a raw capability request and returns a structured risk/evidence result.
    ///
    /// - Parameters:
    ///   - request: The intercepted request from the agent.
    ///   - context: Shared memory/history for stateful scanning (e.g., tracking a download across multiple commands).
    /// - Returns: A `ScannerResult` detailing the risk modifications and evidence.
    func evaluate(request: CapabilityRequest, context: StatefulScanContext) async throws -> ScannerResult
}

/// A thread-safe context object passed through the scanner pipeline.
/// Used to remember recent actions to catch multi-step attacks (e.g., `curl` -> `chmod` -> `./exec`).
public actor StatefulScanContext {
    private var recentCommands: [String: [CapabilityRequest]] = [:]
    
    public init() {}
    
    public func record(request: CapabilityRequest) {
        var agentHistory = recentCommands[request.agentId, default: []]
        agentHistory.append(request)
        // Keep only the last 10 commands per agent to prevent memory bloat
        if agentHistory.count > 10 {
            agentHistory.removeFirst(agentHistory.count - 10)
        }
        recentCommands[request.agentId] = agentHistory
    }
    
    public func history(for agentId: String) -> [CapabilityRequest] {
        return recentCommands[agentId] ?? []
    }
}
