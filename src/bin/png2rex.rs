// Converts a PNG to a REXPaint .xp file using the upper-half-block method.
// Each character cell maps to 1×2 source pixels: fg = top pixel, bg = bottom pixel,
// glyph = 223 (▀). This doubles vertical resolution compared to 1-pixel-per-cell.
//
// Usage: png2rex <input.png> <output.xp> [--width N]
//   --width N   Scale image to N characters wide. Height is computed so the image
//               appears undistorted when rendered with square character cells.
//               Without --width: no scaling (image width = char width).

use std::io::Write;
use image::{DynamicImage, GenericImageView};
use flate2::{write::GzEncoder, Compression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: png2rex <input.png> <output.xp> [--width N]");
        std::process::exit(1);
    }

    let input_path  = &args[1];
    let output_path = &args[2];

    let mut target_char_width: Option<u32> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--width" if i + 1 < args.len() => {
                target_char_width = Some(args[i + 1].parse()?);
                i += 2;
            }
            flag => {
                eprintln!("Unknown flag: {}", flag);
                std::process::exit(1);
            }
        }
    }

    let img = image::open(input_path)?;
    let (orig_w, orig_h) = img.dimensions();

    eprintln!("Image: {}x{} px, color type: {:?}", orig_w, orig_h, img.color());
    if orig_w > 0 && orig_h > 0 {
        let (cx, cy) = (orig_w / 2, orig_h / 2);
        let p = img.get_pixel(cx, cy);
        eprintln!("Center pixel ({},{}): rgba({},{},{},{})", cx, cy, p[0], p[1], p[2], p[3]);
    }

    // With square character cells, displaying char_h rows of half-block characters
    // takes char_h cell heights on screen, but represents char_h*2 source pixels.
    // To keep the on-screen aspect ratio equal to the source image's aspect ratio:
    //   pixel_h = 2 * char_w * orig_h / orig_w
    let (pixel_w, pixel_h) = match target_char_width {
        Some(cw) => {
            let ph = ((2 * cw * orig_h) as f64 / orig_w as f64).round() as u32;
            (cw, ph.max(2))
        }
        None => (orig_w, orig_h),
    };

    let char_w = pixel_w;
    let char_h = (pixel_h + 1) / 2;

    let img = if pixel_w != orig_w || pixel_h != orig_h {
        img.resize_exact(pixel_w, pixel_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // XP binary layout (after zlib decompression):
    //   i32  version (-1)
    //   i32  layer count (1)
    //   i32  layer width
    //   i32  layer height
    //   for x in 0..width, for y in 0..height:   ← column-major
    //     i32  glyph (CP437 code point)
    //     u8   fg_r, fg_g, fg_b
    //     u8   bg_r, bg_g, bg_b
    const UPPER_HALF: i32 = 223; // ▀

    let cell_count = char_w as usize * char_h as usize;
    let mut raw: Vec<u8> = Vec::with_capacity(16 + cell_count * 10);

    raw.extend_from_slice(&(-1i32).to_le_bytes());          // version
    raw.extend_from_slice(&1i32.to_le_bytes());             // layer count
    raw.extend_from_slice(&(char_w as i32).to_le_bytes()); // layer width
    raw.extend_from_slice(&(char_h as i32).to_le_bytes()); // layer height

    for x in 0..char_w {
        for y in 0..char_h {
            let (fr, fg, fb) = sample(&img, x, y * 2,     pixel_h);
            let (br, bg, bb) = sample(&img, x, y * 2 + 1, pixel_h);
            raw.extend_from_slice(&UPPER_HALF.to_le_bytes());
            raw.extend_from_slice(&[fr, fg, fb, br, bg, bb]);
        }
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw)?;
    let compressed = encoder.finish()?;
    std::fs::write(output_path, &compressed)?;

    println!("{}x{} chars -> {}", char_w, char_h, output_path);
    Ok(())
}

/// Returns (r, g, b) for the pixel at (x, y). Out-of-bounds y returns black.
/// Alpha is used only as a hard threshold: fully transparent pixels map to black,
/// everything else uses the RGB values directly without premultiplication.
fn sample(img: &DynamicImage, x: u32, y: u32, max_y: u32) -> (u8, u8, u8) {
    if y >= max_y {
        return (0, 0, 0);
    }
    let p = img.get_pixel(x, y);
    if p[3] < 16 {
        return (0, 0, 0);
    }
    (p[0], p[1], p[2])
}
