use crate::args::Args;
use crate::font::{Code, FontBase, Size, Style};
use crate::result::Result;
use crate::utils::{flood_fill, Rect};
use ab_glyph::{Font, FontVec, GlyphId};
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage};
use std::collections::HashMap;

pub const DIST_UNALIGNED_THRESHOLD: f32 = 32.;
pub const DIST_THRESHOLD: f32 = 10.;
pub const CHAR_THRESHOLD: u8 = 75;
const ASCII_BONUS: f32 = 0.3;

pub trait Glyph {
    fn rect(&self) -> &Rect;
    fn image(&self) -> &Vec<u8>;

    fn get_pixel(&self, x: u32, y: u32) -> f32 {
        if x >= self.rect().width || y >= self.rect().height {
            1.
        } else {
            f32::from(self.image()[(x + y * self.rect().width) as usize]) / 255.
        }
    }
    fn get_pixel_signed(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.rect().width as i32 || y >= self.rect().height as i32 {
            1.
        } else {
            f32::from(self.image()[(x.unsigned_abs() + y.unsigned_abs() * self.rect().width) as usize]) / 255.
        }
    }

    fn distance(&self, other: &dyn Glyph, offset: i32, limit: f32) -> f32 {
        let mut dist: HashMap<(i32, i32), f32> = HashMap::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                dist.insert((dx, dy + offset), 0.);
            }
        }

        for x in 0..self.rect().width {
        for y in 0..self.rect().height {
            for (&(dx, dy), value) in &mut dist {
                if *value < limit {
                    let v_g = self.get_pixel(x, y);
                    if v_g != 1. {
                        let v_o =
                            other.get_pixel_signed(x as i32 + dx, y as i32 + dy);
                        *value += (v_g - v_o).powf(2.);
                    }
                }
            }
        }}
        for x in 0..other.rect().width {
        for y in 0..other.rect().height {
            for (&(dx, dy), value) in &mut dist {
                if *value < limit {
                    let v_g = self.get_pixel_signed(x as i32 - dx, y as i32 - dy);
                    if v_g == 1. {
                        let v_o =
                            other.get_pixel(x.wrapping_add_signed(dx), y.wrapping_add_signed(dy));
                        *value += (v_g - v_o).powf(2.);
                    }
                }
            }
        }}

        *dist.values().min_by(|a, b| a.total_cmp(b)).unwrap()
    }

    fn save(&self, path: &str) -> Result<()> {
        image::save_buffer_with_format(
            path,
            self.image(),
            self.rect().width,
            self.rect().height,
            image::ColorType::L8,
            image::ImageFormat::Png,
        )?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct KnownGlyph {
    pub chr: char,
    pub code: Code,
    pub size: Size,
    pub styles: Vec<Style>,

    pub rect: Rect,
    pub baseline_offset: i32,
    pub image: Vec<u8>,
}

impl Glyph for KnownGlyph {
    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn image(&self) -> &Vec<u8> {
        &self.image
    }
}

impl KnownGlyph {
    pub fn try_from(
        font: &FontVec,
        data: (GlyphId, char, Code, Size, &Vec<Style>),
        args: &Args,
    ) -> Option<KnownGlyph> {
        let (id, chr, code, size, styles) = data;
        let scale = font
            .pt_to_px_scale(size.as_pt(args.pt) * 512. / 96.)
            .unwrap();
        let glyph = id.with_scale(scale);

        if let Some(outlined) = font.outline_glyph(glyph) {
            // if chr.is_ascii() {
            //     println!("{}, {:?}", chr, outlined.px_bounds());
            // }
            let bounds = outlined.px_bounds();
            let point = bounds.max - bounds.min;
            let baseline_offset = bounds.min.y as i32;
            let rect = Rect::new(0, 0, point.x as u32, point.y as u32);

            let mut image = RgbImage::from_pixel(rect.width, rect.height, Rgb([255, 255, 255]));
            outlined.draw(|x, y, v| {
                let c = (255. - v * 255.) as u8;
                image.put_pixel(x, y, Rgb([c, c, c]));
            });

            Some(KnownGlyph {
                chr,
                code,
                size,
                styles: styles.clone(),
                rect,
                baseline_offset,
                image: DynamicImage::ImageRgb8(image).to_luma8().into_raw(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct UnknownGlyph {
    pub rect: Rect,
    pub image: Vec<u8>,

    pub dist: Option<f32>,
    pub guess: Option<KnownGlyph>,
}

impl Glyph for UnknownGlyph {
    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn image(&self) -> &Vec<u8> {
        &self.image
    }
}

impl UnknownGlyph {
    pub fn from(start: (u32, u32), bounds: Rect, image: &DynamicImage) -> UnknownGlyph {
        let pixels = flood_fill(vec![start], &bounds.crop(image).to_luma8(), CHAR_THRESHOLD);

        let x = pixels.iter().map(|(x, _)| *x).min().unwrap();
        let y = pixels.iter().map(|(_, y)| *y).min().unwrap();
        let width = pixels.iter().map(|(px, _)| px - x + 1).max().unwrap();
        let height = pixels.iter().map(|(_, py)| py - y + 1).max().unwrap();

        let mut glyph_image = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
        for (px, py) in &pixels {
            let color = image.get_pixel(*px + bounds.x, *py + bounds.y).to_rgb();
            glyph_image.put_pixel(px - x, py - y, color);
        }

        UnknownGlyph {
            rect: Rect::new(x + bounds.x, y + bounds.y, width, height),
            image: DynamicImage::ImageRgb8(glyph_image).to_luma8().into_raw(),
            dist: None,
            guess: None,
        }
    }

    pub fn join(&self, other: &UnknownGlyph) -> UnknownGlyph {
        let x = self.rect.x.min(other.rect.x);
        let y = self.rect.y.min(other.rect.y);
        let width = (self.rect.x + self.rect.width - x).max(other.rect.x + other.rect.width - x);
        let height = (self.rect.y + self.rect.height - y).max(other.rect.y + other.rect.height - y);

        let mut glyph_image = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
        for _x in 0..self.rect.width {
        for _y in 0..self.rect.height {
            let v = self.image[(_x + _y * self.rect.width) as usize];
            glyph_image.put_pixel(_x + (self.rect.x - x), _y + (self.rect.y - y), Rgb([v, v, v]));
        }}
        for _x in 0..other.rect.width {
        for _y in 0..other.rect.height {
            let v = other.image[(_x + _y * other.rect.width) as usize];
            glyph_image.put_pixel(_x + (other.rect.x - x), _y + (other.rect.y - y), Rgb([v, v, v]));
        }}

        UnknownGlyph {
            rect: Rect::new(x, y, width, height),
            image: DynamicImage::ImageRgb8(glyph_image).to_luma8().into_raw(),
            dist: None,
            guess: None,
        }
    }

    pub fn try_guess(
        &mut self,
        baseline: u32,
        fontbase: &FontBase,
        word_length: usize,
        hint: Option<(Code, Size)>,
        is_aligned: bool,
    ) {
        let bonus = |chr: char| {
            if chr.is_ascii() && word_length > 1 {
                1. - ASCII_BONUS
            } else {
                1.
            }
        };

        let (code, size) = hint.unzip();
        let mut closest = self.dist.unwrap_or(f32::MAX / 1.1) * 1.1;
        for (&key, family) in &fontbase.glyphs {
            if code.is_some() && Some(key) != code {
                continue;
            }

            for dw in [0, -1, 1, -2, 2] {
                for dh in [0, -1, 1, -2, 2] {
                    let width = self.rect.width.saturating_add_signed(dw);
                    let height = self.rect.height.saturating_add_signed(dh);
                    if let Some(glyphs) = family.get(&(width, height)) {
                        for glyph in glyphs {
                            if size.is_some() && Some(glyph.size) != size {
                                continue;
                            }

                            let offset = glyph.baseline_offset - ((self.rect.y) as i32 - baseline as i32);
                            let dist = self.distance(glyph, is_aligned.then_some(offset).unwrap_or(0), closest / (1. - ASCII_BONUS))
                                * bonus(glyph.chr);
                            if dist < closest {
                                closest = dist;
                                self.dist = Some(dist);
                                self.guess = Some(glyph.clone());
                            }

                            if dist < DIST_THRESHOLD {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}