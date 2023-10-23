use crate::fonts::code::Code;
use clap::{arg, command, Parser};
use std::path::PathBuf;

/// Arguments the user can give when using pdf2latex to parse a pdf to a latex file
#[derive(Parser)]
#[command(author, version, about)]
pub struct MainArg {
    /// PDF to convert
    pub input: PathBuf,

    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 8)]
    pub threads: usize,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Parse only selected pages, examples: 1,3,5,7-9,11,20,23-63
    #[arg(short, long)]
    pub pages: Option<String>,
}

/// Arguments the user can give when using pdf2latex to generate `FontBases`
#[derive(Parser)]
#[command(author, version, about)]
pub struct FontArg {
    /// Font files to create
    pub codes: Vec<Code>,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 8)]
    pub threads: usize,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
