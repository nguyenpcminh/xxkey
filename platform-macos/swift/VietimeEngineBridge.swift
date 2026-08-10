import Foundation

public enum KeyEvent: UInt8 {
    case keyboard = 0
    case mouse = 1
}

public enum KeyEventState: UInt8 {
    case keyUp = 1
    case mouseDown = 2
    case mouseUp = 3
    case keyDown = 0
}

public enum HookCode: UInt8 {
    case doNothing = 0
    case willProcess = 1
    case breakWord = 2
    case restore = 3
    case replaceMacro = 4
    case restoreAndStartNewSession = 5
}

public enum ExtCode: UInt8 {
    case wordBreak = 1
    case delete = 2
    case normalKey = 3
    case noEmptyChar = 4
}

public struct HookResult {
    public let code: HookCode
    public let backspaceCount: Int
    public let newCharCount: Int
    public let extCode: ExtCode
    public let characters: [Character]
}

public class VietimeEngineBridge {
    private var enginePtr: OpaquePointer?

    public init() {
        self.enginePtr = vietime_new_engine()
    }

    deinit {
        if let ptr = enginePtr {
            vietime_free_engine(ptr)
        }
    }

    public func reset() {
        if let ptr = enginePtr {
            vietime_reset_engine(ptr)
        }
    }

    public func startNewSession() {
        if let ptr = enginePtr {
            vietime_start_new_session(ptr)
        }
    }

    public func setInputType(_ inputType: UInt8) {
        if let ptr = enginePtr {
            vietime_set_input_type(ptr, inputType)
        }
    }

    public func setModernOrthography(_ modern: Bool) {
        if let ptr = enginePtr {
            vietime_set_modern_orthography(ptr, modern ? 1 : 0)
        }
    }

    public func handleKey(
        event: KeyEvent,
        state: KeyEventState,
        data: UInt16,
        capsStatus: UInt8,
        otherControlKey: Bool
    ) -> HookResult? {
        guard let ptr = enginePtr else { return nil }
        
        let statePtr = vietime_handle_key(
            ptr,
            event.rawValue,
            state.rawValue,
            data,
            capsStatus,
            otherControlKey
        )
        
        guard let sPtr = statePtr else { return nil }
        
        let codeRaw = vietime_get_hook_state_code(sPtr)
        let backspaceCount = Int(vietime_get_hook_state_backspace_count(sPtr))
        let newCharCount = Int(vietime_get_hook_state_new_char_count(sPtr))
        let extCodeRaw = vietime_get_hook_state_ext_code(sPtr)
        
        var chars: [Character] = []
        if newCharCount > 0 {
            for i in (0..<newCharCount).reversed() {
                let val = vietime_get_hook_state_char_at(sPtr, UInt32(i))
                let chCode = (val & 0x200_0000 != 0) ? (val & 0xFFFF) : UInt32(vietime_key_code_to_char(val))
                if let unicodeChar = UnicodeScalar(chCode) {
                    chars.append(Character(unicodeChar))
                }
            }
        }
        
        return HookResult(
            code: HookCode(rawValue: codeRaw) ?? .doNothing,
            backspaceCount: backspaceCount,
            newCharCount: newCharCount,
            extCode: ExtCode(rawValue: extCodeRaw) ?? .normalKey,
            characters: chars
        )
    }
}
