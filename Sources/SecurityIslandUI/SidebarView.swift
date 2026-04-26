import SwiftUI
import SecurityIslandCore

public struct SidebarView: View {
    @EnvironmentObject var bus: DecisionBus
    @Binding var selectedIncidentId: String?
    
    public init(selectedIncidentId: Binding<String?>) {
        self._selectedIncidentId = selectedIncidentId
    }
    
    public var body: some View {
        List(selection: $selectedIncidentId) {
            Section("Monitored Agents") {
                ForEach(bus.incidents) { incident in
                    NavigationLink(value: incident.id) {
                        HStack {
                            Circle()
                                .fill(color(for: incident.state, severity: incident.severity))
                                .frame(width: 10, height: 10)
                            
                            VStack(alignment: .leading) {
                                Text(incident.agentId)
                                    .font(.headline)
                                Text(incident.state.rawValue.replacingOccurrences(of: "_", with: " ").capitalized)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            
                            Spacer()
                            
                            Text("Risk: \(incident.risk)")
                                .font(.caption.monospacedDigit())
                                .foregroundStyle(incident.risk > 80 ? .red : .secondary)
                        }
                    }
                }
            }
        }
        .navigationTitle("Security Island")
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
