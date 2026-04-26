import SwiftUI
import SecurityIslandCore
import SecurityIslandUI

@main
struct SecurityIslandApp: App {
    @StateObject private var bus = DecisionBus()
    @StateObject private var userService = SystemUserService()

    var body: some Scene {
        WindowGroup("Security Island") {
            MainDashboardView()
                .environmentObject(bus)
                .environmentObject(userService)
                .onAppear {
                    setupSwarmPlugins()
                    Task {
                        // Seed mock data for hackathon demo
                        await DemoSeeder.seed(bus: bus)
                    }
                }
        }
        .commands {
            SidebarCommands()
        }
    }
    
    @MainActor
    private func setupSwarmPlugins() {
        // 1. Magika Deep Scanner
        let magika = MagikaScanner()
        bus.register(plugin: magika)
        
        // 2. Telegram Two-Way Adapter
        let token = ProcessInfo.processInfo.environment["SECURITY_ISLAND_TELEGRAM_BOT_TOKEN"] ?? "stub_token"
        let chatId = ProcessInfo.processInfo.environment["SECURITY_ISLAND_TELEGRAM_ALLOWED_CHAT_IDS"] ?? "stub_chat_id"
        
        let telegram = TelegramAdapter(token: token, chatId: chatId)
        telegram.bind(bus: bus)
        bus.register(plugin: telegram)
    }
}
