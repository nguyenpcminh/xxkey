#include "Engine.h"
#include <cstdio>
#include <iostream>
#include <string>

using namespace std;

extern vKeyHookState HookState;

// --- globals the engine expects ---
int vLanguage = 1;
int vInputType = 0; // Telex
int vFreeMark = 0;
int vCodeTable = 0;     // Unicode
int vCheckSpelling = 0; // Disable spelling check!
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
int vAllowConsonantZFWJ = 1;
int vQuickStartConsonant = 0;
int vQuickEndConsonant = 1; // Enable quick end consonant
int vRememberCode = 0;
int vOtherLanguage = 0;
int vTempOffOpenKey = 0;

static void typeKey(Uint16 key) {
  vKeyHandleEvent(vKeyEvent::Keyboard, vKeyEventState::KeyDown, key, 0, false);
}

int main() {
  vKeyInit();

  // Type 30 't's.
  for (int i = 0; i < 30; i++) {
    typeKey(KEY_T);
  }
  // Type 'a' (index 30) - vowel
  typeKey(KEY_A);

  // Type 'g' (index 31) - quick end consonant
  cout << "Typing 'g'..." << endl;
  typeKey(KEY_G);

  // Type SPACE
  cout << "Typing SPACE..." << endl;
  typeKey(KEY_SPACE);
  cout << "Finished without crash." << endl;

  return 0;
}
