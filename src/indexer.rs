use std::fs;
use std::path::Path;

use image::GenericImageView;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

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
pub fn build_index(dir: &Path, output_path: &Path) -> anyhow::Result<String> {
    let mut index = Vec::new();

    // Iterate through the directory entries
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
            continue;
        }
        let Ok(img) = image::open(path) else { continue };
        let absolute_path = fs::canonicalize(path)?;
        let Some(stem) = path.file_stem() else {
            continue;
        };
        index.push(ImageEntry {
            name: stem.to_string_lossy().into_owned(),
            path: absolute_path.to_string_lossy().into_owned(),
            dimensions: img.dimensions(),
        });
    }

    index.sort_by(|a, b| a.name.cmp(&b.name));
    let json_data = serde_json::to_string_pretty(&index)?;

    // fs::write handles &Path perfectly
    fs::write(output_path, &json_data)?;

    Ok(json_data)
}
