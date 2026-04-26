import Foundation

/// The central truth orchestrator for Security Island.
/// Binds directly to SwiftUI.
@MainActor
public final class DecisionBus: ObservableObject {
    @Published public private(set) var incidents: [Incident] = []
    @Published public private(set) var decisions: [Decision] = []
    @Published public var activeSurfaceId: String?
    
    private var plugins: [SecurityPlugin] = []
    
    public init() {}
    
    public func register(plugin: SecurityPlugin) {
        plugins.append(plugin)
    }
    
    public func appendIncident(_ incident: Incident) async {
        var mutableIncident = incident
        
        // Pass through plugins for extensible security layers
        for plugin in plugins {
            do {
                let eval = try await plugin.evaluate(incident: mutableIncident)
                mutableIncident.risk += eval.riskModifier
                mutableIncident.evidence.append(contentsOf: eval.extraEvidence)
                if eval.shouldHalt {
                    mutableIncident.state = .pendingDecision
                    _ = ProcessController.pause(mutableIncident.pgid)
                }
            } catch {
                print("Plugin \(plugin.name) failed: \(error)")
            }
        }
        
        // Automatically pause critical/high risk incidents if they aren't already pending
        if mutableIncident.severity >= .high && mutableIncident.state == .watch {
            mutableIncident.state = .pendingDecision
            _ = ProcessController.pause(mutableIncident.pgid)
        }
        
        incidents.append(mutableIncident)
    }
    
    public func applyDecision(incidentId: String, action: AllowedAction, source: String = "keyboard", actor: String = NSUserName()) {
        guard let index = incidents.firstIndex(where: { $0.id == incidentId }) else { return }
        var incident = incidents[index]
        
        let decision = Decision(
            id: UUID().uuidString,
            incidentId: incidentId,
            source: source,
            action: action,
            actor: actor,
            timestamp: Date()
        )
        decisions.append(decision)
        
        switch action {
        case .allowOnce:
            incident.state = .resolvedAllowed
            _ = ProcessController.resume(incident.pgid)
        case .continueWatched:
            incident.state = .continuedWatched
            _ = ProcessController.resume(incident.pgid)
        case .kill:
            incident.state = .killed
            Task {
                await ProcessController.safeKillSequence(pgid: incident.pgid)
            }
        case .llmJudge:
            incident.state = .queuedForLLM
        }
        
        incidents[index] = incident
    }
}
