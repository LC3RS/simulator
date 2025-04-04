pub mod cli;
pub mod constants;
pub mod enums;
pub mod error;
pub mod memory;
pub mod utils;
pub mod vm;

use clap::Parser;
use cli::Cli;
use crossterm::terminal;
use error::Result;
use vm::Machine;

fn main() -> Result<()> {
    let args = Cli::parse();

    // Setup code
    terminal::enable_raw_mode().expect("Could not turn on raw mode");

    // Run machine
    let mut machine = Machine::default();

    if args.debug {
        machine.enter_debug_mode();
    }

    machine.load_image(args.file)?;
    machine.run();

    // Cleanup code
    terminal::disable_raw_mode().expect("Could not turn off raw mode");

    Ok(())
}
