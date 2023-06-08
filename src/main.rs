mod pdf;
mod poppler;
mod result;
mod text;

fn process(path: &str) -> result::Result<()> {
    let file = pdf::Pdf::load(path)?;
    println!("{}", file.pages.len());
    Ok(())
}

fn main() {
    if let Err(err) = process("test/test_1_toLatex.pdf") {
        eprintln!("{err}");
    }
}
