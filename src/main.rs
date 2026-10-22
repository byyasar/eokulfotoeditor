#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icons;
mod image_processor;
mod ocr;

use eframe::egui::{self, Color32, RichText, TextureHandle, Vec2};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use image_processor::{
    crop_and_scale, detect_portrait_crop, encode_custom_jpeg, load_and_fix_orientation, CropRect,
};
use ocr::read_student_number;

/// İşlenen her bir öğrenci fotoğrafının durumu
#[derive(Clone)]
struct ProcessedPhoto {
    original_path: PathBuf,
    student_number: String,
    jpeg_bytes: Vec<u8>,
    file_size_kb: f64,
    preview_texture: Option<TextureHandle>,
    original_preview_texture: Option<TextureHandle>,
    // Yeniden kadrajlama ve düzenleme için veriler
    original_image: Option<image::DynamicImage>,
    current_crop: CropRect,
    // Çıktıda yer alıp almayacağı (Checkbox)
    is_included: bool,
}

enum WorkerMessage {
    Progress { current: usize, total: usize, file_name: String },
    PhotoProcessed {
        original_path: PathBuf,
        student_number: String,
        jpeg_bytes: Vec<u8>,
        file_size_kb: f64,
        preview_rgb: image::RgbImage,
        orig_preview_rgb: image::RgbImage,
        orig_dyn_img: image::DynamicImage,
        crop_rect: CropRect,
    },
    Done,
}



#[derive(Clone)]
struct AppIcons {
    source: TextureHandle,
    target: TextureHandle,
    play: TextureHandle,
    export: TextureHandle,
    open_folder: TextureHandle,
    zoom_in: TextureHandle,
    zoom_out: TextureHandle,
    reset: TextureHandle,
    close: TextureHandle,
    arrow_up: TextureHandle,
    arrow_down: TextureHandle,
    arrow_left: TextureHandle,
    arrow_right: TextureHandle,
}

impl AppIcons {
    fn load(ctx: &egui::Context) -> Self {
        let load_32 = |name, img: image::RgbaImage| {
            let color = egui::ColorImage::from_rgba_unmultiplied([32, 32], img.as_raw());
            ctx.load_texture(name, color, egui::TextureOptions::LINEAR)
        };

        Self {
            source: load_32("icon_source", icons::create_folder_icon_32()),
            target: load_32("icon_target", icons::create_target_icon_32()),
            play: load_32("icon_play", icons::create_play_icon_32()),
            export: load_32("icon_export", icons::create_export_icon_32()),
            open_folder: load_32("icon_open_folder", icons::create_open_folder_icon_32()),
            zoom_in: load_32("icon_zoom_in", icons::create_zoom_in_icon_32()),
            zoom_out: load_32("icon_zoom_out", icons::create_zoom_out_icon_32()),
            reset: load_32("icon_reset", icons::create_reset_icon_32()),
            close: load_32("icon_close", icons::create_close_icon_32()),
            arrow_up: load_32("icon_arrow_up", icons::create_arrow_up_32()),
            arrow_down: load_32("icon_arrow_down", icons::create_arrow_down_32()),
            arrow_left: load_32("icon_arrow_left", icons::create_arrow_left_32()),
            arrow_right: load_32("icon_arrow_right", icons::create_arrow_right_32()),
        }
    }
}

/// Kaydedilen klasörü işletim sisteminin dosya gezgininde açıp gösterir
fn open_folder_in_explorer(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }
}

/// Uygulama arayüzünü derin siyah (black) modern temaya geçirir
fn apply_black_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Derin siyah arka planlar
    visuals.dark_mode = true;
    visuals.panel_fill = Color32::from_rgb(14, 14, 16);
    visuals.window_fill = Color32::from_rgb(18, 18, 20);
    visuals.extreme_bg_color = Color32::from_rgb(10, 10, 12);
    visuals.faint_bg_color = Color32::from_rgb(22, 22, 26);
    visuals.code_bg_color = Color32::from_rgb(20, 20, 24);

    // Etkileşim bileşenleri ve renk stilleri
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(22, 22, 26);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(42, 42, 48));
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(210, 210, 210));

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(28, 28, 34);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(52, 52, 60));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(225, 225, 225));

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(44, 44, 52);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(85, 85, 95));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, Color32::WHITE);

    visuals.widgets.active.bg_fill = Color32::from_rgb(58, 58, 68);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(120, 120, 140));
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, Color32::WHITE);

    visuals.widgets.open.bg_fill = Color32::from_rgb(32, 32, 38);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 70));

    // Seçim ve kenarlıklar
    visuals.selection.bg_fill = Color32::from_rgb(26, 80, 145);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(80, 170, 255));
    visuals.window_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 45, 52));

    ctx.set_visuals(visuals);
}

struct OkulFotoApp {
    source_folder: Option<PathBuf>,
    target_folder: Option<PathBuf>,
    last_exported_folder: Option<PathBuf>,
    theme_applied: bool,
    is_processing: bool,
    progress_current: usize,
    progress_total: usize,
    status_text: String,

    photos: Vec<ProcessedPhoto>,
    selected_index: Option<usize>,

    // Detay Paneli: Zoom ve Pan (Kaydırma) Kontrolleri
    zoom_level: f32,
    pan_offset: Vec2,

    // Fotoğraf Kalitesi ve Dosya Boyutu Sınırları
    jpeg_quality: u8,
    min_kb: usize,
    max_kb: usize,

    // 32x32 Buton İkonları
    icons: Option<AppIcons>,

    // Thread iletişim
    rx: Option<Receiver<WorkerMessage>>,
}

impl Default for OkulFotoApp {
    fn default() -> Self {
        Self {
            source_folder: None,
            target_folder: None,
            last_exported_folder: None,
            theme_applied: false,
            is_processing: false,
            progress_current: 0,
            progress_total: 0,
            status_text: "Lütfen Kaynak ve Hedef Klasörleri seçin.".to_string(),
            photos: Vec::new(),
            selected_index: None,
            zoom_level: 1.0,
            pan_offset: Vec2::ZERO,
            jpeg_quality: 95,
            min_kb: 20,
            max_kb: 150,
            icons: None,
            rx: None,
        }
    }
}

impl OkulFotoApp {
    fn start_processing(&mut self) {
        let src = match &self.source_folder {
            Some(p) => p.clone(),
            None => {
                self.status_text = "⚠️ Lütfen önce bir Kaynak Klasör seçin!".to_string();
                return;
            }
        };

        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ext_lower == "jpg" || ext_lower == "jpeg" || ext_lower == "png" {
                            files.push(path);
                        }
                    }
                }
            }
        }

        files.sort();

        if files.is_empty() {
            self.status_text = "⚠️ Kaynak klasörde fotoğraf bulunamadı (.jpg, .jpeg, .png).".to_string();
            return;
        }

        self.photos.clear();
        self.selected_index = None;
        self.is_processing = true;
        self.progress_current = 0;
        self.progress_total = files.len();
        self.status_text = format!("İşleme başladı... (0 / {})", files.len());

        let (tx, rx): (Sender<WorkerMessage>, Receiver<WorkerMessage>) = channel();
        self.rx = Some(rx);

        let quality = self.jpeg_quality;
        let min_k = self.min_kb;
        let max_k = self.max_kb;

        thread::spawn(move || {
            let total = files.len();
            for (i, file_path) in files.into_iter().enumerate() {
                let file_name = file_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                let _ = tx.send(WorkerMessage::Progress {
                    current: i + 1,
                    total,
                    file_name: file_name.clone(),
                });

                // Pipeline adımları
                let oriented_img = match load_and_fix_orientation(&file_path) {
                    Ok(img) => img,
                    Err(_) => continue,
                };

                // OCR ile tahtadaki numara
                let detected_num = read_student_number(&oriented_img)
                    .unwrap_or_else(|| format!("belirsiz_{}", i + 1));

                // Yüz tespiti ve vesikalık kırpma
                let crop_rect = detect_portrait_crop(&oriented_img);
                let scaled_img = crop_and_scale(&oriented_img, &crop_rect);

                // JPEG özel kalite ve min/max KB ile kodlama
                let jpeg_bytes = match encode_custom_jpeg(&scaled_img, quality, min_k, max_k) {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                let file_size_kb = jpeg_bytes.len() as f64 / 1024.0;
                let preview_rgb = scaled_img.to_rgb8();

                // Detay panelinde zoom ve inceleme için yüksek kaliteli önizleme
                let orig_thumb = oriented_img.thumbnail(800, 800).to_rgb8();

                let _ = tx.send(WorkerMessage::PhotoProcessed {
                    original_path: file_path,
                    student_number: detected_num,
                    jpeg_bytes,
                    file_size_kb,
                    preview_rgb,
                    orig_preview_rgb: orig_thumb,
                    orig_dyn_img: oriented_img,
                    crop_rect,
                });
            }

            let _ = tx.send(WorkerMessage::Done);
        });
    }

    /// Kalite veya min/max KB değiştiğinde fotoğrafları yeniden kodlar
    fn reencode_all(&mut self) {
        let q = self.jpeg_quality;
        let min_k = self.min_kb;
        let max_k = self.max_kb;

        for photo in &mut self.photos {
            if let Some(ref orig_img) = photo.original_image {
                let scaled = crop_and_scale(orig_img, &photo.current_crop);
                if let Ok(bytes) = encode_custom_jpeg(&scaled, q, min_k, max_k) {
                    photo.file_size_kb = bytes.len() as f64 / 1024.0;
                    photo.jpeg_bytes = bytes;
                }
            }
        }
    }

    /// Seçili fotoğrafın kadrajını elle kaydırıp e-Okul görselini anında yeniden üretir
    fn adjust_crop_and_update(&mut self, ctx: &egui::Context, dx: i32, dy: i32, scale_delta: f32) {
        if let Some(idx) = self.selected_index {
            if idx < self.photos.len() {
                let photo = &mut self.photos[idx];
                if let Some(ref orig_img) = photo.original_image {
                    let (img_w, img_h) = (orig_img.width(), orig_img.height());

                    // Boyut ölçekleme
                    let mut new_w = (photo.current_crop.width as f32 * scale_delta).round() as u32;
                    let mut new_h = (new_w as f32 * (171.0 / 133.0)).round() as u32;

                    new_w = new_w.clamp(50, img_w);
                    new_h = new_h.clamp(64, img_h);

                    // Konum kaydırma (dx, dy)
                    let new_x = (photo.current_crop.x as i32 + dx)
                        .clamp(0, (img_w.saturating_sub(new_w)) as i32) as u32;
                    let new_y = (photo.current_crop.y as i32 + dy)
                        .clamp(0, (img_h.saturating_sub(new_h)) as i32) as u32;

                    photo.current_crop = CropRect {
                        x: new_x,
                        y: new_y,
                        width: new_w,
                        height: new_h,
                    };

                    // Yeniden ölçekle ve seçili kalite/boyut ile JPEG üret
                    let scaled = crop_and_scale(orig_img, &photo.current_crop);
                    if let Ok(bytes) = encode_custom_jpeg(&scaled, self.jpeg_quality, self.min_kb, self.max_kb) {
                        photo.file_size_kb = bytes.len() as f64 / 1024.0;
                        photo.jpeg_bytes = bytes;

                        let preview_rgb = scaled.to_rgb8();
                        let (w, h) = preview_rgb.dimensions();
                        let color_image = egui::ColorImage::from_rgb([w as usize, h as usize], preview_rgb.as_raw());
                        photo.preview_texture = Some(ctx.load_texture(
                            format!("preview_{}", idx),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                }
            }
        }
    }

    fn export_all(&mut self) {
        let dest = match &self.target_folder {
            Some(p) => p.clone(),
            None => {
                self.status_text = "⚠️ Lütfen önce bir Hedef Klasör seçin!".to_string();
                return;
            }
        };

        if self.photos.is_empty() {
            self.status_text = "⚠️ Dışa aktarılacak işlenmiş fotoğraf yok!".to_string();
            return;
        }

        let mut saved_count = 0;
        let mut name_counts = std::collections::HashMap::new();

        for photo in &self.photos {
            if !photo.is_included {
                continue; // Seçili olmayan fotoğraflar çıktıya dahil edilmez
            }

            let base_num = if photo.student_number.trim().is_empty() {
                "isimsiz".to_string()
            } else {
                photo.student_number.trim().to_string()
            };

            let count = name_counts.entry(base_num.clone()).or_insert(0);
            *count += 1;

            let file_name = if *count == 1 {
                format!("{}.jpg", base_num)
            } else {
                format!("{}_dup{}.jpg", base_num, *count - 1)
            };

            let out_path = dest.join(&file_name);
            if fs::write(&out_path, &photo.jpeg_bytes).is_ok() {
                saved_count += 1;
            }
        }

        if saved_count > 0 {
            self.last_exported_folder = Some(dest.clone());
            self.status_text = format!("✅ {} adet e-Okul fotoğrafı kaydedildi -> {:?}", saved_count, dest);
        } else {
            self.status_text = "⚠️ Dışa aktarılacak seçili fotoğraf kaydedilemedi!".to_string();
        }
    }
}

impl eframe::App for OkulFotoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            apply_black_theme(ctx);
            self.theme_applied = true;
        }

        // Arka plan iş parçacığından gelen verileri al
        if let Some(ref rx) = self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    WorkerMessage::Progress { current, total, file_name } => {
                        self.progress_current = current;
                        self.progress_total = total;
                        self.status_text = format!("İşleniyor ({}/{}): {}", current, total, file_name);
                    }
                    WorkerMessage::PhotoProcessed {
                        original_path,
                        student_number,
                        jpeg_bytes,
                        file_size_kb,
                        preview_rgb,
                        orig_preview_rgb,
                        orig_dyn_img,
                        crop_rect,
                    } => {
                        let (w, h) = preview_rgb.dimensions();
                        let color_image = egui::ColorImage::from_rgb(
                            [w as usize, h as usize],
                            preview_rgb.as_raw(),
                        );
                        let texture = ctx.load_texture(
                            format!("preview_{}", self.photos.len()),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );

                        let (ow, oh) = orig_preview_rgb.dimensions();
                        let orig_color_image = egui::ColorImage::from_rgb(
                            [ow as usize, oh as usize],
                            orig_preview_rgb.as_raw(),
                        );
                        let orig_texture = ctx.load_texture(
                            format!("orig_{}", self.photos.len()),
                            orig_color_image,
                            egui::TextureOptions::LINEAR,
                        );

                        self.photos.push(ProcessedPhoto {
                            original_path,
                            student_number,
                            jpeg_bytes,
                            file_size_kb,
                            preview_texture: Some(texture),
                            original_preview_texture: Some(orig_texture),
                            original_image: Some(orig_dyn_img),
                            current_crop: crop_rect,
                            is_included: true,
                        });
                    }
                    WorkerMessage::Done => {
                        self.is_processing = false;
                        self.status_text = format!(
                            "🎉 İşlem tamamlandı! {} fotoğraf hazır. Tıklayarak Zoom/Pan yapabilir ve düzenleyebilirsiniz.",
                            self.photos.len()
                        );
                    }
                }
            }
        }

        // 32x32 Piksel İkon Texture'larını bir kez yükle
        if self.icons.is_none() {
            self.icons = Some(AppIcons::load(ctx));
        }
        let icons = self.icons.clone().unwrap();

        if self.is_processing {
            ctx.request_repaint();
        }

        let mut should_reencode = false;

        // --- ÜST PANEL: Klasörler ve Butonlar ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🎓 e-Okul Fotoğraf Editörü & Otomatik Numaralandırıcı").size(18.0).strong());
                //ui.label(RichText::new("(133x171 • 20-50 KB • 32x32 İkon Desteği)").italics().color(Color32::GRAY));
            });
            ui.add_space(6.0);

            // 1. Satır: Kaynak ve Hedef Klasör Seçimi (32x32 İkonlu Butonlar)
            ui.horizontal(|ui| {
                let btn = egui::Button::image_and_text(&icons.source, RichText::new(" Kaynak Klasör").size(13.0).strong())
                    .min_size(Vec2::new(150.0, 42.0));
                if ui.add(btn).clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.source_folder = Some(folder);
                    }
                }
                if let Some(ref path) = self.source_folder {
                    ui.label(RichText::new(path.to_string_lossy()).small());
                } else {
                    ui.label(RichText::new("Kaynak klasör seçilmedi").color(Color32::from_rgb(140, 140, 150)).small());
                }

                ui.separator();

                let btn_t = egui::Button::image_and_text(&icons.target, RichText::new(" Hedef Klasör").size(13.0).strong())
                    .min_size(Vec2::new(150.0, 42.0));
                if ui.add(btn_t).clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.target_folder = Some(folder);
                    }
                }
                if let Some(ref path) = self.target_folder {
                    ui.label(RichText::new(path.to_string_lossy()).small());
                } else {
                    ui.label(RichText::new("Hedef klasör seçilmedi").color(Color32::from_rgb(140, 140, 150)).small());
                }
            });

            ui.add_space(6.0);

            // 2. Satır: Kalite ve Boyut Ayarları
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚙️ Kalite & Boyut:").strong());

                ui.label("JPEG Kalitesi:");
                let q_resp = ui.add(egui::Slider::new(&mut self.jpeg_quality, 40..=100).suffix("%"));

                ui.separator();

                ui.label("Min Boyut:");
                let min_resp = ui.add(egui::DragValue::new(&mut self.min_kb).range(5..=100).suffix(" KB"));

                ui.label("Max Boyut:");
                let max_resp = ui.add(egui::DragValue::new(&mut self.max_kb).range(self.min_kb..=500).suffix(" KB"));

                if q_resp.changed() || min_resp.changed() || max_resp.changed() {
                    should_reencode = true;
                }

                ui.separator();
                ui.label(RichText::new(format!("(e-Okul: 20-150 KB • Aktif: {} - {} KB)", self.min_kb, self.max_kb)).small().color(Color32::from_rgb(160, 160, 170)));
            });

            ui.add_space(6.0);

            // 3. Satır: İşlemi Başlat, Dışa Aktar ve Kaydedilen Klasörü Aç (32x32 İkonlu Butonlar)
            ui.horizontal(|ui| {
                let start_btn = egui::Button::image_and_text(&icons.play, RichText::new(" İşlemi Başlat").size(13.0).strong())
                    .min_size(Vec2::new(145.0, 42.0));
                if ui.add_enabled(!self.is_processing && self.source_folder.is_some(), start_btn).clicked() {
                    self.start_processing();
                }

                let inc_count = self.photos.iter().filter(|p| p.is_included).count();
                let btn_label = format!(" Seçilenleri Kaydet ({}/{})", inc_count, self.photos.len());
                let export_btn = egui::Button::image_and_text(
                    &icons.export,
                    RichText::new(btn_label).size(13.0).strong().color(Color32::from_rgb(110, 225, 140)),
                )
                .min_size(Vec2::new(195.0, 42.0));

                if ui.add_enabled(!self.is_processing && inc_count > 0 && self.target_folder.is_some(), export_btn).clicked() {
                    self.export_all();
                }

                // Seçilenler kaydedildikten sonra kaydedilen klasörü açmak için buton
                if let Some(ref saved_dir) = self.last_exported_folder {
                    let open_btn = egui::Button::image_and_text(
                        &icons.open_folder,
                        RichText::new(" Klasörü Göster").size(13.0).strong().color(Color32::from_rgb(100, 205, 255)),
                    )
                    .min_size(Vec2::new(165.0, 42.0));

                    if ui.add(open_btn).on_hover_text("Kaydedilen fotoğrafların bulunduğu klasörü dosya gezgininde aç ve göster").clicked() {
                        open_folder_in_explorer(saved_dir);
                    }
                }

                if self.is_processing {
                    ui.spinner();
                }

                ui.label(RichText::new(&self.status_text).color(Color32::from_rgb(130, 200, 255)));
            });

            if self.is_processing || self.progress_total > 0 {
                let frac = if self.progress_total > 0 {
                    self.progress_current as f32 / self.progress_total as f32
                } else {
                    0.0
                };
                ui.add_space(4.0);
                ui.add(egui::ProgressBar::new(frac).show_percentage());
            }

            ui.add_space(8.0);
        });

        if should_reencode {
            self.reencode_all();
        }

        // --- SAĞ DETAY PANELİ: Zoom, Pan (Kaydırma) ve Kadraj Ayarlama ---
        if let Some(idx) = self.selected_index {
            if idx < self.photos.len() {
                let mut crop_action = None;

                egui::SidePanel::right("detail_panel")
                    .default_width(380.0)
                    .min_width(340.0)
                    .max_width(520.0)
                    .show(ctx, |ui| {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.heading(RichText::new("🔍 Fotoğraf İnceleme & Zoom").size(16.0).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::image(&icons.close).min_size(Vec2::new(32.0, 32.0));
                                if ui.add(close_btn).on_hover_text("Detay Panelini Kapat").clicked() {
                                    self.selected_index = None;
                                }
                            });
                        });
                        ui.separator();

                        let photo = &mut self.photos[idx];

                        // --- 1. ÖĞRENCİ NUMARASI DÜZENLEME ---
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Öğrenci Numarası:").strong());
                            ui.add(egui::TextEdit::singleline(&mut photo.student_number).desired_width(120.0));
                        });
                        ui.label(
                            RichText::new(format!(
                                "Boyut: {:.1} KB • Orijinal: {:?}",
                                photo.file_size_kb,
                                photo.original_path.file_name().unwrap_or_default()
                            ))
                            .small()
                            .color(Color32::LIGHT_GRAY),
                        );

                        ui.add_space(8.0);
                        ui.separator();

                        // --- 2. ORİJİNAL FOTOĞRAFTA ZOOM & PAN (TAHTAYI OKUMA) ---
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔎 Tahta İnceleme (Zoom & Sürükle):").strong());
                        });

                        // Zoom Kontrol Butonları (32x32 İkonlu) ve Slider
                        ui.horizontal(|ui| {
                            let zout_btn = egui::Button::image(&icons.zoom_out).min_size(Vec2::new(32.0, 32.0));
                            if ui.add(zout_btn).on_hover_text("Uzaklaştır").clicked() {
                                self.zoom_level = (self.zoom_level - 0.25).max(1.0);
                            }

                            ui.add(egui::Slider::new(&mut self.zoom_level, 1.0..=5.0).text("Zoom"));

                            let zin_btn = egui::Button::image(&icons.zoom_in).min_size(Vec2::new(32.0, 32.0));
                            if ui.add(zin_btn).on_hover_text("Yakınlaştır").clicked() {
                                self.zoom_level = (self.zoom_level + 0.25).min(5.0);
                            }

                            let rst_btn = egui::Button::image(&icons.reset).min_size(Vec2::new(32.0, 32.0));
                            if ui.add(rst_btn).on_hover_text("Konumu ve Zoom'u Sıfırla").clicked() {
                                self.zoom_level = 1.0;
                                self.pan_offset = Vec2::ZERO;
                            }
                        });

                        ui.label(RichText::new("💡 İpucu: Fotoğrafı farenin sol tuşuyla tutarak istediğiniz yöne kaydırabilirsiniz.").small().italics().color(Color32::GRAY));

                        // Etkileşimli Zoom & Pan Görünüm Alanı (Tahta İnceleme)
                        if let Some(ref orig_tex) = photo.original_preview_texture {
                            let view_size = Vec2::new(ui.available_width(), 240.0);
                            let (rect, response) = ui.allocate_exact_size(view_size, egui::Sense::drag());

                            // Fare tekerleğiyle zoom
                            if response.hovered() {
                                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                                if scroll.abs() > 0.1 {
                                    self.zoom_level = (self.zoom_level + scroll * 0.005).clamp(1.0, 5.0);
                                }
                            }

                            // Sürükleme (Pan) ile alanı kaydırma
                            if response.dragged() {
                                self.pan_offset += response.drag_delta();
                            }

                            // Sınırlandırma
                            let max_pan = (self.zoom_level - 1.0) * 180.0;
                            self.pan_offset.x = self.pan_offset.x.clamp(-max_pan, max_pan);
                            self.pan_offset.y = self.pan_offset.y.clamp(-max_pan, max_pan);

                            // Boyama (Painter) - En-Boy Oranı Korunarak (Aspect Fit)
                            let painter = ui.painter_at(rect);
                            painter.rect_filled(rect, 4.0, Color32::from_black_alpha(220));

                            let tex_size = orig_tex.size_vec2();
                            let aspect = (tex_size.x / tex_size.y).max(0.1);

                            let mut fit_w = view_size.x;
                            let mut fit_h = fit_w / aspect;
                            if fit_h > view_size.y {
                                fit_h = view_size.y;
                                fit_w = fit_h * aspect;
                            }

                            let scaled_w = fit_w * self.zoom_level;
                            let scaled_h = fit_h * self.zoom_level;

                            let img_center = rect.center() + self.pan_offset;
                            let img_rect = egui::Rect::from_center_size(img_center, Vec2::new(scaled_w, scaled_h));

                            painter.image(
                                orig_tex.id(),
                                img_rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, Color32::DARK_GRAY));
                        }

                        ui.add_space(8.0);
                        ui.separator();

                        // --- 3. e-OKUL KADRAJI (133x171) & İNTERAKTİF ZOOM/PAN ---
                        ui.label(RichText::new("✂️ Vesikalık Kadrajı (133x171 - Zoom & Kaydır):").strong());
                        ui.label(RichText::new("💡 Farenin sol tuşuyla tutarak kadrajı kaydırabilir, tekerlekle yakınlaşıp uzaklaşabilirsiniz.").small().italics().color(Color32::GRAY));

                        ui.horizontal(|ui| {
                            // Vesikalık İnteraktif Tuval (140 x 180 ~133:171)
                            let v_w = 140.0_f32;
                            let v_h = v_w * (171.0 / 133.0); // 180 px
                            let v_size = Vec2::new(v_w, v_h);

                            let (v_rect, v_response) = ui.allocate_exact_size(v_size, egui::Sense::drag());

                            // 1. Vesikalık üzerinde sürükleyerek kadrajı kaydırma (Pan)
                            if v_response.dragged() {
                                let delta = v_response.drag_delta();
                                if delta.length_sq() > 0.0 {
                                    // Farenin hareket yönüne göre kadrajı kaydır
                                    let factor = if let Some(ref orig) = photo.original_image {
                                        orig.width() as f32 / v_w
                                    } else {
                                        3.0
                                    };
                                    let dx = (-delta.x * factor * 0.4).round() as i32;
                                    let dy = (-delta.y * factor * 0.4).round() as i32;
                                    crop_action = Some((dx, dy, 1.0));
                                }
                            }

                            // 2. Vesikalık üzerinde tekerlekle Zoom (Yakınlaş/Uzaklaş)
                            if v_response.hovered() {
                                let v_scroll = ui.input(|i| i.raw_scroll_delta.y);
                                if v_scroll > 0.5 {
                                    crop_action = Some((0, 0, 0.95)); // Yüze yaklaş
                                } else if v_scroll < -0.5 {
                                    crop_action = Some((0, 0, 1.05)); // Uzaklaş
                                }
                            }

                            // Boyama (Vesikalık Önizlemesi)
                            let v_painter = ui.painter_at(v_rect);
                            v_painter.rect_filled(v_rect, 4.0, Color32::from_rgb(20, 20, 24));

                            if let Some(ref tex) = photo.preview_texture {
                                v_painter.image(
                                    tex.id(),
                                    v_rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                            }
                            v_painter.rect_stroke(
                                v_rect,
                                4.0,
                                if v_response.hovered() || v_response.dragged() {
                                    egui::Stroke::new(2.0_f32, Color32::from_rgb(70, 160, 255))
                                } else {
                                    egui::Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 70))
                                },
                            );

                            ui.add_space(8.0);

                            // Butonlarla da hassas ayar yapabilme (32x32 İkonlar)
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Kadraj Ayarı:").small());
                                ui.horizontal(|ui| {
                                    let btn_up = egui::Button::image(&icons.arrow_up).min_size(Vec2::new(32.0, 32.0));
                                    if ui.add(btn_up).on_hover_text("Yukarı Kaydır").clicked() {
                                        crop_action = Some((0, -20, 1.0));
                                    }

                                    let btn_down = egui::Button::image(&icons.arrow_down).min_size(Vec2::new(32.0, 32.0));
                                    if ui.add(btn_down).on_hover_text("Aşağı Kaydır").clicked() {
                                        crop_action = Some((0, 20, 1.0));
                                    }

                                    let btn_left = egui::Button::image(&icons.arrow_left).min_size(Vec2::new(32.0, 32.0));
                                    if ui.add(btn_left).on_hover_text("Sola Kaydır").clicked() {
                                        crop_action = Some((-20, 0, 1.0));
                                    }

                                    let btn_right = egui::Button::image(&icons.arrow_right).min_size(Vec2::new(32.0, 32.0));
                                    if ui.add(btn_right).on_hover_text("Sağa Kaydır").clicked() {
                                        crop_action = Some((20, 0, 1.0));
                                    }
                                });

                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let btn_zin = egui::Button::image_and_text(&icons.zoom_in, " Yakınlaş").min_size(Vec2::new(88.0, 34.0));
                                    if ui.add(btn_zin).on_hover_text("Kadrajı Büyüt / Yüze Yaklaş").clicked() {
                                        crop_action = Some((0, 0, 0.92));
                                    }

                                    let btn_zout = egui::Button::image_and_text(&icons.zoom_out, " Uzaklaş").min_size(Vec2::new(88.0, 34.0));
                                    if ui.add(btn_zout).on_hover_text("Kadrajı Küçült / Uzaklaş").clicked() {
                                        crop_action = Some((0, 0, 1.08));
                                    }
                                });
                            });
                        });

                        ui.add_space(8.0);

                        if crop_action.is_some() {
                            ctx.request_repaint();
                        }
                    });

                // Panelin dışında borrow serbest olduğunda kadrajı güncelle
                if let Some((dx, dy, sc)) = crop_action {
                    self.adjust_crop_and_update(ctx, dx, dy, sc);
                }
            }
        }

        // --- MERKEZ PANEL: Kaydırılabilir Kart Izgarası (Grid View) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.photos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Henüz işlenmiş fotoğraf yok.\nKaynak klasörü seçip '▶ İşlemi Başlat' butonuna basın.")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                });
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let available_width = ui.available_width();
                let card_width = 155.0;
                let columns = ((available_width / card_width).floor() as usize).max(1);

                let mut to_remove = None;

                egui::Grid::new("photos_grid")
                    .spacing(Vec2::new(14.0, 14.0))
                    .show(ui, |ui| {
                        let mut col = 0;
                        let len = self.photos.len();

                        for i in 0..len {
                            ui.vertical(|ui| {
                                let is_selected = self.selected_index == Some(i);
                                let is_inc = self.photos[i].is_included;
                                let frame = egui::Frame::group(ui.style())
                                    .inner_margin(6.0)
                                    .fill(if is_selected {
                                        Color32::from_rgb(20, 36, 56)
                                    } else if !is_inc {
                                        Color32::from_rgb(18, 18, 20)
                                    } else {
                                        Color32::from_rgb(26, 26, 30)
                                    })
                                    .stroke(if is_selected {
                                        egui::Stroke::new(2.5_f32, Color32::from_rgb(70, 160, 255))
                                    } else if !is_inc {
                                        egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 45, 50))
                                    } else {
                                        egui::Stroke::new(1.0_f32, Color32::from_rgb(58, 58, 65))
                                    });

                                frame.show(ui, |ui| {
                                    // 1. SOL ÜST CHECKBOX & SAĞ ÜST KAPAT (SİL) BUTONU
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.photos[i].is_included, "");
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let close_btn = egui::Button::new(
                                                RichText::new("✖")
                                                    .size(12.0)
                                                    .strong()
                                                    .color(Color32::from_rgb(240, 80, 80)),
                                            )
                                            .frame(false);
                                            if ui.add(close_btn).on_hover_text("Bu fotoğrafı listeden sil").clicked() {
                                                to_remove = Some(i);
                                            }
                                        });
                                    });

                                    ui.add_space(2.0);

                                    // 2. 133x171 ÖNİZLEME BUTONU
                                    if let Some(ref tex) = self.photos[i].preview_texture {
                                        let img_btn = egui::ImageButton::new(tex);
                                        if ui.add(img_btn).clicked() {
                                            self.selected_index = Some(i);
                                            self.zoom_level = 1.0;
                                            self.pan_offset = Vec2::ZERO;
                                        }
                                    }

                                    ui.add_space(4.0);

                                    // 3. DÜZENLENEBİLİR ÖĞRENCİ NUMARASI
                                    let mut num = self.photos[i].student_number.clone();
                                    let text_resp = ui.add(
                                        egui::TextEdit::singleline(&mut num)
                                            .desired_width(120.0)
                                            .hint_text("Öğrenci No"),
                                    );
                                    if text_resp.changed() {
                                        self.photos[i].student_number = num;
                                    }

                                    // 4. DOSYA BOYUTU BİLGİSİ
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("{:.1} KB", self.photos[i].file_size_kb))
                                                .small()
                                                .color(if self.photos[i].is_included {
                                                    Color32::LIGHT_GREEN
                                                } else {
                                                    Color32::GRAY
                                                }),
                                        );
                                        if !self.photos[i].is_included {
                                            ui.label(RichText::new("(Hariç)").small().italics().color(Color32::YELLOW));
                                        }
                                    });
                                });
                            });

                            col += 1;
                            if col >= columns {
                                ui.end_row();
                                col = 0;
                            }
                        }
                    });

                // Kapat butonuna basılan fotoğrafı listeden güvenle sil
                if let Some(del_idx) = to_remove {
                    self.photos.remove(del_idx);
                    if self.selected_index == Some(del_idx) {
                        self.selected_index = None;
                    } else if let Some(sel) = self.selected_index {
                        if sel > del_idx {
                            self.selected_index = Some(sel - 1);
                        }
                    }
                }
            });
        });
    }
}

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../app_icon.png");
    let img = image::load_from_memory(icon_bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> Result<(), eframe::Error> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1040.0, 720.0])
        .with_min_inner_size([800.0, 560.0])
        .with_title("MEB e-Okul Öğrenci Fotoğrafı Hazırlama ve İsimlendirme (Zoom & Pan)");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "okulfotoeditor",
        native_options,
        Box::new(|cc| {
            apply_black_theme(&cc.egui_ctx);
            Ok(Box::new(OkulFotoApp::default()))
        }),
    )
}

