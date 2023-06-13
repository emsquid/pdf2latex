use crate::font::{FontCode, FontFamily};
use crate::result::Result;
use crate::text::{Line, Rect};
use crate::utils::{find_parts, pdf_to_images};
use image::imageops::overlay;
use image::{DynamicImage, Rgba};

const LINE_SPACING: u32 = 5;

pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    pub fn from(image: &DynamicImage) -> Page {
        Page {
            image: image.clone(),
            lines: Page::find_lines(image),
        }
    }

    fn find_lines(image: &DynamicImage) -> Vec<Line> {
        let gray = image.to_luma8();

        let lines = find_parts(gray, LINE_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(0, start, image.width(), end - start + 1);
                Line::new(rect, image)
            })
            .collect();

        lines
    }

    pub fn guess_text(&self) -> Result<String> {
        let mut text = String::new();

        let family = FontFamily::from_code(FontCode::Lmr)?;

        for line in self.lines.iter() {
            for word in line.words.iter() {
                for char in word.chars.iter() {
                    text.push(char.guess(&self.image, &family).chr);
                }
                text.push(' ');
            }
            text.push('\n');
        }

        Ok(text)
    }

    pub fn debug(&self) -> DynamicImage {
        let mut copy = self.image.clone();

        let mut alt = true;

        for line in self.lines.iter() {
            for word in line.words.iter() {
                for chr in word.chars.iter() {
                    let color = match alt {
                        true => Rgba([255, 0, 0, 255]),
                        false => Rgba([0, 255, 0, 255]),
                    };
                    alt = !alt;
                    let sub = image::RgbaImage::from_pixel(chr.rect.width, 2, color);

                    overlay(
                        &mut copy,
                        &sub,
                        i64::from(chr.rect.x),
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
        let pages = pdf_to_images(path, 300)?
            .iter()
            .map(|image| Page::from(image))
            .collect();
        Ok(Pdf { pages })
    }
}
