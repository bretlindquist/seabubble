import Foundation
import Network

/// Native Swift UNIX Domain Socket broker to intercept cmux capability requests.
public final class CmuxSocketBroker {
    private var listener: NWListener?
    private let incomingSocketURL = URL(fileURLWithPath: "/tmp/cmux.sock")
    private let destinationSocketURL = URL(fileURLWithPath: "/tmp/cmux.sock.real")
    
    // In a real implementation, we would retain the decision bus to push incidents.
    // private let bus: DecisionBus
    
    public init() {}
    
    public func start() {
        // Clean up stale socket if it exists
        if FileManager.default.fileExists(atPath: incomingSocketURL.path) {
            try? FileManager.default.removeItem(at: incomingSocketURL)
        }
        
        guard let parameters = createUNIXParameters(),
              let listener = try? NWListener(using: parameters) else {
            print("Failed to configure NWListener for \(incomingSocketURL.path)")
            return
        }
        
        self.listener = listener
        
        listener.newConnectionHandler = { [weak self] connection in
            self?.handle(connection: connection)
        }
        
        listener.stateUpdateHandler = { state in
            switch state {
            case .ready:
                print("🎧 Security Island Broker listening on \(self.incomingSocketURL.path)")
            case .failed(let error):
                print("❌ Broker listener failed: \(error)")
            default:
                break
            }
        }
        
        listener.start(queue: .global(qos: .userInitiated))
    }
    
    public func stop() {
        listener?.cancel()
        listener = nil
        try? FileManager.default.removeItem(at: incomingSocketURL)
    }
    
    private func createUNIXParameters() -> NWParameters? {
        let parameters = NWParameters.tcp
        let endpoint = NWEndpoint.unix(path: incomingSocketURL.path)
        parameters.requiredLocalEndpoint = endpoint
        return parameters
    }
    
    private func handle(connection: NWConnection) {
        connection.stateUpdateHandler = { state in
            if state == .ready {
                print("🔄 Intercepted new cmux automation request.")
                // In production: Read data, parse into `CapabilityRequest`, 
                // run through DecisionBus, and only forward to `.real` if allowed.
                connection.cancel()
            }
        }
        connection.start(queue: .global(qos: .userInitiated))
    }
}
