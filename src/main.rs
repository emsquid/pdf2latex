use clap::Parser;

mod args;
mod font;
mod glyph;
mod latex;
mod pdf;
mod result;
mod text;
mod utils;

fn process(args: &args::Args) -> result::Result<()> {
    let mut pdf = pdf::Pdf::load(&args.input)?;

    pdf.guess(args)?;
    match &args.output {
        Some(output) => latex::Latex::from(&pdf).save(output)?,
        None => println!("{}", pdf.get_content()),
    }
    pdf.pages[0].debug_dist_avg();
    pdf.pages[0].debug_image().save("./test/debug.png")?;

    Ok(())
}

fn main() {
    if let Err(err) = process(&args::Args::parse()) {
        eprintln!("{err}");
    }
}
