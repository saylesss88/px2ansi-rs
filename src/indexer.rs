use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::fs;

/// Represents a single image discovered during the indexing process.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageEntry {
    /// The filename without its extension (e.g., "charizard" instead of "charizard.png")
    pub name: String,
    /// The absolute system path to the image file, ensuring it works from any directory
    pub path: String,
    /// Width and Height in pixels
    pub dimensions: (u32, u32),
}

/// Scans a directory for supported image files and builds a JSON index.
///
/// This function converts all relative image paths into absolute paths using
/// `fs::canonicalize` to ensure the index remains valid regardless of the
/// current working directory.
pub fn build_index(dir: &str, output_path: &str) -> anyhow::Result<String> {
    let mut index = Vec::new();

    // Iterate through the directory entries
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process regular files
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Supported image formats
            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
                // Open image to extract dimensions and verify it's a valid image
                if let Ok(img) = image::open(&path) {
                    // Resolve the absolute path so the index is "portable" across directories
                    let absolute_path = fs::canonicalize(&path)?;

                    index.push(ImageEntry {
                        name: path.file_stem().unwrap().to_string_lossy().into(),
                        path: absolute_path.to_string_lossy().into(),
                        dimensions: img.dimensions(),
                    });
                }
            }
        }
    }

    // 1. Sort alphabetically by name for cleaner 'list' output
    index.sort_by(|a, b| a.name.cmp(&b.name));

    // 2. Serialize to pretty JSON
    let json_data = serde_json::to_string_pretty(&index)?;

    // 3. Write to the specified output file
    fs::write(output_path, &json_data)?;

    Ok(json_data)
}
