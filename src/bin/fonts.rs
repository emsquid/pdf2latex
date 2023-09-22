use clap::Parser;
use pdf2latex::utils::{
    args::{Args, FontArgs},
    font::FontBase,
};

fn main() {
    if let Err(e) = FontBase::try_from(&Args::Font(&FontArgs::parse())) {
        eprintln!("{e}");
    }
}
