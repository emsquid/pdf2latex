use crate::font::{FontBase, Glyph};
use crate::result::Result;
use crate::utils::{distance, find_parts, flood_fill, Rect};
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage};

const WORD_SPACING: u32 = 10;
const CHAR_THRESHOLD: u8 = 175;

#[derive(Clone)]
pub struct UnknownGlyph {
    pub rect: Rect,
    pub image: Vec<u8>,
}

impl UnknownGlyph {
    fn find_rect(start: (u32, u32), bounds: Rect, image: &DynamicImage) -> Rect {
        let base_pixels = flood_fill(vec![start], &bounds.crop(image).to_luma8(), CHAR_THRESHOLD);

        let x = base_pixels.iter().map(|(x, _)| *x).min().unwrap();
        let width = base_pixels.iter().map(|(px, _)| px - x + 1).max().unwrap();

        Rect::new(bounds.x + x - 3, bounds.y, width + 6, bounds.height)
    }

    fn find_pixels(base: Rect, image: &DynamicImage) -> Vec<(u32, u32)> {
        let gray = base.crop(image).to_luma8();

        let mut borders = Vec::new();
        for x in [0, base.width - 1] {
            for y in 0..base.height {
                if gray[(x, y)].0[0] < 255 {
                    borders.push((x, y));
                }
            }
        }

        let unwanted_pixels = flood_fill(borders, &gray, CHAR_THRESHOLD);

        let mut pixels = Vec::new();
        for x in 0..base.width {
            for y in 0..base.height {
                if gray[(x, y)].0[0] < 255 && !unwanted_pixels.contains(&(x, y)) {
                    pixels.push((base.x + x, base.y + y));
                }
            }
        }

        pixels
    }

    pub fn from(start: (u32, u32), bounds: Rect, image: &DynamicImage) -> UnknownGlyph {
        let base = UnknownGlyph::find_rect(start, bounds, image);
        let pixels = UnknownGlyph::find_pixels(base, image);

        let x = pixels.iter().map(|(x, _)| *x).min().unwrap();
        let y = pixels.iter().map(|(_, y)| *y).min().unwrap();
        let width = pixels.iter().map(|(px, _)| px - x + 1).max().unwrap();
        let height = pixels.iter().map(|(_, py)| py - y + 1).max().unwrap();

        let mut glyph_image = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
        for (px, py) in &pixels {
            let color = image.get_pixel(*px, *py).to_rgb();
            glyph_image.put_pixel(px - x, py - y, color);
        }

        UnknownGlyph {
            rect: Rect::new(x, y, width, height),
            image: DynamicImage::ImageRgb8(glyph_image).to_luma8().into_raw(),
        }
    }

    pub fn guess(&self, base: &FontBase) -> Option<Glyph> {
        let mut closest = (None, u32::MAX);
        for family in base.glyphs.values() {
            for dw in -2..=2 {
                for dh in -2..=2 {
                    let width = self.rect.width.saturating_add_signed(dw);
                    let height = self.rect.height.saturating_add_signed(dh);
                    if let Some(glyphs) = family.get(&(width, height)) {
                        for glyph in glyphs {
                            let dist = distance(self, glyph);
                            if dist < closest.1 {
                                closest = (Some(glyph.clone()), dist);
                            }
                        }
                    }
                }
            }
        }

        closest.0
    }

    pub fn save(&self, path: &str) -> Result<()> {
        image::save_buffer_with_format(
            path,
            &self.image,
            self.rect.width,
            self.rect.height,
            image::ColorType::L8,
            image::ImageFormat::Png,
        )?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Word {
    pub rect: Rect,
    pub glyphs: Vec<UnknownGlyph>,
}

impl Word {
    fn find_glyphs(bounds: Rect, image: &DynamicImage) -> Vec<UnknownGlyph> {
        let gray = bounds.crop(image).to_luma8();

        let mut glyphs = Vec::new();
        let mut x = 0;
        'outer: while x < gray.width() {
            for y in 0..gray.height() {
                if gray[(x, y)].0[0] <= CHAR_THRESHOLD {
                    let glyph = UnknownGlyph::from((x, y), bounds, image);
                    x = glyph.rect.x - bounds.x + glyph.rect.width;
                    glyphs.push(glyph);
                    continue 'outer;
                }
            }
            x += 1;
        }

        glyphs
    }

    pub fn new(rect: Rect, image: &DynamicImage) -> Word {
        Word {
            rect,
            glyphs: Word::find_glyphs(rect, image),
        }
    }

    pub fn guess(&self, base: &FontBase) -> String {
        let mut content = String::new();
        for glyph in &self.glyphs {
            if let Some(glyph) = glyph.guess(base) {
                content.push(glyph.chr);
            } else {
                content.push(' ');
            }
        }

        content
    }
}

#[derive(Clone)]
pub struct Line {
    pub rect: Rect,
    pub words: Vec<Word>,
}

impl Line {
    fn find_words(bounds: Rect, image: &DynamicImage) -> Vec<Word> {
        find_parts(&bounds.crop(image).rotate90().to_luma8(), WORD_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(bounds.x + start, bounds.y, end - start + 1, bounds.height);
                Word::new(rect, image)
            })
            .collect()
    }

    pub fn new(rect: Rect, image: &DynamicImage) -> Line {
        Line {
            rect,
            words: Line::find_words(rect, image),
        }
    }

    pub fn guess(&self, base: &FontBase) -> String {
        let mut content = String::new();
        for word in &self.words {
            content.push_str(&word.guess(base));
            content.push(' ');
        }

        content
    }
}
