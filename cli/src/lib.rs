pub mod cli;
pub mod commands;
pub mod config;
pub mod output;
pub mod render;

// Re-export types
pub use crate::commands::Command;
pub use crate::render::build_render_options;
pub use cli::{Cli, Commands};
pub use commands::convert::ConvertCmd;
pub use commands::index::IndexCmd;
pub use commands::list::ListCmd;
pub use commands::show::ShowCmd;
pub use config::Config;
pub use px2ansi::{Density, RenderOptions, RenderStylePreset, ResizeFilter};

use std::path::PathBuf;

#[derive(Debug)]
pub struct ResolvedOptions {
    pub latency: bool,
    pub index_path: PathBuf,
}

impl ResolvedOptions {
    pub fn from_cli_and_config(cli: &Cli, cfg: &Config) -> Self {
        Self {
            latency: cli.latency || cfg.latency,
            index_path: cli
                .index
                .as_deref()
                .map_or_else(|| PathBuf::from(&cfg.index), PathBuf::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use clap::Parser;
    use px2ansi::{CharsetMode, ColorMode, Density, RenderStylePreset, ResizeFilter};
    use std::path::PathBuf;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    // --- ResolvedOptions ---

    #[test]
    fn resolved_options_prefers_cli_over_config_and_combines_latency() {
        let cli = Cli::parse_from([
            "px2ansi-rs",
            "index",
            "./tests",
            "-I",
            "index.json",
            "--latency",
        ]);
        let cfg = Config {
            index: "config_index.json".into(),
            latency: false,
            ..Default::default()
        };
        let opts = ResolvedOptions::from_cli_and_config(&cli, &cfg);
        assert!(opts.latency);
        assert_eq!(opts.index_path, PathBuf::from("index.json"));
    }

    #[test]
    fn resolved_options_falls_back_to_config_index_when_cli_omits_it() {
        let cli = Cli::parse_from(["px2ansi-rs", "list"]);
        let cfg = Config {
            index: "config_index.json".into(),
            latency: false,
            ..Default::default()
        };
        let opts = ResolvedOptions::from_cli_and_config(&cli, &cfg);
        assert_eq!(opts.index_path, PathBuf::from("config_index.json"));
    }

    #[test]
    fn resolved_options_latency_true_when_only_config_sets_it() {
        let cli = Cli::parse_from(["px2ansi-rs", "list"]);
        let cfg = Config {
            index: "index.json".into(),
            latency: true,
            ..Default::default()
        };
        let opts = ResolvedOptions::from_cli_and_config(&cli, &cfg);
        assert!(opts.latency);
    }

    #[test]
    fn resolved_options_latency_false_when_neither_sets_it() {
        let cli = Cli::parse_from(["px2ansi-rs", "list"]);
        let cfg = Config {
            latency: false,
            ..Default::default()
        };
        let opts = ResolvedOptions::from_cli_and_config(&cli, &cfg);
        assert!(!opts.latency);
    }

    // --- build_render_options ---

    #[test]
    fn build_render_options_applies_overrides_and_no_color() {
        let opts = build_render_options(None, None, Some(80), None, None, false);
        // let opts = build_render_options(None, None, Some(80), None, true, None);
        assert_eq!(opts.width(), Some(80));
        assert_eq!(opts.color_mode(), ColorMode::None);

        let opts2 = build_render_options(None, None, None, None, None, false);
        assert_eq!(opts2.width(), None);
        assert_eq!(opts2.color_mode(), ColorMode::None);
    }

    #[test]
    fn build_render_options_braille_preset_sets_charset() {
        let opts = build_render_options(
            Some(RenderStylePreset::Braille),
            None,
            None,
            None,
            None,
            false,
        );
        assert_eq!(opts.charset(), CharsetMode::Braille);
    }

    #[test]
    fn build_render_options_full_block_sets_full_flag() {
        let opts = build_render_options(
            Some(RenderStylePreset::FullBlock),
            None,
            None,
            None,
            None,
            false,
        );
        assert_eq!(opts.charset(), CharsetMode::Unicode);
        assert!(opts.style().is_full());
    }

    #[test]
    fn build_render_options_dense_sets_heavy_density() {
        let opts = build_render_options(
            Some(RenderStylePreset::Dense),
            None,
            None,
            None,
            None,
            false,
        );
        assert!(matches!(opts.style().density(), Density::Heavy));
    }

    #[test]
    fn build_render_options_density_override_beats_preset() {
        // Dense preset sets Heavy, but explicit Light should win
        let opts = build_render_options(
            Some(RenderStylePreset::Ascii),
            Some(Density::Light),
            None,
            None,
            None,
            false,
        );
        assert!(matches!(opts.style().density(), Density::Light));
    }

    #[test]
    fn build_render_options_nearest_filter() {
        let opts = build_render_options(None, None, None, Some(ResizeFilter::Nearest), None, false);
        assert_eq!(opts.filter(), image::imageops::FilterType::Nearest);
    }

    // --- CLI parsing ---

    #[test]
    fn cli_parses_convert_with_all_flags() {
        let cli = Cli::parse_from([
            "px2ansi-rs",
            "convert",
            "input.png",
            "--style",
            "braille",
            "--width",
            "120",
            "--filter",
            "nearest",
            "--color-mode",
            "none",
            "--output",
            "out.txt",
        ]);

        let Commands::Convert {
            input,
            style,
            width,
            filter,
            color_mode,
            output,
            ..
        } = cli.command
        else {
            unreachable!("Cli::parse_from should have produced Commands::Convert");
        };

        assert_eq!(input, PathBuf::from("input.png"));
        assert_eq!(style, Some(RenderStylePreset::Braille));
        assert_eq!(width, Some(120));
        assert_eq!(filter, Some(ResizeFilter::Nearest));
        assert_eq!(color_mode, Some(ColorMode::TrueColor));
        assert_eq!(output, Some(PathBuf::from("out.txt")));
    }

    #[test]
    fn cli_parses_show_with_style_and_interactive() {
        let cli = Cli::parse_from(["px2ansi-rs", "show", "pikachu", "--style", "ascii", "-i"]);

        let Commands::Show {
            name,
            style,
            interactive,
            ..
        } = cli.command
        else {
            unreachable!("Cli::parse_from should have produced Commands::Show");
        };

        assert_eq!(name, "pikachu");
        assert_eq!(style, Some(RenderStylePreset::Ascii));
        assert!(interactive);
    }

    #[test]
    fn cli_show_defaults_to_random() {
        let cli = Cli::parse_from(["px2ansi-rs", "show"]);

        // This is much more concise and avoids the explicit panic
        assert!(
            matches!(cli.command, Commands::Show { ref name, .. } if name == "random"),
            "Expected Show command with name 'random', got {:?}",
            cli.command
        );
    }

    #[test]
    fn cli_parses_list_with_count() -> TestResult {
        let cli = Cli::parse_from(["px2ansi-rs", "list", "--count", "10"]);

        let Commands::List { count } = cli.command else {
            return Err("Expected List command variant".into());
        };

        assert_eq!(count, Some(10));
        Ok(())
    }

    #[test]
    fn cli_parses_index_with_output() -> TestResult {
        let cli = Cli::parse_from([
            "px2ansi-rs",
            "index",
            "./sprites",
            "--output",
            "sprites.json",
        ]);

        // If it's NOT Commands::Index, it returns the error immediately
        let Commands::Index { dir, output } = cli.command else {
            return Err("Expected Index command variant".into());
        };

        assert_eq!(dir, PathBuf::from("./sprites"));
        assert_eq!(output, Some(PathBuf::from("sprites.json")));

        Ok(())
    }

    #[test]
    fn cli_global_latency_flag_works_on_show() {
        let cli = Cli::parse_from(["px2ansi-rs", "--latency", "show", "pikachu"]);
        assert!(cli.latency);
    }

    #[test]
    fn cli_global_index_flag_works_on_list() {
        let cli = Cli::parse_from(["px2ansi-rs", "-I", "custom.json", "list"]);
        assert_eq!(cli.index.as_deref(), Some("custom.json"));
    }

    // --- RenderStylePreset parsing ---

    #[test]
    fn style_preset_parses_all_variants() {
        use std::str::FromStr;
        let cases = [
            ("ansi", RenderStylePreset::Ansi),
            ("braille", RenderStylePreset::Braille),
            ("fade", RenderStylePreset::Fade),
            ("ascii", RenderStylePreset::Ascii),
            ("kanji", RenderStylePreset::Kanji),
            ("chinese", RenderStylePreset::Chinese),
            ("full-block", RenderStylePreset::FullBlock),
            ("dense", RenderStylePreset::Dense),
        ];
        for (input, expected) in cases {
            assert_eq!(
                RenderStylePreset::from_str(input),
                Ok(expected),
                "failed to parse '{input}'"
            );
        }
    }

    #[test]
    fn style_preset_rejects_invalid() {
        use std::str::FromStr;

        assert!(
            RenderStylePreset::from_str("invalid").is_err(),
            "\"invalid\" should not parse as a RenderStylePreset"
        );
        assert!(
            RenderStylePreset::from_str("").is_err(),
            "empty string should not parse as a RenderStylePreset"
        );
    }
}
