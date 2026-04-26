import Foundation
import os

let logger = Logger(subsystem: "com.securityisland.daemon", category: "main")

logger.notice("Starting native Swift Security Island Daemon...")

let uid = getuid()
let socketPath = "/tmp/security-island/\(uid)/cmux.sock"

// Ensure the directory exists
let socketDir = URL(fileURLWithPath: socketPath).deletingLastPathComponent()
try? FileManager.default.createDirectory(at: socketDir, withIntermediateDirectories: true, attributes: nil)

let listener = CmuxListener(socketPath: socketPath)
listener.start()

// Keep the daemon running
RunLoop.main.run()
