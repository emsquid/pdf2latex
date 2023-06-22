use clap::Parser;

mod args;
mod dictionary;
mod font;
mod glyph;
mod latex;
mod pdf;
mod result;
mod text;
mod utils;

fn process(args: &args::Args) -> result::Result<()> {
    let mut file = pdf::Pdf::load(&args.input)?;

    file.guess(args)?;
    match &args.output {
        Some(output) => file.save_content(output)?,
        None => println!("\n{}", file.get_content()?),
    }

    Ok(())
}

fn main() {
    if let Err(err) = process(&args::Args::parse()) {
        eprintln!("{err}");
    }
}
