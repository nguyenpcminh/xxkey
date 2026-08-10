import SwiftUI
import Cocoa
import InputMethodKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var eventTap: CFMachPort?
    var runLoopSource: CFRunLoopSource?
    
    // Rust Engine Bridge
    let engine = VietimeEngineBridge()
    var isVietnamese = true
    var inputType: UInt8 = 0
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Run in background (accessory policy hides dock icon)
        NSApp.setActivationPolicy(.accessory)
        
        setupStatusMenu()
        setupEventTap()
        
        // Listen to settings changes
        NotificationCenter.default.addObserver(self, selector: #selector(onSettingsInputTypeChanged(_:)), name: Notification.Name("SettingsInputTypeChanged"), object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(onSettingsModernChanged(_:)), name: Notification.Name("SettingsModernChanged"), object: nil)
    }
    
    func setupStatusMenu() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        updateStatusIcon()
        
        let menu = NSMenu()
        
        let titleItem = NSMenuItem(title: "Bộ gõ XXKey (Rust Core)", action: nil, keyEquivalent: "")
        titleItem.isEnabled = false
        menu.addItem(titleItem)
        menu.addItem(NSMenuItem.separator())
        
        let toggleItem = NSMenuItem(title: "Chế độ: Tiếng Việt", action: #selector(toggleLanguage), keyEquivalent: " ")
        toggleItem.keyEquivalentModifierMask = [.control] // Control + Space global hotkey
        menu.addItem(toggleItem)
        
        menu.addItem(NSMenuItem.separator())
        
        // Input Method Submenu
        let inputTypeItem = NSMenuItem(title: "Kiểu gõ", action: nil, keyEquivalent: "")
        let inputSubmenu = NSMenu()
        
        let telexItem = NSMenuItem(title: "Telex", action: #selector(changeInputType(_:)), keyEquivalent: "")
        telexItem.tag = 0
        telexItem.state = .on
        inputSubmenu.addItem(telexItem)
        
        let vniItem = NSMenuItem(title: "VNI", action: #selector(changeInputType(_:)), keyEquivalent: "")
        vniItem.tag = 1
        inputSubmenu.addItem(vniItem)
        
        let st1Item = NSMenuItem(title: "Simple Telex 1", action: #selector(changeInputType(_:)), keyEquivalent: "")
        st1Item.tag = 2
        inputSubmenu.addItem(st1Item)
        
        let st2Item = NSMenuItem(title: "Simple Telex 2", action: #selector(changeInputType(_:)), keyEquivalent: "")
        st2Item.tag = 3
        inputSubmenu.addItem(st2Item)
        
        inputTypeItem.submenu = inputSubmenu
        menu.addItem(inputTypeItem)
        
        menu.addItem(NSMenuItem.separator())
        
        let resetItem = NSMenuItem(title: "Reset bộ nhớ bộ gõ", action: #selector(resetEngine), keyEquivalent: "")
        menu.addItem(resetItem)
        
        menu.addItem(NSMenuItem.separator())
        
        let quitItem = NSMenuItem(title: "Thoát bộ gõ", action: #selector(quitApp), keyEquivalent: "q")
        menu.addItem(quitItem)
        
        statusItem?.menu = menu
    }
    
    @objc func changeInputType(_ sender: NSMenuItem) {
        AppSettingsState.shared.inputType = sender.tag
    }
    
    @objc func onSettingsInputTypeChanged(_ notification: Notification) {
        if let val = notification.object as? Int {
            let tag = UInt8(val)
            self.inputType = tag
            self.engine.setInputType(tag)
            
            // Update checkmarks in status menu
            if let submenu = statusItem?.menu?.items.first(where: { $0.title == "Kiểu gõ" })?.submenu {
                for item in submenu.items {
                    item.state = (item.tag == val) ? .on : .off
                }
            }
        }
    }

    @objc func onSettingsModernChanged(_ notification: Notification) {
        if let val = notification.object as? Bool {
            self.engine.setModernOrthography(val)
        }
    }
    
    @objc func toggleLanguage() {
        isVietnamese.toggle()
        updateStatusIcon()
        engine.reset()
    }
    
    @objc func resetEngine() {
        engine.reset()
    }
    
    @objc func quitApp() {
        NSApp.terminate(nil)
    }
    
    func updateStatusIcon() {
        if let button = statusItem?.button {
            button.title = isVietnamese ? "🇻🇳 V" : "🇺🇸 E"
            button.font = NSFont.systemFont(ofSize: 13, weight: .semibold)
        }
        if let menu = statusItem?.menu {
            if menu.items.count > 2 {
                menu.items[2].title = isVietnamese ? "Chế độ: Tiếng Việt" : "Chế độ: Tiếng Anh"
            }
        }
    }
    
    func setupEventTap() {
        let eventMask = (1 << CGEventType.keyDown.rawValue) |
                        (1 << CGEventType.keyUp.rawValue) |
                        (1 << CGEventType.flagsChanged.rawValue) |
                        (1 << CGEventType.leftMouseDown.rawValue) |
                        (1 << CGEventType.rightMouseDown.rawValue)
        
        let selfRef = Unmanaged.passUnretained(self).toOpaque()
        
        eventTap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: CGEventMask(eventMask),
            callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
                guard let refcon = refcon else { return Unmanaged.passUnretained(event) }
                let mySelf = Unmanaged<AppDelegate>.fromOpaque(refcon).takeUnretainedValue()
                return mySelf.handleEvent(proxy: proxy, type: type, event: event)
            },
            userInfo: selfRef
        )
        
        if let tap = eventTap {
            runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
            CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
            CGEvent.tapEnable(tap: tap, enable: true)
            print("Event tap successfully installed.")
        } else {
            print("Failed to create event tap. Make sure Accessibility permissions are enabled.")
            showAccessibilityAlert()
        }
    }
    
    func handleEvent(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        if type == .leftMouseDown || type == .rightMouseDown {
            engine.reset()
            return Unmanaged.passUnretained(event)
        }
        
        // 1. Skip events generated by our own app to avoid infinite typing loops
        if event.getIntegerValueField(.eventSourceUserData) == 42 {
            return Unmanaged.passUnretained(event)
        }
        
        let flags = event.flags
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
        
        // 2. Global switch hotkey: Control + Space
        if type == .keyDown {
            let isControl = flags.contains(.maskControl)
            if keyCode == 49 && isControl {
                toggleLanguage()
                return nil // Swallow event
            }
        }
        
        // 3. Skip if English mode is active
        if !isVietnamese {
            return Unmanaged.passUnretained(event)
        }
        
        // 4. Feed events to Rust Core Engine
        if type == .keyDown {
            let isShift = flags.contains(.maskShift)
            let isAlphaShift = flags.contains(.maskAlphaShift)
            let capsStatus: UInt8 = isShift ? 1 : (isAlphaShift ? 2 : 0)
            
            // Check modifier keys (Cmd, Option, Ctrl)
            let otherControlKey = flags.contains(.maskCommand) || flags.contains(.maskAlternate) || flags.contains(.maskControl)
            
            if let result = engine.handleKey(
                event: .keyboard,
                state: .keyDown,
                data: UInt16(keyCode),
                capsStatus: capsStatus,
                otherControlKey: otherControlKey
            ) {
                if result.code == .doNothing {
                    return Unmanaged.passUnretained(event)
                } else if result.code == .willProcess || result.code == .restore || result.code == .restoreAndStartNewSession {
                    // Send backspaces
                    if result.backspaceCount > 0 {
                        for _ in 0..<result.backspaceCount {
                            sendBackspace(proxy: proxy)
                        }
                    }
                    
                    // Send characters from engine
                    var newStr = ""
                    if !result.characters.isEmpty {
                        newStr = String(result.characters)
                    }
                    
                    // If it is a restore event, append the literal key that was pressed (only if it is a mark key)
                    if result.code == .restore {
                        if self.isMarkKey(inputType: self.inputType, code: UInt16(keyCode)) {
                            let chVal = vietime_key_code_to_char(UInt32(keyCode))
                            if chVal != 0, let scalar = UnicodeScalar(chVal) {
                                var literalChar = String(scalar)
                                let isShift = flags.contains(.maskShift)
                                let isAlphaShift = flags.contains(.maskAlphaShift)
                                if isShift || isAlphaShift {
                                    literalChar = literalChar.uppercased()
                                }
                                newStr += literalChar
                            }
                        }
                    }
                    
                    if !newStr.isEmpty {
                        sendUnicodeString(newStr, proxy: proxy)
                    }
                    
                    if result.code == .restoreAndStartNewSession {
                        engine.reset()
                    }
                    
                    return nil // Consume original keypress
                }
            }
        }
        
        return Unmanaged.passUnretained(event)
    }
    
    private func isMarkKey(inputType: UInt8, code: UInt16) -> Bool {
        if inputType == 0 || inputType == 2 || inputType == 3 { // Telex, SimpleTelex1, SimpleTelex2
            return code == 1 || code == 3 || code == 15 || code == 38 || code == 7
        } else if inputType == 1 { // VNI
            return code == 18 || code == 19 || code == 20 || code == 23 || code == 21
        }
        return false
    }
    
    private func sendBackspace(proxy: CGEventTapProxy) {
        let source = CGEventSource(stateID: .privateState)
        
        let eventDown = CGEvent(keyboardEventSource: source, virtualKey: 51, keyDown: true)
        let eventUp = CGEvent(keyboardEventSource: source, virtualKey: 51, keyDown: false)
        
        eventDown?.setIntegerValueField(.eventSourceUserData, value: 42)
        eventUp?.setIntegerValueField(.eventSourceUserData, value: 42)
        
        eventDown?.tapPostEvent(proxy)
        eventUp?.tapPostEvent(proxy)
    }
    
    private func sendUnicodeString(_ string: String, proxy: CGEventTapProxy) {
        let source = CGEventSource(stateID: .privateState)
        let utf16Chars = Array(string.utf16)
        
        let eventDown = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true)
        let eventUp = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false)
        
        eventDown?.keyboardSetUnicodeString(stringLength: utf16Chars.count, unicodeString: utf16Chars)
        eventUp?.keyboardSetUnicodeString(stringLength: utf16Chars.count, unicodeString: utf16Chars)
        
        eventDown?.setIntegerValueField(.eventSourceUserData, value: 42)
        eventUp?.setIntegerValueField(.eventSourceUserData, value: 42)
        
        eventDown?.tapPostEvent(proxy)
        eventUp?.tapPostEvent(proxy)
    }
    
    func showAccessibilityAlert() {
        let alert = NSAlert()
        alert.messageText = "Yêu cầu quyền trợ năng (Accessibility)"
        alert.informativeText = "Để gõ tiếng Việt toàn hệ thống, vui lòng mở:\nSystem Settings -> Privacy & Security -> Accessibility\nvà tích chọn cho phép ứng dụng này."
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Mở Cài đặt Trợ năng")
        alert.addButton(withTitle: "Bỏ qua")
        
        let res = alert.runModal()
        if res == .alertFirstButtonReturn {
            if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                NSWorkspace.shared.open(url)
            }
        }
    }
}

class AppSettingsState: ObservableObject {
    static let shared = AppSettingsState()
    
    @Published var inputType: Int = 0 {
        didSet {
            NotificationCenter.default.post(name: Notification.Name("SettingsInputTypeChanged"), object: inputType)
        }
    }
    
    @Published var modernOrthography: Bool = true {
        didSet {
            NotificationCenter.default.post(name: Notification.Name("SettingsModernChanged"), object: modernOrthography)
        }
    }
}

struct SettingsView: View {
    @ObservedObject var state = AppSettingsState.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            Text("Cấu hình Bộ Gõ XXKey")
                .font(.title2)
                .fontWeight(.bold)
            
            Picker("Kiểu gõ:", selection: $state.inputType) {
                Text("Telex").tag(0)
                Text("VNI").tag(1)
                Text("Simple Telex 1").tag(2)
                Text("Simple Telex 2").tag(3)
            }
            .pickerStyle(.radioGroup)
            
            Toggle("Sử dụng chính tả hiện đại (Modern Orthography)", isOn: $state.modernOrthography)
            
            Divider()
            
            HStack {
                Button("Kiểm tra quyền Trợ năng (Accessibility)") {
                    let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true]
                    AXIsProcessTrustedWithOptions(options as CFDictionary)
                }
                .buttonStyle(.borderedProminent)
                
                Spacer()
                
                Text("Rust Core v1.0")
                    .font(.footnote)
                    .foregroundColor(.secondary)
            }
        }
        .frame(width: 400, height: 260)
        .padding()
    }
}

@main
struct XXKeyApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var body: some Scene {
        Settings {
            SettingsView()
        }
    }
}
