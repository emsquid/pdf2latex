use clap::Parser;
use pdf2latex::utils::args::{Args, MainArgs};
mod latex;
mod pdf;
mod text;
use anyhow::Result;

/// Process the arguments given by the user
fn process(args: MainArgs) -> Result<()> {
    let main_args = Args::Main(&args);
    // Load the pdf
    let mut pdf = pdf::Pdf::load(&args.input)?;

    // Guess its content and either save it or print it
    pdf.guess(&main_args)?;
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
    if let Err(err) = process(MainArgs::parse()) {
        eprintln!("{err}");
    }
}
