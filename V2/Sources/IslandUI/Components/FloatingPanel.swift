import SwiftUI
import AppKit

/// A custom NSPanel subclass that acts as an ultra-thin, floating material panel.
public class FloatingPanel<Content: View>: NSPanel {
    
    public init(
        contentRect: NSRect,
        backing: NSWindow.BackingStoreType = .buffered,
        defer flag: Bool = false,
        @ViewBuilder rootView: () -> Content
    ) {
        super.init(
            contentRect: contentRect,
            styleMask: [.nonactivatingPanel, .titled, .resizable, .closable, .fullSizeContentView],
            backing: backing,
            defer: flag
        )
        
        // Window configuration for an ultra-thin floating panel
        self.isFloatingPanel = true
        self.level = .floating
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        self.titleVisibility = .hidden
        self.titlebarAppearsTransparent = true
        self.isMovableByWindowBackground = true
        self.isOpaque = false
        self.backgroundColor = .clear
        
        // Setup NSVisualEffectView for behind-window frosted glass blending
        let visualEffectView = NSVisualEffectView(frame: contentRect)
        visualEffectView.material = .popover // Can be adjusted to .hudWindow or .menu based on needs
        visualEffectView.blendingMode = .behindWindow
        visualEffectView.state = .active
        
        visualEffectView.wantsLayer = true
        visualEffectView.layer?.cornerRadius = 16.0
        visualEffectView.layer?.masksToBounds = true
        
        // Host the SwiftUI view
        let hostingController = NSHostingController(rootView: rootView())
        hostingController.view.autoresizingMask = [.width, .height]
        hostingController.view.frame = visualEffectView.bounds
        
        // Ensure the hosting view is transparent so the material shows through
        hostingController.view.wantsLayer = true
        hostingController.view.layer?.backgroundColor = NSColor.clear.cgColor
        
        visualEffectView.addSubview(hostingController.view)
        
        // Set as the panel's content view
        self.contentView = visualEffectView
    }
    
    public override var canBecomeKey: Bool {
        return true
    }
    
    public override var canBecomeMain: Bool {
        return true
    }
}

/// A convenient wrapper to present this panel from within SwiftUI state if needed.
public struct FloatingPanelModifier<PanelContent: View>: ViewModifier {
    @Binding var isPresented: Bool
    var contentRect: NSRect
    let panelContent: () -> PanelContent

    @State private var panel: FloatingPanel<PanelContent>?

    public func body(content: Content) -> some View {
        content
            .onChange(of: isPresented) { newValue in
                if newValue {
                    if panel == nil {
                        panel = FloatingPanel(contentRect: contentRect) {
                            panelContent()
                        }
                    }
                    panel?.makeKeyAndOrderFront(nil)
                } else {
                    panel?.close()
                    panel = nil
                }
            }
    }
}

public extension View {
    /// Modifies a view to present a FloatingPanel.
    func floatingPanel<PanelContent: View>(
        isPresented: Binding<Bool>,
        contentRect: NSRect = NSRect(x: 0, y: 0, width: 300, height: 300),
        @ViewBuilder content: @escaping () -> PanelContent
    ) -> some View {
        self.modifier(FloatingPanelModifier(isPresented: isPresented, contentRect: contentRect, panelContent: content))
    }
}
