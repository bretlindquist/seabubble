import Foundation

/// The severity level of a security incident.
public enum Severity: String, Codable, Comparable {
    case low, medium, high, critical
    
    public static func < (lhs: Severity, rhs: Severity) -> Bool {
        let order: [Severity: Int] = [.low: 0, .medium: 1, .high: 2, .critical: 3]
        return order[lhs]! < order[rhs]!
    }
}

/// The state of an incident in the system.
public enum IncidentState: String, Codable {
    case safe
    case watch
    case pendingDecision
    case queuedForLLM
    case resolvedAllowed
    case continuedWatched
    case killed
    case error
}

/// Actions available to the human or LLM judge.
public enum AllowedAction: String, Codable, CaseIterable {
    case allowOnce
    case continueWatched
    case kill
    case llmJudge
}

/// Represents an active or historical security event triggered by an agent.
public struct Incident: Identifiable, Codable {
    public let id: String
    public let agentId: String
    public let capability: String
    public let payload: String
    
    public let pid: Int32
    public let pgid: Int32
    
    public var state: IncidentState
    public var riskScore: Int
    public var severity: Severity
    
    public var evidence: [String]
    public let createdAt: Date
    public let allowedActions: [AllowedAction]
    
    public init(
        id: String = UUID().uuidString,
        agentId: String,
        capability: String,
        payload: String,
        pid: Int32,
        pgid: Int32,
        state: IncidentState = .safe,
        riskScore: Int = 0,
        severity: Severity = .low,
        evidence: [String] = [],
        createdAt: Date = Date(),
        allowedActions: [AllowedAction] = [.allowOnce, .continueWatched, .kill, .llmJudge]
    ) {
        self.id = id
        self.agentId = agentId
        self.capability = capability
        self.payload = payload
        self.pid = pid
        self.pgid = pgid
        self.state = state
        self.riskScore = riskScore
        self.severity = severity
        self.evidence = evidence
        self.createdAt = createdAt
        self.allowedActions = allowedActions
    }
}
