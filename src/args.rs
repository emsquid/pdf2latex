use crate::fonts::code::Code;
use clap::{arg, command, Parser};
use std::path::PathBuf;

pub enum Args<'a> {
    Main(&'a MainArg),
    Font(&'a FontArg),
}

impl<'a> Args<'a> {
    #[must_use]
    pub fn verbose(&self) -> bool {
        match self {
            Args::Main(args) => args.verbose,
            Args::Font(args) => args.verbose,
        }
    }

    #[must_use]
    pub fn create(&self) -> Option<&Vec<Code>> {
        match self {
            Args::Main(_) => None,
            Args::Font(args) => Some(&args.create),
        }
    }

    #[must_use]
    pub fn threads(&self) -> usize {
        match self {
            Args::Main(args) => args.threads,
            Args::Font(args) => args.threads,
        }
    }
}

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
}

/// Arguments the user can give when using pdf2latex to generate `FontBases`
#[derive(Parser)]
#[command(author, version, about)]
pub struct FontArg {
    /// Font files to create
    pub create: Vec<Code>,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 8)]
    pub threads: usize,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
