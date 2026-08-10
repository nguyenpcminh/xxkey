# XXKey

### Bộ gõ tiếng Việt mã nguồn mở tối ưu cho macOS, Windows và Linux (Core Rust)

**XXKey** là bộ gõ tiếng Việt mã nguồn mở thế hệ mới, ban đầu được phát triển dưới dạng một bản fork từ dự án **OpenKey** (của tác giả _tuyenvm_). 

Để tối ưu hóa hiệu năng, tăng cường an toàn bộ nhớ và mang lại khả năng tương thích đa nền tảng hoàn hảo (macOS, Windows, Linux), dự án đã tiến hành thiết kế và chuyển đổi toàn bộ phần lõi xử lý phím sang ngôn ngữ **Rust** (lấy tên là **`vietime-engine`**). Lõi xử lý này kế thừa chuẩn xác thuật toán xử lý phím gốc của OpenKey (C++), vượt qua bộ kiểm thử tự động (differential testing) với hơn 1.800 test vector để đảm bảo hành vi gõ dấu tiếng Việt chính xác 100%, không bị lỗi gạch chân hay mất phím.

---

## ⚡ Các tính năng nổi bật

### ⌨️ Hỗ trợ kiểu gõ và bảng mã đa dạng
- **Kiểu gõ:** Telex, VNI, Simple Telex 1, Simple Telex 2.
- **Bảng mã:** Lõi Rust hỗ trợ các bảng mã thông dụng như Unicode dựng sẵn, Unicode tổ hợp, TCVN3 (ABC), VNI Windows, Vietnamese Locale CP 1258. *Lưu ý: Hiện giao diện UI mặc định sử dụng Unicode dựng sẵn, tính năng lựa chọn bảng mã trên UI đang được phát triển.*

### 🧠 Xử lý thông minh & Tiện ích
- **Khởi động cùng hệ thống (Auto-launch):** Hỗ trợ tùy chọn tự động khởi động cùng hệ điều hành khi đăng nhập (sử dụng `SMAppService` trên macOS, ghi Registry trên Windows và tệp `.desktop` trên Linux).
- **Khôi phục phím gõ sai (Restore spelling):** Tự động hoàn tác về ký tự gốc khi phát hiện từ sai quy tắc chính tả tiếng Việt.
- **Chính tả hiện đại (Modern Orthography):** Hỗ trợ đặt dấu chuẩn theo phong cách mới hoặc phong cách cổ truyền.
- **Giao diện cấu hình gọn nhẹ:** Sử dụng giao diện Slint UI trên Windows/Linux chỉ tiêu tốn **~5MB RAM** khi chạy, vượt trội so với các giải pháp dựa trên Webview (như Electron/Tauri).
- **Phụ âm ghép & Gõ nhanh:** Hỗ trợ Telex nhanh (`cc` -> `ch`, `gg` -> `gi`, `kk` -> `kh`...) và tự động viết hoa chữ cái đầu câu.
- **Gõ tắt (Macro):** Khởi tạo và sử dụng bảng từ gõ tắt (lõi xử lý gõ tắt đã sẵn sàng).

---

## 🏗️ Cấu trúc thư mục dự án

Dự án được tổ chức dưới dạng một Cargo Workspace thống nhất:

| Thư mục | Vai trò | Trạng thái phát triển |
| :--- | :--- | :--- |
| **[`vietime-engine`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/vietime-engine)** | Lõi xử lý chính (State machine, spelling) | Hoàn thành 100% |
| **[`platform-macos`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-macos)** | Front-end & Keyboard Hook cho macOS | Hoàn thành (Chạy ổn định qua Event Tap) |
| **[`platform-win`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-win)** | Daemon bắt phím chạy ẩn cho Windows | Đang phát triển (Khung bắt phím/hook thô) |
| **[`platform-linux`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-linux)** | Daemon bắt phím & Tích hợp IBus cho Linux | Đang phát triển (Khung stub D-Bus/IBus) |
| **[`ui-settings`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-settings)** | Giao diện cấu hình bộ gõ cho Windows/Linux | Hoàn thành (Sử dụng Slint UI) |
| **[`ui-tray`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-tray)** | Khay hệ thống cho Windows/Linux | Đang phát triển (Skeleton) |
| **[`ui-candidate`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-candidate)** | Giao diện gợi ý từ/bảng gõ | Đang phát triển (Skeleton) |

---

## 🛠️ Hướng dẫn cài đặt & Biên dịch

> [!IMPORTANT]
> Để tránh xung đột phím, vui lòng tắt hoàn toàn các bộ gõ tiếng Việt khác (như UniKey, EVKey, bộ gõ mặc định của macOS) trước khi khởi chạy XXKey.

### Yêu cầu môi trường
- Đã cài đặt **Rust toolchain** (cargo/rustc).
- Trên macOS: Yêu cầu cài đặt **Xcode Command Line Tools** (`swiftc`).

### Các lệnh build nhanh (Makefile)

Tại thư mục gốc của dự án, bạn có thể sử dụng các lệnh Makefile sau:

- **Biên dịch và chạy thử ứng dụng trên macOS:**
  ```bash
  make run
  ```
- **Chỉ biên dịch và tạo Bundle `build/XXKey.app` trên macOS:**
  ```bash
  make build-app
  ```
- **Tự động đóng gói bộ cài cho cả 3 hệ điều hành (macOS, Windows, Linux):**
  ```bash
  make zips
  ```
  Lệnh này sẽ sinh ra 3 file lưu trữ tại thư mục `build/`:
  - `XXKey-macOS.zip` (Chứa app bundle chạy ngay).
  - `XXKey-Windows.zip` (Chứa Rust core pre-compiled, source code và settings UI).
  - `XXKey-Linux.zip` (Chứa Rust core pre-compiled, source code và settings UI).

---

## 📥 Trạng thái sử dụng cho từng nền tảng

### 1. Cho macOS (Đã hoàn thiện)
1. Giải nén `XXKey-macOS.zip` và kéo **XXKey.app** vào thư mục **Applications**.
2. **Cấp quyền Trợ năng (Accessibility):**
   - Vào **System Settings** -> **Privacy & Security** -> **Accessibility**.
   - Tích chọn cho phép **XXKey** nghe phím từ hệ thống.
3. Trong giao diện cài đặt, bạn có thể tích chọn **Khởi động cùng hệ thống** để tự động mở bộ gõ khi bật máy.

### 2. Cho Windows (Đang phát triển Daemon)
1. Hiện tại giao diện cấu hình [`ui-settings`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-settings) đã hoàn thiện và hỗ trợ cấu hình các tính năng bao gồm cả tự động khởi động cùng hệ thống (ghi Registry).
2. Daemon bắt phím [`platform-win`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-win) đang trong quá trình tích hợp sâu với lõi Rust và khay hệ thống.

### 3. Cho Linux (Đang phát triển Daemon)
1. Giao diện cấu hình [`ui-settings`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/ui-settings) đã hoàn thiện, hỗ trợ lưu cấu hình và tự tạo tệp `.desktop` để khởi động cùng hệ thống.
2. Khung dịch vụ tích hợp IBus/Fcitx5 [`platform-linux`](file:///Users/itaccountvn.ab-inbev.com/Desktop/tools/xxkey/platform-linux) đang được xây dựng để xử lý sự kiện phím trên môi trường Linux.

---

## 📜 Giấy phép & Cam kết
Bộ gõ XXKey được phát hành hoàn toàn miễn phí dưới giấy phép nguồn mở **GPL-3.0**. 

Cam kết minh bạch: **Bộ gõ không chứa mã độc, không theo dõi và không thu thập bất kỳ dữ liệu nhập liệu nào của người dùng.**
