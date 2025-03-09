pub mod constants;
pub mod enums;
pub mod memory;
pub mod utils;
pub mod vm;

use clap::Parser;
use std::{error::Error, path::PathBuf};
use vm::Machine;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Image file to load
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,

    /// Turn on debug-mode
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let mut machine = Machine::default();

    if args.debug {
        machine.enter_debug_mode();
    }

    machine.load_image(args.file)?;
    machine.run();

    Ok(())
}
