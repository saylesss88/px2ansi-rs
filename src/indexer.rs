use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::fs;

/// Represents a single image discovered during the indexing process.
///
/// We store the dimensions so that future features (like filtering by size)
/// can work without re-reading the actual image files.
#[derive(Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    /// The filename without its extension (e.g, "charizard" instead of "charizard.png")
    pub name: String,
    /// The full system path to the image file
    pub path: String,
    /// Width and Height in pixels
    pub dimensions: (u32, u32),
}

/// Scans a directory for supported image files and builds a JSON index.
///
/// This function ignores non-image files and subdirectories. It actually opens
/// each image briefly to extract dimensions, so it may take a moment for
/// very large directories.
pub fn build_index(dir: &str) -> anyhow::Result<String> {
    let mut index = Vec::new();

    // Iterate through the directory entries
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // We only care about regular files, not subdirectories or symlinks
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Filter for common web/system image formats
            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
                // Open image just to get metadata (dimensions), if file is corrupted, we just skip it
                if let Ok(img) = image::open(&path) {
                    index.push(ImageEntry {
                        // Use file_stem so the user can 'show' by name without typing .png
                        name: path.file_stem().unwrap().to_string_lossy().into(),
                        path: path.to_string_lossy().into(),
                        dimensions: img.dimensions(),
                    });
                }
            }
        }
    }
    // Sort the index alphabetically by name for a more predictable JSON file
    index.sort_by(|a, b| a.name.cmp(&b.name));

    // Serialize to Pretty JSON so it's readable if the user opens it in an editor
    Ok(serde_json::to_string_pretty(&index)?)
}
