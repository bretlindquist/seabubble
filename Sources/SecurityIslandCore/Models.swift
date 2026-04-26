import Foundation

public enum IncidentState: String, Codable {
    case safe
    case watch
    case pendingDecision = "pending_decision"
    case queuedForLLM = "queued_for_llm"
    case resolvedAllowed = "resolved_allowed"
    case continuedWatched = "continued_watched"
    case killed
    case error
}

public enum Severity: String, Codable, Comparable {
    case low, medium, high, critical
    
    public static func < (lhs: Severity, rhs: Severity) -> Bool {
        let order: [Severity: Int] = [.low: 0, .medium: 1, .high: 2, .critical: 3]
        return order[lhs]! < order[rhs]!
    }
}

public enum AllowedAction: String, Codable, CaseIterable {
    case allowOnce = "allow_once"
    case continueWatched = "continue_watched"
    case kill
    case llmJudge = "llm_judge"
}

public struct FilterResults: Codable {
    public var regex: String?
    public var bashAst: String?
    public var magika: String?
    public var llm: String?
    
    public init(regex: String? = nil, bashAst: String? = nil, magika: String? = nil, llm: String? = nil) {
        self.regex = regex
        self.bashAst = bashAst
        self.magika = magika
        self.llm = llm
    }
}

public struct Incident: Identifiable, Codable {
    public let id: String
    public let agentId: String
    public let paneId: String
    public let pid: pid_t
    public let pgid: pid_t
    public var state: IncidentState
    public var risk: Int
    public let severity: Severity
    public let reason: String
    public let ruleId: String?
    public let rawRedacted: String
    public let normalized: String
    public var evidence: [String]
    public var filterResults: FilterResults
    public let createdAt: Date
    public let allowedActions: [AllowedAction]
    
    enum CodingKeys: String, CodingKey {
        case id = "incident_id"
        case agentId = "agent_id"
        case paneId = "pane_id"
        case pid, pgid, state, risk, severity, reason
        case ruleId = "rule_id"
        case rawRedacted = "raw_redacted"
        case normalized, evidence
        case filterResults = "filter_results"
        case createdAt = "created_at"
        case allowedActions = "allowed_actions"
    }
}

public struct Decision: Identifiable, Codable {
    public let id: String
    public let incidentId: String
    public let source: String
    public let action: AllowedAction
    public let actor: String
    public let timestamp: Date
    
    enum CodingKeys: String, CodingKey {
        case id = "decision_id"
        case incidentId = "incident_id"
        case source, action, actor, timestamp
    }
}
