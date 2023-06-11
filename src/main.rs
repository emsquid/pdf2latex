mod pdf;
mod result;
mod text;
mod utils;

fn process(path: &str) -> result::Result<()> {
    let file = pdf::Pdf::load(path)?;

    println!("{}", file.pages[0].get_text()?);

    Ok(())
}

fn main() {
    if let Err(err) = process("test/test_1_toLatex.pdf") {
        eprintln!("{err}");
    }
}
