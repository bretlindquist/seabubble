import Foundation

public enum ScannerStage: String, Codable {
    case preForwardFast = "pre_forward_fast"
    case preForwardBlocking = "pre_forward_blocking"
}

public struct IncidentTemplate {
    public var stage: ScannerStage
    public var state: IncidentState
    public var severity: Severity
    public var reason: String
    public var ruleId: String?
    public var risk: Int
    public var evidence: [String]
    public var regex: String?
    public var bashAst: String?
    public var magika: String?
    public var allowedActions: [AllowedAction]
    
    public init(stage: ScannerStage, state: IncidentState, severity: Severity, reason: String, ruleId: String? = nil, risk: Int, evidence: [String], regex: String? = nil, bashAst: String? = nil, magika: String? = nil, allowedActions: [AllowedAction]) {
        self.stage = stage
        self.state = state
        self.severity = severity
        self.reason = reason
        self.ruleId = ruleId
        self.risk = risk
        self.evidence = evidence
        self.regex = regex
        self.bashAst = bashAst
        self.magika = magika
        self.allowedActions = allowedActions
    }
}

public protocol Scanner {
    func scan(req: CapabilityRequest) -> [IncidentTemplate]
}
