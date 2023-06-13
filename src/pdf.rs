use crate::result::Result;
use crate::text::{Line, Rect};
use crate::utils::{find_parts, get_rasterized_glyphs, pdf_to_images};
use image::imageops::overlay;
use image::{DynamicImage, Rgba};

const LINE_SPACING: u32 = 3;

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

    pub fn get_text(&self) -> Result<String> {
        let mut text = String::new();

        let font = "fonts/lmroman10-regular.otf";
        let glyphs = get_rasterized_glyphs(font)?;

        for line in self.lines.iter() {
            for word in line.words.iter() {
                for char in word.chars.iter() {
                    text.push(char.guess(&self.image, &glyphs));
                }
                text.push(' ');
            }
        }

        Ok(text)
    }
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &str) -> Result<Pdf> {
        let pages = pdf_to_images(path, 200)?
            .iter()
            .map(|image| Page::from(image))
            .collect();
        Ok(Pdf { pages })
    }
}
