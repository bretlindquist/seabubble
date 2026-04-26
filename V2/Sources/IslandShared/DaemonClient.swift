import Foundation

/// A modern async/await wrapper around NSXPCConnection.
public actor DaemonClient {
    private let connection: NSXPCConnection
    
    public init() {
        self.connection = NSXPCConnection(machServiceName: islandMachServiceName, options: [])
        self.connection.remoteObjectInterface = NSXPCInterface(with: IslandDaemonXPCProtocol.self)
        
        // Setup interruption and invalidation handlers so we don't leak continuations
        self.connection.interruptionHandler = {
            print("⚠️ XPC Connection interrupted.")
        }
        self.connection.invalidationHandler = {
            // Suppress invalidation logging on clean deinit
        }
        
        self.connection.resume()
    }
    
    deinit {
        self.connection.invalidate()
    }
    
    private var proxy: IslandDaemonXPCProtocol {
        get throws {
            let remote = connection.remoteObjectProxyWithErrorHandler { error in
                print("❌ XPC Proxy Error: \(error.localizedDescription)")
            }
            guard let proxy = remote as? IslandDaemonXPCProtocol else {
                throw NSError(domain: "DaemonClient", code: 1, userInfo: [NSLocalizedDescriptionKey: "Failed to cast XPC Proxy. The Daemon may not be running or registered."])
            }
            return proxy
        }
    }
    
    public func ping() async throws -> Bool {
        let p = try proxy
        return try await withCheckedThrowingContinuation { continuation in
            p.ping { response in
                continuation.resume(returning: response)
            }
        }
    }
    
    public func getStatus() async throws -> String {
        let p = try proxy
        return try await withCheckedThrowingContinuation { continuation in
            p.getStatus { status in
                continuation.resume(returning: status)
            }
        }
    }
    
    public func simulateRequest(agentId: String, capability: String, payload: String, cwd: String) async throws -> Incident {
        let p = try proxy
        return try await withCheckedThrowingContinuation { continuation in
            p.simulateRequest(agentId: agentId, capability: capability, payload: payload, cwd: cwd) { data in
                guard let data = data else {
                    continuation.resume(throwing: NSError(domain: "DaemonClient", code: 2, userInfo: [NSLocalizedDescriptionKey: "Daemon returned nil data for simulation."]))
                    return
                }
                do {
                    let incident = try JSONDecoder().decode(Incident.self, from: data)
                    continuation.resume(returning: incident)
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }
    
    public func submitDecision(incidentId: String, action: AllowedAction) async throws -> String {
        let p = try proxy
        return try await withCheckedThrowingContinuation { continuation in
            p.submitDecision(incidentId: incidentId, actionRawValue: action.rawValue) { success, message in
                if success {
                    continuation.resume(returning: message)
                } else {
                    continuation.resume(throwing: NSError(domain: "DaemonClient", code: 3, userInfo: [NSLocalizedDescriptionKey: message]))
                }
            }
        }
    }
}
