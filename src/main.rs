mod font;
mod pdf;
mod result;
mod text;
mod utils;

fn process(path: &str) -> result::Result<()> {
    let file = pdf::Pdf::load(path)?;

    // file.pages[0].lines[0].words[0].chars[0].save("test/debug.png")?;
    // file.pages[0].debug().save("test/debug.png")?;
    println!("{}", file.pages[0].guess_cpu_cool()?);

    Ok(())
}

fn main() {
    if let Err(err) = process("test/test_3_toLatex.pdf") {
        eprintln!("{err}");
    }
}
