mod pdf;
mod result;
mod text;
mod utils;

fn process(path: &str) -> result::Result<()> {
    let file = pdf::Pdf::load(path)?;

    file.pages[0].debug().save("test/test.png")?;

    Ok(())
}

fn main() {
    if let Err(err) = process("test/test_1_toLatex.pdf") {
        eprintln!("{err}");
    }
}
