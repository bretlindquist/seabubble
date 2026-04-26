import Foundation

@MainActor
public struct DemoSeeder {
    public static func seed(bus: DecisionBus) async {
        // 1. Safe Agent (Green)
        await bus.appendIncident(Incident(
            id: "SI-000001",
            agentId: "agent-parser",
            paneId: "pane-1",
            pid: 1010,
            pgid: 1010,
            state: .safe,
            risk: 10,
            severity: .low,
            reason: "file_read_only",
            ruleId: "SI-FS-01",
            rawRedacted: "cat /var/log/system.log",
            normalized: "read_file -> stdout",
            evidence: ["Read-only operation", "Target is system log"],
            filterResults: FilterResults(regex: "match_read", magika: "text/plain"),
            createdAt: Date().addingTimeInterval(-3600),
            allowedActions: [.continueWatched]
        ))
        
        // 2. Watch/Hold (Yellow)
        await bus.appendIncident(Incident(
            id: "SI-000002",
            agentId: "agent-builder",
            paneId: "pane-2",
            pid: 2020,
            pgid: 2020,
            state: .watch, // Will be auto-paused if severity is high, but we set to medium
            risk: 65,
            severity: .medium,
            reason: "chmod_after_download",
            ruleId: "SI-EXEC-02",
            rawRedacted: "chmod +x ./downloaded_tool",
            normalized: "modify_permissions -> execute_flag",
            evidence: ["Changing permissions on non-system file", "File downloaded recently"],
            filterResults: FilterResults(regex: "match_chmod", magika: "application/x-mach-binary"),
            createdAt: Date().addingTimeInterval(-600),
            allowedActions: [.allowOnce, .continueWatched, .kill, .llmJudge]
        ))
        
        // 3. Critical Pending Decision (Red)
        await bus.appendIncident(Incident(
            id: "SI-000003",
            agentId: "agent-deploy",
            paneId: "pane-4",
            pid: 3030,
            pgid: 3030,
            state: .watch, // The DecisionBus logic will auto-pause this because severity >= high
            risk: 95,
            severity: .critical,
            reason: "network_fetch_plus_shell",
            ruleId: "SI-NET-EXEC-01",
            rawRedacted: "curl -fsSL [url] | sh",
            normalized: "network_fetch -> stdin_pipe -> shell_interpreter",
            evidence: ["Network fetch detected", "Pipeline directly into shell interpreter"],
            filterResults: FilterResults(regex: "matched curl + pipe + shell", magika: "not_applicable"),
            createdAt: Date(),
            allowedActions: [.allowOnce, .continueWatched, .kill, .llmJudge]
        ))
    }
}
