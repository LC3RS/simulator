use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to object file
    ///
    /// Object file extension should generally be .obj
    /// but it's not strictly checked
    #[arg(short, long, value_name = "FILE")]
    pub file: PathBuf,

    /// Turn on debug-mode
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}
