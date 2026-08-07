# XXKey

### Bộ gõ tiếng Việt nguồn mở cho macOS và Windows (Fork từ OpenKey)

**XXKey** là bộ gõ tiếng Việt mã nguồn mở được phát triển dưới dạng một bản fork từ dự án **OpenKey** (của tác giả _tuyenvm_).

Kế thừa những giá trị cốt lõi của OpenKey, XXKey tập trung cải thiện hiệu năng, tối ưu hóa các API hệ thống và mang lại trải nghiệm gõ tiếng Việt mượt mà trên macOS & Windows mà không gặp phải lỗi gạch chân hay trễ phím (nhờ kỹ thuật gửi phím `Backspace` hiện đại thay vì cơ chế preedit mặc định của macOS).

---

## ⚡ Tính năng nổi bật

### ⌨️ Hỗ trợ kiểu gõ và bảng mã đa dạng

- **Kiểu gõ hỗ trợ:** Telex, VNI, Simple Telex.
- **Bảng mã thông dụng:** Unicode dựng sẵn, Unicode tổ hợp, TCVN3 (ABC), VNI Windows, Vietnamese Locale CP 1258, v.v.

### 🧠 Tính năng thông minh & Tiện ích

- **Tự ghi nhớ bảng mã theo ứng dụng:** XXKey tự động ghi nhận và chuyển đổi bảng mã phù hợp cho từng ứng dụng riêng biệt (ví dụ: Photoshop/AutoCAD dùng VNI/TCVN3, còn trình duyệt dùng Unicode) khi bạn chuyển đổi cửa sổ làm việc.
- **Chuyển chế độ gõ thông minh:** Tự động khôi phục chế độ gõ (Anh hoặc Việt) tương ứng với từng ứng dụng cụ thể.
- **Viết hoa chữ cái đầu câu:** Tự động viết hoa chữ cái đầu câu sau các dấu ngắt câu hoặc khi xuống dòng.
- **Tính năng gõ tắt (Macro):** Hỗ trợ soạn thảo và sử dụng bảng từ gõ tắt không giới hạn độ dài hay số lượng ký tự.
- **Khôi phục phím khi gõ từ sai (Restore key):** Tự động khôi phục lại các ký tự gốc khi phát hiện từ gõ sai quy tắc chính tả tiếng Việt.
- **Hỗ trợ gõ tắt phụ âm:**
  - Phụ âm đầu: `f` -> `ph`, `j` -> `gi`, `w` -> `qu`.
  - Phụ âm cuối: `g` -> `ng`, `h` -> `nh`, `k` -> `ch`.
  - Cho phép sử dụng các ký tự `f, z, w, j` làm phụ âm đầu.
- **Chế độ gõ nhanh (Quick Telex):** Gõ nhanh các cụm phụ âm ghép (`cc`=ch, `gg`=gi, `kk`=kh, `nn`=ng, `qq`=qu, `pp`=ph, `tt`=th).
- **Chế độ "Gửi từng phím" (Send key by key):** Tăng độ tương thích khi chơi game hoặc gõ trên các ứng dụng đặc thù.

---

## 📥 Hướng dẫn cài đặt

> [!IMPORTANT]
> Để tránh xung đột phím, vui lòng tắt hoàn toàn các bộ gõ tiếng Việt khác trước khi sử dụng XXKey.

### Cho macOS

1. Tải về file `.dmg` mới nhất từ mục [Releases](https://github.com/nguyenpcminh/OpenKey/releases/latest).
2. Mở file `.dmg` và kéo ứng dụng **XXKey.app** vào thư mục **Applications** (Ứng dụng).
3. **Cấp quyền truy cập hệ thống (Trợ năng):**
   - Truy cập **System Settings** (Cài đặt hệ thống) -> **Privacy & Security** (Quyền riêng tư & Bảo mật) -> **Accessibility** (Trợ năng).
   - Kích hoạt và cấp quyền cho **XXKey.app**.
   - _Lưu ý: Quyền này bắt buộc phải bật để bộ gõ có thể xử lý phím bấm._

### Cho Windows

1. Tải về file `.zip` dành cho Windows từ mục [Releases](https://github.com/nguyenpcminh/OpenKey/releases/latest).
2. Giải nén vào một thư mục bất kỳ trên máy tính của bạn.
3. Chạy file thực thi `XXKey.exe`. Khuyến nghị chạy dưới quyền **Administrator** (Run as Administrator) để có thể gõ tiếng Việt bình thường trong game và các ứng dụng hệ thống.

---

## 🛠️ Hướng dẫn tự biên dịch (Build từ mã nguồn)

Nếu bạn muốn tự tay xây dựng ứng dụng từ mã nguồn:

### macOS

- **Yêu cầu:** macOS Mojave trở lên và Xcode 10 trở lên.
- **Cách build:**
  1. Clone mã nguồn của dự án:
     ```bash
     git clone https://github.com/nguyenpcminh/OpenKey.git
     ```
  2. Mở Xcode và mở dự án tại đường dẫn `Sources/OpenKey/macOS/OpenKey.xcodeproj`.
  3. Trên thanh menu Xcode, chọn **Product** -> **Archive** để bắt đầu biên dịch phiên bản Release.
  4. Sau khi biên dịch hoàn tất, chọn **Distribute App** để xuất file ứng dụng `XXKey.app`.

### Windows

- **Yêu cầu:** Microsoft Visual Studio 2017 trở lên.
- **Cách build:**
  1. Mở solution trong thư mục `Sources/OpenKey/win32`.
  2. Chọn cấu hình build `Release` và thực hiện build project.

---

## 📜 Giấy phép & Cam kết bảo mật

XXKey được phát hành hoàn toàn miễn phí dưới giấy phép **GPL (GNU General Public License)**.

- Ứng dụng cam kết minh bạch, **hoàn toàn không chứa mã độc, keylogger, quảng cáo hay bất kỳ đoạn mã theo dõi nào**.
- Bạn có quyền tự do tải mã nguồn, chỉnh sửa và đóng góp cho dự án. Mọi bản phân phối cải tiến cũng phải được mở mã nguồn và ghi nhận nguồn gốc từ XXKey / OpenKey.
