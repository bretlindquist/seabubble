import Foundation

/// The unique identifier for the Security Island Daemon's XPC Service.
/// This must match the MachService name in the Daemon's Info.plist and SMAppService registration.
public let islandMachServiceName = "com.bretlindquist.SecurityIslandDaemon"

/// The formal XPC Protocol defining the API surface of the Daemon.
/// It must be exposed to Objective-C runtime (@objc) to work with NSXPCConnection.
@objc public protocol IslandDaemonXPCProtocol {
    
    /// A simple health check to ensure the Daemon is reachable.
    /// - Parameter reply: The callback invoked when the daemon receives the ping.
    func ping(reply: @escaping (Bool) -> Void)
    
    /// Requests the current status of the Daemon.
    /// - Parameter reply: The callback returning a JSON or formatted string of the current status.
    func getStatus(reply: @escaping (String) -> Void)
    
    /// Allows the CLI to simulate a capability request to test the scanner pipeline.
    func simulateRequest(agentId: String, capability: String, payload: String, cwd: String, reply: @escaping (Data?) -> Void)
    
    /// Submits a human decision (Allow, Kill, Watch) for a pending incident.
    /// - Parameters:
    ///   - incidentId: The ID of the incident being judged.
    ///   - actionRawValue: The raw string value of the `AllowedAction`.
    ///   - reply: Returns true if the daemon accepted and enforced the action, false otherwise.
    func submitDecision(incidentId: String, actionRawValue: String, reply: @escaping (Bool, String) -> Void)
}
