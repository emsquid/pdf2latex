use crate::font::{FontFamily, FontGlyph};
use crate::result::Result;
use crate::utils::{find_parts, flood_fill, squared_distance};
use image::{DynamicImage, GenericImageView, Pixel, Rgb};

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

#[derive(Clone)]
pub struct UnknownGlyph {
    pub rect: Rect,
    pub image: Vec<u8>,
}

impl UnknownGlyph {
    fn find_rect(start: (u32, u32), bounds: Rect, image: &DynamicImage) -> Rect {
        let base_pixels = flood_fill(vec![start], &bounds.crop(image).to_luma8(), CHAR_THRESHOLD);

        let x = base_pixels.iter().map(|(x, _)| *x).min().unwrap();
        let width = base_pixels
            .iter()
            .map(|(px, _)| px.saturating_sub(x) + 1)
            .max()
            .unwrap();

        Rect::new(bounds.x + x - 2, bounds.y, width + 4, bounds.height)
    }

    fn find_pixels(base: Rect, image: &DynamicImage) -> Vec<(u32, u32)> {
        let gray = base.crop(image).to_luma8();

        let mut borders = Vec::new();
        for x in [0, base.width - 1] {
            for y in 0..base.height {
                if gray[(x, y)].0[0] < 255 {
                    borders.push((x, y))
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
        let width = pixels
            .iter()
            .map(|(px, _)| px.saturating_sub(x) + 1)
            .max()
            .unwrap();
        let height = pixels
            .iter()
            .map(|(_, py)| py.saturating_sub(y) + 1)
            .max()
            .unwrap();

        let mut glyph_image = image::RgbImage::from_pixel(64, 64, Rgb([255, 255, 255]));
        for (px, py) in pixels.iter() {
            if px - x < 64 && py - y < 64 {
                let color = image.get_pixel(*px, *py).to_rgb();
                glyph_image.put_pixel(px - x, py - y, color)
            }
        }

        UnknownGlyph {
            rect: Rect::new(x, y, width, height),
            image: DynamicImage::ImageRgb8(glyph_image).to_luma8().into_raw(),
        }
    }

    pub fn guess(&self, family: &FontFamily) -> FontGlyph {
        let mut closest = (family.glyphs[0].clone(), std::f32::MAX);
        for glyph in family.glyphs.iter() {
            let dist = squared_distance(&self.image, &glyph.image);
            if dist < closest.1 {
                closest = (glyph.clone(), dist);
            }
        }

        closest.0
    }

    pub fn save(&self, path: &str) -> Result<()> {
        image::save_buffer_with_format(
            path,
            &self.image,
            64,
            64,
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
}

#[derive(Clone)]
pub struct Line {
    pub rect: Rect,
    pub words: Vec<Word>,
}

impl Line {
    fn find_words(bounds: Rect, image: &DynamicImage) -> Vec<Word> {
        let words = find_parts(bounds.crop(image).rotate90().to_luma8(), WORD_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(bounds.x + start, bounds.y, end - start + 1, bounds.height);
                Word::new(rect, image)
            })
            .collect();

        words
    }

    pub fn new(rect: Rect, image: &DynamicImage) -> Line {
        Line {
            rect,
            words: Line::find_words(rect, image),
        }
    }
}
