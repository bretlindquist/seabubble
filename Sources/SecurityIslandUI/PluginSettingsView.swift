import SwiftUI
import SecurityIslandCore

public struct MockPlugin: Identifiable {
    public let id: String
    public let name: String
    public let description: String
    public var isEnabled: Bool
}

public struct PluginSettingsView: View {
    @EnvironmentObject var daemonClient: DaemonControlClient
    
    @State private var plugins: [MockPlugin] = [
        MockPlugin(id: "com.seabubble.telegram", name: "Telegram Adapter", description: "Forward alerts to Telegram", isEnabled: true),
        MockPlugin(id: "com.seabubble.llm", name: "LLM Judge", description: "Use LLM to judge incidents automatically", isEnabled: false),
        MockPlugin(id: "com.seabubble.slack", name: "Slack Integration", description: "Forward alerts to Slack channels", isEnabled: false)
    ]
    
    public init() {}
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack {
                Text("Plugin Settings")
                    .font(.title)
                    .fontWeight(.bold)
                Spacer()
                Button(action: {
                    daemonClient.sendReloadPlugins()
                }) {
                    Label("Reload Plugins", systemImage: "arrow.clockwise")
                }
                .buttonStyle(.borderedProminent)
            }
            .padding(.bottom, 10)
            
            Text("Manage your active security plugins. Reloading plugins triggers the daemon to rescan the ~/.seabubble/plugins/ directory.")
                .font(.subheadline)
                .foregroundColor(.secondary)
            
            List {
                ForEach($plugins) { $plugin in
                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(plugin.name)
                                .font(.headline)
                            Text(plugin.description)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        Spacer()
                        Toggle("", isOn: $plugin.isEnabled)
                            .onChange(of: plugin.isEnabled) { newValue in
                                daemonClient.sendTogglePlugin(id: plugin.id, enabled: newValue)
                            }
                    }
                    .padding(.vertical, 4)
                }
            }
            .listStyle(.inset)
            
            Spacer()
        }
        .padding()
        .frame(minWidth: 400, minHeight: 300)
    }
}