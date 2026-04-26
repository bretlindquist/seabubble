import Foundation
import OSLog
import IslandShared

let logger = Logger(subsystem: "com.bretlindquist.SecurityIslandDaemon", category: "App")

/// The delegate responsible for accepting or rejecting incoming XPC connections.
class DaemonXPCListenerDelegate: NSObject, NSXPCListenerDelegate {
    let pipeline: ScannerPipeline
    
    init(pipeline: ScannerPipeline) {
        self.pipeline = pipeline
        super.init()
    }
    
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        logger.debug("Received new XPC connection request.")
        
        newConnection.exportedInterface = NSXPCInterface(with: IslandDaemonXPCProtocol.self)
        newConnection.exportedObject = DaemonService(pipeline: pipeline)
        
        // 3. Begin listening on this specific connection
        newConnection.resume()
        
        logger.notice("Accepted new XPC connection.")
        return true
    }
}

@main
struct IslandDaemonApp {
    static func main() {
        print("Starting Security Island Daemon (V2)...")
        logger.notice("Starting Security Island Daemon (V2)...")

        let pipeline = ScannerPipeline()
        
        // Start the Unix Socket Broker to listen for live cmux traffic
        let uid = getuid()
        let publicSocketPath = "/tmp/security-island/\(uid)/cmux.sock"
        
        // Ensure the directory exists
        let dirPath = "/tmp/security-island/\(uid)"
        if !FileManager.default.fileExists(atPath: dirPath) {
            try? FileManager.default.createDirectory(atPath: dirPath, withIntermediateDirectories: true)
        }
        
        // We must hold a strong reference to the broker so the NWListener doesn't instantly deallocate
        let broker: CmuxSocketBroker
        do {
            broker = try CmuxSocketBroker(socketPath: publicSocketPath, pipeline: pipeline)
            broker.start()
        } catch {
            logger.fault("Failed to start cmux broker: \(error.localizedDescription)")
            return
        }

        // Register the scanners
        Task {
            await pipeline.register(plugin: ShellIntentScanner())
            await pipeline.register(plugin: ExecutionChainScanner())
            logger.notice("Scanners initialized.")
        }
        
        let delegate = DaemonXPCListenerDelegate(pipeline: pipeline)

        let listener = NSXPCListener(machServiceName: islandMachServiceName)
        listener.delegate = delegate
        listener.resume()

        logger.notice("Daemon listening on Mach Service: \(islandMachServiceName)")
        print("Daemon listening on Mach Service: \(islandMachServiceName)")

        // The Daemon must run infinitely on the main runloop to process XPC requests.
        RunLoop.main.run()
        
        // Keep broker alive (though RunLoop.main.run() never returns)
        _ = broker
    }
}
