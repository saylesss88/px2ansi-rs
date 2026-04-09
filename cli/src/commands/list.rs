use anyhow::Result;
use colored::Colorize;
use std::{io::Write, path::PathBuf};

/// Parameters for listing the contents of an index.
#[derive(Debug)]
pub struct ListCmd {
    /// Path to the JSON index file.
    pub index_path: PathBuf,
    /// Maximum number of entries to display.
    pub count: Option<usize>,
}

impl ListCmd {
    pub fn run<W: Write>(&self, writer: &mut W) -> Result<()> {
        let content = std::fs::read_to_string(&self.index_path)?;
        let entries: Vec<px2ansi::indexer::ImageEntry> = serde_json::from_str(&content)?;

        let limit = self.count.unwrap_or(entries.len()).min(entries.len());

        writeln!(
            writer,
            "{} Showing {} of {} entries:",
            "Index:".magenta().bold(),
            limit,
            entries.len()
        )?;

        for entry in entries.iter().take(limit) {
            writeln!(
                writer,
                "  • {:<20} {}x{}px",
                entry.name.cyan(),
                entry.dimensions.0.to_string().dimmed(),
                entry.dimensions.1.to_string().dimmed()
            )?;
        }
        Ok(())
    }
}
