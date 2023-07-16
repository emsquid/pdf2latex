use crate::font::Code;
use clap::{arg, command, Parser};
use std::path::PathBuf;

/// Arguments the user can give when using pdf2latex
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// PDF to convert
    pub input: PathBuf,

    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 8)]
    pub threads: usize,

    /// Create font files
    #[arg(short, long, value_enum, num_args(1..))]
    pub create: Option<Vec<Code>>,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
