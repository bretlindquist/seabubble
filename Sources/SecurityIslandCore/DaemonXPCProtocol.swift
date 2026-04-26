import Foundation

@objc public protocol SecurityIslandDaemonXPCProtocol {
    /// Applies a decision to a specific incident.
    /// - Parameters:
    ///   - incidentId: The unique identifier of the incident.
    ///   - action: The action to apply (e.g., "allow", "deny").
    ///   - reply: A block called upon completion with success status and an optional error message.
    func applyDecision(incidentId: String, action: String, withReply reply: @escaping (Bool, String?) -> Void)
    
    /// Requests the current status of the daemon.
    /// - Parameter reply: A block returning a serialized status dictionary or JSON string.
    func getDaemonStatus(withReply reply: @escaping (String?) -> Void)
}

/// Optional: Protocol for the Daemon to send events back to the UI.
@objc public protocol SecurityIslandUIListenerProtocol {
    /// Delivers a new incident to the UI.
    /// - Parameter serializedIncident: JSON or serialized representation of the incident.
    func didReceiveIncident(_ serializedIncident: String)
}
