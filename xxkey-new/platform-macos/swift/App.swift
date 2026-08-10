import SwiftUI

@main
struct VietimeApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
        .windowStyle(.hiddenTitleBar)
    }
}

struct ContentView: View {
    @State private var engine = VietimeEngineBridge()
    @State private var textInput = ""
    @State private var typedBuffer = ""
    @State private var history: [String] = []

    var body: some View {
        VStack(spacing: 20) {
            // Header
            VStack(spacing: 5) {
                Text("Vietime Input Engine")
                    .font(.system(.title, design: .rounded))
                    .fontWeight(.bold)
                    .foregroundColor(.primary)
                Text("Rust core powered by SwiftUI GUI")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            .padding(.top, 10)

            // Input Section
            VStack(alignment: .leading, spacing: 8) {
                Text("Type here to test Telex processing:")
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundColor(.secondary)

                TextField("Type key sequences...", text: $textInput)
                    .textFieldStyle(.plain)
                    .padding(12)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(8)
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color.secondary.opacity(0.2), lineWidth: 1)
                    )
                    .onChange(of: textInput) { newValue in
                        processInput(newValue)
                    }
            }

            // Status Monitor
            HStack(spacing: 40) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("TYPED BUFFER")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(.secondary)
                    Text(typedBuffer.isEmpty ? "(Empty)" : typedBuffer)
                        .font(.system(.title3, design: .monospaced))
                        .foregroundColor(typedBuffer.isEmpty ? .secondary : .accentColor)
                }

                Spacer()
            }
            .padding()
            .background(Color.primary.opacity(0.03))
            .cornerRadius(8)

            // Action Row
            HStack {
                Button(action: {
                    engine.reset()
                    textInput = ""
                    typedBuffer = ""
                    history.removeAll()
                }) {
                    Text("Clear Engine State")
                        .fontWeight(.semibold)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
            }
        }
        .padding(30)
        .frame(width: 480, height: 350)
    }

    private func processInput(_ value: String) {
        // If empty, reset engine
        if value.isEmpty {
            engine.reset()
            typedBuffer = ""
            return
        }
        
        // Feed only the last character for demo purposes
        guard let lastChar = value.last else { return }
        let code = macOSKeyCode(for: lastChar)
        
        if let result = engine.handleKey(
            event: .keyboard,
            state: .keyDown,
            data: code,
            capsStatus: 0,
            otherControlKey: false
        ) {
            // Apply deletions and insertions to typedBuffer
            let bpc = result.backspaceCount
            if bpc > 0 && bpc <= typedBuffer.count {
                typedBuffer.removeLast(bpc)
            }
            for ch in result.characters {
                typedBuffer.append(ch)
            }
            if result.code == .doNothing && result.extCode == .wordBreak {
                // Word break, start new session
                typedBuffer.append(" ")
            }
        }
    }

    private func macOSKeyCode(for char: Character) -> UInt16 {
        // Simple mapping table mirroring macOS platforms/mac.h
        switch char {
        case "a": return 0
        case "s": return 1
        case "d": return 2
        case "f": return 3
        case "h": return 4
        case "g": return 5
        case "z": return 6
        case "x": return 7
        case "c": return 8
        case "v": return 9
        case "b": return 11
        case "q": return 12
        case "w": return 13
        case "e": return 14
        case "r": return 15
        case "y": return 16
        case "t": return 17
        case "o": return 31
        case "u": return 32
        case "i": return 34
        case "p": return 35
        case "l": return 37
        case "j": return 38
        case "k": return 40
        case "n": return 45
        case "m": return 46
        case " ": return 49
        default: return 256
        }
    }
}
