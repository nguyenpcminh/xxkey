import Cocoa
import InputMethodKit
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var eventTap: CFMachPort?
    var runLoopSource: CFRunLoopSource?

    // Rust Engine Bridge
    let engine = VietimeEngineBridge()
    var isVietnamese = true
    var inputType: UInt8 = 0

    private func isGhostAutocompleteApp(_ bundleId: String) -> Bool {
        let b = bundleId.lowercased()

        return b.contains("chrome") || b.contains("safari") || b.contains("firefox")
            || b.contains("edge") || b.contains("brave") || b.contains("opera")
            || b.contains("coccoc") || b.contains("thebrowser")  // Arc
            || b.contains("orion") || b.contains("vivaldi") || b == "com.apple.finder"
            || b.contains("com.apple.spotlight") || b.contains("searchui")
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)

        // Important: initialize Rust engine config before installing event tap.
        self.inputType = UInt8(AppSettingsState.shared.inputType)
        self.engine.setInputType(self.inputType)
        self.engine.setModernOrthography(AppSettingsState.shared.modernOrthography)

        setupStatusMenu()
        setupEventTap()

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(onSettingsInputTypeChanged(_:)),
            name: Notification.Name("SettingsInputTypeChanged"),
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(onSettingsModernChanged(_:)),
            name: Notification.Name("SettingsModernChanged"),
            object: nil
        )
    }

    func setupStatusMenu() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        updateStatusIcon()

        let menu = NSMenu()

        let titleItem = NSMenuItem(
            title: "Bộ gõ XXKey (Rust Core)",
            action: nil,
            keyEquivalent: ""
        )
        titleItem.isEnabled = false
        menu.addItem(titleItem)
        menu.addItem(NSMenuItem.separator())

        let toggleItem = NSMenuItem(
            title: "Chế độ: Tiếng Việt",
            action: #selector(toggleLanguage),
            keyEquivalent: " "
        )
        toggleItem.keyEquivalentModifierMask = [.control]
        menu.addItem(toggleItem)

        menu.addItem(NSMenuItem.separator())

        let inputTypeItem = NSMenuItem(title: "Kiểu gõ", action: nil, keyEquivalent: "")
        let inputSubmenu = NSMenu()

        let telexItem = NSMenuItem(
            title: "Telex", action: #selector(changeInputType(_:)), keyEquivalent: "")
        telexItem.tag = 0
        telexItem.state = AppSettingsState.shared.inputType == 0 ? .on : .off
        inputSubmenu.addItem(telexItem)

        let vniItem = NSMenuItem(
            title: "VNI", action: #selector(changeInputType(_:)), keyEquivalent: "")
        vniItem.tag = 1
        vniItem.state = AppSettingsState.shared.inputType == 1 ? .on : .off
        inputSubmenu.addItem(vniItem)

        let st1Item = NSMenuItem(
            title: "Simple Telex 1", action: #selector(changeInputType(_:)), keyEquivalent: "")
        st1Item.tag = 2
        st1Item.state = AppSettingsState.shared.inputType == 2 ? .on : .off
        inputSubmenu.addItem(st1Item)

        let st2Item = NSMenuItem(
            title: "Simple Telex 2", action: #selector(changeInputType(_:)), keyEquivalent: "")
        st2Item.tag = 3
        st2Item.state = AppSettingsState.shared.inputType == 3 ? .on : .off
        inputSubmenu.addItem(st2Item)

        inputTypeItem.submenu = inputSubmenu
        menu.addItem(inputTypeItem)

        menu.addItem(NSMenuItem.separator())

        let resetItem = NSMenuItem(
            title: "Reset bộ nhớ bộ gõ",
            action: #selector(resetEngine),
            keyEquivalent: ""
        )
        menu.addItem(resetItem)

        menu.addItem(NSMenuItem.separator())

        let quitItem = NSMenuItem(
            title: "Thoát bộ gõ",
            action: #selector(quitApp),
            keyEquivalent: "q"
        )
        menu.addItem(quitItem)

        statusItem?.menu = menu
        updateStatusIcon()
    }

    @objc func changeInputType(_ sender: NSMenuItem) {
        AppSettingsState.shared.inputType = sender.tag
    }

    @objc func onSettingsInputTypeChanged(_ notification: Notification) {
        guard let val = notification.object as? Int else { return }

        let tag = UInt8(val)
        self.inputType = tag
        self.engine.setInputType(tag)
        resetCompositionState(resetEngineCore: true)

        if let submenu = statusItem?.menu?.items.first(where: { $0.title == "Kiểu gõ" })?.submenu {
            for item in submenu.items {
                item.state = item.tag == val ? .on : .off
            }
        }
    }

    @objc func onSettingsModernChanged(_ notification: Notification) {
        guard let val = notification.object as? Bool else { return }

        self.engine.setModernOrthography(val)
        resetCompositionState(resetEngineCore: true)
    }

    @objc func toggleLanguage() {
        isVietnamese.toggle()
        updateStatusIcon()
        resetCompositionState(resetEngineCore: true)
    }

    @objc func resetEngine() {
        resetCompositionState(resetEngineCore: true)
    }

    @objc func quitApp() {
        NSApp.terminate(nil)
    }

    func updateStatusIcon() {
        if let button = statusItem?.button {
            button.title = isVietnamese ? "🇻🇳 V" : "🇺🇸 E"
            button.font = NSFont.systemFont(ofSize: 13, weight: .semibold)
        }

        if let menu = statusItem?.menu, menu.items.count > 2 {
            menu.items[2].title = isVietnamese ? "Chế độ: Tiếng Việt" : "Chế độ: Tiếng Anh"
        }
    }

    func setupEventTap() {
        let eventMask =
            (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue)
            | (1 << CGEventType.flagsChanged.rawValue) | (1 << CGEventType.leftMouseDown.rawValue)
            | (1 << CGEventType.rightMouseDown.rawValue)

        let selfRef = Unmanaged.passUnretained(self).toOpaque()

        eventTap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: CGEventMask(eventMask),
            callback: { proxy, type, event, refcon -> Unmanaged<CGEvent>? in
                guard let refcon = refcon else {
                    return Unmanaged.passUnretained(event)
                }

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

    func handleEvent(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent) -> Unmanaged<
        CGEvent
    >? {
        if type == .leftMouseDown || type == .rightMouseDown {
            engine.reset()
            return Unmanaged.passUnretained(event)
        }

        // Ignore events generated by this app.
        if event.getIntegerValueField(.eventSourceUserData) == 42 {
            return Unmanaged.passUnretained(event)
        }

        let flags = event.flags
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)

        // Control + Space toggles Vietnamese/English.
        if type == .keyDown {
            let isControl = flags.contains(.maskControl)
            if keyCode == 49 && isControl {
                toggleLanguage()
                return nil
            }
        }

        if !isVietnamese {
            return Unmanaged.passUnretained(event)
        }

        guard type == .keyDown else {
            return Unmanaged.passUnretained(event)
        }

        let isShift = flags.contains(.maskShift)
        let isAlphaShift = flags.contains(.maskAlphaShift)
        let capsStatus: UInt8 = isShift ? 1 : (isAlphaShift ? 2 : 0)

        let otherControlKey =
            flags.contains(.maskCommand) || flags.contains(.maskAlternate)
            || flags.contains(.maskControl)

        guard
            let result = engine.handleKey(
                event: .keyboard,
                state: .keyDown,
                data: UInt16(keyCode),
                capsStatus: capsStatus,
                otherControlKey: otherControlKey
            )
        else {
            return Unmanaged.passUnretained(event)
        }

        if result.code == .doNothing {
            switch result.extCode {
            case .wordBreak:
                // DoNothing + WordBreak must start a new session.
                engine.startNewSession()
                return Unmanaged.passUnretained(event)

            case .normalKey:
                // Raw key goes through.
                return Unmanaged.passUnretained(event)

            default:
                // Delete or unknown ext code.
                return Unmanaged.passUnretained(event)
            }
        }

        if result.code == .willProcess || result.code == .restore
            || result.code == .restoreAndStartNewSession
        {

            let bpc = Int(result.backspaceCount)

            // In browsers / Spotlight the omnibox shows a ghost autocomplete
            // suggestion; the first backspace only clears the ghost instead of
            // deleting a real character. Prepend an invisible ZWNJ so that
            // ghost-eaten backspace consumes it and exactly bpc chars are left.
            var zwnjFix = false
            if let frontmostApp = NSWorkspace.shared.frontmostApplication {
                let bundleId = frontmostApp.bundleIdentifier ?? ""
                zwnjFix = self.isGhostAutocompleteApp(bundleId)
            }

            if zwnjFix {
                sendUnicodeString("\u{200C}", proxy: proxy)
            }

            for _ in 0..<(bpc + (zwnjFix ? 1 : 0)) {
                sendBackspace(proxy: proxy)
            }

            var newStr = ""

            if !result.characters.isEmpty {
                newStr = String(result.characters)
            }

            if result.code == .restore {
                if self.isMarkKey(inputType: self.inputType, code: UInt16(keyCode)) {
                    let chVal = vietime_key_code_to_char(UInt32(keyCode))
                    if chVal != 0, let scalar = UnicodeScalar(chVal) {
                        var literalChar = String(scalar)

                        if flags.contains(.maskShift) || flags.contains(.maskAlphaShift) {
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
                engine.startNewSession()
            }

            return nil
        }

        return Unmanaged.passUnretained(event)
    }

    private func resetCompositionState(resetEngineCore: Bool) {
        if resetEngineCore {
            engine.reset()
        }
    }

    private func isMarkKey(inputType: UInt8, code: UInt16) -> Bool {
        if inputType == 0 || inputType == 2 || inputType == 3 {
            // Telex, SimpleTelex1, SimpleTelex2
            return code == 1  // s
                || code == 3  // f
                || code == 15  // r
                || code == 38  // x
                || code == 7  // j
        } else if inputType == 1 {
            // VNI
            return code == 18  // 1
                || code == 19  // 2
                || code == 20  // 3
                || code == 23  // 4
                || code == 21  // 5
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

        eventDown?.keyboardSetUnicodeString(
            stringLength: utf16Chars.count,
            unicodeString: utf16Chars
        )

        eventUp?.keyboardSetUnicodeString(
            stringLength: utf16Chars.count,
            unicodeString: utf16Chars
        )

        eventDown?.setIntegerValueField(.eventSourceUserData, value: 42)
        eventUp?.setIntegerValueField(.eventSourceUserData, value: 42)

        eventDown?.tapPostEvent(proxy)
        eventUp?.tapPostEvent(proxy)
    }

    func showAccessibilityAlert() {
        let alert = NSAlert()
        alert.messageText = "Yêu cầu quyền trợ năng (Accessibility)"
        alert.informativeText = """
            Để gõ tiếng Việt toàn hệ thống, vui lòng mở:
            System Settings -> Privacy & Security -> Accessibility
            và tích chọn cho phép ứng dụng này.
            """
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Mở Cài đặt Trợ năng")
        alert.addButton(withTitle: "Bỏ qua")

        let res = alert.runModal()
        if res == .alertFirstButtonReturn {
            if let url = URL(
                string:
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            {
                NSWorkspace.shared.open(url)
            }
        }
    }
}

class AppSettingsState: ObservableObject {
    static let shared = AppSettingsState()

    @Published var inputType: Int = 0 {
        didSet {
            NotificationCenter.default.post(
                name: Notification.Name("SettingsInputTypeChanged"),
                object: inputType
            )
        }
    }

    @Published var modernOrthography: Bool = true {
        didSet {
            NotificationCenter.default.post(
                name: Notification.Name("SettingsModernChanged"),
                object: modernOrthography
            )
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

            Toggle(
                "Sử dụng chính tả hiện đại (Modern Orthography)",
                isOn: $state.modernOrthography
            )

            Divider()

            HStack {
                Button("Kiểm tra quyền Trợ năng (Accessibility)") {
                    let options = [
                        kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true
                    ]
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
