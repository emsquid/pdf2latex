use clap::Parser;
use pdf2latex::args::{Args, FontArgs};
use pdf2latex::fonts::FontBase;

fn main() {
    if let Err(e) = FontBase::try_from(&Args::Font(&FontArgs::parse())) {
        eprintln!("{e}");
    }
}
