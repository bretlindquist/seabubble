import SwiftUI
import SecurityIslandCore

public struct MainDashboardView: View {
    @EnvironmentObject var bus: DecisionBus
    @EnvironmentObject var userService: SystemUserService
    @State private var selectedIncidentId: String?
    
    public init() {}
    
    public var body: some View {
        NavigationSplitView {
            SidebarView(selectedIncidentId: $selectedIncidentId)
                .frame(minWidth: 280)
        } detail: {
            if let id = selectedIncidentId, let incident = bus.incidents.first(where: { $0.id == id }) {
                ForensicDetailView(incident: incident)
            } else {
                VStack(spacing: 24) {
                    Image(systemName: "shield.lefthalf.filled")
                        .font(.system(size: 80, weight: .light))
                        .foregroundStyle(Color.accentColor.opacity(0.8))
                    
                    VStack(spacing: 8) {
                        Text("Security Island")
                            .font(.title.bold())
                        Text("Waiting for cmux telemetry...")
                            .font(.body)
                            .foregroundStyle(.secondary)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color(NSColor.underPageBackgroundColor))
            }
        }
        .toolbar {
            ToolbarItem(placement: .status) {
                HStack {
                    Image(systemName: userService.isVisible ? "lock.open.fill" : "lock.fill")
                        .foregroundStyle(userService.isVisible ? .red : .green)
                        .imageScale(.small)
                    Text(userService.displayUsername)
                        .font(.system(.subheadline, design: .monospaced))
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(Color(NSColor.controlBackgroundColor))
                .clipShape(Capsule())
                .onTapGesture {
                    withAnimation(.snappy) {
                        userService.toggleVisibility()
                    }
                }
                .help("Click to reveal/hide Host OS Identity")
            }
        }
    }
}
