import SwiftUI

@main
struct SeabubbleApp: App {
    // For now, the main window can just host the TelegramOnboardingView
    // to prove the UI compiles and runs.
    var body: some Scene {
        WindowGroup {
            TelegramOnboardingView()
                .frame(minWidth: 400, minHeight: 300)
        }
    }
}
