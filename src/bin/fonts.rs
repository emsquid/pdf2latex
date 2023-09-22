use clap::Parser;
use pdf2latex::args::FontArg;
use pdf2latex::fonts::FontBase;

fn main() {
    let args = FontArg::parse();
    for &code in &args.codes {
        if let Err(e) = FontBase::create_family(code, &args) {
            eprintln!("{e}");
        }
    }
}
