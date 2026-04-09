use anyhow::Result;
use colored::Colorize;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use px2ansi::indexer::ImageEntry;
use px2ansi::render::RenderOptions;
use rand::prelude::IndexedRandom;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ShowCmd {
    pub name: String,
    pub index_path: PathBuf,
    pub render: RenderOptions,
    pub interactive: bool,
}

impl ShowCmd {
    /// Locates and renders an image from a managed index.
    ///
    /// This is the primary way users interact with the tool. It manages the
    /// search-and-display logic, including fuzzy matching and random selection.
    ///
    /// It supports:
    /// 1. `interactive`: A fuzzy-search TUI for when you don't know the exact name.
    /// 2. `random`: For when you're feeling adventurous.
    /// 3. `name`: Tries an exact match, then falls back to a fuzzy search.
    pub fn run<W: Write>(&self, writer: &mut W) -> Result<()> {
        let entries: Vec<ImageEntry> =
            serde_json::from_str(&std::fs::read_to_string(&self.index_path)?)?;
        anyhow::ensure!(!entries.is_empty(), "Index is empty.");

        let entry_opt = if self.interactive {
            prompt_search(&entries)?
        } else {
            search_index(&entries, &self.name)?
        };

        match entry_opt {
            Some(e) => {
                if self.interactive {
                    println!("Showing: {}", e.name.cyan().bold());
                }
                let img = image::open(&e.path)?;
                self.render.render_centered(&img, writer)?;
            }
            None => {
                // User pressed Esc in FuzzySelect, just exit
                return Ok(());
            }
        }
        Ok(())
    }
}

/// Searches the index for an image entry based on the provided name.
///
/// It follows a prioritized search strategy:
/// 1. **Random**: If the name is "random", it picks a random entry from the index.
/// 2. **Exact Match**: It first tries to find a name that matches the input perfectly.
/// 3. **Fuzzy Match**: If no exact match exists, it calculates a fuzzy similarity score.
///
/// If the fuzzy score is too low (30 or below), it errors out to avoid showing
/// something completely unrelated.
fn search_index<'a>(entries: &'a [ImageEntry], name: &str) -> Result<Option<&'a ImageEntry>> {
    if name.to_lowercase() == "random" {
        return Ok(entries.choose(&mut rand::rng()));
    }
    if let Some(e) = entries.iter().find(|e| e.name == name) {
        return Ok(Some(e));
    }
    let matcher = SkimMatcherV2::default();
    let best = entries
        .iter()
        .filter_map(|e| matcher.fuzzy_match(&e.name, name).map(|score| (score, e)))
        .max_by_key(|(score, _)| *score);
    match best {
        Some((score, e)) if score > 30 => {
            println!(
                "{} No exact match for '{}'. Showing: {} (score: {})",
                "Fuzzy:".yellow(),
                name,
                e.name.cyan(),
                score
            );
            Ok(Some(e))
        }
        Some((score, e)) => anyhow::bail!("Best match '{}' score ({}) too low.", e.name, score),
        None => anyhow::bail!("No match found for '{name}'"),
    }
}

/// Spawns an interactive terminal UI for browsing the image index.
///
/// This uses a fuzzy-search selector that allows the user to type and filter
/// through the entire index in real-time. It's particularly useful when you
/// can't remember the exact filename.
///
/// Returns `Ok(None)` if the user cancels the selection (e.g., by pressing Esc).
fn prompt_search(entries: &[ImageEntry]) -> Result<Option<&ImageEntry>> {
    let items: Vec<&String> = entries.iter().map(|e| &e.name).collect();
    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Search for a sprite")
        .items(&items)
        .interact_opt()?;
    Ok(selection.map(|idx| &entries[idx]))
}
