pub mod constants;
pub mod enums;
pub mod memory;
pub mod utils;
pub mod vm;

use clap::Parser;
use std::{error::Error, path::PathBuf};
use termios::{
    tcsetattr, BRKINT, ECHO, ICANON, ICRNL, IGNBRK, IGNCR, INLCR, ISTRIP, IXON, PARMRK, TCSANOW,
};
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
    // Setup code
    let stdin = 0;
    let termios = termios::Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios;
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &new_termios).unwrap();

    // Run machine
    let args = Cli::parse();
    let mut machine = Machine::default();

    if args.debug {
        machine.enter_debug_mode();
    }

    machine.load_image(args.file)?;
    machine.run();

    // Cleanup code
    tcsetattr(stdin, TCSANOW, &termios).unwrap();

    Ok(())
}
