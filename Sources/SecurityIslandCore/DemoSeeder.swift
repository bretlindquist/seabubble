import Foundation

@MainActor
public struct DemoSeeder {
    public static func seed(bus: DecisionBus) async {
        
        // Let's also mock a FocusEvent landing 2 seconds after boot
        Task {
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            await MainActor.run {
                bus.activeSurfaceId = "surface:4"
            }
        }
        
        // 1. Safe Agent (Green)
        await bus.appendIncident(Incident(
            id: "SI-000001",
            actor: ActorContext(uid: 501, process: "codex", agentId: "agent-parser"),
            cmux: CmuxContext(workspaceId: "workspace:1", surfaceId: "surface:1", socketPath: "/tmp/cmux.sock"),
            request: CapabilityRequest(capability: "terminal.send_text", payload: "cat /var/log/system.log", cwd: "/var/log"),
            pid: 1010,
            pgid: 1010,
            state: .safe,
            risk: 10,
            severity: .low,
            reason: "file_read_only",
            ruleId: "SI-FS-01",
            evidence: ["Read-only operation", "Target is system log"],
            filterResults: FilterResults(regex: "match_read", magika: "text/plain"),
            createdAt: Date().addingTimeInterval(-3600),
            allowedActions: [.continueWatched]
        ))
        
        // 2. Watch/Hold (Yellow)
        await bus.appendIncident(Incident(
            id: "SI-000002",
            actor: ActorContext(uid: 501, process: "claude-code", agentId: "agent-builder"),
            cmux: CmuxContext(workspaceId: "workspace:2", surfaceId: "surface:2", socketPath: "/tmp/cmux.sock"),
            request: CapabilityRequest(capability: "terminal.send_text", payload: "chmod +x ./downloaded_tool", cwd: "/Users/dev/tmp"),
            pid: 2020,
            pgid: 2020,
            state: .watch, 
            risk: 65,
            severity: .medium,
            reason: "chmod_after_download",
            ruleId: "SI-EXEC-02",
            evidence: ["Changing permissions on non-system file", "File downloaded recently"],
            filterResults: FilterResults(regex: "match_chmod", magika: "application/x-mach-binary"),
            createdAt: Date().addingTimeInterval(-600),
            allowedActions: [.allowOnce, .continueWatched, .kill, .llmJudge]
        ))
        
        // 3. Critical Pending Decision (Red - Browser Automation)
        await bus.appendIncident(Incident(
            id: "SI-000003",
            actor: ActorContext(uid: 501, process: "browser-use", agentId: "agent-deploy"),
            cmux: CmuxContext(workspaceId: "workspace:3", surfaceId: "surface:4", socketPath: "/tmp/cmux.sock"),
            request: CapabilityRequest(capability: "browser.eval", payload: "document.cookie", cwd: "https://internal.admin.panel"),
            pid: 3030,
            pgid: 3030,
            state: .watch, // Bus will auto-pause
            risk: 98,
            severity: .critical,
            reason: "browser_cookie_exfiltration",
            ruleId: "SI-BROWSER-01",
            evidence: ["Browser JS execution detected", "Attempting to access document.cookie on internal domain"],
            filterResults: FilterResults(regex: "matched document.cookie", magika: "not_applicable"),
            createdAt: Date(),
            allowedActions: [.allowOnce, .continueWatched, .kill, .llmJudge]
        ))
    }
}
