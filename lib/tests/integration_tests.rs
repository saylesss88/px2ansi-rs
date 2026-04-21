//! Integration tests for the `px2ansi` library.
//!
//! These tests verify the public API from the outside, exactly as a
//! downstream user would use it.

use image::{DynamicImage, Rgba, RgbaImage};
use px2ansi::{
    CharsetMode, ColorMode, Density, RenderOptions, RenderStylePreset, ResizeFilter,
    indexer::{ImageEntry, build_index},
};
use std::path::Path;
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;

// --- Test Helpers ---

/// Helper to load the index and parse it, reducing boilerplate in tests
fn load_index(path: &Path) -> Result<Vec<ImageEntry>, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(path)?;
    let entries = serde_json::from_str(&json)?;
    Ok(entries)
}

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
    assert_eq!(opts.color_mode(), ColorMode::TrueColor);
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
    let opts = RenderOptions::builder().color_mode(ColorMode::None).build();
    assert_eq!(opts.color_mode(), ColorMode::None);
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
fn render_ansi_produces_output() -> TestResult {
    let img = make_test_image(8, 8, [255, 100, 50, 255]);
    let opts = RenderOptions::default();
    let mut buf = Vec::new();

    opts.render(&img, &mut buf)?;

    assert!(!buf.is_empty(), "render should produce output");
    Ok(())
}

#[test]
fn render_centered_produces_output() -> TestResult {
    let img = make_test_image(8, 8, [255, 100, 50, 255]);
    let opts = RenderOptions::default();
    let mut buf = Vec::new();

    opts.render_centered(&img, &mut buf)?;

    assert!(!buf.is_empty());
    Ok(())
}

#[test]
fn render_output_contains_ansi_escapes() -> TestResult {
    let img = make_test_image(4, 4, [200, 100, 50, 255]);
    let opts = RenderOptions::default();
    let mut buf = Vec::new();

    opts.render(&img, &mut buf)?;

    // ANSI escape sequences start with ESC (0x1b)
    assert!(
        buf.contains(&0x1b),
        "output should contain ANSI escape codes"
    );
    Ok(())
}

#[test]
fn render_no_color_has_no_ansi_escapes() -> TestResult {
    let img = make_test_image(4, 4, [200, 100, 50, 255]);
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Ascii)
        .color_mode(ColorMode::None)
        .build();
    let mut buf = Vec::new();

    opts.render(&img, &mut buf)?;

    assert!(
        !buf.contains(&0x1b),
        "no-color mode should not emit ANSI escapes"
    );
    Ok(())
}

#[test]
fn render_transparent_image_produces_spaces() -> TestResult {
    let img = make_test_image(4, 4, [0, 0, 0, 0]);
    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Ascii)
        .build();
    let mut buf = Vec::new();

    opts.render(&img, &mut buf)?;

    let output = String::from_utf8_lossy(&buf);
    let printable: String = output.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(printable.is_empty() || !printable.contains('█'));
    Ok(())
}

#[test]
fn all_charset_modes_produce_output() -> TestResult {
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

        opts.render(&img, &mut buf)?;

        assert!(!buf.is_empty(), "{preset:?} produced no output");
    }
    Ok(())
}

#[test]
fn render_writer_trait_works_with_cursor() -> TestResult {
    let img = make_test_image(4, 4, [100, 200, 50, 255]);
    let opts = RenderOptions::default();
    let mut cursor = std::io::Cursor::new(Vec::new());

    opts.render(&img, &mut cursor)?;

    assert!(!cursor.into_inner().is_empty());
    Ok(())
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
    let dir = TempDir::new().expect("failed to create temporary directory for test");
    let sprites_dir = dir.path().join("sprites");
    std::fs::create_dir_all(&sprites_dir).expect("failed to create sprites directory");

    // Save a test image
    let img = make_test_image(16, 16, [255, 0, 0, 255]);
    img.save(sprites_dir.join("test_sprite.png"))
        .expect("unable to save image");

    let index_path = dir.path().join("index.json");

    px2ansi::indexer::build_index(dir.path(), &index_path).expect("Failed to build index");

    let json = std::fs::read_to_string(&index_path).unwrap_or_else(|e| {
        eprintln!("Warning: failed to read index file: {e}");
        String::new()
    });

    assert!(index_path.exists(), "index file should be created");
    assert!(!json.is_empty());
}

#[test]
fn build_index_contains_correct_entry() {
    let dir = TempDir::new().expect("failed to create temporary directory");
    let img = make_test_image(32, 24, [0, 255, 0, 255]);
    img.save(dir.path().join("mysprite.png"))
        .expect("unable to save image");

    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path).expect("unable to build index");

    let json = std::fs::read_to_string(&index_path).unwrap_or_else(|e| {
        eprintln!("Warning: failed to read index file: {e}");
        String::new()
    });
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).expect("unable to serailize data");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "mysprite");
    assert_eq!(entries[0].dimensions, (32, 24));
}

#[test]
fn build_index_ignores_non_image_files() -> TestResult {
    let dir = TempDir::new()?;
    let root = dir.path();

    // Using ? here is much cleaner than .unwrap()
    std::fs::write(root.join("notes.txt"), "not an image")?;
    std::fs::write(root.join("data.json"), "{}")?;

    let img = make_test_image(8, 8, [0, 0, 255, 255]);
    img.save(root.join("sprite.png"))?;

    let index_path = root.join("index.json");
    build_index(root, &index_path)?;

    let entries = load_index(&index_path)?;
    assert_eq!(entries.len(), 1, "Only the PNG should be indexed");
    Ok(())
}

#[test]
fn build_index_is_sorted_alphabetically() -> TestResult {
    let dir = TempDir::new()?;
    let root = dir.path();
    let img = make_test_image(8, 8, [255, 255, 0, 255]);

    // Setup files
    img.save(root.join("zebra.png"))?;
    img.save(root.join("apple.png"))?;
    img.save(root.join("mango.png"))?;

    let index_path = root.join("index.json");
    build_index(root, &index_path)?;

    let entries = load_index(&index_path)?;

    // Map names for a cleaner assertion message
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["apple", "mango", "zebra"]);

    Ok(())
}

#[test]
fn build_index_scans_subdirectories() -> TestResult {
    let dir = TempDir::new()?;
    let root = dir.path();
    let subdir = root.join("pokemon");

    std::fs::create_dir_all(&subdir).expect("Failed to create test subdirectory");

    let img = make_test_image(8, 8, [255, 0, 255, 255]);
    img.save(root.join("top_level.png"))?;
    img.save(subdir.join("nested.png"))?;

    let index_path = root.join("index.json");
    build_index(root, &index_path)?;

    let entries = load_index(&index_path)?;
    assert_eq!(entries.len(), 2, "Should find images in subdirectories");

    Ok(())
}
#[test]
fn build_index_empty_directory_produces_empty_array() -> TestResult {
    let dir = TempDir::new()?;
    let index_path = dir.path().join("index.json");
    build_index(dir.path(), &index_path)?;

    let json = std::fs::read_to_string(&index_path)?;
    let entries: Vec<ImageEntry> = serde_json::from_str(&json)?;
    assert!(entries.is_empty());

    Ok(())
}

// --- Rasterization ---

#[cfg(feature = "rasterize")]
mod rasterize_tests {
    use super::*;
    use px2ansi::{RasterTheme, rasterize_ansi_with_theme};

    // Re-using the same type alias for consistency
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn rasterize_ansi_produces_valid_image() -> TestResult {
        let img = make_test_image(8, 8, [255, 100, 50, 255]);
        let opts = RenderOptions::default();
        let mut buf = Vec::new();

        opts.render(&img, &mut buf)?;

        let png = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight)?;

        assert!(png.width() > 0);
        assert!(png.height() > 0);
        Ok(())
    }

    #[test]
    fn rasterize_empty_input_fails_gracefully() {
        // We DON'T return TestResult here because we WANT to assert an error
        let result = rasterize_ansi_with_theme(&[], RasterTheme::TokyoNight);
        assert!(result.is_err(), "empty input should return an error");
    }

    #[test]
    fn rasterize_different_themes_produce_different_backgrounds() -> TestResult {
        let img = make_test_image(4, 4, [255, 255, 255, 255]);
        let opts = RenderOptions::builder()
            .preset(RenderStylePreset::Ascii)
            .build();
        let mut buf = Vec::new();
        opts.render(&img, &mut buf)?;

        let dark = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight)?;
        let light = rasterize_ansi_with_theme(&buf, RasterTheme::White)?;

        let dark_px = dark.get_pixel(0, 0);
        let light_px = light.get_pixel(0, 0);

        assert_ne!(
            dark_px, light_px,
            "different themes should produce different backgrounds"
        );
        Ok(())
    }
}
