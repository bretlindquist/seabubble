import Foundation
import Network

public enum DaemonConnectionState: String {
    case disconnected
    case connecting
    case connected
    case reconnecting
    case failed
}

/// Native Swift UNIX Domain Socket Client to receive Incidents from the Rust Daemon.
public final class DaemonControlClient: ObservableObject {
    @Published public private(set) var connectionState: DaemonConnectionState = .disconnected
    @Published public private(set) var lastError: String?
    @Published public private(set) var incidentErrors: [String: String] = [:]
    
    private var connection: NWConnection?
    private let controlSocketURL: URL
    private var receiveBuffer = Data()
    private let maxFrameSize = 1_048_576
    
    private weak var bus: DecisionBus?
    
    public init(bus: DecisionBus) {
        self.bus = bus
        let path = ProcessInfo.processInfo.environment["SECURITY_ISLAND_CONTROL_SOCKET"]?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        if let path, !path.isEmpty {
            self.controlSocketURL = URL(fileURLWithPath: path)
        } else {
            self.controlSocketURL = URL(fileURLWithPath: "/tmp/security-island/\(getuid())/control.sock")
        }
    }
    
    public func start() {
        updateConnectionState(.connecting)
        let endpoint = NWEndpoint.unix(path: controlSocketURL.path)
        let parameters = NWParameters.tcp
        parameters.requiredLocalEndpoint = nil // We are the client
        
        let connection = NWConnection(to: endpoint, using: parameters)
        self.connection = connection
        receiveBuffer.removeAll(keepingCapacity: true)
        
        connection.stateUpdateHandler = { [weak self] state in
            switch state {
            case .ready:
                print("✅ Connected to Rust Daemon control socket.")
                self?.updateConnectionState(.connected)
                self?.receiveLoop()
            case .waiting(let error):
                print("⏳ Control socket waiting: \(error)")
                self?.scheduleReconnect(error: error)
            case .failed(let error):
                print("❌ Control socket failed: \(error)")
                self?.scheduleReconnect(error: error)
            case .cancelled:
                self?.updateConnectionState(.disconnected)
            default:
                break
            }
        }
        
        connection.start(queue: .global(qos: .background))
    }
    
    private func scheduleReconnect(error: NWError) {
        updateConnectionState(.reconnecting, error: String(describing: error))
        DispatchQueue.global().asyncAfter(deadline: .now() + 2) { [weak self] in
            self?.start()
        }
    }
    
    private func updateConnectionState(_ state: DaemonConnectionState, error: String? = nil) {
        DispatchQueue.main.async { [weak self] in
            self?.connectionState = state
            self?.lastError = error
        }
    }
    
    private func receiveLoop() {
        guard let connection = connection else { return }
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            if let data = data, !data.isEmpty {
                self?.appendReceived(data)
            }
            
            if let error = error {
                print("⚠️ Control socket receive failed: \(error)")
                self?.scheduleReconnect(error: error)
                return
            }
            
            if isComplete {
                print("⚠️ Control socket disconnected. Reconnecting...")
                self?.updateConnectionState(.reconnecting)
                self?.start()
                return
            }
            
            self?.receiveLoop()
        }
    }
    
    private func appendReceived(_ data: Data) {
        receiveBuffer.append(data)
        
        if receiveBuffer.count > maxFrameSize {
            print("⚠️ Control socket frame exceeded max size; dropping buffer.")
            receiveBuffer.removeAll(keepingCapacity: true)
            return
        }
        
        while let newlineRange = receiveBuffer.firstRange(of: Data([0x0A])) {
            let frame = receiveBuffer[..<newlineRange.lowerBound]
            receiveBuffer.removeSubrange(...newlineRange.lowerBound)
            
            if !frame.isEmpty {
                process(data: Data(frame))
            }
        }
    }
    
    private func process(data: Data) {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        
        do {
            if let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
               let type = json["type"] as? String,
               let payloadDict = json["payload"] {
                
                let payloadData = try JSONSerialization.data(withJSONObject: payloadDict)
                
                if type == "incident" || type == "Incident" {
                    let incident = try decoder.decode(Incident.self, from: payloadData)
                    Task { @MainActor [weak self] in
                        await self?.bus?.appendIncident(incident)
                    }
                } else if type == "focus" || type == "Focus" {
                    struct FocusEvent: Decodable { let surface_id: String }
                    let focus = try decoder.decode(FocusEvent.self, from: payloadData)
                    Task { @MainActor [weak self] in
                        self?.bus?.activeSurfaceId = focus.surface_id
                    }
                } else if type == "decision_ack" || type == "DecisionAck" {
                    let ack = try decoder.decode(DecisionAck.self, from: payloadData)
                    Task { @MainActor [weak self] in
                        if ack.accepted {
                            self?.bus?.applyDecision(incidentId: ack.incidentId, action: ack.action)
                            self?.incidentErrors.removeValue(forKey: ack.incidentId)
                        } else if let message = ack.message {
                            print("⚠️ Daemon rejected decision for \(ack.incidentId): \(message)")
                            self?.incidentErrors[ack.incidentId] = message
                        }
                    }
                }
            } else {
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
        
        guard var data = try? JSONSerialization.data(withJSONObject: payload),
              let connection = connection else { return }
        data.append(0x0A)
        
        connection.send(content: data, completion: .contentProcessed({ [weak self] error in
            if let error = error {
                print("❌ Failed to send decision to daemon: \(error)")
                DispatchQueue.main.async {
                    self?.incidentErrors[incidentId] = "Failed to send: \(error.localizedDescription)"
                }
            }
        }))
    }
    
    public func sendReloadPlugins() {
        let payload: [String: Any] = [
            "type": "ReloadPlugins"
        ]
        
        guard var data = try? JSONSerialization.data(withJSONObject: payload),
              let connection = connection else { return }
        data.append(0x0A)
        
        connection.send(content: data, completion: .contentProcessed({ error in
            if let error = error {
                print("❌ Failed to send ReloadPlugins to daemon: \(error)")
            } else {
                print("✅ Sent ReloadPlugins to daemon")
            }
        }))
    }
    
    public func sendTogglePlugin(id: String, enabled: Bool) {
        let payload: [String: Any] = [
            "type": "TogglePlugin",
            "plugin_id": id,
            "enabled": enabled
        ]
        
        guard var data = try? JSONSerialization.data(withJSONObject: payload),
              let connection = connection else { return }
        data.append(0x0A)
        
        connection.send(content: data, completion: .contentProcessed({ error in
            if let error = error {
                print("❌ Failed to send TogglePlugin to daemon: \(error)")
            } else {
                print("✅ Sent TogglePlugin for \(id) to daemon")
            }
        }))
    }
    
    public func lastIncidentError(for incidentId: String) -> String? {
        return incidentErrors[incidentId]
    }
}

private struct DecisionAck: Decodable {
    let incidentId: String
    let action: AllowedAction
    let accepted: Bool
    let message: String?
    
    enum CodingKeys: String, CodingKey {
        case incidentId = "incident_id"
        case action, accepted, message
    }
}
