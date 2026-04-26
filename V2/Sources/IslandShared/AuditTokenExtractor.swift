import Foundation
import Darwin

/// Extracts un-spoofable process lineage (PID, UID, PGID) from an incoming Unix Domain Socket connection.
/// This prevents malicious agents from spoofing their identity by sending fake credentials in JSON payloads.
public struct AuditTokenExtractor {
    
    /// Extracts the audit_token_t from a connected POSIX socket file descriptor.
    ///
    /// - Parameter fd: The raw file descriptor of the connected socket.
    /// - Returns: The extracted `audit_token_t` or nil if it fails.
    public static func extractToken(from fd: Int32) -> audit_token_t? {
        var token = audit_token_t()
        var len = socklen_t(MemoryLayout<audit_token_t>.size)
        
        // Darwin's getsockopt mechanism to retrieve the peer's audit token
        let result = getsockopt(fd, SOL_LOCAL, LOCAL_PEERTOKEN, &token, &len)
        
        if result == 0 {
            return token
        } else {
            print("⚠️ Failed to extract audit_token_t. Errno: \(errno)")
            return nil
        }
    }
    
    /// Extracts the PID (Process ID) from an audit_token_t.
    public static func getPid(from token: audit_token_t) -> pid_t {
        // audit_token_to_pid is a private API in Darwin, but it's widely used by security tools.
        // We have to extract it manually from the struct array.
        // The audit_token_t struct on macOS contains 8 UInt32 values.
        // Value 5 is the PID.
        return pid_t(token.val.5)
    }
    
    /// Extracts the UID (User ID) from an audit_token_t.
    public static func getUid(from token: audit_token_t) -> uid_t {
        // Value 1 is the Audit UID. Value 2 is the Real UID.
        return uid_t(token.val.2)
    }
    
    /// Extracts the PGID (Process Group ID) from an audit_token_t.
    /// Note: audit_token_t doesn't directly expose pgid in public headers,
    /// so we must retrieve the PID first and then query the kernel for the PGID.
    public static func getPgid(from token: audit_token_t) -> pid_t {
        let pid = getPid(from: token)
        return getpgid(pid)
    }
}
