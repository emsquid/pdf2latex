use crate::font::{FontFamily, Glyph};
use crate::utils::{find_parts, flood_fill, squared_distance};
use image::imageops::overlay;
use image::{DynamicImage, Rgb};

const WORD_SPACING: u32 = 7;
const CHAR_THRESHOLD: u8 = 175;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn crop(&self, image: &DynamicImage) -> DynamicImage {
        image.crop_imm(self.x, self.y, self.width, self.height)
    }
}

pub struct Char {
    pub rect: Rect,
}

impl Char {
    pub fn new(rect: Rect) -> Char {
        Char { rect }
    }

    pub fn find_rect(bounds: Rect, start: (u32, u32), image: &DynamicImage) -> Rect {
        let mut rect = bounds.clone();

        let pixels = flood_fill(start, rect.crop(image).to_luma8(), CHAR_THRESHOLD);

        let x = pixels.iter().map(|(x, _)| x).min().unwrap();
        rect.x += x;
        rect.width = pixels
            .iter()
            .map(|(px, _)| px.saturating_sub(*x) + 1)
            .max()
            .unwrap();

        let parts = find_parts(rect.crop(image).to_luma8(), 0);

        let y = parts[0].0;
        rect.y += y;
        rect.height = parts[parts.len() - 1].1 - y + 1;

        rect
    }

    pub fn get_glyph(&self, image: &DynamicImage) -> Vec<u8> {
        let mut bottom = image::RgbImage::from_pixel(64, 64, Rgb([255, 255, 255]));
        let top = self.rect.crop(image).to_rgb8();
        overlay(&mut bottom, &top, 0, 0);

        DynamicImage::ImageRgb8(bottom).to_luma8().into_raw()
    }

    pub fn guess(&self, image: &DynamicImage, family: &FontFamily) -> Glyph {
        let reference = self.get_glyph(image);

        let mut closest = (family.glyphs[0].clone(), std::f32::MAX);

        for glyph in family.glyphs.iter() {
            // TODO: temporary fix
            let dist = squared_distance(&reference, &glyph.image);
            if dist < closest.1 {
                closest = (glyph.clone(), dist);
            }
        }

        closest.0
    }
}

pub struct Word {
    pub rect: Rect,
    pub chars: Vec<Char>,
}

impl Word {
    pub fn new(rect: Rect, image: &DynamicImage) -> Word {
        Word {
            rect,
            chars: Word::find_chars(rect, image),
        }
    }

    fn find_chars(bounds: Rect, image: &DynamicImage) -> Vec<Char> {
        let gray = bounds.crop(image).to_luma8();

        let mut chars = Vec::new();
        let mut x = 0;

        'outer: while x < gray.width() {
            for y in 0..gray.height() {
                if gray[(x, y)].0[0] <= CHAR_THRESHOLD {
                    let rect = Char::find_rect(bounds, (x, y), image);
                    chars.push(Char::new(rect));
                    x = rect.x - bounds.x + rect.width;
                    continue 'outer;
                }
            }
            x += 1;
        }

        chars
    }
}

pub struct Line {
    pub rect: Rect,
    pub words: Vec<Word>,
}

impl Line {
    pub fn new(rect: Rect, image: &DynamicImage) -> Line {
        Line {
            rect,
            words: Line::find_words(rect, image),
        }
    }

    fn find_words(bounds: Rect, image: &DynamicImage) -> Vec<Word> {
        let gray = bounds.crop(image).rotate90().to_luma8();

        let words = find_parts(gray, WORD_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(bounds.x + start, bounds.y, end - start + 1, bounds.height);
                Word::new(rect, image)
            })
            .collect();

        words
    }
}
