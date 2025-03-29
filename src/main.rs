pub mod cli;
pub mod constants;
pub mod enums;
pub mod error;
pub mod memory;
pub mod utils;
pub mod vm;

use clap::Parser;
use cli::Cli;
use error::Result;
use termios::{
    tcsetattr, BRKINT, ECHO, ICANON, ICRNL, IGNBRK, IGNCR, INLCR, ISTRIP, IXON, PARMRK, TCSANOW,
};
use vm::Machine;

fn main() -> Result<()> {
    // Setup code
    let stdin = 0;
    let termios = termios::Termios::from_fd(stdin)?;
    let mut new_termios = termios;
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &new_termios)?;

    // Run machine
    let args = Cli::parse();
    let mut machine = Machine::default();

    if args.debug {
        machine.enter_debug_mode();
    }

    machine.load_image(args.file)?;
    machine.run();

    // Cleanup code
    tcsetattr(stdin, TCSANOW, &termios)?;

    Ok(())
}
