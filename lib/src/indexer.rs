use std::{fs, path::Path};

use image::GenericImageView;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Represents a single image discovered during the indexing process.
///
/// Entries are produced by [`build_index`] and can be deserialized from
/// the generated JSON index file for use in search and display workflows.
///
/// # Examples
///
/// ```no_run
/// use px2ansi::indexer::ImageEntry;
///
/// let json = std::fs::read_to_string("index.json").unwrap();
/// let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();
///
/// for entry in &entries {
///     println!("{}: {}x{}", entry.name, entry.dimensions.0, entry.dimensions.1);
/// }
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageEntry {
    /// The filename without its extension (e.g., "charizard" instead of "charizard.png")
    pub name: String,
    /// The absolute system path to the image file, ensuring it works from any directory
    pub path: String,
    /// Width and Height in pixels
    pub dimensions: (u32, u32),
}

/// Builds a searchable JSON index of image files found within a directory.
///
/// Recursively scans `dir` for supported image formats (`png`, `jpg`, `jpeg`,
/// `webp`, `bmp`), extracts their dimensions, and writes a sorted JSON manifest
/// to `output_path`. Returns the JSON string on success.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use px2ansi::indexer::build_index;
///
/// let json = build_index(
///     Path::new("/home/user/sprites"),
///     Path::new("/home/user/sprites/index.json"),
/// ).expect("failed to build index");
///
/// println!("Index contains {} bytes of JSON", json.len());
/// ```
///
/// # Errors
///
/// This function returns an error if:
/// * **Path Resolution Fails:** Any valid image file encountered cannot be canonicalized
///   (e.g., due to insufficient permissions or a broken symlink).
/// * **Serialization Error:** The collected index cannot be serialized into a JSON string
///   via `serde_json`.
/// * **I/O Failure:** The final JSON index cannot be written to the `output_path` (e.g.,
///   the directory doesn't exist, is read-only, or the disk is full).
pub fn build_index(dir: &Path, output_path: &Path) -> anyhow::Result<String> {
    // Collect valid paths first — WalkDir is not Send so can't be parallelized directly
    let paths: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| {
            let ext = e
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp")
        })
        .collect();

    // Process images in parallel with parallel feature (rayon), sequential otherwise
    #[cfg(feature = "parallel")]
    let mut index: Vec<ImageEntry> = paths
        .par_iter()
        .filter_map(|entry| process_entry(entry.path()))
        .collect();

    #[cfg(not(feature = "parallel"))]
    let mut index: Vec<ImageEntry> = paths
        .iter()
        .filter_map(|entry| process_entry(entry.path()))
        .collect();

    index.sort_by(|a, b| a.name.cmp(&b.name));
    let json_data = serde_json::to_string_pretty(&index)?;
    fs::write(output_path, &json_data)?;
    Ok(json_data)
}

/// Processes a single image path into an [`ImageEntry`].
/// Returns `None` if the image cannot be opened or the path has no stem.
fn process_entry(path: &Path) -> Option<ImageEntry> {
    let img = image::open(path).ok()?;
    let absolute_path = fs::canonicalize(path).ok()?;
    let stem = path.file_stem()?;
    Some(ImageEntry {
        name: stem.to_string_lossy().into_owned(),
        path: absolute_path.to_string_lossy().into_owned(),
        dimensions: img.dimensions(),
    })
}
