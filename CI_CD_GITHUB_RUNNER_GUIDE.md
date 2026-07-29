# Hướng Dẫn Setup Tự Động Đẩy Lệnh Compile Về Máy PC Nhà (Self-Hosted Runner)

Tài liệu này hướng dẫn cách ép hệ thống GitHub Actions phân luồng tác vụ nặng (compile file `.exe` cho Tauri) trượt thẳng từ Cloud rớt xuống máy PC cấu hình cao ở nhà bạn để xử lý, nhằm tối ưu hóa CPU tự có thay vì dùng Server mặc định của Github.

## Bước 1: Setup Máy Chủ Chạy Ngầm (Cài Trên Lúc Ở PC PC Mạnh)
1. Mở trình duyệt trên máy PC ở nhà, truy cập vào Repository của project này trên Github.
2. Chuyển sang Tab **Settings** của Repo > Ở menu phía bên tay trái, tìm chữ **Actions** > Nhấn vào dòng **Runners**.
3. Nhấp vào nút màu xanh lá cây cực to nằm ở góc trên cùng **New self-hosted runner**.
4. Màn hình sẽ hiện ra bảng chọn. Bạn chọn Hệ Điều Hành là **Windows** và kiến trúc là **`x64`**.
5. Github sẽ đưa ra cho bạn danh sách khoảng 5-6 dòng lệnh liên tiếp (Ví dụ: `Invoke-WebRequest -Uri https://github.com/actions...`). Bạn chỉ cần mở một bảng PowerShell, Copy Dán và chạy y chang theo đúng thứ tự các lệnh đó trên PC là hoàn thành! Máy tính nhà bạn đã kết nối với mã nguồn trên Cloud.
6. (Tùy chọn) Chạy thêm cụm `.\svc.sh install` để biến cái công cụ đó thành một dịch vụ ẩn chạy tàng hình cùng với Windows mỗi khi khởi động PC!

## Bước 2: Áp Dụng Lệnh Nhánh Vào `.github/workflows/release.yml`
Mở file cấu hình CI/CD đã có là `.github/workflows/release.yml` ra. Tìm đến dòng ở Job hiện tại đang ghi cứng ngắc thông số như thế này:

```yaml
jobs:
  release:
    runs-on: windows-latest
```

Xoá chữ `windows-latest` đi và thay bằng đoạn mã IF logic quét nội dung commit này:

```yaml
jobs:
  release:
    runs-on: ${{ contains(github.event.head_commit.message, '[local]') && 'self-hosted' || 'windows-latest' }}
```

## Bước 3: Hưởng Thành Quả Của Ma Thuật Server!
Từ nay trở đi, bạn được quyền tự do điều khiển "Bóc lột cấu hình PC nhà" hoặc "Dùng ké Cloud của Github Cloud":

- 🟢 Khi đang đi ra ngoài và dùng **Máy PC / Laptop Yếu**:
  Bạn cứ commit code bình thường như không có gì xảy ra `git commit -m "Sửa xong giao diện"`. Đoạn script sẽ check lỗi, không thấy chữ `[local]` thì lập lệnh cho Server Microsoft bung nguồn lực ra compile tự động.
  
- 🔴 Khi đang ngồi máy ở Công Ty muốn xuất file và muốn **Máy Hộ Gia Đình biên dịch file thay cho Server Github quá chậm chạp**:
  Bạn đẩy lệnh đi đính kèm thêm từ khóa `[local]`:
  `git commit -m "Viết xong backend, chuẩn bị release! [local]"` 
  
  Mọi thứ lập tức thay đổi: Hệ thống của Github đọc được cụm `[local]`, nó sẽ vứt nhiệm vụ sang một bên, chuyển ngay tín hiệu ping về thẳng mạng lan gia đình, thức tỉnh cái máy tính siêu mạnh ở nhà đang chạy ẩn đằng sau! Lệnh compile khổng lồ sẽ được hệ thống PC dồi dào CPU ở nhà gánh team 100%, nhanh chóng zip được file `.exe` cuối cùng rồi đẩy trả hàng lên giao diện Release trên mạng xã hội cho bạn!
