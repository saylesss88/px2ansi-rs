pub mod convert;
pub mod index;
pub mod list;
pub mod show;

use anyhow::Result;
use convert::ConvertCmd;
use index::IndexCmd;
use list::ListCmd;
use show::ShowCmd;
use std::io::Write;

/// The internal representation of the action the user wants to perform.
/// This bridges the gap between raw CLI arguments and execution logic.
pub enum Command {
    Convert(ConvertCmd),
    Index(IndexCmd),
    List(ListCmd),
    Show(ShowCmd),
}

/// Dispatches the provided command to its respective handler.
///
/// This is the central entry point for executing CLI logic. It routes the
/// [`Command`] variant to the appropriate `run` method, passing the
/// mutable writer along for output.
///
/// # Errors
///
/// This function returns an error if the underlying command execution fails.
/// Common failure points include:
///
/// * **I/O Errors**: Issues writing to the provided `writer` (e.g., broken pipe).
/// * **Processing Errors**: Command-specific failures such as file not found
///   during conversion or invalid index references.
pub fn handle_command<W: Write>(cmd: &Command, writer: &mut W) -> Result<()> {
    match cmd {
        Command::Convert(convert) => convert.run(writer),
        Command::Index(index) => index.run(writer),
        Command::List(list) => list.run(writer),
        Command::Show(show) => show.run(writer),
    }
}
