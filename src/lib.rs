use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, Rgba};
use std::path::Path;

/// Convert an image file to ANSI art
pub fn convert_file<P: AsRef<Path>>(path: P) -> Result<String, image::ImageError> {
    let img = ImageReader::open(path)?.decode()?;
    Ok(image_to_ansi(&img))
}

fn pixels_to_ansi(px1: Rgba<u8>, px2: Rgba<u8>) -> String {
    let mut res = String::new();

    // Reset if either pixel is transparent
    if px1[3] != 255 || px2[3] != 255 {
        res.push_str("\x1b[0m");
    }

    // Both transparent = space
    if px1[3] != 255 && px2[3] != 255 {
        res.push(' ');
        return res;
    }

    // Top pixel opaque
    if px1[3] == 255 {
        res.push_str(&format!("\x1b[38;2;{};{};{}m", px1[0], px1[1], px1[2]));

        // Bottom pixel opaque = use as background
        if px2[3] == 255 {
            res.push_str(&format!("\x1b[48;2;{};{};{}m", px2[0], px2[1], px2[2]));
        }
        res.push('▀'); // Upper half block
    }
    // Only bottom pixel opaque
    else if px2[3] == 255 {
        res.push_str(&format!("\x1b[38;2;{};{};{}m", px2[0], px2[1], px2[2]));
        res.push('▄'); // Lower half block
    }

    res
}

pub fn image_to_ansi(img: &DynamicImage) -> String {
    let (width, height) = img.dimensions();
    let mut output = String::new();

    // Process two rows at a time (combining into one terminal row)
    for y in (0..height).step_by(2) {
        for x in 0..width {
            let px1 = img.get_pixel(x, y);
            let px2 = if y + 1 < height {
                img.get_pixel(x, y + 1)
            } else {
                Rgba([0, 0, 0, 0]) // Transparent if odd height
            };

            output.push_str(&pixels_to_ansi(px1, px2));
        }
        output.push_str("\x1b[0m\n");
    }

    output
}
