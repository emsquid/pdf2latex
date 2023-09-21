use clap::Parser;
use utils::{
    args::{self, Args},
    result,
};

// mod args;
mod latex;
mod pdf;
mod text;

/// Process the arguments given by the user
fn process(args: &args::MainArgs) -> result::Result<()> {
    let main_args = Args::MainArgs(args);
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
    if let Err(err) = process(&args::MainArgs::parse()) {
        eprintln!("{err}");
    }
}
