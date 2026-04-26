import Foundation

/// Represents a raw request intercepted from the `cmux` environment before it is approved.
public struct CapabilityRequest: Codable {
    public let agentId: String
    public let capability: String
    public let payload: String
    public let cwd: String
    public let processId: Int32
    
    public init(agentId: String, capability: String, payload: String, cwd: String, processId: Int32) {
        self.agentId = agentId
        self.capability = capability
        self.payload = payload
        self.cwd = cwd
        self.processId = processId
    }
}

/// The result returned by a `ScannerPlugin` after evaluating a request.
public struct ScannerResult {
    /// How much risk this specific scanner adds to the overall score (0-100).
    public let riskModifier: Int
    
    /// If true, the Daemon will immediately flag this request and require human intervention.
    public let requiresIntervention: Bool
    
    /// Structured evidence explaining *why* the scanner made this decision.
    public let evidence: [String]
    
    /// Suggested severity modification (if the scanner determines this is a critical hit).
    public let suggestedSeverity: Severity?
    
    public init(riskModifier: Int = 0, requiresIntervention: Bool = false, evidence: [String] = [], suggestedSeverity: Severity? = nil) {
        self.riskModifier = riskModifier
        self.requiresIntervention = requiresIntervention
        self.evidence = evidence
        self.suggestedSeverity = suggestedSeverity
    }
}
