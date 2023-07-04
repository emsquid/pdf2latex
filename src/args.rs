use clap::{arg, command, Parser};
use std::path::PathBuf;

use crate::font::Code;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// PDF to convert
    pub input: PathBuf,

    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Create font files
    #[arg(short, long, value_enum)]
    pub create: Option<Vec<Code>>,

    /// Silent mode
    #[arg(short, long, default_value_t = false)]
    pub silent: bool,
}
