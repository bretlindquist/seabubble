import Foundation
import Network
import os

class CmuxListener {
    private let listener: NWListener
    private let logger = Logger(subsystem: "com.securityisland.daemon", category: "cmux")
    private let socketPath: String

    init(socketPath: String) {
        self.socketPath = socketPath
        let endpoint = NWEndpoint.unix(path: socketPath)
        
        let parameters = NWParameters.tcp
        parameters.requiredLocalEndpoint = endpoint
        
        // Remove existing socket file if it exists so we can bind
        try? FileManager.default.removeItem(atPath: socketPath)
        
        do {
            self.listener = try NWListener(using: parameters)
        } catch {
            fatalError("Failed to initialize listener: \(error)")
        }
    }

    func start() {
        listener.stateUpdateHandler = { [weak self] state in
            guard let self = self else { return }
            switch state {
            case .ready:
                self.logger.notice("Cmux listener ready on \(self.socketPath)")
            case .failed(let error):
                self.logger.error("Cmux listener failed: \(error.localizedDescription)")
            case .cancelled:
                self.logger.notice("Cmux listener cancelled")
            default:
                break
            }
        }

        listener.newConnectionHandler = { [weak self] connection in
            self?.logger.info("New connection received")
            connection.start(queue: .global())
            // Implementation of connection handling would go here
        }

        listener.start(queue: .main)
    }
}
