import SwiftUI

public enum TelegramOnboardingState {
    case intro
    case createBot
    case tokenInput
    case verifying
    case success
    case error(String)
}

public struct TelegramOnboardingView: View {
    @State private var currentState: TelegramOnboardingState = .intro
    @State private var botToken: String = ""
    
    public init() {}
    
    public var body: some View {
        VStack {
            Spacer()
            
            switch currentState {
            case .intro:
                introView
            case .createBot:
                createBotView
            case .tokenInput:
                tokenInputView
            case .verifying:
                verifyingView
            case .success:
                successView
            case .error(let message):
                errorView(message: message)
            }
            
            Spacer()
        }
        .padding(30)
        .frame(width: 480, height: 380)
    }
    
    private var introView: some View {
        VStack(spacing: 16) {
            Image(systemName: "paperplane.fill")
                .resizable()
                .scaledToFit()
                .frame(width: 64, height: 64)
                .foregroundColor(.blue)
            
            Text("Connect Telegram")
                .font(.title)
                .fontWeight(.semibold)
            
            Text("Seabubble can integrate with Telegram to send you notifications and allow remote control of your island.")
                .multilineTextAlignment(.center)
                .foregroundColor(.secondary)
            
            Button("Get Started") {
                withAnimation { currentState = .createBot }
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .padding(.top, 10)
        }
    }
    
    private var createBotView: some View {
        VStack(spacing: 16) {
            Text("Create Your Bot")
                .font(.title2)
                .fontWeight(.semibold)
            
            VStack(alignment: .leading, spacing: 12) {
                stepRow(number: 1, text: "Open Telegram and search for @BotFather")
                stepRow(number: 2, text: "Send the message /newbot")
                stepRow(number: 3, text: "Follow the prompts to name your bot")
                stepRow(number: 4, text: "Copy the HTTP API token provided at the end")
            }
            .padding()
            .background(Color(nsColor: .controlBackgroundColor))
            .cornerRadius(8)
            
            HStack {
                Button("Open Telegram") {
                    if let url = URL(string: "tg://resolve?domain=BotFather") {
                        NSWorkspace.shared.open(url)
                    }
                }
                
                Spacer()
                
                Button("Back") {
                    withAnimation { currentState = .intro }
                }
                
                Button("Next") {
                    withAnimation { currentState = .tokenInput }
                }
                .buttonStyle(.borderedProminent)
            }
            .padding(.top, 10)
        }
    }
    
    private func stepRow(number: Int, text: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
            Text("\(number).")
                .fontWeight(.bold)
                .foregroundColor(.secondary)
            Text(text)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
    
    private var tokenInputView: some View {
        VStack(spacing: 16) {
            Text("Enter Bot Token")
                .font(.title2)
                .fontWeight(.semibold)
            
            Text("Paste the HTTP API token you received from BotFather.")
                .multilineTextAlignment(.center)
                .foregroundColor(.secondary)
            
            SecureField("1234567890:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsaw", text: $botToken)
                .textFieldStyle(.roundedBorder)
                .controlSize(.large)
                .padding(.vertical, 10)
            
            HStack {
                Button("Back") {
                    withAnimation { currentState = .createBot }
                }
                
                Spacer()
                
                Button("Connect") {
                    verifyToken()
                }
                .buttonStyle(.borderedProminent)
                .disabled(botToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
    }
    
    private var verifyingView: some View {
        VStack(spacing: 20) {
            ProgressView()
                .controlSize(.large)
            
            Text("Verifying token and connecting...")
                .foregroundColor(.secondary)
        }
    }
    
    private var successView: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .resizable()
                .scaledToFit()
                .frame(width: 64, height: 64)
                .foregroundColor(.green)
            
            Text("Connected Successfully")
                .font(.title)
                .fontWeight(.semibold)
            
            Text("Your Telegram bot is now linked to Seabubble.")
                .multilineTextAlignment(.center)
                .foregroundColor(.secondary)
            
            Button("Done") {
                // Action to close or proceed
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .padding(.top, 10)
        }
    }
    
    private func errorView(message: String) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle.fill")
                .resizable()
                .scaledToFit()
                .frame(width: 64, height: 64)
                .foregroundColor(.red)
            
            Text("Connection Failed")
                .font(.title2)
                .fontWeight(.semibold)
            
            Text(botToken.isEmpty ? "Unknown error." : "Could not verify the token.")
                .multilineTextAlignment(.center)
                .foregroundColor(.secondary)
            
            HStack {
                Button("Start Over") {
                    botToken = ""
                    withAnimation { currentState = .intro }
                }
                
                Button("Try Again") {
                    withAnimation { currentState = .tokenInput }
                }
                .buttonStyle(.borderedProminent)
            }
            .padding(.top, 10)
        }
    }
    
    private func verifyToken() {
        withAnimation { currentState = .verifying }
        
        // Mock verification delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
            if botToken.count > 20 {
                withAnimation { currentState = .success }
            } else {
                withAnimation { currentState = .error("Invalid token format.") }
            }
        }
    }
}

#Preview {
    TelegramOnboardingView()
}