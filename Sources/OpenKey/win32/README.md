# XXKey for Windows

### Bộ gõ tiếng Việt nguồn mở cho Windows (Fork từ OpenKey)

**XXKey** (phiên bản dành cho Windows) là bộ gõ tiếng Việt mã nguồn mở được phát triển dựa trên nền tảng của **OpenKey** (tác giả *tuyenvm*). 

Mã nguồn của ứng dụng được công bố công khai dưới giấy phép **GPL**, đảm bảo tính minh bạch, an toàn tuyệt đối, không có keylogger, quảng cáo hay backdoor.

XXKey mặc định chạy ở chế độ **Administrator** (Quản trị viên) để tương thích tốt nhất và cho phép gõ tiếng Việt mượt mà trong các tựa game, ứng dụng hệ thống và các trình duyệt web.

---

## ⚡ Tính năng nổi bật

### ⌨️ Hỗ trợ kiểu gõ và bảng mã phong phú
- **Kiểu gõ:** Telex, VNI, Simple Telex.
- **Bảng mã thông dụng:** Unicode dựng sẵn, Unicode tổ hợp, TCVN3 (ABC), VNI Windows, Vietnamese Locale CP 1258.

### 🧠 Tính năng nâng cao & Tiện ích
- **Tự ghi nhớ bảng mã theo ứng dụng:** XXKey tự động ghi nhớ và chuyển đổi bảng mã phù hợp cho từng ứng dụng riêng biệt (ví dụ: Photoshop/AutoCAD dùng VNI/TCVN3, còn trình duyệt dùng Unicode) khi bạn chuyển đổi cửa sổ làm việc.
- **Chuyển chế độ thông minh:** Tự động khôi phục chế độ gõ (Anh hoặc Việt) tương ứng với từng ứng dụng cụ thể.
- **Viết hoa chữ cái đầu câu:** Tự động viết hoa chữ cái đầu tiên sau khi kết thúc một câu hoặc khi xuống dòng.
- **Tính năng gõ tắt (Macro):** Hỗ trợ bảng soạn thảo các từ gõ tắt, hỗ trợ ký tự và bảng mã bất kỳ mà không giới hạn số lượng hay độ dài.
- **Khôi phục phím khi gõ từ sai (Restore key):** Tự động khôi phục lại các ký tự gốc khi phát hiện từ gõ sai quy tắc chính tả tiếng Việt.
- **Hỗ trợ gõ tắt phụ âm:**
  - Phụ âm đầu: `f` -> `ph`, `j` -> `gi`, `w` -> `qu`.
  - Phụ âm cuối: `g` -> `ng`, `h` -> `nh`, `k` -> `ch`.
  - Cho phép dùng `f, z, w, j` làm phụ âm đầu.
- **Chế độ gõ nhanh (Quick Telex):** Gõ nhanh các cụm phụ âm ghép (`cc`=ch, `gg`=gi, `kk`=kh, `nn`=ng, `qq`=qu, `pp`=ph, `tt`=th).
- **Phím tắt chuyển đổi nhanh:** Cho phép tùy chỉnh phím tắt chuyển đổi Anh - Việt linh hoạt.
- **Hỗ trợ đầy đủ:** Hoạt động tốt trên Windows Vista trở lên, bao gồm các ứng dụng Metro trên Windows 10/11.

---

## 📥 Hướng dẫn cài đặt

1. Tải về file `.zip` phiên bản mới nhất từ mục [Releases](https://github.com/nguyenpcminh/xxkey/releases).
2. Giải nén tệp tin đã tải vào một thư mục bất kỳ trên máy tính.
3. Chạy file thực thi `XXKey.exe`. 
4. Khi chạy lần đầu, đồng ý cấp quyền chạy dưới quyền **Administrator** (chọn `Yes` trên hộp thoại xác nhận của Windows).
5. Biểu tượng chữ `V` (hoặc `E`) xuất hiện dưới khay hệ thống (System Tray). Bạn có thể nhấp đúp vào biểu tượng này để mở Bảng điều khiển và bắt đầu cấu hình.

---

## 🛠️ Hướng dẫn tự biên dịch (Build từ mã nguồn)

* **Yêu cầu:** Microsoft Visual Studio 2017 trở lên.
* **Cách build:**
  1. Clone mã nguồn của dự án.
  2. Mở solution trong thư mục `Sources/OpenKey/win32`.
  3. Chọn cấu hình build `Release` và thực hiện build project.

---

## 📜 Giấy phép & Cam kết bảo mật

XXKey cho Windows được phát hành hoàn toàn miễn phí dưới giấy phép **GPL (GNU General Public License)**. 
- Cam kết không chứa mã độc, keylogger, quảng cáo hay bất kỳ đoạn mã theo dõi nào.
- Mọi bản phân phối cải tiến cũng phải được mở mã nguồn và ghi nhận nguồn gốc từ XXKey / OpenKey.
