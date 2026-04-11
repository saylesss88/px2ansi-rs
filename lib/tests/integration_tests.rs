//! Integration tests for the `px2ansi` library.
//!
//! These tests verify the public API from the outside, exactly as a
//! downstream user would use it.

use image::{DynamicImage, Rgba, RgbaImage};
use px2ansi::{
    CharsetMode, Density, RenderOptions, RenderStylePreset, ResizeFilter,
    indexer::{ImageEntry, build_index},
};
use std::io::Cursor;
use tempfile::TempDir;

// --- Test Helpers ---

/// Creates a small solid-color RGBA test image.
fn make_test_image(width: u32, height: u32, color: [u8; 4]) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgba(color);
    }
    DynamicImage::ImageRgba8(img)
}

/// Creates a simple gradient test image.
fn make_gradient_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // We use .try_into().unwrap() because we know the result fits in u8.
        // Math: (0..width) * 255 / width is always <= 255.
        let r: u8 = (x * 255 / width).try_into().unwrap_or(255);
        let g: u8 = (y * 255 / height).try_into().unwrap_or(255);

        *pixel = Rgba([r, g, 128, 255]);
    }
    DynamicImage::ImageRgba8(img)
}

/// Creates a test image with a transparent background.
// fn make_sprite_image(width: u32, height: u32) -> DynamicImage {
//     let mut img = RgbaImage::new(width, height);
//     for (x, y, pixel) in img.enumerate_pixels_mut() {
//         // Solid colored center, transparent border
//         if x > width / 4 && x < 3 * width / 4 && y > height / 4 && y < 3 * height / 4 {
//             *pixel = Rgba([255, 100, 50, 255]);
//         } else {
//             *pixel = Rgba([0, 0, 0, 0]);
//         }
//     }
//     DynamicImage::ImageRgba8(img)
// }

// --- RenderOptions Builder ---

#[test]
fn builder_defaults_are_sensible() {
    let opts = RenderOptions::default();
    assert_eq!(opts.charset(), CharsetMode::Ansi);
    assert!(opts.color());
    assert_eq!(opts.width(), None);
}

#[test]
fn builder_preset_braille() {
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Braille)
        .build();
    assert_eq!(opts.charset(), CharsetMode::Braille);
}

#[test]
fn builder_preset_full_block_sets_full_flag() {
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::FullBlock)
        .build();
    assert_eq!(opts.charset(), CharsetMode::Unicode);
    assert!(opts.style().is_full());
}

#[test]
fn builder_no_color_disables_color() {
    let opts = RenderOptions::builder().color(false).build();
    assert!(!opts.color());
}

#[test]
fn builder_width_override() {
    let opts = RenderOptions::builder().width(120).build();
    assert_eq!(opts.width(), Some(120));
}

#[test]
fn builder_density_override() {
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Ascii)
        .density(Density::Light)
        .build();
    assert!(matches!(opts.style().density(), Density::Light));
}

#[test]
fn builder_filter_nearest() {
    let opts = RenderOptions::builder()
        .filter(ResizeFilter::Nearest)
        .build();
    assert_eq!(opts.filter(), image::imageops::FilterType::Nearest);
}

// --- Rendering ---

#[test]
fn render_ansi_produces_output() {
    let img = make_test_image(8, 8, [255, 100, 50, 255]);
    let opts = RenderOptions::default();
    let mut buf = Vec::new();
    opts.render(&img, &mut buf).unwrap();
    assert!(!buf.is_empty(), "render should produce output");
}

#[test]
fn render_centered_produces_output() {
    let img = make_test_image(8, 8, [255, 100, 50, 255]);
    let opts = RenderOptions::default();
    let mut buf = Vec::new();
    opts.render_centered(&img, &mut buf).unwrap();
    assert!(!buf.is_empty());
}

#[test]
fn render_output_contains_ansi_escapes() {
    let img = make_test_image(4, 4, [200, 100, 50, 255]);
    let opts = RenderOptions::default();
    let mut buf = Vec::new();
    opts.render(&img, &mut buf).unwrap();
    // ANSI escape sequences start with ESC (0x1b)
    assert!(
        buf.contains(&0x1b),
        "output should contain ANSI escape codes"
    );
}

#[test]
fn render_no_color_has_no_ansi_escapes() {
    let img = make_test_image(4, 4, [200, 100, 50, 255]);
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Ascii)
        .color(false)
        .build();
    let mut buf = Vec::new();
    opts.render(&img, &mut buf).unwrap();
    assert!(
        !buf.contains(&0x1b),
        "no-color mode should not emit ANSI escapes"
    );
}

#[test]
fn render_transparent_image_produces_spaces() {
    let img = make_test_image(4, 4, [0, 0, 0, 0]); // fully transparent
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Ascii)
        .build();
    let mut buf = Vec::new();
    opts.render(&img, &mut buf).unwrap();
    let output = String::from_utf8_lossy(&buf);
    // Should be mostly spaces and newlines
    let printable: String = output.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(printable.is_empty() || !printable.contains('█'));
}

#[test]
fn all_charset_modes_produce_output() {
    let img = make_gradient_image(16, 16);
    let presets = [
        RenderStylePreset::Ansi,
        RenderStylePreset::Unicode,
        RenderStylePreset::Braille,
        RenderStylePreset::Fade,
        RenderStylePreset::Ascii,
        RenderStylePreset::Kanji,
        RenderStylePreset::Chinese,
        RenderStylePreset::FullBlock,
    ];
    for preset in presets {
        let opts = RenderOptions::builder().preset(preset).build();
        let mut buf = Vec::new();
        opts.render(&img, &mut buf)
            .unwrap_or_else(|e| panic!("render failed for {preset:?}: {e}"));
        assert!(!buf.is_empty(), "{preset:?} produced no output");
    }
}

#[test]
fn render_writer_trait_works_with_cursor() {
    let img = make_test_image(4, 4, [100, 200, 50, 255]);
    let opts = RenderOptions::default();
    let mut cursor = Cursor::new(Vec::new());
    opts.render(&img, &mut cursor).unwrap();
    assert!(!cursor.into_inner().is_empty());
}

#[test]
fn prepare_image_respects_width_override() {
    let img = make_test_image(100, 100, [255, 255, 255, 255]);
    let opts = RenderOptions::builder().width(40).build();
    let prepared = opts.prepare_image(&img);
    assert_eq!(prepared.width(), 40);
}

// --- Indexer ---

#[test]
fn build_index_creates_json_file() {
    let dir = TempDir::new().unwrap();
    let sprites_dir = dir.path().join("sprites");
    std::fs::create_dir(&sprites_dir).unwrap();

    // Save a test image
    let img = make_test_image(16, 16, [255, 0, 0, 255]);
    img.save(sprites_dir.join("test_sprite.png")).unwrap();

    let index_path = dir.path().join("index.json");
    let json = build_index(&sprites_dir, &index_path).unwrap();

    assert!(index_path.exists(), "index file should be created");
    assert!(!json.is_empty());
}

#[test]
fn build_index_contains_correct_entry() {
    let dir = TempDir::new().unwrap();
    let img = make_test_image(32, 24, [0, 255, 0, 255]);
    img.save(dir.path().join("mysprite.png")).unwrap();

    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path).unwrap();

    let json = std::fs::read_to_string(&index_path).unwrap();
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "mysprite");
    assert_eq!(entries[0].dimensions, (32, 24));
}

#[test]
fn build_index_ignores_non_image_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not an image").unwrap();
    std::fs::write(dir.path().join("data.json"), "{}").unwrap();

    let img = make_test_image(8, 8, [0, 0, 255, 255]);
    img.save(dir.path().join("sprite.png")).unwrap();

    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path).unwrap();

    let json = std::fs::read_to_string(&index_path).unwrap();
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();

    assert_eq!(entries.len(), 1, "only the PNG should be indexed");
}

#[test]
fn build_index_is_sorted_alphabetically() {
    let dir = TempDir::new().unwrap();
    let img = make_test_image(8, 8, [255, 255, 0, 255]);

    img.save(dir.path().join("zebra.png")).unwrap();
    img.save(dir.path().join("apple.png")).unwrap();
    img.save(dir.path().join("mango.png")).unwrap();

    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path).unwrap();

    let json = std::fs::read_to_string(&index_path).unwrap();
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();

    assert_eq!(entries[0].name, "apple");
    assert_eq!(entries[1].name, "mango");
    assert_eq!(entries[2].name, "zebra");
}

#[test]
fn build_index_scans_subdirectories() {
    let dir = TempDir::new().unwrap();
    let subdir = dir.path().join("pokemon");
    std::fs::create_dir(&subdir).unwrap();

    let img = make_test_image(8, 8, [255, 0, 255, 255]);
    img.save(dir.path().join("top_level.png")).unwrap();
    img.save(subdir.join("nested.png")).unwrap();

    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path).unwrap();

    let json = std::fs::read_to_string(&index_path).unwrap();
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();

    assert_eq!(entries.len(), 2, "should find images in subdirectories");
}

#[test]
fn build_index_empty_directory_produces_empty_array() {
    let dir = TempDir::new().unwrap();
    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path).unwrap();

    let json = std::fs::read_to_string(&index_path).unwrap();
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();
    assert!(entries.is_empty());
}

// --- Rasterization ---

#[cfg(feature = "rasterize")]
mod rasterize_tests {
    use super::*;
    use px2ansi::{RasterTheme, rasterize_ansi_with_theme};

    #[test]
    fn rasterize_ansi_produces_valid_image() {
        let img = make_test_image(8, 8, [255, 100, 50, 255]);
        let opts = RenderOptions::default();
        let mut buf = Vec::new();
        opts.render(&img, &mut buf).unwrap();

        let result = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight);
        assert!(result.is_ok(), "rasterization should succeed");
        let png = result.unwrap();
        assert!(png.width() > 0);
        assert!(png.height() > 0);
    }

    #[test]
    fn rasterize_empty_input_fails_gracefully() {
        let result = rasterize_ansi_with_theme(&[], RasterTheme::TokyoNight);
        assert!(result.is_err(), "empty input should return an error");
    }

    #[test]
    fn rasterize_different_themes_produce_different_backgrounds() {
        let img = make_test_image(4, 4, [255, 255, 255, 255]);
        let opts = RenderOptions::builder()
            .preset(RenderStylePreset::Ascii)
            .build();
        let mut buf = Vec::new();
        opts.render(&img, &mut buf).unwrap();

        let dark = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight).unwrap();
        let light = rasterize_ansi_with_theme(&buf, RasterTheme::White).unwrap();

        // Top-left pixel should differ between themes
        let dark_px = dark.get_pixel(0, 0);
        let light_px = light.get_pixel(0, 0);
        assert_ne!(
            dark_px, light_px,
            "different themes should produce different backgrounds"
        );
    }
}
