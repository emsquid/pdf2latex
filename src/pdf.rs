use std::sync::Arc;

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
                Line::new(rect, image)
            })
            .collect()
    }

    pub fn from(image: &DynamicImage) -> Page {
        Page {
            image: image.clone(),
            lines: Page::find_lines(image),
        }
    }

    pub fn debug(&self) -> DynamicImage {
        let mut copy = self.image.clone();
        let mut alt = true;
        for line in &self.lines {
            for word in &line.words {
                for glyph in &word.glyphs {
                    let color = if alt {
                        Rgba([0, 0, 255, 255])
                    } else {
                        Rgba([0, 255, 0, 255])
                    };
                    alt = !alt;
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

    pub fn guess(&self) -> Result<String> {
        let fontbase = Arc::new(FontBase::new()?);

        let mut text = String::new();
        for line in &self.lines {
            text.push_str(&line.guess(&fontbase));
            text.push('\n');
        }

        Ok(text)
    }

    pub fn guess_threaded(&self) -> Result<String> {
        let fontbase = Arc::new(FontBase::new()?);

        let mut handles = Vec::new();
        for line in self.lines.clone() {
            let fontbase = Arc::clone(&fontbase);
            let handle = std::thread::spawn(move || line.guess(&fontbase));
            handles.push(handle);
        }

        let mut content = String::new();
        for handle in handles {
            content.push_str(&handle.join().unwrap());
            content.push('\n');
        }

        Ok(content)
    }
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &str) -> Result<Pdf> {
        let pages = pdf_to_images(path, 300)?.iter().map(Page::from).collect();

        Ok(Pdf { pages })
    }
}
