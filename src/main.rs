mod poppler;

fn main() {
    let images = poppler::pdf_to_images("test/test_2_toLatex.pdf");
    println!("{}", images.len())
}
