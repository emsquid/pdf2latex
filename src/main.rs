use clap::Parser;

mod args;
mod font;
mod glyph;
mod latex;
mod pdf;
mod result;
mod text;
mod utils;

/// Process the arguments given by the user
fn process(args: &args::Args) -> result::Result<()> {
    // Load the pdf
    let mut pdf = pdf::Pdf::load(&args.input)?;

    // Guess its content and either save it or print it
    pdf.guess(args)?;
    match &args.output {
        Some(output) => latex::LaTeX::from(&pdf).save(output)?,
        None => println!("{}", pdf.get_content()),
    }

    // Do some debugging
    pdf.pages[0].debug_dist_avg();
    pdf.pages[0].debug_image().save("./test/debug.png")?;

    Ok(())
}

fn main() {
    if let Err(err) = process(&args::Args::parse()) {
        eprintln!("{err}");
    }
}
