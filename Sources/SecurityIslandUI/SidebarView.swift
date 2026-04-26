import SwiftUI
import SecurityIslandCore

public struct SidebarView: View {
    @EnvironmentObject var bus: DecisionBus
    @Binding var selectedIncidentId: String?
    
    public init(selectedIncidentId: Binding<String?>) {
        self._selectedIncidentId = selectedIncidentId
    }
    
    public var body: some View {
        ScrollViewReader { proxy in
            List(selection: $selectedIncidentId) {
                Section("Settings") {
                    NavigationLink(value: "__plugin_settings__") {
                        Label("Plugin Settings", systemImage: "puzzlepiece.extension")
                    }
                }
                
                Section("Monitored Agents") {
                    ForEach(bus.incidents) { incident in
                        NavigationLink(value: incident.id) {
                            HStack {
                                Circle()
                                    .fill(color(for: incident.state, severity: incident.severity))
                                    .frame(width: 10, height: 10)
                                
                                VStack(alignment: .leading) {
                                    Text(incident.actor.agentId)
                                        .font(.headline)
                                    Text(incident.state.rawValue.replacingOccurrences(of: "_", with: " ").capitalized)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                
                                Spacer()
                                
                                if bus.activeSurfaceId == incident.cmux.surfaceId {
                                    Image(systemName: "bolt.fill")
                                        .foregroundStyle(.blue)
                                        .font(.caption)
                                        .help("Active cmux surface")
                                }
                                
                                Text("Risk: \(incident.risk)")
                                    .font(.caption.monospacedDigit())
                                    .foregroundStyle(incident.risk > 80 ? .red : .secondary)
                            }
                        }
                        .id(incident.id)
                    }
                }
            }
            .navigationTitle("Security Island")
            .onChange(of: bus.activeSurfaceId) { newSurfaceId in
                // When cmux changes focus, auto-select the incident for that pane if it requires attention
                if let surfaceId = newSurfaceId,
                   let matchingIncident = bus.incidents.first(where: { $0.cmux.surfaceId == surfaceId }) {
                    withAnimation {
                        selectedIncidentId = matchingIncident.id
                        proxy.scrollTo(matchingIncident.id, anchor: .center)
                    }
                }
            }
        }
    }
    
    private func color(for state: IncidentState, severity: Severity) -> Color {
        switch state {
        case .killed: return .gray
        case .resolvedAllowed, .safe: return .green
        case .queuedForLLM: return .purple
        case .pendingDecision:
            return severity >= .high ? .red : .yellow
        default:
            return severity >= .high ? .red : (severity == .medium ? .yellow : .green)
        }
    }
}
