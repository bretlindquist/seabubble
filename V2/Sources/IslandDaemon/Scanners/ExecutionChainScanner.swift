import Foundation
import IslandShared

/// A stateful scanner that tracks the history of an agent's commands.
/// It detects multi-step attacks, such as downloading a payload, changing its permissions, and executing it.
public struct ExecutionChainScanner: ScannerPlugin {
    public let id = "com.island.scanner.execution-chain"
    public let name = "Execution Chain Scanner"
    
    public init() {}
    
    public func evaluate(request: CapabilityRequest, context: StatefulScanContext) async throws -> ScannerResult {
        // Only run on terminal capabilities
        guard request.capability.contains("terminal") else {
            return ScannerResult()
        }
        
        let history = await context.history(for: request.agentId)
        let payload = request.payload.lowercased()
        
        var riskModifier = 0
        var evidence: [String] = []
        var requiresIntervention = false
        var suggestedSeverity: Severity? = nil
        
        // Rule: Download -> Chmod -> Execute sequence
        let isChmodExecute = payload.contains("chmod +x") && payload.contains("./")
        
        if isChmodExecute {
            // Look back in history to see if the target was recently downloaded
            let hasRecentDownload = history.contains { pastReq in
                let p = pastReq.payload.lowercased()
                return p.contains("curl") || p.contains("wget") || p.contains("git clone")
            }
            
            if hasRecentDownload {
                riskModifier += 85
                requiresIntervention = true
                suggestedSeverity = .critical
                evidence.append("Detected a multi-step execution chain: A file was recently downloaded, made executable, and launched.")
            } else {
                riskModifier += 30
                suggestedSeverity = .medium
                evidence.append("Modifying permissions and executing a local binary.")
            }
        }
        
        return ScannerResult(
            riskModifier: riskModifier,
            requiresIntervention: requiresIntervention,
            evidence: evidence,
            suggestedSeverity: suggestedSeverity
        )
    }
}
