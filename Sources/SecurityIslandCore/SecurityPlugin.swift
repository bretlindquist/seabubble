import Foundation

public enum PluginStatus {
    case active, inactive, error(String)
}

public struct PluginEvaluation {
    public let riskModifier: Int
    public let extraEvidence: [String]
    public let shouldHalt: Bool
    
    public init(riskModifier: Int = 0, extraEvidence: [String] = [], shouldHalt: Bool = false) {
        self.riskModifier = riskModifier
        self.extraEvidence = extraEvidence
        self.shouldHalt = shouldHalt
    }
}

/// Extensible protocol for Security Island modules (e.g. Magika scanner, LLM Judge).
public protocol SecurityPlugin {
    var id: String { get }
    var name: String { get }
    var version: String { get }
    var status: PluginStatus { get }
    
    /// Evaluates an incident asynchronously. Plugins can augment evidence or increase risk.
    func evaluate(incident: Incident) async throws -> PluginEvaluation
}
