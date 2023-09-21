use clap::Parser;
use utils::{
    args::{self, Args},
    font::FontBase,
};

fn main() {
    if let Err(e) = FontBase::try_from(&Args::FontBaseArgs(&args::FontBaseArgs::parse())) {
        eprintln!("{e}");
    }
}
