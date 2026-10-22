//! 32x32 piksel boyutunda uygulama genelindeki tüm buton ikonlarını üreten modül.
//! Dış dosya gerektirmeden pürüzsüz RGBA vektörel çizimler sunar.

use image::{ImageBuffer, Rgba, RgbaImage};

/// 32x32 Klasör İkonu (Kaynak Klasör)
pub fn create_folder_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let tab = Rgba([255, 152, 0, 255]);
    let back = Rgba([255, 179, 0, 255]);
    let front = Rgba([255, 202, 40, 255]);

    for y in 5..10 { for x in 4..14 { img.put_pixel(x, y, tab); } }
    for y in 9..27 { for x in 4..28 { img.put_pixel(x, y, back); } }
    for y in 12..28 { for x in 3..29 { img.put_pixel(x, y, front); } }
    img
}

/// 32x32 Hedef Klasör İkonu (Mavi Klasör + Aşağı Ok)
pub fn create_target_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let bg = Rgba([66, 165, 245, 255]);
    let front = Rgba([33, 150, 243, 255]);
    let white = Rgba([255, 255, 255, 255]);

    for y in 5..10 { for x in 4..14 { img.put_pixel(x, y, bg); } }
    for y in 9..27 { for x in 4..28 { img.put_pixel(x, y, bg); } }
    for y in 12..28 { for x in 3..29 { img.put_pixel(x, y, front); } }

    // Küçük beyaz aşağı ok
    for y in 15..22 { for x in 15..18 { img.put_pixel(x, y, white); } }
    for i in 0..4 {
        for x in (16 - i)..=(16 + i) {
            img.put_pixel(x as u32, (21 + i) as u32, white);
        }
    }
    img
}

/// 32x32 İşlemi Başlat İkonu (Yeşil Play Dairesi)
pub fn create_play_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let green = Rgba([76, 175, 80, 255]);
    let white = Rgba([255, 255, 255, 255]);

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            if dx * dx + dy * dy <= 14.0 * 14.0 {
                img.put_pixel(x, y, green);
            }
        }
    }

    for y in 10..=22 {
        let max_x = 12 + (6 - (y as i32 - 16).abs()) * 10 / 6;
        for x in 12..=max_x.min(23) {
            img.put_pixel(x as u32, y, white);
        }
    }
    img
}

/// 32x32 Dışa Aktar İkonu (Turkuaz İndirme Dairesi)
pub fn create_export_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let cyan = Rgba([38, 166, 154, 255]);
    let white = Rgba([255, 255, 255, 255]);

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            if dx * dx + dy * dy <= 14.0 * 14.0 {
                img.put_pixel(x, y, cyan);
            }
        }
    }

    // Ok gövdesi
    for y in 8..17 { for x in 15..18 { img.put_pixel(x, y, white); } }
    // Ok ucu
    for i in 0..5 {
        for x in (16 - i)..=(16 + i) {
            img.put_pixel(x as u32, (16 + i) as u32, white);
        }
    }
    // Taban
    for y in 23..25 { for x in 9..24 { img.put_pixel(x, y, white); } }
    img
}

/// 32x32 Yakınlaştır (Zoom In) İkonu (Büyüteç +)
pub fn create_zoom_in_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let blue = Rgba([41, 128, 185, 255]);
    let white = Rgba([255, 255, 255, 255]);

    // Büyüteç halkası
    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 14.0;
            let dy = y as f32 - 14.0;
            let d2 = dx * dx + dy * dy;
            if d2 <= 9.0 * 9.0 && d2 >= 6.0 * 6.0 {
                img.put_pixel(x, y, blue);
            }
        }
    }
    // Büyüteç sapı
    for i in 0..7 {
        img.put_pixel((20 + i) as u32, (20 + i) as u32, blue);
        img.put_pixel((21 + i) as u32, (20 + i) as u32, blue);
        img.put_pixel((20 + i) as u32, (21 + i) as u32, blue);
    }
    // Ortadaki artı (+) işareti
    for y in 10..=18 { for x in 13..=15 { img.put_pixel(x, y, white); } }
    for y in 13..=15 { for x in 10..=18 { img.put_pixel(x, y, white); } }
    img
}

/// 32x32 Uzaklaştır (Zoom Out) İkonu (Büyüteç -)
pub fn create_zoom_out_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let blue = Rgba([41, 128, 185, 255]);
    let white = Rgba([255, 255, 255, 255]);

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 14.0;
            let dy = y as f32 - 14.0;
            let d2 = dx * dx + dy * dy;
            if d2 <= 9.0 * 9.0 && d2 >= 6.0 * 6.0 {
                img.put_pixel(x, y, blue);
            }
        }
    }
    for i in 0..7 {
        img.put_pixel((20 + i) as u32, (20 + i) as u32, blue);
        img.put_pixel((21 + i) as u32, (20 + i) as u32, blue);
        img.put_pixel((20 + i) as u32, (21 + i) as u32, blue);
    }
    // Ortadaki eksi (-) işareti
    for y in 13..=15 { for x in 10..=18 { img.put_pixel(x, y, white); } }
    img
}

/// 32x32 Sıfırla (Reset) İkonu (Döngüsel Ok)
pub fn create_reset_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let gray = Rgba([149, 165, 166, 255]);

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            let d2 = dx * dx + dy * dy;
            if d2 <= 10.0 * 10.0 && d2 >= 7.0 * 7.0 {
                // Sağ alt çeyreği açık bırak
                if !(dx > 0.0 && dy > 0.0) {
                    img.put_pixel(x, y, gray);
                }
            }
        }
    }
    // Ok başı
    for y in 14..=20 {
        for x in 16..=22 {
            if (x as i32 - y as i32).abs() <= 2 {
                img.put_pixel(x, y, gray);
            }
        }
    }
    img
}

/// 32x32 Kapat İkonu (Kırmızı Çarpı Dairesi)
pub fn create_close_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let red = Rgba([231, 76, 60, 255]);
    let white = Rgba([255, 255, 255, 255]);

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            if dx * dx + dy * dy <= 12.0 * 12.0 {
                img.put_pixel(x, y, red);
            }
        }
    }

    // Çarpı işareti
    for i in 11..=21 {
        img.put_pixel(i, i, white);
        img.put_pixel(i + 1, i, white);
        img.put_pixel(i, 32 - i, white);
        img.put_pixel(i + 1, 32 - i, white);
    }
    img
}

/// 32x32 Yukarı Ok İkonu
pub fn create_arrow_up_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let color = Rgba([52, 152, 219, 255]);

    for y in 14..24 { for x in 14..18 { img.put_pixel(x, y, color); } }
    for i in 0..6 {
        for x in (16 - i)..=(15 + i) {
            img.put_pixel(x as u32, (14 - i) as u32, color);
        }
    }
    img
}

/// 32x32 Aşağı Ok İkonu
pub fn create_arrow_down_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let color = Rgba([52, 152, 219, 255]);

    for y in 8..18 { for x in 14..18 { img.put_pixel(x, y, color); } }
    for i in 0..6 {
        for x in (16 - i)..=(15 + i) {
            img.put_pixel(x as u32, (18 + i) as u32, color);
        }
    }
    img
}

/// 32x32 Sol Ok İkonu
pub fn create_arrow_left_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let color = Rgba([52, 152, 219, 255]);

    for y in 14..18 { for x in 14..24 { img.put_pixel(x, y, color); } }
    for i in 0..6 {
        for y in (16 - i)..=(15 + i) {
            img.put_pixel((14 - i) as u32, y as u32, color);
        }
    }
    img
}

/// 32x32 Sağ Ok İkonu
pub fn create_arrow_right_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let color = Rgba([52, 152, 219, 255]);

    for y in 14..18 { for x in 8..18 { img.put_pixel(x, y, color); } }
    for i in 0..6 {
        for y in (16 - i)..=(15 + i) {
            img.put_pixel((18 + i) as u32, y as u32, color);
        }
    }
    img
}

/// 32x32 Açık Klasör İkonu (Kaydedilen Klasörü Aç / Göster)
pub fn create_open_folder_icon_32() -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
    let tab = Rgba([245, 124, 0, 255]);
    let back = Rgba([255, 167, 38, 255]);
    let paper = Rgba([240, 244, 248, 255]);
    let paper_line = Rgba([144, 202, 249, 255]);
    let front = Rgba([255, 202, 40, 255]);
    let front_dark = Rgba([255, 179, 0, 255]);
    let badge_bg = Rgba([46, 125, 50, 255]); // Zümrüt yeşili rozet
    let white = Rgba([255, 255, 255, 255]);

    // Klasör Arka Sekmesi
    for y in 4..9 {
        for x in 4..14 {
            img.put_pixel(x, y, tab);
        }
    }
    // Klasör Arka Gövdesi
    for y in 8..23 {
        for x in 4..28 {
            img.put_pixel(x, y, back);
        }
    }

    // İçinden çıkan doküman / fotoğraf kağıdı (Açık olduğunu belli eden beyaz kağıt)
    for y in 7..17 {
        for x in 8..24 {
            img.put_pixel(x, y, paper);
        }
    }
    // Kağıt üzerindeki minik detay çizgileri
    for x in 10..22 {
        img.put_pixel(x, 10, paper_line);
        img.put_pixel(x, 13, paper_line);
    }

    // Açık Ön Kapak (Eğimli perspektif)
    for y in 14..28 {
        let offset = (y as i32 - 14) / 2;
        let start_x = (3 - offset).max(1) as u32;
        let end_x = (27 + offset).min(30) as u32;
        for x in start_x..=end_x {
            if y == 14 || x == start_x || x == end_x {
                img.put_pixel(x, y, front_dark);
            } else {
                img.put_pixel(x, y, front);
            }
        }
    }

    // Sağ alt köşede yeşil daire rozet (Klasörü Aç / Göster sembolü)
    for y in 18..30 {
        for x in 18..30 {
            let dx = x as f32 - 24.0;
            let dy = y as f32 - 24.0;
            if dx * dx + dy * dy <= 5.2 * 5.2 {
                img.put_pixel(x, y, badge_bg);
            }
        }
    }
    // Rozet içi yukarı/dışarı bakan beyaz ok
    for y in 22..27 {
        for x in 23..26 {
            img.put_pixel(x, y, white);
        }
    }
    for i in 0..4 {
        for x in (24 - i)..=(24 + i) {
            img.put_pixel(x as u32, (22 - i) as u32, white);
        }
    }

    img
}

