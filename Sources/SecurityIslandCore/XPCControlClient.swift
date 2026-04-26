import Foundation

/// Stub implementation showing how `NSXPCConnection` will replace the Unix socket client.
public final class XPCControlClient: ObservableObject {
    @Published public private(set) var connectionState: DaemonConnectionState = .disconnected
    @Published public private(set) var lastError: String?
    
    private var connection: NSXPCConnection?
    private weak var bus: DecisionBus?
    
    public init(bus: DecisionBus) {
        self.bus = bus
    }
    
    public func start() {
        updateConnectionState(.connecting)
        
        // Instantiate the XPC connection to the daemon's Mach service
        let newConnection = NSXPCConnection(machServiceName: "com.seabubble.daemon.xpc", options: [])
        
        // The protocol that the remote daemon implements
        newConnection.remoteObjectInterface = NSXPCInterface(with: SecurityIslandDaemonXPCProtocol.self)
        
        // Optional: If we want to receive callbacks (like incidents) from the daemon
        // newConnection.exportedInterface = NSXPCInterface(with: SecurityIslandUIListenerProtocol.self)
        // newConnection.exportedObject = self
        
        newConnection.interruptionHandler = { [weak self] in
            print("⚠️ XPC Connection Interrupted")
            self?.updateConnectionState(.reconnecting)
        }
        
        newConnection.invalidationHandler = { [weak self] in
            print("❌ XPC Connection Invalidated")
            self?.updateConnectionState(.disconnected)
            self?.connection = nil
        }
        
        newConnection.resume()
        self.connection = newConnection
        updateConnectionState(.connected)
        print("✅ XPC Connection resumed to com.seabubble.daemon.xpc")
    }
    
    public func stop() {
        connection?.invalidate()
    }
    
    public func sendDecision(action: String, incidentId: String) {
        guard let proxy = connection?.remoteObjectProxyWithErrorHandler({ error in
            print("❌ XPC Error sending decision: \(error)")
        }) as? SecurityIslandDaemonXPCProtocol else {
            return
        }
        
        proxy.applyDecision(incidentId: incidentId, action: action) { success, errorMsg in
            if success {
                print("✅ XPC Daemon accepted decision for \(incidentId)")
            } else {
                print("⚠️ XPC Daemon rejected decision for \(incidentId): \(errorMsg ?? "Unknown")")
            }
        }
    }
    
    public func requestStatus() {
        guard let proxy = connection?.remoteObjectProxyWithErrorHandler({ error in
            print("❌ XPC Error requesting status: \(error)")
        }) as? SecurityIslandDaemonXPCProtocol else {
            return
        }
        
        proxy.getDaemonStatus { statusJson in
            if let status = statusJson {
                print("ℹ️ XPC Daemon Status: \(status)")
            }
        }
    }
    
    private func updateConnectionState(_ state: DaemonConnectionState, error: String? = nil) {
        DispatchQueue.main.async { [weak self] in
            self?.connectionState = state
            self?.lastError = error
        }
    }
}
