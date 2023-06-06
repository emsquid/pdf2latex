use image::DynamicImage;
use pdfium_render::prelude::*;

fn pdf_to_images(path: &str, password: Option<&str>) -> Result<Vec<DynamicImage>, PdfiumError> {
    let lib = Pdfium::pdfium_platform_library_name_at_path("./lib/");
    let pdfium = Pdfium::new(Pdfium::bind_to_library(lib).unwrap());

    let document = pdfium.load_pdf_from_file(path, password)?;

    let mut images = Vec::new();
    for page in document.pages().iter() {
        images.push(page.render(1654, 2339, None)?.as_image())
    }

    Ok(images)
}

fn main() {
    let images = pdf_to_images("test/test_1_toLatex.pdf", None).expect("Error");
    images
        .get(0)
        .unwrap()
        .save("test/test_1_toLatex.png")
        .unwrap();
}
