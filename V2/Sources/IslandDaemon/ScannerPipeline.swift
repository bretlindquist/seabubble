import Foundation
import IslandShared

/// The core engine inside the Daemon that manages the scanner lifecycle.
public actor ScannerPipeline {
    private var plugins: [ScannerPlugin] = []
    private let context = StatefulScanContext()
    
    public init() {}
    
    /// Registers a new scanner to participate in the evaluation pipeline.
    public func register(plugin: ScannerPlugin) {
        plugins.append(plugin)
        print("🔌 Registered Scanner Plugin: \(plugin.name) [\(plugin.id)]")
    }
    
    /// Pushes a raw request through all registered scanners to build a unified Incident.
    public func evaluate(request: CapabilityRequest, pgid: Int32) async throws -> Incident {
        // 1. Record the request in the stateful context so future scans remember it.
        await context.record(request: request)
        
        var totalRiskScore = 0
        var highestSeverity: Severity = .low
        var aggregatedEvidence: [String] = []
        var requiresIntervention = false
        
        // 2. Route the request through every registered scanner concurrently.
        // We use a task group to ensure the pipeline is as fast as the slowest scanner.
        try await withThrowingTaskGroup(of: ScannerResult.self) { group in
            for plugin in plugins {
                group.addTask {
                    return try await plugin.evaluate(request: request, context: self.context)
                }
            }
            
            for try await result in group {
                totalRiskScore += result.riskModifier
                aggregatedEvidence.append(contentsOf: result.evidence)
                if result.requiresIntervention {
                    requiresIntervention = true
                }
                if let requestedSeverity = result.suggestedSeverity, requestedSeverity > highestSeverity {
                    highestSeverity = requestedSeverity
                }
            }
        }
        
        // 3. Synthesize the final state based on the aggregate results.
        let finalState: IncidentState
        if requiresIntervention || highestSeverity >= .high {
            finalState = .pendingDecision
        } else if totalRiskScore > 40 || highestSeverity == .medium {
            finalState = .watch
        } else {
            finalState = .safe
        }
        
        // 4. Construct the final Incident object that will be exposed via XPC.
        return Incident(
            agentId: request.agentId,
            capability: request.capability,
            payload: request.payload,
            pid: request.processId,
            pgid: pgid,
            state: finalState,
            riskScore: min(totalRiskScore, 100), // Cap at 100
            severity: highestSeverity,
            evidence: aggregatedEvidence
        )
    }
}
