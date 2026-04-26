import Foundation
import OSLog
import IslandShared

/// The actual object that performs the work when an XPC request comes in.
/// It must conform to the @objc IslandDaemonXPCProtocol.
class DaemonService: NSObject, IslandDaemonXPCProtocol {
    
    private let pipeline: ScannerPipeline
    private var pendingIncidents: [String: Incident] = [:]
    private let queue = DispatchQueue(label: "com.island.daemon.state")
    
    init(pipeline: ScannerPipeline) {
        self.pipeline = pipeline
        super.init()
        
        NotificationCenter.default.addObserver(
            forName: NSNotification.Name("com.island.incident.emitted"),
            object: nil,
            queue: nil
        ) { [weak self] notification in
            guard let incident = notification.object as? Incident else { return }
            if incident.state == .pendingDecision {
                self?.queue.sync {
                    logger.info("💾 Storing real pending incident from cmux proxy: \(incident.id)")
                    self?.pendingIncidents[incident.id] = incident
                }
            }
        }
    }
    
    func ping(reply: @escaping (Bool) -> Void) {
        logger.debug("Received XPC ping.")
        reply(true)
    }
    
    func getStatus(reply: @escaping (String) -> Void) {
        logger.debug("Received XPC status request.")
        var pendingCount = 0
        queue.sync { pendingCount = pendingIncidents.count }
        
        let status = """
        Security Island Daemon [V2]
        Status: Active
        Scanners Loaded: 2
        Pending Decisions: \(pendingCount)
        """
        reply(status)
    }
    
    func simulateRequest(agentId: String, capability: String, payload: String, cwd: String, reply: @escaping (Data?) -> Void) {
        logger.info("Simulating request: \(capability) | \(payload)")
        let request = CapabilityRequest(
            agentId: agentId,
            capability: capability,
            payload: payload,
            cwd: cwd,
            processId: 9999 // Mock PID for simulation
        )
        
        Task {
            do {
                let incident = try await pipeline.evaluate(request: request, pgid: 9999)
                
                // If it requires a decision, store it so the CLI/UI can act on it later.
                if incident.state == .pendingDecision {
                    self.queue.sync {
                        logger.info("💾 Storing pending incident: \(incident.id)")
                        self.pendingIncidents[incident.id] = incident
                    }
                }
                
                let data = try JSONEncoder().encode(incident)
                reply(data)
            } catch {
                logger.error("❌ Simulation failed: \(error.localizedDescription)")
                reply(nil)
            }
        }
    }
    
    func submitDecision(incidentId: String, actionRawValue: String, reply: @escaping (Bool, String) -> Void) {
        logger.notice("Received decision \(actionRawValue) for incident \(incidentId)")
        
        guard let action = AllowedAction(rawValue: actionRawValue) else {
            logger.error("Unknown action: \(actionRawValue)")
            reply(false, "Unknown action: \(actionRawValue)")
            return
        }
        
        var targetIncident: Incident?
        queue.sync {
            targetIncident = pendingIncidents[incidentId]
        }
        
        guard let incident = targetIncident else {
            logger.error("Incident \(incidentId) is not pending or does not exist.")
            reply(false, "Incident \(incidentId) is not pending or does not exist.")
            return
        }
        
        logger.notice("⚖️ Applying decision \(action.rawValue) to incident \(incidentId) (PGID: \(incident.pgid))")
        
        var success = true
        var message = "Decision applied."
        
        // Apply the POSIX math based on the human decision.
        switch action {
        case .allowOnce, .continueWatched:
            if !ProcessController.resume(pgid: incident.pgid) {
                success = false
                message = "Failed to send SIGCONT to pgid \(incident.pgid)"
                logger.error("\(message)")
            }
        case .kill:
            if !ProcessController.terminate(pgid: incident.pgid) {
                success = false
                message = "Failed to send SIGTERM to pgid \(incident.pgid)"
                logger.error("\(message)")
            }
        case .llmJudge:
            message = "Incident queued for LLM appeal. Process remains paused."
            logger.notice("\(message)")
        }
        
        if success && action != .llmJudge {
            let _ = queue.sync {
                pendingIncidents.removeValue(forKey: incidentId)
                logger.info("🗑 Removed incident \(incidentId) from pending queue.")
            }
        }
        
        reply(success, message)
    }
}
