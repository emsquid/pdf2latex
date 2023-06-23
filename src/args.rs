use clap::{arg, command, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// PDF to convert
    pub input: PathBuf,

    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Font size
    #[arg(short, long, default_value_t = 11, value_parser = clap::value_parser!(u32).range(10..=12))]
    pub size: u32,
}
