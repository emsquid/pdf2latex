use crate::font::{Code, FontBase, Size, Style};
use crate::result::Result;
use crate::utils::{flood_fill, Rect};
use ab_glyph::{Font, FontVec, GlyphId};
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage};

pub const CHAR_THRESHOLD: u8 = 175;

#[derive(Clone)]
pub struct Known {
    pub chr: char,
    pub code: Code,
    pub size: Size,
    pub styles: Vec<Style>,

    pub rect: Rect,
    pub image: Vec<u8>,
}

impl Known {
    pub fn try_from(
        font: &FontVec,
        id: GlyphId,
        chr: char,
        code: Code,
        size: Size,
        styles: &[Style],
    ) -> Option<Known> {
        // TODO: improve scale
        let scale = font.pt_to_px_scale(size.as_pt() * 400.0 / 96.0).unwrap();
        let glyph = id.with_scale(scale);

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let point = bounds.max - bounds.min;
            let rect = Rect::new(0, 0, point.x as u32, point.y as u32);

            let mut image = RgbImage::from_pixel(rect.width, rect.height, Rgb([255, 255, 255]));
            outlined.draw(|x, y, v| {
                let c = (255.0 - v * 255.0) as u8;
                image.put_pixel(x, y, Rgb([c, c, c]));
            });

            Some(Known {
                chr,
                code,
                size,
                styles: styles.to_owned(),
                rect,
                image: DynamicImage::ImageRgb8(image).to_luma8().into_raw(),
            })
        } else {
            None
        }
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
pub struct Unknown {
    pub rect: Rect,
    pub image: Vec<u8>,

    pub guess: Option<Known>,
}

impl Unknown {
    fn find_rect(start: (u32, u32), bounds: Rect, image: &DynamicImage) -> Rect {
        let base_pixels = flood_fill(vec![start], &bounds.crop(image).to_luma8(), CHAR_THRESHOLD);

        let x = base_pixels.iter().map(|(x, _)| *x).min().unwrap();
        let width = base_pixels.iter().map(|(px, _)| px - x + 1).max().unwrap();

        Rect::new(bounds.x + x - 5, bounds.y, width + 10, bounds.height)
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

    pub fn from(start: (u32, u32), bounds: Rect, image: &DynamicImage) -> Unknown {
        let base = Unknown::find_rect(start, bounds, image);
        let pixels = Unknown::find_pixels(base, image);

        let x = pixels.iter().map(|(x, _)| *x).min().unwrap();
        let y = pixels.iter().map(|(_, y)| *y).min().unwrap();
        let width = pixels.iter().map(|(px, _)| px - x + 1).max().unwrap();
        let height = pixels.iter().map(|(_, py)| py - y + 1).max().unwrap();

        let mut glyph_image = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
        for (px, py) in &pixels {
            let color = image.get_pixel(*px, *py).to_rgb();
            glyph_image.put_pixel(px - x, py - y, color);
        }

        Unknown {
            rect: Rect::new(x, y, width, height),
            image: DynamicImage::ImageRgb8(glyph_image).to_luma8().into_raw(),
            guess: None,
        }
    }

    fn distance(&self, other: &Known) -> u32 {
        let width = u32::max(self.rect.width, other.rect.width);
        let height = u32::max(self.rect.height, other.rect.height);

        let mut dist = 0;
        for x in 0..width {
            for y in 0..height {
                if x < self.rect.width
                    && y < self.rect.height
                    && x < other.rect.width
                    && y < other.rect.height
                {
                    let v_g = u32::from(self.image[(x + y * self.rect.width) as usize]);
                    let v_o = u32::from(other.image[(x + y * other.rect.width) as usize]);
                    dist += (v_g - v_o).pow(2);
                } else if x < self.rect.width && y < self.rect.height {
                    let v_g = u32::from(self.image[(x + y * self.rect.width) as usize]);
                    dist += (255 - v_g).pow(2);
                } else if x < other.rect.width && y < other.rect.height {
                    let v_o = u32::from(other.image[(x + y * other.rect.width) as usize]);
                    dist += (255 - v_o).pow(2);
                }
            }
        }

        dist
    }

    pub fn guess(&mut self, fontbase: &FontBase, hint: Option<(Code, Size)>) {
        let (mut code, mut size) = (None, None);
        let mut closest = u32::MAX;

        if let (Some((h_code, h_size)), Some(known)) = (hint, &self.guess) {
            (code, size) = (Some(h_code), Some(h_size));
            closest = self.distance(&known) * 105 / 100;
        }

        for (key, family) in &fontbase.glyphs {
            if code.is_some() && key != &code.unwrap() {
                continue;
            }
            for dw in -2..=2 {
                for dh in -2..=2 {
                    let width = self.rect.width.saturating_add_signed(dw);
                    let height = self.rect.height.saturating_add_signed(dh);
                    if let Some(glyphs) = family.get(&(width, height)) {
                        for glyph in glyphs {
                            if size.is_some() && glyph.size != size.unwrap() {
                                continue;
                            }
                            let dist = self.distance(glyph);
                            if glyph.chr.is_ascii() && dist <= closest {
                                closest = dist;
                                self.guess = Some(glyph.clone());
                            }
                        }
                    }
                }
            }
        }
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
