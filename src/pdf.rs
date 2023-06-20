use crate::font::FontBase;
use crate::result::Result;
use crate::text::Line;
use crate::utils::{find_parts, pdf_to_images, Rect};
use image::imageops::overlay;
use image::{DynamicImage, Rgba};

const LINE_SPACING: u32 = 5;

pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    fn find_lines(image: &DynamicImage) -> Vec<Line> {
        find_parts(&image.to_luma8(), LINE_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(0, start, image.width(), end - start + 1);
                Line::from(rect, image)
            })
            .collect()
    }

    pub fn from(image: &DynamicImage) -> Page {
        Page {
            image: image.clone(),
            lines: Page::find_lines(image),
        }
    }

    pub fn guess(&mut self, fontbase: &FontBase) {
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for line in &mut self.lines {
                let handle = scope.spawn(move || line.guess(fontbase));
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    }

    pub fn get_content(&self) -> String {
        let mut content = String::new();
        for line in &self.lines {
            content.push_str(&line.get_content());
            content.push('\n');
        }

        content
    }

    pub fn debug(&self) -> DynamicImage {
        let mut copy = self.image.clone();
        let mut alt = true;
        for line in &self.lines {
            for word in &line.words {
                for glyph in &word.glyphs {
                    alt = !alt;
                    let color = if alt {
                        Rgba([0, 0, 255, 255])
                    } else {
                        Rgba([0, 255, 0, 255])
                    };
                    let sub = image::RgbaImage::from_pixel(glyph.rect.width, 2, color);

                    overlay(
                        &mut copy,
                        &sub,
                        i64::from(glyph.rect.x),
                        i64::from(line.rect.y + line.rect.height + 1),
                    );
                }
            }
        }

        copy
    }
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &str) -> Result<Pdf> {
        let pages = pdf_to_images(path)?.iter().map(Page::from).collect();

        Ok(Pdf { pages })
    }

    pub fn guess(&mut self) -> Result<()> {
        let fontbase = FontBase::new()?;
        for page in &mut self.pages {
            page.guess(&fontbase);
        }

        Ok(())
    }
}
