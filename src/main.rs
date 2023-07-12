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
    let mut pdf = pdf::Pdf::load(&args.input)?;

    pdf.guess(args)?;
    match &args.output {
        Some(output) => pdf.save_content(output)?,
        None => println!("\n{}", pdf.get_content()),
    }
    pdf.pages[0].debug_dist_avg();
    pdf.pages[0].debug_image().save("./test/debug.png")?;

    let latex = latex::Latex::from(pdf);
    latex.save("test/test.tex")?;

    Ok(())
}

fn main() {
    if let Err(err) = process(&args::Args::parse()) {
        eprintln!("{err}");
    }
}
