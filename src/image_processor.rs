//! Görüntü işleme pipeline'ı: EXIF yönelimi düzeltme, yüz tespiti ve vesikalık auto-crop,
//! 133x171 Lanczos3 ölçekleme ve 20-50 KB e-Okul JPEG kodlama.

use anyhow::{anyhow, Context, Result};
use image::{imageops, DynamicImage, GenericImageView, GrayImage};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// EXIF Orientation etiketini okuyarak fotoğrafı doğru yönüne döndürür.
pub fn load_and_fix_orientation<P: AsRef<Path>>(path: P) -> Result<DynamicImage> {
    let path_ref = path.as_ref();
    let file = File::open(path_ref).with_context(|| format!("Dosya açılamadı: {:?}", path_ref))?;
    let mut bufreader = std::io::BufReader::new(&file);

    let exifreader = exif::Reader::new();
    let orientation = exifreader
        .read_from_container(&mut bufreader)
        .ok()
        .and_then(|exif_data| {
            exif_data
                .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
        })
        .unwrap_or(1);

    let mut img = image::open(path_ref).with_context(|| format!("Görsel yüklenemedi: {:?}", path_ref))?;

    img = match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    };

    Ok(img)
}

/// YCbCr ten rengi piksel kontrolü (Klasik yüz tespiti)
#[inline]
fn is_skin_pixel(r: u8, g: u8, b: u8) -> bool {
    let r_f = r as f32;
    let g_f = g as f32;
    let b_f = b as f32;

    let y = 0.299 * r_f + 0.587 * g_f + 0.114 * b_f;
    let cb = 128.0 - 0.168736 * r_f - 0.331264 * g_f + 0.5 * b_f;
    let cr = 128.0 + 0.5 * r_f - 0.418688 * g_f - 0.081312 * b_f;

    (60.0..=245.0).contains(&y) && (77.0..=133.0).contains(&cb) && (133.0..=175.0).contains(&cr)
}

/// Yüz tespiti ve e-Okul standartlarında (133:171 oranında) vesikalık kadraj kutusunu hesaplar.
/// Tahtadaki yazılar kırpma alanı dışında kalacak şekilde kadrajı yüze ve omuzlara odaklar.
pub fn detect_portrait_crop(img: &DynamicImage) -> CropRect {
    let (width, height) = img.dimensions();

    let scale = (width.max(height) as f32 / 360.0).max(1.0);
    let sw = ((width as f32) / scale).round() as u32;
    let sh = ((height as f32) / scale).round() as u32;

    let small_img = img.thumbnail_exact(sw, sh);
    let mut skin_mask = GrayImage::new(sw, sh);

    for y in 0..sh {
        for x in 0..sw {
            let p = small_img.get_pixel(x, y);
            if is_skin_pixel(p[0], p[1], p[2]) {
                skin_mask.put_pixel(x, y, image::Luma([255]));
            }
        }
    }

    let win_w = (sw / 6).max(8);
    let win_h = (sh / 6).max(8);

    let mut max_density = 0u32;
    let mut best_cx = sw / 2;
    let mut best_cy = (sh as f32 * 0.38) as u32;

    let start_y = (sh as f32 * 0.08) as u32;
    let end_y = (sh as f32 * 0.65).min(sh as f32 - win_h as f32) as u32;
    let start_x = (sw as f32 * 0.15) as u32;
    let end_x = (sw as f32 * 0.85).min(sw as f32 - win_w as f32) as u32;

    for y in (start_y..end_y).step_by(2) {
        for x in (start_x..end_x).step_by(2) {
            let mut density = 0u32;
            for wy in 0..win_h {
                for wx in 0..win_w {
                    if skin_mask.get_pixel(x + wx, y + wy)[0] == 255 {
                        density += 1;
                    }
                }
            }
            if density > max_density {
                max_density = density;
                best_cx = x + win_w / 2;
                best_cy = y + win_h / 2;
            }
        }
    }

    let orig_cx = (best_cx as f32 * scale).round() as u32;
    let orig_cy = (best_cy as f32 * scale).round() as u32;

    // 133:171 oranında vesikalık boyut hesaplama
    let estimated_face_h = ((win_h as f32 * scale) * 1.5).max(height as f32 * 0.28);
    let crop_h = (estimated_face_h * 2.2).min(height as f32 * 0.85) as u32;
    let crop_w = (crop_h as f32 * (133.0 / 171.0)).round() as u32;

    let start_crop_y = orig_cy.saturating_sub((crop_h as f32 * 0.35) as u32);
    let start_crop_x = orig_cx.saturating_sub(crop_w / 2);

    let final_x = start_crop_x.min(width.saturating_sub(crop_w));
    let final_y = start_crop_y.min(height.saturating_sub(crop_h));
    let final_w = crop_w.min(width - final_x);
    let final_h = crop_h.min(height - final_y);

    CropRect {
        x: final_x,
        y: final_y,
        width: final_w,
        height: final_h,
    }
}

/// Vesikalık kadrajına göre kırpar ve tam 133 x 171 piksele ölçekler.
pub fn crop_and_scale(img: &DynamicImage, rect: &CropRect) -> DynamicImage {
    let cropped = img.crop_imm(rect.x, rect.y, rect.width.max(1), rect.height.max(1));
    let resized = imageops::resize(&cropped.to_rgb8(), 133, 171, imageops::FilterType::Lanczos3);
    DynamicImage::ImageRgb8(resized)
}

/// Minimum dosya boyutu şartını garantiye almak için JPEG Comment segmenti ekler.
pub fn ensure_min_file_size(jpeg_bytes: Vec<u8>, min_bytes: usize) -> Result<Vec<u8>> {
    let current_len = jpeg_bytes.len();
    if current_len >= min_bytes {
        return Ok(jpeg_bytes);
    }

    if jpeg_bytes.len() < 2 || jpeg_bytes[0] != 0xFF || jpeg_bytes[1] != 0xD8 {
        return Err(anyhow!("Geçersiz JPEG verisi: SOI eksik"));
    }

    let needed = min_bytes.saturating_sub(current_len);
    if needed <= 4 {
        return Ok(jpeg_bytes);
    }

    let payload_len = needed - 4;
    let segment_len = (payload_len + 2) as u16;
    let len_bytes = segment_len.to_be_bytes();

    let mut padded = Vec::with_capacity(current_len + needed);
    padded.extend_from_slice(&jpeg_bytes[0..2]); // SOI (0xFF 0xD8)
    padded.push(0xFF);
    padded.push(0xFE); // COM marker
    padded.extend_from_slice(&len_bytes);

    let tag = b"e-Okul Foto Padding ";
    let mut written = 0;
    while written < payload_len && written < tag.len() {
        padded.push(tag[written]);
        written += 1;
    }
    for _ in written..payload_len {
        padded.push(b' ');
    }

    padded.extend_from_slice(&jpeg_bytes[2..]);
    Ok(padded)
}

/// 133x171 boyutundaki görseli kullanıcının belirlediği kalite (%20 - %100) ve
/// min_kb / max_kb sınırlarında JPEG olarak kodlar.
pub fn encode_custom_jpeg(
    img: &DynamicImage,
    mut quality: u8,
    min_kb: usize,
    max_kb: usize,
) -> Result<Vec<u8>> {
    let rgb = img.to_rgb8();
    let max_bytes = max_kb * 1024;
    let min_bytes = min_kb * 1024;

    let mut buf = Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.encode(rgb.as_raw(), 133, 171, image::ExtendedColorType::Rgb8)?;

    let mut bytes = buf.into_inner();
    while bytes.len() > max_bytes && quality > 20 {
        quality = quality.saturating_sub(5);
        let mut retry_buf = Cursor::new(Vec::new());
        let mut retry_enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut retry_buf, quality);
        retry_enc.encode(rgb.as_raw(), 133, 171, image::ExtendedColorType::Rgb8)?;
        bytes = retry_buf.into_inner();
    }

    if bytes.len() < min_bytes {
        bytes = ensure_min_file_size(bytes, min_bytes)?;
    }

    Ok(bytes)
}

/// Varsayılan e-Okul kodlayıcı (%95 kalite, 20 KB min, 150 KB max)
#[allow(dead_code)]
pub fn encode_eokul_jpeg(img: &DynamicImage) -> Result<Vec<u8>> {
    encode_custom_jpeg(img, 95, 20, 150)
}
