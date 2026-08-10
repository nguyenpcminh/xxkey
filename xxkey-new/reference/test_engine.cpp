//
//  test_engine.cpp
//  Assert-based self-check for the OpenKey Vietnamese engine.
//
//  Compile (from Sources/OpenKey/engine):
//    g++ -std=c++17 -DLINUX Engine.cpp Vietnamese.cpp Macro.cpp SmartSwitchKey.cpp ConvertTool.cpp test_engine.cpp -o test_engine
//  Run:
//    ./test_engine
//
//  Exits non-zero if any assertion fails.
//
//  This harness simulates how the macOS/Windows front-end consumes the
//  engine: it keeps a "screen buffer" (wstring), and for each key event it
//  applies backspaceCount deletions then appends newCharCount characters in
//  typing order. charData is filled by the engine last-char-first, so the
//  typing order is charData[newCharCount-1] .. charData[0].

#include "Engine.h"
#include <cstdio>
#include <cstring>
#include <string>

using namespace std;

extern vKeyHookState HookState; // defined in Engine.cpp

// --- globals the engine expects ---
int vLanguage = 1;
int vInputType = 0;        // Telex
int vFreeMark = 0;
int vCodeTable = 0;        // Unicode
int vCheckSpelling = 1;
int vUseModernOrthography = 1;
int vQuickTelex = 0;
int vSwitchKeyStatus = 0;
int vRestoreIfWrongSpelling = 0;
int vFixRecommendBrowser = 0;
int vUseMacro = 0;
int vUseMacroInEnglishMode = 0;
int vAutoCapsMacro = 0;
int vUseSmartSwitchKey = 0;
int vUpperCaseFirstChar = 0;
int vTempOffSpelling = 0;
int vAllowConsonantZFWJ = 0;
int vQuickStartConsonant = 0;
int vQuickEndConsonant = 0;
int vRememberCode = 0;
int vOtherLanguage = 0;
int vTempOffOpenKey = 0;

static int _failCount = 0;
static int _checkCount = 0;

// Simulated screen buffer (typing order).
static wstring screen;

static void check(bool cond, const char* what) {
    _checkCount++;
    if (!cond) {
        _failCount++;
        printf("FAIL: %s  (screen=[%ls])\n", what, screen.c_str());
    }
}

// Apply one keydown; simulates the front-end handling of the hook result.
static void typeKey(Uint16 key) {
    vKeyHandleEvent(vKeyEvent::Keyboard, vKeyEventState::KeyDown, key, 0, false);

    if (HookState.code == vDoNothing) {
        // No transformation: the key event is passed through untouched,
        // so the plain character appears on screen.
        wchar_t ch = (wchar_t)keyCodeToCharacter(key);
        if (ch != 0)
            screen.push_back(ch);
        if (HookState.extCode == 1) // word break -> new session
            startNewSession();
        return;
    }

    // vWillProcess / vRestore: backspace the transformed chars...
    if (HookState.backspaceCount > 0) {
        int n = (int)HookState.backspaceCount;
        if (n > (int)screen.size())
            n = (int)screen.size();
        screen.erase(screen.size() - n, n);
    }

    // ...then append the new characters in typing order (reverse of charData).
    for (int i = HookState.newCharCount - 1; i >= 0; i--) {
        Uint32 c = HookState.charData[i];
        wchar_t ch;
        if (c & CHAR_CODE_MASK)
            ch = (wchar_t)(c & 0xFFFF);
        else {
            // raw keycode: map through the keyboard table like the real front-end does
            ch = (wchar_t)keyCodeToCharacter(c);
            if (ch == 0)
                ch = (wchar_t)c;
        }
        if (ch != 0)
            screen.push_back(ch);
    }

    if (HookState.code == vRestoreAndStartNewSession)
        startNewSession();
}

// Map an ASCII character to its macOS key code (platforms/mac.h).
static Uint16 charToKeyCode(char ch) {
    switch (ch) {
        case 'a': return KEY_A;   case 'b': return KEY_B;
        case 'c': return KEY_C;   case 'd': return KEY_D;
        case 'e': return KEY_E;   case 'f': return KEY_F;
        case 'g': return KEY_G;   case 'h': return KEY_H;
        case 'i': return KEY_I;   case 'j': return KEY_J;
        case 'k': return KEY_K;   case 'l': return KEY_L;
        case 'm': return KEY_M;   case 'n': return KEY_N;
        case 'o': return KEY_O;   case 'p': return KEY_P;
        case 'q': return KEY_Q;   case 'r': return KEY_R;
        case 's': return KEY_S;   case 't': return KEY_T;
        case 'u': return KEY_U;   case 'v': return KEY_V;
        case 'w': return KEY_W;   case 'x': return KEY_X;
        case 'y': return KEY_Y;   case 'z': return KEY_Z;
        case '1': return KEY_1;   case '2': return KEY_2;
        case '3': return KEY_3;   case '4': return KEY_4;
        case '5': return KEY_5;   case '6': return KEY_6;
        case '7': return KEY_7;   case '8': return KEY_8;
        case '9': return KEY_9;   case '0': return KEY_0;
        case ' ': return KEY_SPACE;
        default:  return KEY_EMPTY;
    }
}

static void typeText(const char* text) {
    for (const char* p = text; *p; p++) {
        Uint16 key = charToKeyCode(*p);
        if (key != KEY_EMPTY)
            typeKey(key);
    }
}

static void newSession() {
    // Mimic RequestNewSession() in the real front-end: a mouse event tells
    // the engine to start a fresh session and resets HookState.
    vKeyHandleEvent(vKeyEvent::Mouse, vKeyEventState::MouseDown, 0);
    startNewSession();
    screen.clear();
}

int main() {
    vKeyInit();

    // --- Basic Telex ---
    newSession();
    typeText("vieetj nam");
    check(screen == L"việt nam", "vieetj nam -> việt nam");

    newSession();
    typeText("vieetj");
    check(screen == L"việt", "vieetj -> việt");

    // Tone on diphthong "oa":
    // modern orthography -> mark on 'a' ("toà"), old orthography -> mark on 'o' ("tòa").
    vUseModernOrthography = 1;
    newSession();
    typeText("toaf");
    check(screen == L"toà", "toaf -> toà (modern mark on a)");
    vUseModernOrthography = 0;
    newSession();
    typeText("toaf");
    check(screen == L"tòa", "toaf -> tòa (old mark on o)");
    vUseModernOrthography = 1;

    // A with circumflex: aas -> ấ
    newSession();
    typeText("aas");
    check(screen == L"ấ", "aas -> ấ");

    // "huỵch" (fixed by commit 9b0efa4)
    newSession();
    typeText("huyjch");
    check(screen == L"huỵch", "huyjch -> huỵch");

    // "q u" handling: quan stays
    newSession();
    typeText("quan");
    check(screen == L"quan", "quan -> quan");

    // Word break resets engine
    newSession();
    typeText("chaof ");
    check(screen == L"chào ", "chaof[space] -> chào ");

    newSession();
    typeText("ba");
    check(screen == L"ba", "ba -> ba");

    // Simple Telex 1: aw -> ă
    vInputType = vSimpleTelex1;
    newSession();
    typeText("aw");
    check(screen == L"ă", "simple telex1 aw -> ă");
    vInputType = vTelex;

    // VNI input: a6 -> â
    vInputType = vVNI;
    newSession();
    typeText("a6");
    check(screen == L"â", "VNI a6 -> â");
    vInputType = vTelex;

    if (_failCount == 0)
        printf("ALL %d CHECKS PASSED\n", _checkCount);
    else
        printf("%d/%d CHECKS FAILED\n", _failCount, _checkCount);
    return _failCount == 0 ? 0 : 1;
}
