use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, PowerShell, Zsh},
};
use std::env;
use std::io::Error;

// Include the CLI structure defined in the main source tree.
//
// This allows the build script to inspect the `Cli` struct at compile time
// to generate shell completion files without duplicating the CLI logic.
include!("src/cli.rs");

fn main() -> Result<(), Error> {
    // Cargo sets the OUT_DIR environment variable to a directory specific to the crate.
    // This is where we should place any generated files during the build process.
    let Some(outdir) = env::var_os("OUT_DIR") else {
        return Ok(());
    };

    // Create a clap::Command instance from our Cli struct.
    // This contains all the metadata about subcommands, arguments, and help text.
    let mut cmd = Cli::command();

    // The binary name as it will be called in the terminal.
    let bin_name = "px2ansi-rs";

    // Generate completion scripts for the most popular shells.
    // These files will be named _px2ansi-rs (Zsh), px2ansi-rs.bash, etc.
    generate_to(Bash, &mut cmd, bin_name, &outdir)?;
    generate_to(Zsh, &mut cmd, bin_name, &outdir)?;
    generate_to(Fish, &mut cmd, bin_name, &outdir)?;
    generate_to(PowerShell, &mut cmd, bin_name, &outdir)?;

    // Inform the developer where the files were saved during the build.
    println!(
        "cargo:warning=completion scripts generated in {}",
        outdir.display()
    );

    Ok(())
}
