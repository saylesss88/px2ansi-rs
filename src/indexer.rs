use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct ImageEntry {
    pub name: String,
    pub path: String,
    pub dimensions: (u32, u32),
}

pub fn build_index(dir: &str) -> anyhow::Result<String> {
    let mut index = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process files with image extensions
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
                // Open image just to get metadata (dimensions)
                if let Ok(img) = image::open(&path) {
                    index.push(ImageEntry {
                        name: path.file_stem().unwrap().to_string_lossy().into(),
                        path: path.to_string_lossy().into(),
                        dimensions: img.dimensions(),
                    });
                }
            }
        }
    }

    // Serialize to Pretty JSON
    Ok(serde_json::to_string_pretty(&index)?)
}
