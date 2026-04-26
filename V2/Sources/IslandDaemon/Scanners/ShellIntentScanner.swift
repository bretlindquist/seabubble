import Foundation
import IslandShared

/// A fast, hot-path scanner that evaluates the structural intent of a shell command.
/// It moves past basic string matching to catch pipelines, privilege escalation, and execution chains.
public struct ShellIntentScanner: ScannerPlugin {
    public let id = "com.island.scanner.shell-intent"
    public let name = "Shell Intent Scanner"
    
    public init() {}
    
    public func evaluate(request: CapabilityRequest, context: StatefulScanContext) async throws -> ScannerResult {
        // Only run on terminal capabilities
        guard request.capability.contains("terminal") else {
            return ScannerResult()
        }
        
        let payload = request.payload.lowercased()
        var riskModifier = 0
        var evidence: [String] = []
        var requiresIntervention = false
        var suggestedSeverity: Severity? = nil
        
        // Rule 1: Direct Network-to-Execution Pipeline (The classic `curl | sh`)
        if (payload.contains("curl") || payload.contains("wget")) && (payload.contains("| sh") || payload.contains("| bash")) {
            riskModifier += 80
            requiresIntervention = true
            suggestedSeverity = .critical
            evidence.append("Detected direct network-to-shell execution pipeline (e.g., curl | sh).")
        }
        
        // Rule 2: Privilege Escalation
        if payload.contains("sudo ") {
            riskModifier += 50
            suggestedSeverity = .high
            evidence.append("Command explicitly requests root privilege escalation (sudo).")
        }
        
        // Rule 3: Destructive Intent
        if payload.contains("rm -rf") {
            riskModifier += 90
            requiresIntervention = true
            suggestedSeverity = .critical
            evidence.append("Detected highly destructive filesystem command (rm -rf).")
        }
        
        return ScannerResult(
            riskModifier: riskModifier,
            requiresIntervention: requiresIntervention,
            evidence: evidence,
            suggestedSeverity: suggestedSeverity
        )
    }
}
