import Foundation
import Darwin

/// Handles low-level POSIX process group math and Darwin-specific interception logic.
public struct ProcessController {
    
    /// Freezes an entire process group.
    /// This is the primary interception mechanism when an incident hits `pendingDecision`.
    /// - Parameter pgid: The process group ID.
    /// - Returns: True if successful, false otherwise.
    @discardableResult
    public static func pause(pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGSTOP)
    }
    
    /// Resumes a frozen process group.
    /// - Parameter pgid: The process group ID.
    /// - Returns: True if successful, false otherwise.
    @discardableResult
    public static func resume(pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGCONT)
    }
    
    /// Requests graceful termination of a process group.
    /// - Parameter pgid: The process group ID.
    /// - Returns: True if successful, false otherwise.
    @discardableResult
    public static func terminate(pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGTERM)
    }
    
    /// Forcibly kills a process group.
    /// - Parameter pgid: The process group ID.
    /// - Returns: True if successful, false otherwise.
    @discardableResult
    public static func forceKill(pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGKILL)
    }
    
    /// Sends a POSIX signal to an entire process group safely.
    private static func sendSignal(to pgid: pid_t, signal: Int32) -> Bool {
        guard pgid > 1 else {
            print("❌ SECURITY ERROR: Refusing to send signal \(signal) to unsafe pgid \(pgid) (kernel/init).")
            return false
        }
        
        let result = Darwin.killpg(pgid, signal)
        if result != 0 {
            print("⚠️ Failed to send signal \(signal) to pgid \(pgid). Errno: \(errno)")
        }
        return result == 0
    }
}
