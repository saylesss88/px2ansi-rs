use anyhow::Result;
use std::path::PathBuf;

/// Parameters for creating a new asset index from a directory.
#[derive(Debug)]
pub struct IndexCmd {
    /// The source directory containing image files.
    pub dir: PathBuf,
    /// The destination path for the generated JSON index.
    pub output: PathBuf,
}

impl IndexCmd {
    /// Scans the source directory and writes a JSON manifest to the output path.
    pub fn run(&self) -> Result<()> {
        px2ansi_rs::indexer::build_index(&self.dir, &self.output)?;
        Ok(())
    }
}
