import SwiftUI
import SecurityIslandCore

public struct ForensicDetailView: View {
    @EnvironmentObject var bus: DecisionBus
    let incident: Incident
    
    public init(incident: Incident) {
        self.incident = incident
    }
    
    public var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                
                // HEADER
                HStack {
                    VStack(alignment: .leading) {
                        Text("Incident: \(incident.id)")
                            .font(.title2.bold())
                        Text("Agent: \(incident.agentId) | Pane: \(incident.paneId)")
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    VStack(alignment: .trailing) {
                        Text(incident.state.rawValue.uppercased())
                            .font(.caption.bold())
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(stateColor.opacity(0.2))
                            .foregroundStyle(stateColor)
                            .cornerRadius(4)
                        
                        Text("PGID: \(incident.pgid)")
                            .font(.caption.monospacedDigit())
                            .foregroundStyle(.secondary)
                    }
                }
                
                Divider()
                
                // FORENSIC DATA
                VStack(alignment: .leading, spacing: 10) {
                    Text("Command Executed").font(.headline)
                    Text(incident.rawRedacted)
                        .font(.system(.body, design: .monospaced))
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.black.opacity(0.8))
                        .foregroundStyle(.green)
                        .cornerRadius(8)
                    
                    Text("Normalized Intent").font(.headline).padding(.top, 10)
                    Text(incident.normalized)
                        .font(.system(.subheadline, design: .monospaced))
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.gray.opacity(0.1))
                        .cornerRadius(8)
                    
                    Text("Evidence").font(.headline).padding(.top, 10)
                    ForEach(incident.evidence, id: \.self) { ev in
                        HStack(alignment: .top) {
                            Image(systemName: "exclamationmark.triangle.fill")
                                .foregroundStyle(.yellow)
                                .font(.caption)
                            Text(ev)
                        }
                    }
                }
                
                Divider()
                
                // DECISION ACTIONS
                if incident.state == .pendingDecision || incident.state == .watch {
                    Text("Human Decision Required").font(.headline)
                    
                    HStack(spacing: 16) {
                        if incident.allowedActions.contains(.allowOnce) {
                            actionButton("Allow Once", icon: "checkmark.seal.fill", color: .green) {
                                apply(.allowOnce)
                            }
                            .keyboardShortcut("a", modifiers: [.command])
                        }
                        
                        if incident.allowedActions.contains(.continueWatched) {
                            actionButton("Continue Watched", icon: "eye.fill", color: .blue) {
                                apply(.continueWatched)
                            }
                            .keyboardShortcut("c", modifiers: [.command])
                        }
                        
                        if incident.allowedActions.contains(.llmJudge) {
                            actionButton("LLM Judge", icon: "brain.head.profile", color: .purple) {
                                apply(.llmJudge)
                            }
                            .keyboardShortcut("l", modifiers: [.command])
                        }
                        
                        Spacer()
                        
                        if incident.allowedActions.contains(.kill) {
                            actionButton("KILL", icon: "xmark.octagon.fill", color: .red) {
                                apply(.kill)
                            }
                            .keyboardShortcut("k", modifiers: [.command])
                        }
                    }
                } else {
                    HStack {
                        Image(systemName: "info.circle.fill")
                        Text("No pending decisions available for this state.")
                    }
                    .foregroundStyle(.secondary)
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(8)
                }
                
            }
            .padding()
        }
    }
    
    private var stateColor: Color {
        if incident.state == .killed { return .gray }
        if incident.state == .safe || incident.state == .resolvedAllowed { return .green }
        return incident.severity >= .high ? .red : .yellow
    }
    
    private func apply(_ action: AllowedAction) {
        bus.applyDecision(incidentId: incident.id, action: action)
    }
    
    private func actionButton(_ title: String, icon: String, color: Color, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack {
                Image(systemName: icon)
                Text(title).bold()
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(color.opacity(0.2))
            .foregroundStyle(color)
            .cornerRadius(8)
        }
        .buttonStyle(.plain)
    }
}
