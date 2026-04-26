import SwiftUI
import AppKit
import ServiceManagement
import SecurityIslandCore
import SecurityIslandUI

@main
struct SecurityIslandApp: App {
    @StateObject private var bus: DecisionBus
    @StateObject private var userService = SystemUserService()
    @StateObject private var daemonClient: DaemonControlClient
    @StateObject private var islandController: FloatingIslandPanelController
    @State private var didStart = false

    private let demoMode: Bool
    private let presentationMode: PresentationMode

    init() {
        let busInstance = DecisionBus()
        let daemonClient = DaemonControlClient(bus: busInstance)
        let presentationMode = PresentationMode.current()
        _bus = StateObject(wrappedValue: busInstance)
        _daemonClient = StateObject(wrappedValue: daemonClient)
        _islandController = StateObject(wrappedValue: FloatingIslandPanelController(bus: busInstance, daemonClient: daemonClient))
        self.demoMode = ProcessInfo.processInfo.environment["SECURITY_ISLAND_DEMO"] == "1"
        self.presentationMode = presentationMode

        if #available(macOS 13.0, *) {
            do {
                let service = SMAppService.agent(plistName: "com.seabubble.daemon.plist")
                if service.status == .notRegistered {
                    try service.register()
                    print("Successfully registered SMAppService daemon.")
                } else {
                    print("SMAppService daemon already registered or requires approval. Status: \(service.status)")
                }
            } catch {
                print("Failed to register SMAppService daemon: \(error)")
            }
        }
    }

    var body: some Scene {
        WindowGroup("Security Island") {
            Group {
                if presentationMode == .floatingIsland {
                    Color.clear
                        .frame(width: 1, height: 1)
                        .onAppear {
                            islandController.show()
                            DispatchQueue.main.async {
                                for window in NSApp.windows where window !== islandController.panel {
                                    window.orderOut(nil)
                                }
                            }
                        }
                } else {
                    MainDashboardView()
                        .environmentObject(bus)
                        .environmentObject(userService)
                        .environmentObject(daemonClient)
                }
            }
            .environmentObject(bus)
            .environmentObject(userService)
            .environmentObject(daemonClient)
            .onAppear {
                guard !didStart else { return }
                didStart = true
                setupSwarmPlugins()
                daemonClient.start()

                if demoMode {
                    Task {
                        await DemoSeeder.seed(bus: bus)
                    }
                }
            }
        }
        .commands {
            SidebarCommands()
        }
    }
    
    @MainActor
    private func setupSwarmPlugins() {
        guard let token = ProcessInfo.processInfo.environment["SECURITY_ISLAND_TELEGRAM_BOT_TOKEN"]?.trimmingCharacters(in: .whitespacesAndNewlines),
              let chatId = ProcessInfo.processInfo.environment["SECURITY_ISLAND_TELEGRAM_ALLOWED_CHAT_IDS"]?.trimmingCharacters(in: .whitespacesAndNewlines),
              !token.isEmpty,
              !chatId.isEmpty,
              token != "stub_token",
              chatId != "stub_chat_id" else {
            return
        }

        let telegram = TelegramAdapter(token: token, chatId: chatId)
        telegram.bind(bus: bus)
        bus.register(plugin: telegram)
    }
}
