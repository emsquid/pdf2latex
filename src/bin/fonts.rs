use clap::Parser;
use pdf2latex::args::{Args, FontArg};
use pdf2latex::fonts::FontBase;

fn main() {
    if let Err(e) = FontBase::try_from(&Args::Font(&FontArg::parse())) {
        eprintln!("{e}");
    }
}
