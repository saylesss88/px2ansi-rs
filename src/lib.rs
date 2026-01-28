use image::ImageReader;
use image::{DynamicImage, GenericImageView, Rgba};
use std::path::Path;

/// Convert an image file to ANSI art
pub fn convert_file<P: AsRef<Path>>(path: P) -> Result<String, image::ImageError> {
    let img = ImageReader::open(path)?.decode()?;
    Ok(image_to_ansi(&img))
}

fn pixels_to_ansi(px1: Rgba<u8>, px2: Rgba<u8>) -> String {
    let mut res = String::new();
    let top_alpha = px1[3];
    let bot_alpha = px2[3];

    // Check > 0 instead of == 255 to catch semi-transparent pixels
    if top_alpha > 0 {
        // Use single backslash \x1b for actual Escape char
        res.push_str(&format!("\x1b[38;2;{};{};{}m", px1[0], px1[1], px1[2]));

        if bot_alpha > 0 {
            res.push_str(&format!("\x1b[48;2;{};{};{}m▀", px2[0], px2[1], px2[2]));
        } else {
            res.push_str("\x1b[49m▀");
        }
    } else {
        if bot_alpha > 0 {
            res.push_str(&format!(
                "\x1b[38;2;{};{};{}m\x1b[49m▄",
                px2[0], px2[1], px2[2]
            ));
        } else {
            res.push_str("\x1b[0m ");
        }
    }
    res
}

pub fn image_to_ansi(img: &DynamicImage) -> String {
    let (width, height) = img.dimensions();
    let mut output = String::new();

    for y in (0..height).step_by(2) {
        for x in 0..width {
            let px1 = img.get_pixel(x, y);
            let px2 = if y + 1 < height {
                img.get_pixel(x, y + 1)
            } else {
                Rgba([0, 0, 0, 0])
            };

            output.push_str(&pixels_to_ansi(px1, px2));
        }
        // Fix newline escape
        output.push_str("\x1b[0m\n");
    }

    output
}
