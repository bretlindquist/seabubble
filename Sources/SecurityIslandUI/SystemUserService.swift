import Foundation
import Combine

/// Service to manage the display and privacy masking of the logged-in OS user.
public final class SystemUserService: ObservableObject {
    @Published public var isVisible: Bool = false
    public let actualUsername: String
    
    public init() {
        self.actualUsername = NSUserName()
    }
    
    public var displayUsername: String {
        if isVisible {
            return actualUsername
        } else {
            // Provide a masked version, matching length if > 4, or default to 5 dots
            let count = max(actualUsername.count, 5)
            return String(repeating: "•", count: count)
        }
    }
    
    public func toggleVisibility() {
        isVisible.toggle()
    }
}
