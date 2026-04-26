import Foundation
import Darwin

/// Controls POSIX Process Groups natively on Darwin (macOS).
public struct ProcessController {
    
    /// Sends a POSIX signal to an entire process group.
    @discardableResult
    public static func sendSignal(to pgid: pid_t, signal: Int32) -> Bool {
        // Darwin's killpg(pgid, signal). Returns 0 on success.
        let result = Darwin.killpg(pgid, signal)
        if result != 0 {
            print("Failed to send signal \(signal) to pgid \(pgid). Errno: \(errno)")
        }
        return result == 0
    }
    
    /// Freezes the process group immediately.
    @discardableResult
    public static func pause(_ pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGSTOP)
    }
    
    /// Resumes the process group.
    @discardableResult
    public static func resume(_ pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGCONT)
    }
    
    /// Requests graceful termination.
    @discardableResult
    public static func terminate(_ pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGTERM)
    }
    
    /// Forcibly kills the process group.
    @discardableResult
    public static func forceKill(_ pgid: pid_t) -> Bool {
        return sendSignal(to: pgid, signal: SIGKILL)
    }
    
    /// The standard Security Island kill sequence: SIGTERM, wait, SIGKILL.
    public static func safeKillSequence(pgid: pid_t) async {
        terminate(pgid)
        
        // Wait 1 second to allow graceful shutdown
        try? await Task.sleep(nanoseconds: 1_000_000_000)
        
        // Ensure process group is completely dead
        forceKill(pgid)
    }
}
