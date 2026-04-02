use clap::CommandFactory;
use clap_mangen::Man;
use std::path::PathBuf;

// We need to duplicate the Cli definition here or expose it from the lib.
// The easiest way is to expose it from lib.rs.
use px2ansi_rs::Cli;

fn main() -> anyhow::Result<()> {
    let out_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let man_dir = out_dir.join("man");
    std::fs::create_dir_all(&man_dir)?;

    let cmd = Cli::command();

    // Generate top-level man page
    let man = Man::new(cmd.clone());
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    std::fs::write(man_dir.join("px2ansi-rs.1"), buffer)?;

    // Generate per-subcommand man pages
    for subcommand in cmd.get_subcommands() {
        let name = format!("px2ansi-rs-{}", subcommand.get_name());
        let name: &'static str = Box::leak(name.into_boxed_str());
        let sub_man = Man::new(subcommand.clone().name(name));
        let mut buffer = Vec::new();
        sub_man.render(&mut buffer)?;
        std::fs::write(man_dir.join(format!("{name}.1")), buffer)?;
        println!("Generated man/{name}.1");
    }

    println!("Generated man/px2ansi-rs.1");
    Ok(())
}
