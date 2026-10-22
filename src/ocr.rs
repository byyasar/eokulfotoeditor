//! Tahtadaki el yazısı veya tebeşir/tahta kalemiyle yazılmış öğrenci numarasını (0-9)
//! klasik görüntü işleme ve topolojik analizle okuyan OCR modülü.

use image::{DynamicImage, GenericImageView, GrayImage};

#[derive(Debug, Clone)]
struct ComponentBox {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

/// Görüntüye adaptif yerel eşikleme (Adaptive Thresholding) uygular.
fn adaptive_threshold(gray: &GrayImage, block_size: u32, c: i32) -> GrayImage {
    let (w, h) = gray.dimensions();
    let mut binary = GrayImage::new(w, h);
    let half_block = (block_size / 2) as i32;

    for y in 0..h {
        let y_min = (y as i32 - half_block).max(0) as u32;
        let y_max = (y as i32 + half_block).min(h as i32 - 1) as u32;

        for x in 0..w {
            let x_min = (x as i32 - half_block).max(0) as u32;
            let x_max = (x as i32 + half_block).min(w as i32 - 1) as u32;

            let mut sum: u32 = 0;
            let mut count: u32 = 0;

            for by in y_min..=y_max {
                for bx in x_min..=x_max {
                    sum += gray.get_pixel(bx, by)[0] as u32;
                    count += 1;
                }
            }

            let mean = (sum / count) as i32;
            let pixel_val = gray.get_pixel(x, y)[0] as i32;

            // Koyu yazılar (beyaz tahta) için pixel < mean - c
            if pixel_val < mean - c {
                binary.put_pixel(x, y, image::Luma([255]));
            } else {
                binary.put_pixel(x, y, image::Luma([0]));
            }
        }
    }

    binary
}

/// 8-bağlantılı bileşenler analizi ile olası karakter kutularını bulur.
fn find_components(binary: &GrayImage, min_area: u32, max_area: u32) -> Vec<ComponentBox> {
    let (w, h) = binary.dimensions();
    let mut visited = vec![false; (w * h) as usize];
    let mut components = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if visited[idx] || binary.get_pixel(x, y)[0] == 0 {
                continue;
            }

            let mut queue = std::collections::VecDeque::new();
            queue.push_back((x, y));
            visited[idx] = true;

            let mut min_x = x;
            let mut max_x = x;
            let mut min_y = y;
            let mut max_y = y;
            let mut count = 0u32;

            while let Some((cx, cy)) = queue.pop_front() {
                count += 1;
                min_x = min_x.min(cx);
                max_x = max_x.max(cx);
                min_y = min_y.min(cy);
                max_y = max_y.max(cy);

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;

                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            let n_idx = (ny as u32 * w + nx as u32) as usize;
                            if !visited[n_idx] && binary.get_pixel(nx as u32, ny as u32)[0] == 255 {
                                visited[n_idx] = true;
                                queue.push_back((nx as u32, ny as u32));
                            }
                        }
                    }
                }
            }

            let comp_w = max_x - min_x + 1;
            let comp_h = max_y - min_y + 1;

            if count >= min_area && count <= max_area {
                let aspect = comp_w as f32 / comp_h as f32;
                if aspect >= 0.15 && aspect <= 1.4 && comp_h >= 12 {
                    components.push(ComponentBox {
                        x: min_x,
                        y: min_y,
                        w: comp_w,
                        h: comp_h,
                    });
                }
            }
        }
    }

    components
}

fn normalize_char_patch(binary: &GrayImage, c: &ComponentBox) -> [[bool; 16]; 16] {
    let mut patch = [[false; 16]; 16];
    for py in 0..16 {
        for px in 0..16 {
            let orig_x = c.x + (px as f32 / 16.0 * c.w as f32) as u32;
            let orig_y = c.y + (py as f32 / 16.0 * c.h as f32) as u32;
            if binary.get_pixel(orig_x.min(binary.width() - 1), orig_y.min(binary.height() - 1))[0] == 255 {
                patch[py][px] = true;
            }
        }
    }
    patch
}

fn count_holes(patch: &[[bool; 16]; 16]) -> (usize, f32) {
    let mut grid = [[false; 18]; 18];
    for y in 0..16 {
        for x in 0..16 {
            grid[y + 1][x + 1] = patch[y][x];
        }
    }

    let mut visited = [[false; 18]; 18];
    let mut q = std::collections::VecDeque::new();
    q.push_back((0, 0));
    visited[0][0] = true;

    while let Some((cx, cy)) = q.pop_front() {
        for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < 18 && ny >= 0 && ny < 18 {
                let ux = nx as usize;
                let uy = ny as usize;
                if !visited[uy][ux] && !grid[uy][ux] {
                    visited[uy][ux] = true;
                    q.push_back((ux, uy));
                }
            }
        }
    }

    let mut holes = 0;
    let mut total_hole_y = 0.0;
    let mut hole_pixels = 0;

    for y in 1..17 {
        for x in 1..17 {
            if !grid[y][x] && !visited[y][x] {
                holes += 1;
                let mut hq = std::collections::VecDeque::new();
                hq.push_back((x, y));
                visited[y][x] = true;

                while let Some((hx, hy)) = hq.pop_front() {
                    total_hole_y += hy as f32 - 1.0;
                    hole_pixels += 1;
                    for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = hx as i32 + dx;
                        let ny = hy as i32 + dy;
                        if nx >= 1 && nx < 17 && ny >= 1 && ny < 17 {
                            let ux = nx as usize;
                            let uy = ny as usize;
                            if !visited[uy][ux] && !grid[uy][ux] {
                                visited[uy][ux] = true;
                                hq.push_back((ux, uy));
                            }
                        }
                    }
                }
            }
        }
    }

    let avg_hole_y = if hole_pixels > 0 {
        total_hole_y / hole_pixels as f32
    } else {
        8.0
    };

    (holes, avg_hole_y)
}

fn classify_digit(c: &ComponentBox, patch: &[[bool; 16]; 16]) -> char {
    let aspect = c.w as f32 / c.h as f32;

    if aspect < 0.40 {
        return '1';
    }

    let (holes, hole_y) = count_holes(patch);

    if holes >= 2 {
        return '8';
    }

    if holes == 1 {
        if hole_y < 6.5 {
            return '9';
        } else if hole_y > 9.5 {
            return '6';
        } else {
            let left_top = patch[3][3];
            let right_top = patch[3][12];
            if left_top && right_top {
                return '0';
            } else {
                return '4';
            }
        }
    }

    let top_row_sum: usize = (2..14).filter(|&x| patch[2][x] || patch[3][x]).count();
    let bottom_row_sum: usize = (2..14).filter(|&x| patch[13][x] || patch[14][x]).count();
    let mid_row_sum: usize = (2..14).filter(|&x| patch[8][x]).count();

    if top_row_sum >= 8 && bottom_row_sum <= 4 {
        return '7';
    }

    if bottom_row_sum >= 8 && top_row_sum >= 5 {
        return '2';
    }

    let left_edge_sum: usize = (4..12).filter(|&y| patch[y][2] || patch[y][3]).count();
    let right_edge_sum: usize = (4..12).filter(|&y| patch[y][12] || patch[y][13]).count();

    if right_edge_sum > left_edge_sum + 2 && mid_row_sum >= 4 {
        return '3';
    }

    if patch[4][3] && bottom_row_sum >= 5 {
        return '5';
    }

    if aspect < 0.55 {
        '1'
    } else {
        '0'
    }
}

/// Fotoğraftaki tahta bölgesini analiz ederek öğrenci numarasını okur.
pub fn read_student_number(img: &DynamicImage) -> Option<String> {
    let (w, h) = img.dimensions();
    let scale = (w.max(h) as f32 / 1200.0).max(1.0);
    let sw = (w as f32 / scale) as u32;
    let sh = (h as f32 / scale) as u32;

    let gray = img.thumbnail_exact(sw, sh).to_luma8();
    let binary = adaptive_threshold(&gray, 31, 8);

    let min_area = (sh / 80).pow(2);
    let max_area = (sh / 6).pow(2);

    let components = find_components(&binary, min_area, max_area);
    if components.is_empty() {
        return None;
    }

    let mut best_line: Vec<ComponentBox> = Vec::new();

    for i in 0..components.len() {
        let base = &components[i];
        let mut line = vec![base.clone()];

        for j in 0..components.len() {
            if i == j {
                continue;
            }
            let other = &components[j];

            let y_diff = (base.y as i32 - other.y as i32).abs();
            let h_diff = (base.h as i32 - other.h as i32).abs();

            if y_diff < (base.h as i32 * 45 / 100) && h_diff < (base.h as i32 * 50 / 100) {
                line.push(other.clone());
            }
        }

        if line.len() >= 2 && line.len() <= 6 && line.len() > best_line.len() {
            best_line = line;
        }
    }

    if best_line.len() < 2 {
        return None;
    }

    best_line.sort_by_key(|c| c.x);

    let mut res = String::new();
    for comp in &best_line {
        let patch = normalize_char_patch(&binary, comp);
        res.push(classify_digit(comp, &patch));
    }

    Some(res)
}
