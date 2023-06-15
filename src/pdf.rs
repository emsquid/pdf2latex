use crate::font::{FontCode, FontFamily};
use crate::result::Result;
use crate::text::{Line, Rect};
use crate::utils::{find_parts, pdf_to_images};
use futures::future::ready;
use futures::{stream, StreamExt};
use image::imageops::overlay;
use image::{DynamicImage, Rgba};

const LINE_SPACING: u32 = 5;

pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    fn find_lines(image: &DynamicImage) -> Vec<Line> {
        let lines = find_parts(image.to_luma8(), LINE_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(0, start, image.width(), end - start + 1);
                Line::new(rect, image)
            })
            .collect();

        lines
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
        for line in self.lines.iter() {
            for word in line.words.iter() {
                for glyph in word.glyphs.iter() {
                    let color = match alt {
                        true => Rgba([0, 0, 255, 255]),
                        false => Rgba([0, 255, 0, 255]),
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

    #[tokio::main]
    pub async fn guess_cpu_cool(&self) -> Result<String> {
        let family = FontFamily::from_code(FontCode::Lmr)?;

        let mut text = String::new();
        stream::iter(self.lines.iter())
            .map(|line| {
                let family = &family;
                async move {
                    let mut line_text = String::new();
                    for word in line.words.iter() {
                        for glyph in word.glyphs.iter() {
                            line_text.push(glyph.guess(family).chr);
                        }
                        line_text.push(' ');
                    }
                    line_text.push('\n');
                    line_text
                }
            })
            .buffered(8)
            .for_each(|line_text| {
                text.push_str(&line_text);
                ready(())
            })
            .await;

        Ok(text)
    }

    pub fn guess_cpu_hot(&self) -> Result<String> {
        let family = FontFamily::from_code(FontCode::Lmr)?;

        let mut text = String::new();
        let mut handles = Vec::new();
        for line in self.lines.clone() {
            let family = family.clone();
            let handle = std::thread::spawn(move || {
                let mut line_text = String::new();
                for word in line.words.iter() {
                    for glyph in word.glyphs.iter() {
                        line_text.push(glyph.guess(&family).chr);
                    }
                    line_text.push(' ');
                }
                line_text.push('\n');
                line_text
            });
            handles.push(handle);
        }

        for handle in handles {
            text.push_str(&handle.join().unwrap());
        }

        Ok(text)
    }
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &str) -> Result<Pdf> {
        let pages = pdf_to_images(path, 300)?
            .iter()
            .map(|image| Page::from(image))
            .collect();

        Ok(Pdf { pages })
    }
}
