# XXKey

### Bộ gõ tiếng Việt mã nguồn mở tối ưu cho macOS & Windows (Rust Core)

**XXKey** là bộ gõ tiếng Việt mã nguồn mở thế hệ mới, ban đầu được phát triển dưới dạng một bản fork từ dự án **OpenKey** (của tác giả _tuyenvm_). 

Hiện tại, **XXKey** hỗ trợ hoàn chỉnh hai hệ điều hành:
- **macOS:** Chạy ẩn siêu nhẹ qua hệ thống Event Tap, tích hợp thanh trạng thái menu bar và cửa sổ Cài đặt trực quan.
- **Windows 11 / 10:** Chạy ẩn qua Low-Level Keyboard Hook (`WH_KEYBOARD_LL`), tích hợp khay hệ thống (System Tray) đầy đủ, tiêm phím nguyên tử (`SendInput`) và giao diện Slint UI siêu nhẹ.

Phiên bản cho **Linux** (IBus / Fcitx5) đang nằm trong lộ trình hoàn thiện tiếp theo.

Để tối ưu hóa hiệu năng, tăng cường an toàn bộ nhớ và mang lại khả năng tương thích đa nền tảng lâu dài, toàn bộ phần lõi xử lý phím đã được chuyển đổi sang ngôn ngữ **Rust** (**`vietime-engine`**). Lõi xử lý này kế thừa chuẩn xác thuật toán xử lý phím gốc của OpenKey (C++), vượt qua bộ kiểm thử tự động (differential testing) với hơn 1.800 test vector để đảm bảo hành vi gõ dấu tiếng Việt chính xác 100%, không bị lỗi gạch chân hay mất phím.

---

## ⚡ Các tính năng nổi bật

### ⌨️ Hỗ trợ kiểu gõ và bảng mã đa dạng
- **Kiểu gõ:** Telex, VNI, Simple Telex 1, Simple Telex 2.
- **Bảng mã:** Lõi Rust hỗ trợ các bảng mã thông dụng: Unicode dựng sẵn, Unicode tổ hợp, TCVN3 (ABC), VNI Windows, Vietnamese Locale CP 1258.

### 🧠 Xử lý thông minh & Tiện ích
- **Khởi động cùng hệ thống (Autostart):**
  - macOS: Tích hợp qua `SMAppService`.
  - Windows: Tích hợp qua Windows Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
- **Khôi phục phím gõ sai (Restore spelling):** Tự động hoàn tác về ký tự gốc khi phát hiện từ sai quy tắc chính tả tiếng Việt.
- **Chính tả hiện đại (Modern Orthography):** Tuỳ chọn đặt dấu kiểu mới (*oà, uý*) hoặc kiểu cũ (*òa, úy*).
- **Giao diện cấu hình siêu nhẹ:**
  - macOS: Cửa sổ Cài đặt SwiftUI trực quan tích hợp ngay trên Status Menu.
  - Windows / Linux: Giao diện Slint UI độc lập chỉ tiêu tốn **~5MB RAM**, tiết kiệm tài nguyên tối đa so với các giải pháp dựa trên Webview (Electron/Tauri).
- **Khay hệ thống (System Tray) thông minh:**
  - Hiển thị tooltip trạng thái động (chế độ gõ, kiểu gõ, bảng mã).
  - Nhấp đúp chuột để mở nhanh cửa sổ Cài đặt.
  - Menu chuột phải chuyển đổi nhanh kiểu gõ / chế độ tiếng Anh - tiếng Việt.
- **Phím tắt chuyển chế độ (Hotkeys):** Hỗ trợ `Ctrl + Shift` hoặc `Alt + Z` để chuyển đổi nhanh giữa tiếng Việt và tiếng Anh.
- **Phụ âm ghép & Gõ nhanh:** Hỗ trợ Telex nhanh (`cc` -> `ch`, `gg` -> `gi`, `kk` -> `kh`...) và tự động viết hoa chữ cái đầu câu.
- **Gõ tắt (Macro):** Khởi tạo và sử dụng bảng từ gõ tắt.

---

## 🏗️ Cấu trúc thư mục dự án

Dự án được tổ chức dưới dạng một Cargo Workspace thống nhất:

| Thư mục | Vai trò | Trạng thái |
| :--- | :--- | :--- |
| **[`vietime-engine`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/vietime-engine)** | Lõi xử lý chính (State machine, spelling, grammar) | Hoàn thành 100% |
| **[`platform-macos`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-macos)** | Front-end SwiftUI & Event Tap hook cho macOS | Hoàn thành (Chạy ổn định qua Event Tap) |
| **[`platform-win`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-win)** | Daemon Win32 Hook (`WH_KEYBOARD_LL`), System Tray, Injector | Hoàn thành (Hỗ trợ đầy đủ Windows 11/10) |
| **[`ui-tray`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-tray)** | Tích hợp khay hệ thống, context menu và dynamic tooltip | Hoàn thành |
| **[`ui-settings`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-settings)** | Giao diện cấu hình bộ gõ bằng Slint UI | Hoàn thành |
| **[`platform-linux`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-linux)** | Daemon bắt phím & Tích hợp IBus/Fcitx5 cho Linux | Đang phát triển (Khung stub D-Bus/IBus) |
| **[`ui-candidate`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-candidate)** | Giao diện gợi ý từ / bảng ứng viên gõ | Đang phát triển (Skeleton) |

---

## 🛠️ Hướng dẫn cài đặt & Biên dịch

> [!IMPORTANT]
> Để tránh xung đột phím, vui lòng tắt hoàn toàn các bộ gõ tiếng Việt khác (như UniKey, EVKey, bộ gõ mặc định của macOS/Windows) trước khi khởi chạy XXKey.

### Yêu cầu môi trường
- Đã cài đặt **Rust toolchain** (cargo/rustc 2024 edition).
- Trên macOS: Yêu cầu **Xcode Command Line Tools** (`swiftc`).
- Trên Windows: Yêu cầu **MSVC C++ Build Tools** hoặc MinGW-w64.

### 1. Biên dịch trên macOS
Tại thư mục gốc của dự án:
- **Biên dịch và chạy thử ứng dụng:**
  ```bash
  make run
  ```
- **Chỉ biên dịch và tạo Bundle `build/XXKey.app` (có tự động ký mã ad-hoc):**
  ```bash
  make build-app
  ```

### 2. Biên dịch trên Windows
Mở terminal (PowerShell hoặc Command Prompt) tại thư mục gốc dự án:
```powershell
cargo build --release -p platform-win -p ui-settings
```
Sau khi hoàn tất, các file thực thi sẽ nằm tại:
- `target\release\xxkey-daemon.exe` (Chương trình gõ phím chạy ẩn dưới khay hệ thống)
- `target\release\xxkey-settings.exe` (Giao diện cấu hình bộ gõ Slint)

### 3. Đóng gói phân phối (Multi-platform)
- **Tự động đóng gói ZIP cho các nền tảng:**
  ```bash
  make zips
  ```
  Sẽ sinh ra các gói tại thư mục `build/`:
  - `XXKey-macOS.zip` (App Bundle sẵn sàng chạy)
  - `XXKey-Windows.zip` (Mã nguồn và core biên dịch sẵn cho Windows)
  - `XXKey-Linux.zip` (Mã nguồn và core biên dịch sẵn cho Linux)
- Ngoài ra, dự án tích hợp **GitHub Actions CI/CD** tự động biên dịch native trên runner của cả 3 hệ điều hành khi đẩy tag phiên bản dạng `v*`.

---

## 📥 Hướng dẫn sử dụng cho từng nền tảng

### 1. Cho macOS
1. Tải bản phát hành mới nhất từ mục [Releases](https://github.com/nguyenpcminh/xxkey/releases).
2. Giải nén `XXKey-macOS.zip` và kéo **XXKey.app** vào thư mục **Applications**.
3. **Cấp quyền Trợ năng (Accessibility):**
   - Vào **System Settings** -> **Privacy & Security** -> **Accessibility**.
   - Bật cho phép **XXKey** nhận diện phím từ hệ thống.
4. Biểu tượng bộ gõ sẽ hiển thị trên Menu bar. Nhấp vào để bật/tắt chính tả kiểu mới hoặc chọn **Cài đặt** để mở giao diện tuỳ chỉnh.

### 2. Cho Windows (Windows 11 / 10)
1. Tải `XXKey-Windows.zip` từ mục [Releases](https://github.com/nguyenpcminh/xxkey/releases) và giải nén vào thư mục bạn muốn cài đặt.
2. Khởi chạy `xxkey-daemon.exe`. Biểu tượng bộ gõ sẽ xuất hiện dưới khay hệ thống (System Tray).
3. **Thao tác nhanh:**
   - **Nhấp chuột trái** vào icon khay hệ thống: Bật/Tắt nhanh tiếng Việt / tiếng Anh.
   - **Nhấp đúp chuột**: Mở giao diện Cài đặt (`xxkey-settings.exe`).
   - **Nhấp chuột phải**: Mở menu ngữ cảnh để chọn kiểu gõ, bảng mã, hoặc thoát chương trình.
   - **Phím tắt:** Sử dụng `Ctrl + Shift` hoặc `Alt + Z` để đổi chế độ gõ bất kỳ lúc nào.
4. Trong giao diện Cài đặt, tích chọn **"Khởi động cùng Windows"** để bộ gõ tự chạy mỗi khi đăng nhập máy tính.

### 3. Cho Linux
- Hiện đang trong quá trình phát triển tích hợp IBus/Fcitx5 daemon.

---

## 📜 Giấy phép & Cam kết
Bộ gõ XXKey được phát hành hoàn toàn miễn phí dưới giấy phép nguồn mở **GPL-3.0**. 

Cam kết bảo mật: **Bộ gõ không chứa mã độc, không kết nối mạng gửi dữ liệu, không theo dõi và không lưu trữ nội dung phím gõ của người dùng.**

