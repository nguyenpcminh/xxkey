# HANDOFF — XXKey Rust Port (xxkey-new)

Người chuyển giao: Claude Code
Ngày: 2026-08-10
Mục tiêu: chuyển đổi trình gõ tiếng Việt XXKey/OpenKey (C++) sang **Rust core**, giữ nguyên source cũ, tạo folder mới `xxkey-new/`, test kỹ mọi trường hợp, chính xác tối đa.

---

## 1. Bối cảnh & kiến trúc

Source C++ gốc nằm tại `Sources/OpenKey/engine/` (engine thuần, không phụ thuộc platform GUI). Các file:

| File | Vai trò |
|---|---|
| `Engine.cpp` (1624 dòng) | **State machine chính** — xử lý từng phím, mark, tone, âm cuối, word break, macro. |
| `Vietnamese.cpp` (575) | Bảng dữ liệu: `_vowel`, `_vowelCombine`, `_consonantD`, `_vowelForMark`, `_codeTable` (5 bảng mã), `_quickTelex`, `_characterMap`, `keyCodeToCharacter`. |
| `Macro.cpp` (292) | Macro: convert text → keycodes, `findMacro`, lưu/đọc file. |
| `SmartSwitchKey.cpp` (73) | Tự chuyển ngôn ngữ theo app (bundleId). |
| `ConvertTool.cpp` (179) | Công cụ đổi bảng mã / hoa thường / bỏ dấu. |
| `DataType.h` (155) | Enums, masks (CAPS/TONE/TONEW/MARK/STANDALONE/CHAR_CODE), `vKeyHookState`. |
| `Engine.h` (243) | API + 22 biến config `v*` global. |
| `platforms/mac.h` | Key codes macOS. |
| `test_engine.cpp` | Harness assert sẵn có (27 checks, đều pass). |

### Mô hình dữ liệu (bắt buộc giữ nguyên để audit byte-for-byte)
Mỗi ô trong `TypingWord` là `u32`:
- bit 0-15: character/key code
- bit 16: caps (`CAPS_MASK` 0x10000)
- bit 17: tone `^` (`TONE_MASK` 0x20000)
- bit 18: tone `w` (`TONEW_MASK` 0x40000)
- bit 19-23: mark (sắc/huyền/hỏi/ngã/nặng) `MARK_MASK` 0xF80000
- bit 24: standalone (`STANDALONE_MASK` 0x1000000)
- bit 25: is character code (`CHAR_CODE_MASK` 0x2000000)
- `PURE_CHARACTER_MASK` 0x80000000
- `END_CONSONANT_MASK` 0x4000, `CONSONANT_ALLOW_MASK` 0x8000 (đánh dấu trong bảng từ điển)

### Vòng xử lý phím (`vKeyHandleEvent` → Rust `Engine::handle_key`)
Input: `(event, state, data, capsStatus, otherControlKey)`. Output: `HookState { code, backspaceCount, newCharCount, extCode, charData[32], macroKey, macroData }`. Front-end xóa `backspaceCount` ký tự trên màn hình rồi chèn `newCharCount` ký tự từ `charData` theo **thứ tự đảo** (`charData[ncc-1] .. charData[0]`).

Quyết định chính:
1. **Word break / number / ctrl / mouse** → code=DoNothing, extCode=WordBreak (1). Kèm xử lý macro, quick consonant, restore-if-wrong-spelling.
2. **Space** → check spelling, macro, quick consonant, restore; save word.
3. **Delete (backspace)** → extCode=Delete (2), pop special_char/space, giảm index, checkGrammar(1).
4. **Key thường** → insertState → nếu không phải special key → insertKey; nếu special key → `handleMainKey`.

### `handleMainKey` (Rust: `handle_main_key`)
Thứ tự: Z (bỏ mark) → `[`/`]` (standalone) → D (đ) → mark key (s/f/r/x/j) → vowel (a/o/e/w + VNI số). Mỗi nhánh dùng `checkCorrectVowel` so khớp đuôi từ với bảng `_vowel`/`_vowelForMark`/`_consonantD`, rồi gọi `insertAOE`/`insertW`/`insertMark`/`insertD`.

---

## 2. Trạng thái HIỆN TẠI (đã làm)

### 2.1 Khung workspace đã tồn tại từ phiên trước
`xxkey-new/` là cargo workspace:
```
xxkey-new/
  Cargo.toml            # workspace, edition 2024, members
  vietime-engine/       # <-- CRATE CHÍNH (no_std, pure Rust)
    Cargo.toml          # lib name = vietime_engine, no_std core, tests run on std
    src/lib.rs          # no_std + forbid(unsafe_code) + warn(missing_docs)
    src/datatype.rs     # port của DataType.h (enums, masks, HookState)
    src/keycode.rs      # port của platforms/mac.h (KEY_A=0 ...)
    src/vietnamese.rs   # port của Vietnamese.cpp (bảng const tĩnh + lookup fn)
    src/engine.rs       # port của Engine.cpp (Engine struct + handle_key + helpers)
  platform-win/ platform-macos/ platform-linux/   # stub (1 dòng lib.rs)
  ui-settings/ ui-candidate/ ui-tray/             # stub (1 dòng lib.rs)
  reference/            # bản copy C++ gốc (KHÔNG dùng làm nguồn — xem mục 2.2)
  target/               # đã build, có Cargo.lock
```
`reference/` là **bản C++ CŨ** (còn bug) — **không** phải nguồn tham chiếu. Nguồn đúng là `Sources/OpenKey/engine/`.

### 2.2 Việc ĐÃ HOÀN THÀNH trong phiên này
1. **Đọc & phân tích toàn bộ source C++ gốc** (Engine.cpp, Vietnamese, Macro, ConvertTool, SmartSwitchKey, DataType, Engine.h, mac.h, test_engine.cpp). Task #1 done.
2. **Xác minh `reference/` ≠ `Sources/`**: `reference/Engine.cpp` là bản cũ có bug (ví dụ `handleModernMark` rule 3.1 dùng `CHR(VSI+2)` thay vì `CHR(VSI+3)`, thiếu guard `vowelCount>=2`). `Vietnamese.cpp`/`Macro.cpp`/`SmartSwitchKey.cpp`/`ConvertTool.cpp`/header giống hệt sau khi bỏ comment/space. → **Phải port theo `Sources/` (bản đã sửa).**
3. **Xây dựng ORACLE C++ từ `Sources/`**: biên dịch thành `/tmp/xxkey-oracle/oracle_dump` — đọc chuỗi phím + config `(inputType, modernOrthography)`, in kết quả màn hình dạng hex codepoint. `test_engine.cpp` gốc chạy OK (27/27 pass).
4. **Sửa lỗi build Rust ban đầu** (5 lỗi compile):
   - `datatype.rs`: `extern crate alloc; use alloc::vec::Vec;` ở đầu file (trước kia `extern crate alloc` ở cuối file → `Vec` không tìm thấy).
   - `engine.rs`: `extern crate alloc;` ở đầu + xoá `extern crate alloc;` thừa ở cuối; xoá hàm dead-code `key_for_mark_primary(&self)` (gây E0596, không dùng).
   → **cargo build -p vietime-engine: EXIT 0.**
5. **Sửa 2 bug faithfulness lớn đã phát hiện**:
   - **BUG 1 — `self.key` sai kiểu**: C++ `key` là `int` 32-bit, dùng `key |= TONE_MASK` (0x20000) để lookup bảng `KEY_A|TONE_MASK`. Rust để `key: u16` → `TONE_MASK as u16 = 0` → mất bit, lookup sai. **Đã sửa thành `key: u32`** và viết lại `get_character_code` đúng.
   - **BUG 2 — `check_spelling` dùng biến local thay vì global counter**: C++ tái dùng global `j`, `ii` làm biến đếm vòng lặp và **đọc lại giá trị cuối sau vòng lặp** (`if (j == _spellingEndIndex)`, `j + ii - 1 < _spellingEndIndex`). Port cũ dùng `jj` local → stale/0 → sai. **Đã viết lại `check_spelling` faithful** (giữ `j`/`ii` làm biến đếm, match đúng biểu thức AND hai stripped value như C++: `(_consonantTable[i][j] & ~(vQuickStartConsonant ? END_CONSONANT_MASK:0)) != CHR(j) && (...CONSONANT_ALLOW_MASK...) != CHR(j)`).
   - **BUG 3 — `check_for_standalone_char` panic khi index=0**: `self.index - 1` underflow → panic ở `chr()`. **Đã guard `self.index > 0`** ở nhánh đầu (C++ `_index-1` khi index=0 là UB, quan sát thực tế không match).
6. **Xây dựng DIFFERENTIAL TEST khung**:
   - `/tmp/xxkey-oracle/gen_golden.sh` — sinh **1826 golden vector** từ oracle: input types 0-3, modern 0/1, phủ vựng từ phổ biến + hiếm, backspace (`_`), dấu câu/break keys.
   - Golden file đã copy vào `vietime-engine/tests/data/golden.txt`.
   - `vietime-engine/tests/golden.rs` — integration test đọc golden, mô phỏng screen buffer (y hệt `test_engine.cpp`), so sánh output.
   - **Kết quả hiện tại**: `cargo test` → **148/1826 vector lệch** (73 telex + còn lại VNI/simple). Trước khi sửa bug 1-3 các vector này nhiều hơn và còn panic; giờ không panic nữa.

### 2.3 Các file đã tạo/sửa (trong phiên này)
| File | Trạng thái |
|---|---|
| `vietime-engine/src/datatype.rs` | Sửa: thêm `extern crate alloc` + `use alloc::vec::Vec` đầu file |
| `vietime-engine/src/engine.rs` | Sửa: `key: u32`, viết lại `check_spelling` + `get_character_code`, guard standalone, xoá dead code |
| `vietime-engine/tests/golden.rs` | Mới: differential test chống oracle |
| `vietime-engine/tests/data/golden.txt` | Mới: 1826 golden vector |
| `/tmp/xxkey-oracle/` | Mới (ngoài repo): oracle C++ + gen_golden.sh + golden.txt |

> ⚠️ `src/difftest.rs` đã bị xoá (phiên bản thử đặt logic difftest trong lib gây lỗi no_std). Logic hiện nằm gọn trong `tests/golden.rs`.

---

## 3. VIỆC CÒN LẠI (kế hoạch — theo thứ tự)

### BƯỚC A — Sửa 148 golden mismatch còn lại (ƯU TIÊN CAO NHẤT)
Đây là phần khó nhất. Dùng output test hiện có làm danh sách việc. Quan sát ban đầu:
- Nhóm `aas→1EA5` (oracle) nhưng Rust cho `a a s` nghĩa là **`handle_main_key` chưa kích hoạt cho phím a/o/e/w** đúng — cần debug: có thể do `is_special_key`/`temp_disable_key`/`check_spelling` set `tempDisableKey=true` quá sớm, khiến `vKeyHandleEvent` đi nhánh `!IS_SPECIALKEY(data) || tempDisableKey` → DoNothing + insertKey. Kiểm chứng: với từ "a", Rust cho `0061` (khớp); "aa" phải cho `00E2` nhưng golden test đang fail → khả năng cao `tempDisableKey` bị set sai do `check_spelling` chưa đúng tiếp (ví dụ single 'a' → spelling chưa đủ). **Khuyến nghị: viết unit test nhỏ cho riêng `check_spelling` cho từng prefix rồi so với `tempDisableKey` của C++** (thêm debug print vào oracle nếu cần).
- Sau khi sửa nhóm này, chạy lại `cargo test` → còn các nhóm khác (VNI number mapping, simple telex). Lặp cho tới 0 fail.

### BƯỚC B — Đóng gói test kỹ lưỡng (task #3)
Sau khi golden pass 100%, bổ sung test case "người" cho từng tính năng (kể cả case hiếm):
1. **Bảng mã**: 5 bảng (Unicode, TCVN3, VNI-Windows, Unicode Compound, CP1258) — verify `getCharacterCode` cho từng ô (mở rộng golden theo code_table).
2. **Telex**: mọi tổ hợp mark × (vowel đơn/kép/ba) × âm cuối; case "thuơn"/"ưoi"/"ưom"/"ưoc" (checkGrammar), "qu", "gi", "thoòng" (insertAOE O), "huyjch" (bug cũ đã fix), modern vs old orthography.
3. **VNI**: số mapping (1-9 sắc huyền hỏi nặng ngã, 6=^, 7/8=w/ư, 9=đ), "d9", "a8".
4. **Simple Telex 1/2**: aw→ă, ew→ơ, ow→ơ, uw→ư, ww→ư.
5. **Word break / dấu câu / số đầu từ / ctrl / mouse**: reset session đúng.
6. **Backspace**: đơn, nhiều lần, giữa từ, xoá cả từ, sau space, sau dấu câu, checkGrammar sau xoá.
7. **Macro**: `convert`, `findMacro`, auto-caps (Btw→By the way), `addMacro`/`deleteMacro`, lưu/đọc file định dạng UniKey.
8. **SmartSwitchKey**: cache, get/set theo bundleId.
9. **ConvertTool**: đổi bảng mã giữa 5 bảng, all-caps/non-caps/first-letter/each-word, remove mark, unicode compound mark index.
10. **Quick Telex**: cc/gg/kk/nn/qq/pp/tt/uu.
11. **Quick consonant**: f→ph, j→gi, w→qu (start); g→ng, h→nh, k→ch (end).
12. **Caps**: shift/capslock với mark/tone, uppercase first char (sau . và Enter), ZFWJ consonant allow.
13. **Long word > 32 chars** (`_longWordHelper`, `_typingStates`).
14. **Restore if wrong spelling**; **fix recommend browser**; **temp off spelling/engine**.
15. **Empty/edge inputs**: gõ liên tiếp mark key, tone key không hợp lệ, index tràn.

### BƯỚC C — Build toàn bộ workspace (task #4)
- `cargo build` (workspace) 0 lỗi, 0 panic trong release.
- `cargo test` (workspace) 100% pass.
- Kiểm tra no_std: `cargo build --no-default-features` hoặc target no_std (vd `thumbv7em-none-eabihf`) để chắc lib không rò rỉ std vào non-test build.
- Cảnh báo unused variables (`insert_d`, `insert_aoe`, `insert_w` nhận `is_caps` không dùng — giữ signature giống C++ nhưng thêm `_` prefix hoặc `#[allow]`).

### BƯỚC D — Các crate khác (nếu cần)
- `platform-*`, `ui-*` hiện là stub 1 dòng. Core engine ở `vietime-engine`. Nếu yêu cầu "bộ source mới dùng Rust làm core", ưu tiên hoàn thiện engine + test trước; GUI/platform có thể để stub hoặc port sau.

### BƯỚC E — Commit (task #5)
- Chưa có commit nào trong phiên này. Nên commit từng phần:
  1. `build-fix: sửa no_std alloc + xoá dead code` (datatype.rs, engine.rs)
  2. `fix: key u32 + check_spelling faithful + standalone guard`
  3. `test: golden differential suite chống oracle C++` (tests/golden.rs + golden.txt)
  4. ... mỗi lô sửa mismatch thành 1 commit.
- Theo user: "Chạy tự check task tự làm mọi thứ tới khi hoàn thành. Mỗi bước check kỹ push từng phần lên git."

---

## 4. Môi trường build (quan trọng — không có cargo trong PATH mặc định)

Cargo/rustc **KHÔNG** có trong PATH (cài qua puccinialin cache). Mỗi lệnh cargo phải export:
```bash
export PATH="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo/bin:$PATH"
export CARGO_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo"
export RUSTUP_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/rustup"
```
Cargo 1.97.1, rustc 1.97.1. CWD phải là `xxkey-new/`.

⚠️ **rtk hook** (trong `~/.claude/settings.json`) intercept mọi Bash và nén output — lỗi cargo build đầy đủ bị mất. Cách lấy raw:
- Redirect ra file rồi Read: `cargo build > /tmp/x.log 2>&1`
- Hoặc `rtk proxy bash -c 'cargo build ...'`
- `grep` trực tiếp cũng bị lọc → grep vào file rồi đọc file.

C++ oracle build: `g++ -std=c++17 -DLINUX Engine.cpp Vietnamese.cpp Macro.cpp SmartSwitchKey.cpp ConvertTool.cpp oracle_dump.cpp -o oracle_dump` (clang 21 có sẵn).

---

## 5. Các lỗi faithfulness đã biết (CHỖ NÀY CÓ THỂ CÓ BUG KHÁC)

Port Rust là "dịch nguyên bản" nhưng có những chỗ C++ dựa vào hành vi global/UB mà Rust không có. Đã sửa 3; **còn rà soát kỹ các hàm sau** (tìm chỗ C++ đọc lại biến đếm vòng lặp hoặc index tràn):
- `find_and_calculate_vowel` — C++ dùng `iii` global, vòng `for (iii=_index-1; iii>=0; iii--)` với `iii` là `int`. Rust đã dùng `while iii>0 { iii-=1 }` — kiểm tra biên khi `_index=0` (C++ `iii=-1`, không vào vòng; Rust `while iii>0` không vào — OK).
- `handle_modern_mark` / `handle_old_mark` — C++ truy cập `CHR(VSI+1)`, `CHR(VSI+2)`, `TypingWord[VSI+1]` không check biên khi VSI ở cuối. Rust dùng `.get()`/panic? Kiểm tra từng ô.
- `insert_w` nhánh vowelCount>1: C++ đọc `TypingWord[VSI]`, `TypingWord[VSI+1]` — khi VSI+1 == index (chưa có) là UB. Rust panic nếu index-tràn.
- `check_grammar` dùng `l = VSI` global.
- `vKeyHandleEvent` nhánh `[ ]`: C++ so `(Uint16)hData[0]` với bracket — Rust so `state.char_data[0]`.
- **Thứ tự `_vowelForMark`**: C++ `std::map` sort key tăng dần → **`[A(0), E(14), Y(16), O(31), U(32), I(34)]`**. Port Rust hiện dùng `[A,O,E,I,U,Y]` — **SAI THỨ TỰ**, C++ break ở match đầu tiên nên thứ tự ảnh hưởng kết quả. Cần sửa theo thứ tự ASCII. (Chưa sửa — có thể là 1 trong các mismatch hiện tại.)
- Cần tái lập **`tone on diphthong`**: `toaf`→`toà` (modern) / `tòa` (old) đã pass trong oracle; xác nhận Rust.

---

## 6. Lệnh hữu dụng

```bash
# Build + test engine
cd xxkey-new && <export env> && cargo build -p vietime-engine
cd xxkey-new && <export env> && cargo test -p vietime-engine --test golden
cd xxkey-new && <export env> && cargo test -p vietime-engine

# Sinh lại golden từ oracle
/tmp/xxkey-oracle/gen_golden.sh /tmp/xxkey-oracle/golden.txt
cp /tmp/xxkey-oracle/golden.txt xxkey-new/vietime-engine/tests/data/golden.txt

# Oracle thủ công
/tmp/xxkey-oracle/oracle_dump <inputType> <modern> "<keyseq>"
```

---

## 7. Tóm tắt 1 dòng cho người nhận việc
> Port Rust engine (`vietime-engine`) đã dựng xong khung + build OK + test framework differential chống oracle C++ chạy được. **Nhiệm vụ tiếp: sửa cho hết 148/1826 golden vector lệch (bắt đầu từ nhóm `aas`/`aa` do `tempDisableKey` từ `check_spelling`), sửa thứ tự `_vowelForMark`, rồi bổ sung test toàn diện (mục B) và build/test workspace 100% (mục C), commit từng phần (mục E).**
