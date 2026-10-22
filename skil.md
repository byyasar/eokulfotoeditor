# Proje Tanımı: Rust ile e-Okul Öğrenci Fotoğrafı Hazırlama ve Otomatik İsimlendirme Uygulaması

Rust programlama dilini kullanarak masaüstü grafik arayüzüne (GUI) sahip, toplu fotoğraf işleme ve OCR (Optik Karakter Tanıma) tabanlı bir uygulama geliştirmeni istiyorum. 

## 1. Kullanılacak Teknoloji Yığını (Tech Stack)
- **Dil:** Rust (Edition 2021)
- **GUI Framework:** `eframe` / `egui` (Hızlı, tepkisel arayüz ve kolay görsel önizleme yönetimi için).
- **Dosya Seçimi:** `rfd` (Native klasör seçme ve kaydetme diyalogları için).
- **Görüntü İşleme ve Yüz Tespiti:** `image`, `imageproc` ve yüz tespiti için `rustface` (veya sistemde sorunsuz çalışacak hafif bir ONNX/OpenCV alternatifi).
- **OCR (Yazı Okuma):** `leptess` / `tesseract` (veya saf Rust tabanlı `ocrs` kütüphanesi) - Öğrencinin arkasındaki tahtada yazan rakamları (öğrenci numarasını) okumak için.
- **Asenkron / Многоlu İşlem:** Arayüzün donmaması için görüntü işleme aşamalarında `rayon` veya `tokio` / thread-channel yapısı.

## 2. Temel İş Mantığı (Görüntü İşleme Pipeline'ı)
Seçilen klasördeki telefonla çekilmiş her bir fotoğraf için sırasıyla şu işlemler uygulanmalıdır:
1. **EXIF Yönelim Düzeltmesi:** Telefon fotoğraflarındaki EXIF orientation verisini okuyup fotoğrafı doğru açıyla düzelt.
2. **OCR ile Öğrenci Numarası Okuma:** Görüntünün tamamında veya arka plan (tahta) bölgesinde OCR çalıştırarak sadece rakamlardan (0-9) oluşan öğrenci numarasını tespit et. Eğer birden fazla sayı varsa veya okunamazsa varsayılan olarak `belirsiz_X` ata.
3. **Yüz Tespiti ve Vesikalık Kırpma (Auto-Crop):** Fotoğraftaki öğrencinin yüzünü tespit et. Yüzü merkeze alarak, başın üstünde biraz boşluk ve omuzları kapsayacak şekilde **133:171 (veya yaklaşık 3:4)** en-boy oranında dikey bir dikdörtgen olarak kırp. (Tahtadaki yazılar kırpma alanı dışında kalacak şekilde kadrajı yüze odakla).
4. **e-Okul Standartlarına Boyutlandırma ve Sıkıştırma:** 
   - Kırpılan görüntüyü tam olarak **133x171 piksel** boyutuna yeniden boyutlandır (`FilterType::Lanczos3` kullanarak).
   - Çıktı formatı `.jpg` olmalı ve dosya boyutu **15 KB ile 150 KB arasında** (tercihen 20-50 KB civarında) olacak şekilde JPEG kalitesini (quality) dinamik olarak ayarla.

## 3. Kullanıcı Arayüzü (GUI) Gereksinimleri
Uygulama ekranı iki ana bölümden (Sol Panel ve Sağ Önizleme Izgarası) oluşmalıdır:
- **Üst/Sol Kontrol Paneli:**
  - "Kaynak Klasör Seç" butonu (Telefon fotoğraflarının olduğu klasör).
  - "Hedef Klasör Seç" butonu (İşlenmiş e-Okul fotoğraflarının kaydedileceği klasör).
  - "İşlemi Başlat" butonu ve ilerleme çubuğu (`ProgressBar`).
  - "Tümünü Kaydet / Dışa Aktar" butonu.
- **Merkez Önizleme Alanı (Grid View):**
  - İşlenen fotoğraflar kaydırılabilir bir ızgara (`ScrollArea` + Grid) içinde yan yana/alt alta kartlar halinde gösterilmelidir.
  - Her kartta: **133x171 boyutundaki işlenmiş küçük önizleme resmi**, tahmini dosya boyutu (KB cinsinden) ve altında OCR ile okunan **Öğrenci Numarası (dosya adı)** yer almalıdır.
- **Tıklama ve İsim Değiştirme (Kritik Özellik):**
  - Önizleme ekranındaki herhangi bir resme veya altındaki isme tıklandığında, o resmin ismi anında düzenlenebilir bir metin kutusuna (`TextEdit`) dönüşmeli veya seçili resim için sağ tarafta bir detay paneli açılarak yeni öğrenci numarası elle girilebilmelidir.
  - Ayrıca kullanıcı isterse orijinal fotoğrafı küçük bir pencerede/ipucunda (tooltip veya yan görsel) görüp tahtadaki numarayı kendi gözüyle kontrol edebilmelidir (OCR yanlış okumuşsa düzeltmek için).

## 4. İstenen Çıktılar
1. Tüm bağımlılıkları içeren eksiksiz `Cargo.toml` dosyası.
2. Modüler ve temiz yazılmış `src/main.rs` (gerekirse `image_processor.rs`, `ocr.rs`, `ui.rs` olarak ayrılmış) kaynak kodları.
3. Harici kütüphaneler (ör. Tesseract veya yüz tanıma model dosyası) gerekiyorsa bunların proje dizinine nasıl ekleneceğini anlatan kısa kurulum adımları.