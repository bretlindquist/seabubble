import Foundation

public final class TelegramAdapter: SecurityPlugin {
    public let id = "telegram-v1"
    public let name = "Telegram Bot Notifier"
    public let version = "1.0.0"
    public var status: PluginStatus = .active
    
    private let token: String
    private let chatId: String
    private var bus: DecisionBus?
    
    public init(token: String, chatId: String) {
        self.token = token
        self.chatId = chatId
    }
    
    @MainActor
    public func bind(bus: DecisionBus) {
        self.bus = bus
        print("[\(name)] Bound to DecisionBus. Ready to poll for Chat ID: \(chatId).")
    }
    
    public func evaluate(incident: Incident) async throws -> PluginEvaluation {
        // We do not block the bus to send the notification.
        Task {
            await sendAlert(for: incident)
        }
        // Telegram adapter is an observer/notifier, it doesn't modify risk natively.
        return PluginEvaluation()
    }
    
    private func sendAlert(for incident: Incident) async {
        let message = """
        🏝 SECURITY ISLAND HOLD 🏝
        Agent: \(incident.actor.agentId)
        Risk: \(incident.risk)
        Severity: \(incident.severity.rawValue.uppercased())
        Reason: \(incident.reason)
        Capability: \(incident.request.capability)
        Payload: \(incident.request.payload)
        
        Decision required: [Allow] [Watch] [Kill]
        """
        
        // For the Hackathon MVP, we stub the actual URLSession POST
        // to prevent network timeouts if the env token is missing.
        print("\n==================================")
        print("[\(name)] 📡 Sending via Telegram API:")
        print("Endpoint: https://api.telegram.org/bot<REDACTED>/sendMessage")
        print("Payload:\n\(message)")
        print("==================================\n")
    }
}
