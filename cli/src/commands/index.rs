use anyhow::Result;
use colored::Colorize;
use std::{io::Write, path::PathBuf};

/// Parameters for creating a new asset index from a directory.
#[derive(Debug)]
pub struct IndexCmd {
    /// The source directory containing image files.
    pub dir: PathBuf,
    /// The destination path for the generated JSON index.
    pub output: PathBuf,
}

impl IndexCmd {
    /// Runs the command.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the provided writer fails or if
    /// the command logic encounters a processing error.
    pub fn run<W: Write>(&self, writer: &mut W) -> Result<()> {
        px2ansi::indexer::build_index(&self.dir, &self.output)?;

        writeln!(
            writer,
            "{} Successfully indexed {} to {}",
            "Success:".green().bold(),
            self.dir.display(),
            self.output.display()
        )?;

        Ok(())
    }
}
