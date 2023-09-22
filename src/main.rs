use anyhow::Result;
use clap::Parser;
use pdf2latex::args::Args;
use pdf2latex::args::MainArgs;
use pdf2latex::latex::LaTeX;
use pdf2latex::pdf::Pdf;

/// Process the arguments given by the user
fn process(args: &MainArgs) -> Result<()> {
    let main_args = Args::Main(args);
    // Load the pdf
    let mut pdf = Pdf::load(&args.input)?;

    // Guess its content and either save it or print it
    pdf.guess(&main_args)?;
    match &args.output {
        Some(output) => LaTeX::from(&pdf).save(output)?,
        None => println!("{}", pdf.get_content()),
    }

    // Do some debugging
    pdf.pages[0].debug_dist_avg();
    pdf.pages[0].debug_image().save("./test/debug.png")?;

    Ok(())
}

fn main() {
    if let Err(err) = process(&MainArgs::parse()) {
        eprintln!("{err}");
    }
}
