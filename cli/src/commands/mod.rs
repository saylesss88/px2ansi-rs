pub mod convert;
pub mod index;
pub mod list;
pub mod show;

use anyhow::Result;
use convert::ConvertCmd;
use index::IndexCmd;
use list::ListCmd;
use show::ShowCmd;

/// The internal representation of the action the user wants to perform.
/// This bridges the gap between raw CLI arguments and execution logic.
pub enum Command {
    Convert(ConvertCmd),
    Index(IndexCmd),
    List(ListCmd),
    Show(ShowCmd),
}

pub fn handle_command(cmd: &Command) -> Result<()> {
    match cmd {
        Command::Convert(convert) => convert.run(),
        Command::Index(index) => index.run(),
        Command::List(list) => list.run(),
        Command::Show(show) => show.run(),
    }
}
