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
}
