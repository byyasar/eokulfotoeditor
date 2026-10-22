# Okul Fotoğraf Editörü & Otomatik Numaralandırıcı (Rust & egui)

Milli Eğitim Bakanlığı (MEB) **e-Okul** fotoğraf yükleme standartlarına tam uyumlu, sınıf/öğrenci toplu çekim fotoğraflarını yapay zeka/klasik görüntü işleme algoritmalarıyla otomatik olarak tespit eden, kırpan, numaralandıran ve boyutlandıran masaüstü GUI uygulaması.

---

## 🚀 Özellikler

- **e-Okul Standartları Uyumlu:**
  - Tam **133 × 171 piksel** en boy oranı ve çözünürlük.
  - MEB dosya boyutu alt ve üst sınırı garantisi: **20 KB - 150 KB JPEG**.
  - Minimum boyut kuralı (20 KB altı olmama) için akıllı JPEG COM marker dolgusu.
- **Akıllı Yüz ve Tahta Tespiti:**
  - Cilt tonu segmentasyonu ve morfolojik filtrelerle vesikalık kadrajını otomatik bulma.
  - Sınıf tahtasındaki öğrenci numaralarını tespit etme ve segmentleme.
- **İnteraktif Görsel İnceleme & Düzenleme (GUI):**
  - **Zoom & Pan (Yakınlaştırma ve Kaydırma):** Hem tahta inceleme hem de vesikalık kırpma ekranlarında mouse tekerleğiyle yakınlaştırma ve sürükleme desteği.
  - **Kadraj Kaydırma Butonları:** İstenen yöne piksel hassasiyetinde kadraj ayarlama (32x32 vektörel ikonlar).
  - **Görsel Seçim / Çıkarma:** Sol üst onay kutusu (checkbox) ile çıktıya dahil etme/etmeme; sağ üst kapatma butonu ile listeden kalıcı çıkarma.
  - **Dinamik Kalite & Boyut Ayarı:** Hedef JPEG kalitesi (%40 - %100) ve min/max KB sınırlarını arayüzden belirleyebilme.
- **Hafif ve Hızlı:**
  - Rust ve `eframe`/`egui` ile sıfır harici Python/OpenCV bağımlılığı, ultra hızlı çalıştırma.

---

## 🛠️ Kurulum ve Çalıştırma

Projeyi derlemek ve çalıştırmak için sisteminizde [Rust](https://rustup.rs/) kurulu olmalıdır:

```bash
# Projeyi derleyin ve çalıştırın
cargo run --release
```

---

## 📂 Proje Yapısı

- `src/main.rs`: `eframe`/`egui` kullanıcı arayüzü, görsel önizlemeler, zoom/pan ve kontrol paneli.
- `src/image_processor.rs`: EXIF yönlendirme düzeltme, yüz tespiti, otomatik kırpma, Lanczos3 ölçeklendirme ve JPEG kodlayıcı.
- `src/ocr.rs`: Tahta üzerindeki rakamları ayrıştırma ve okuma algoritmaları.
- `src/icons.rs`: 32x32 piksel vektörel UI ikonları.


## 🛠️ Kurulum ve Çalıştırma

Projeyi derlemek ve çalıştırmak için sisteminizde [Rust](https://rustup.rs/) kurulu olmalıdır:

```bash
# Projeyi derleyin ve çalıştırın
cargo run --release
```

---