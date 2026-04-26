import Foundation
import Network

/// Native Swift UNIX Domain Socket broker to intercept cmux capability requests.
public final class CmuxSocketBroker {
    private var listener: NWListener?
    private let incomingSocketURL = URL(fileURLWithPath: "/tmp/cmux.sock")
    private let destinationSocketURL = URL(fileURLWithPath: "/tmp/cmux.sock.real")
    
    // Hold a reference to the central bus to inject real incidents
    private weak var bus: DecisionBus?
    
    public init(bus: DecisionBus) {
        self.bus = bus
    }
    
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
        connection.stateUpdateHandler = { [weak self] state in
            switch state {
            case .ready:
                print("🔄 Intercepted new cmux automation request from socket.")
                self?.receiveMessage(on: connection)
            case .failed(let error):
                print("❌ Connection failed: \(error)")
                connection.cancel()
            case .cancelled:
                print("🛑 Connection cancelled.")
            default:
                break
            }
        }
        connection.start(queue: .global(qos: .userInitiated))
    }
    
    /// Recursively reads from the NWConnection and attempts to decode incoming JSON.
    private func receiveMessage(on connection: NWConnection) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self, weak connection] data, _, isComplete, error in
            guard let self = self, let connection = connection else { return }
            
            if let data = data, !data.isEmpty {
                self.process(data: data)
            }
            
            if let error = error {
                print("⚠️ Receive error: \(error)")
                connection.cancel()
                return
            }
            
            if isComplete {
                connection.cancel()
            } else {
                // Read next chunk
                self.receiveMessage(on: connection)
            }
        }
    }
    
    private func process(data: Data) {
        // In a true cmux environment, this would be a JSON-RPC CapabilityRequest.
        // For production resilience, if it fails to decode, we log and drop safely.
        let decoder = JSONDecoder()
        do {
            let incident = try decoder.decode(Incident.self, from: data)
            Task { @MainActor [weak self] in
                guard let self = self, let bus = self.bus else { return }
                await bus.appendIncident(incident)
                print("✅ Successfully routed socket incident \(incident.id) to DecisionBus.")
            }
        } catch {
            print("⚠️ Failed to decode incoming socket data into Incident: \(error.localizedDescription)")
            if let str = String(data: data, encoding: .utf8) {
                print("Raw payload received: \(str)")
            }
        }
    }
}
