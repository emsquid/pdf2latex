use clap::{arg, command, Parser};
use std::path::PathBuf;

use crate::code::Code;

pub enum Args<'a> {
    MainArgs(&'a MainArgs),
    FontBaseArgs(&'a FontBaseArgs),
}

impl<'a> Args<'a> {
    pub fn verbose(&self) -> bool {
        match self {
            Args::MainArgs(args) => args.verbose,
            Args::FontBaseArgs(args) => args.verbose,
        }
    }

    pub fn create(&self) -> Option<&Vec<Code>> {
        match self {
            Args::MainArgs(_) => None,
            Args::FontBaseArgs(args) => args.create.as_ref(),
        }
    }

    pub fn threads(&self) -> usize {
        match self {
            Args::MainArgs(args) => args.threads,
            Args::FontBaseArgs(args) => args.threads,
        }
    }
}

/// Arguments the user can give when using pdf2latex to parse a pdf to a latex file
#[derive(Parser)]
#[command(author, version, about)]
pub struct MainArgs {
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
}

/// Arguments the user can give when using pdf2latex to generate FontBases
#[derive(Parser)]
#[command(author, version, about)]
pub struct FontBaseArgs {
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
