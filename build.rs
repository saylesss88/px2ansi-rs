use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, PowerShell, Zsh},
};
use std::env;
use std::io::Error;

// This allows us to use the Cli struct defined in your external file
include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();
    let bin_name = "px2ansi-rs";

    // Generate completions for common shells
    generate_to(Bash, &mut cmd, bin_name, &outdir)?;
    generate_to(Zsh, &mut cmd, bin_name, &outdir)?;
    generate_to(Fish, &mut cmd, bin_name, &outdir)?;
    generate_to(PowerShell, &mut cmd, bin_name, &outdir)?;

    println!("cargo:warning=completion scripts generated in {:?}", outdir);

    Ok(())
}
