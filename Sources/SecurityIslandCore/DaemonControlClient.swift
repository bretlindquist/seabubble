import Foundation
import Network

/// Native Swift UNIX Domain Socket Client to receive Incidents from the Rust Daemon.
public final class DaemonControlClient {
    private var connection: NWConnection?
    private let controlSocketURL = URL(fileURLWithPath: "/tmp/security-island-control.sock")
    
    private weak var bus: DecisionBus?
    
    public init(bus: DecisionBus) {
        self.bus = bus
    }
    
    public func start() {
        let endpoint = NWEndpoint.unix(path: controlSocketURL.path)
        let parameters = NWParameters.tcp
        parameters.requiredLocalEndpoint = nil // We are the client
        
        let connection = NWConnection(to: endpoint, using: parameters)
        self.connection = connection
        
        connection.stateUpdateHandler = { [weak self] state in
            switch state {
            case .ready:
                print("✅ Connected to Rust Daemon control socket.")
                self?.receiveLoop()
            case .failed(let error):
                print("❌ Control socket failed: \(error)")
                // Attempt reconnect after delay
                DispatchQueue.global().asyncAfter(deadline: .now() + 2) {
                    self?.start()
                }
            default:
                break
            }
        }
        
        connection.start(queue: .global(qos: .background))
    }
    
    private func receiveLoop() {
        guard let connection = connection else { return }
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            if let data = data, !data.isEmpty {
                self?.process(data: data)
            }
            
            if error != nil || isComplete {
                print("⚠️ Control socket disconnected. Reconnecting...")
                self?.start()
                return
            }
            
            self?.receiveLoop()
        }
    }
    
    private func process(data: Data) {
        let decoder = JSONDecoder()
        
        // In the new architecture, the daemon sends a tagged ControlMessage
        // {"type": "Focus", "payload": {"surface_id": "surface:4"}}
        // {"type": "Incident", "payload": { ... }}
        
        // For Hackathon MVP backward compatibility with our mock payload,
        // we will try to decode it as an Incident first.
        do {
            if let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
               let type = json["type"] as? String,
               let payloadDict = json["payload"] {
                
                let payloadData = try JSONSerialization.data(withJSONObject: payloadDict)
                
                if type == "Incident" {
                    let incident = try decoder.decode(Incident.self, from: payloadData)
                    Task { @MainActor [weak self] in
                        await self?.bus?.appendIncident(incident)
                    }
                } else if type == "Focus" {
                    struct FocusEvent: Decodable { let surface_id: String }
                    let focus = try decoder.decode(FocusEvent.self, from: payloadData)
                    Task { @MainActor [weak self] in
                        self?.bus?.activeSurfaceId = focus.surface_id
                    }
                }
            } else {
                // Fallback for direct Incident JSON (from DemoSeeder / old tests)
                let incident = try decoder.decode(Incident.self, from: data)
                Task { @MainActor [weak self] in
                    await self?.bus?.appendIncident(incident)
                }
            }
        } catch {
            print("⚠️ Failed to decode control message from daemon: \(error)")
        }
    }
    
    public func sendDecision(action: AllowedAction, incidentId: String) {
        let payload: [String: Any] = [
            "action": action.rawValue,
            "incident_id": incidentId
        ]
        
        guard let data = try? JSONSerialization.data(withJSONObject: payload),
              let connection = connection else { return }
        
        connection.send(content: data, completion: .contentProcessed({ error in
            if let error = error {
                print("❌ Failed to send decision to daemon: \(error)")
            }
        }))
    }
}
