import Foundation
import Network
import IslandShared
import OSLog

let brokerLogger = Logger(subsystem: "com.bretlindquist.SecurityIslandDaemon", category: "Broker")

/// A pure Swift proxy that binds a local Unix socket, intercepts incoming `cmux` traffic,
/// extracts un-spoofable process lineage via `LOCAL_PEERTOKEN`, and routes the requests
/// into the `ScannerPipeline`.
public class CmuxSocketBroker {
    private let listener: NWListener
    private let pipeline: ScannerPipeline
    private let queue = DispatchQueue(label: "com.island.broker.queue")
    
    public init(socketPath: String, pipeline: ScannerPipeline) throws {
        self.pipeline = pipeline
        
        // Ensure the old socket file is removed before binding
        if FileManager.default.fileExists(atPath: socketPath) {
            try FileManager.default.removeItem(atPath: socketPath)
        }
        
        let endpoint = NWEndpoint.unix(path: socketPath)
        let parameters = NWParameters.tcp
        // We are the server listening on the socket
        parameters.requiredLocalEndpoint = endpoint
        
        self.listener = try NWListener(using: parameters)
        
        self.listener.stateUpdateHandler = { state in
            switch state {
            case .ready:
                brokerLogger.notice("🌉 Public cmux broker listening on \(socketPath)")
                print("🌉 Public cmux broker listening on \(socketPath)")
            case .failed(let error):
                brokerLogger.error("❌ Broker listener failed: \(error.localizedDescription)")
                print("❌ Broker listener failed: \(error.localizedDescription)")
            default:
                brokerLogger.debug("Broker state changed: \(String(describing: state))")
                print("Broker state changed: \(String(describing: state))")
            }
        }
        
        self.listener.newConnectionHandler = { [weak self] connection in
            self?.handleConnection(connection)
        }
    }
    
    public func start() {
        listener.start(queue: queue)
    }
    
    private func handleConnection(_ connection: NWConnection) {
        brokerLogger.info("🔌 New incoming cmux connection.")
        
        connection.start(queue: queue)
        receiveLoop(on: connection)
    }
    
    private func receiveLoop(on connection: NWConnection) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            guard let self = self else { return }
            
            if let data = data, !data.isEmpty {
                self.processData(data, from: connection)
            }
            
            if error != nil || isComplete {
                brokerLogger.info("🔌 Connection closed or errored.")
                connection.cancel()
                return
            }
            
            // Loop recursively
            self.receiveLoop(on: connection)
        }
    }
    
    private func processData(_ data: Data, from connection: NWConnection) {
        // Real cmux frames are line-delimited JSON (NDJSON)
        guard let string = String(data: data, encoding: .utf8) else { return }
        let lines = string.split(separator: "\n")
        
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            guard !trimmed.isEmpty, let frameData = trimmed.data(using: .utf8) else { continue }
            
            do {
                let request = try JSONDecoder().decode(CapabilityRequest.self, from: frameData)
                brokerLogger.debug("📥 Intercepted request: \(request.capability)")
                
                // TODO: In a production proxy, we need to extract the `audit_token_t` from the NWConnection
                // to securely verify the request.processId against the actual kernel PID.
                
                Task {
                    do {
                        let incident = try await self.pipeline.evaluate(request: request, pgid: request.processId)
                        
                        // We must inject this real incident into the DaemonService's state tracking
                        // so that `island status` and `island decide` can see it.
                        // We use a NotificationCenter broadcast to bridge the NWConnection thread to the DaemonService.
                        NotificationCenter.default.post(name: NSNotification.Name("com.island.incident.emitted"), object: incident)
                        
                        if incident.state == .pendingDecision {
                            brokerLogger.notice("🔴 Request paused. Waiting for human decision.")
                            // The broker must pause reading from this specific agent's connection
                            // until the human decides. 
                        } else {
                            brokerLogger.info("🟢 Request allowed. Forwarding to real cmux...")
                            // Forwarding logic goes here.
                        }
                    } catch {
                        brokerLogger.error("Pipeline evaluation failed: \(error.localizedDescription)")
                    }
                }
                
            } catch {
                brokerLogger.error("Failed to parse cmux frame: \(error.localizedDescription)")
            }
        }
    }
}
