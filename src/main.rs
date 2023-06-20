mod dictionary;
mod font;
mod glyph;
mod latex;
mod pdf;
mod result;
mod text;
mod utils;

fn process(path: &str) -> result::Result<()> {
    let mut file = pdf::Pdf::load(path)?;
    file.guess()?;

    // file.pages[0].lines[1].words[0].glyphs[0].save("test/debug1.png")?;
    // file.pages[0].debug().save("test/debug.png")?;
    println!("{}", file.get_content()?);

    Ok(())
}

fn main() {
    if let Err(err) = process("test/test_1_toLatex.pdf") {
        eprintln!("{err}");
    }
}
