use std::{fs, path::Path};

use image::GenericImageView;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

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

/// Attempts to decode a single directory entry into an [`ImageEntry`].
///
/// Returns `Ok(None)` for non-image files or entries that should be skipped,
/// and `Ok(Some(_))` on success.  Image-open failures are silently skipped;
/// canonicalization failures are propagated.
fn try_index_entry(entry: walkdir::DirEntry) -> anyhow::Result<Option<ImageEntry>> {
    let path = entry.path().to_owned();
    if !path.is_file() {
        return Ok(None);
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
        return Ok(None);
    }
    let Ok(img) = image::open(&path) else {
        return Ok(None);
    };
    let absolute_path = fs::canonicalize(&path)?;
    let Some(stem) = path.file_stem() else {
        return Ok(None);
    };
    Ok(Some(ImageEntry {
        name: stem.to_string_lossy().into_owned(),
        path: absolute_path.to_string_lossy().into_owned(),
        dimensions: img.dimensions(),
    }))
}

/// Builds a searchable JSON index of image files found within a directory.
///
/// Recursively scans `dir` for supported image formats (`png`, `jpg`, `jpeg`,
/// `webp`, `bmp`), extracts their dimensions, and writes a sorted JSON manifest
/// to `output_path`. Returns the JSON string on success.
///
/// When the `parallel` feature is enabled, image decoding is performed
/// concurrently across all available CPU cores via rayon.
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
    let mut index = Vec::new();

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::{ParallelBridge, ParallelIterator};

        // par_bridge drives the WalkDir iterator from a single thread and
        // distributes DirEntry items to the rayon thread pool.  Each
        // try_index_entry call opens and decodes an image independently, so
        // CPU-bound decoding work is parallelised while the iterator stays
        // sequential.
        let results: Vec<anyhow::Result<Option<ImageEntry>>> = WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .par_bridge()
            .map(try_index_entry)
            .collect();

        for result in results {
            if let Some(entry) = result? {
                index.push(entry);
            }
        }
    }

    #[cfg(not(feature = "parallel"))]
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if let Some(e) = try_index_entry(entry)? {
            index.push(e);
        }
    }

    index.sort_by(|a, b| a.name.cmp(&b.name));
    let json_data = serde_json::to_string_pretty(&index)?;

    fs::write(output_path, &json_data)?;

    Ok(json_data)
}
