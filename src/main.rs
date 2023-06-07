mod pdf;
mod poppler;
mod text;

fn main() {
    let test = pdf::Pdf::load("test/test_2_toLatex.pdf");
    let rect = test.pages[0].lines[2].words[1].glyphs[3].rect;
    rect.crop(test.pages[0].image.clone())
        .save("test/1.png")
        .unwrap();
}
