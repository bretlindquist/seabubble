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
                .frame(minWidth: 250)
        } detail: {
            if let id = selectedIncidentId, let incident = bus.incidents.first(where: { $0.id == id }) {
                ForensicDetailView(incident: incident)
            } else {
                VStack(spacing: 16) {
                    Image(systemName: "shield.lefthalf.filled")
                        .font(.system(size: 64))
                        .foregroundStyle(.secondary)
                    Text("Select an incident to view forensics.")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .status) {
                HStack {
                    Image(systemName: userService.isVisible ? "lock.open.fill" : "lock.fill")
                        .foregroundStyle(userService.isVisible ? .red : .green)
                    Text(userService.displayUsername)
                        .font(.system(.body, design: .monospaced))
                }
                .padding(6)
                .background(Color.gray.opacity(0.2))
                .cornerRadius(6)
                .onTapGesture {
                    withAnimation {
                        userService.toggleVisibility()
                    }
                }
                .help("Click to reveal/hide OS User")
            }
        }
    }
}
