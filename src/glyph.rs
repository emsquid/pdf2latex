use crate::font::{Code, FontBase, Size, Style};
use crate::result::Result;
use crate::utils::{find_parts, flood_fill, Rect};
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage};
use std::collections::HashMap;
use std::process::Command;

pub const DIST_UNALIGNED_THRESHOLD: f32 = 32.;
pub const DIST_THRESHOLD: f32 = 10.;
pub const CHAR_THRESHOLD: u8 = 75;

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
            f32::from(
                self.image()[(x.unsigned_abs() + y.unsigned_abs() * self.rect().width) as usize],
            ) / 255.
        }
    }

    fn distance(&self, other: &dyn Glyph, offset: i32, limit: f32) -> f32 {
        let mut dist = HashMap::new();
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
                            let v_o = other.get_pixel_signed(x as i32 + dx, y as i32 + dy);
                            *value += (v_g - v_o).powf(2.);
                        }
                    }
                }
            }
        }

        for x in 0..other.rect().width {
            for y in 0..other.rect().height {
                for (&(dx, dy), value) in &mut dist {
                    if *value < limit {
                        let v_g = self.get_pixel_signed(x as i32 - dx, y as i32 - dy);
                        if v_g == 1. {
                            let v_o = other
                                .get_pixel(x.wrapping_add_signed(dx), y.wrapping_add_signed(dy));
                            *value += (v_g - v_o).powf(2.);
                        }
                    }
                }
            }
        }

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

#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct KnownGlyph {
    pub base: String,
    pub code: Code,
    pub size: Size,
    pub styles: Vec<Style>,
    pub modifiers: Vec<String>,
    pub math: bool,

    pub rect: Rect,
    pub image: Vec<u8>,
    pub offset: i32,
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
    pub fn from(
        base: &str,
        code: Code,
        size: Size,
        styles: Vec<Style>,
        modifiers: Vec<String>,
        math: bool,
        id: u32,
    ) -> Result<KnownGlyph> {
        let (image, offset) = Self::render(
            base,
            code,
            size,
            styles.clone(),
            modifiers.clone(),
            math,
            id,
        )?;

        Ok(KnownGlyph {
            base: base.to_string(),
            code,
            size,
            styles,
            modifiers,
            math,
            rect: Rect::new(0, 0, image.width(), image.height()),
            image: image.to_luma8().into_raw(),
            offset,
        })
    }

    fn render(
        base: &str,
        code: Code,
        size: Size,
        styles: Vec<Style>,
        modifiers: Vec<String>,
        math: bool,
        id: u32,
    ) -> Result<(DynamicImage, i32)> {
        let latex = Self::latex(base, size, styles, modifiers, math);
        let doc = format!(
            "\\documentclass[11pt, border=4pt]{{standalone}}
            \\usepackage{{amsmath, amssymb, amsthm}}
            \\usepackage{{euscript}}
            \\begin{{document}}
            . {{\\fontfamily{{{code}}}\\selectfont
                {latex}
            }}
            \\end{{document}}"
        );
        std::fs::write(format!("temp/{id}.tex"), doc)?;

        Command::new("pdflatex")
            .args(["-output-directory=temp", &format!("temp/{id}.tex")])
            .output()?;

        let output = Command::new("pdftoppm")
            .args(["-r", "512", &format!("temp/{id}.pdf")])
            .output()?;

        std::fs::remove_file(format!("temp/{id}.tex"))?;
        std::fs::remove_file(format!("temp/{id}.aux"))?;
        std::fs::remove_file(format!("temp/{id}.log"))?;
        std::fs::remove_file(format!("temp/{id}.pdf"))?;

        let image = image::load_from_memory(&output.stdout)?;
        Ok(Self::find_glyph(&image))
    }

    fn latex(
        base: &str,
        size: Size,
        styles: Vec<Style>,
        modifiers: Vec<String>,
        math: bool,
    ) -> String {
        let mut result = modifiers.iter().fold(String::from(base), |acc, modif| {
            format!("\\{modif}{{{acc}}}")
        });
        result = if math { format!("${result}$") } else { result };
        result = styles.iter().fold(result, |acc, style| style.apply(acc));
        size.apply(result)
    }

    fn find_baseline(image: &DynamicImage) -> u32 {
        let image = image.crop_imm(0, 0, 45, image.height());

        find_parts(&image.to_luma8(), 0)
            .last()
            .unwrap_or(&(0, image.height()))
            .1
    }

    fn find_glyph(image: &DynamicImage) -> (DynamicImage, i32) {
        let baseline = Self::find_baseline(&image);
        let image = image.crop_imm(45, 0, image.width(), image.height());

        let vertical = find_parts(&image.to_luma8(), 0);
        let y = vertical.first().unwrap_or(&(0, 0)).0;
        let height = vertical.last().unwrap_or(&(0, image.height())).1 - y + 1;

        let horizontal = find_parts(&image.rotate90().to_luma8(), 0);
        let x = horizontal.first().unwrap_or(&(0, 0)).0;
        let width = horizontal.last().unwrap_or(&(0, image.width())).1 - x + 1;

        let offset = (y + height - 1 - baseline) as i32;

        (image.crop_imm(x, y, width, height), offset)
    }

    pub fn get_latex(
        &self,
        size: &mut Size,
        styles: &mut Vec<Style>,
        math: &mut bool,
        init: &mut bool,
    ) -> String {
        let mut text = String::from("");

        if !self.math && *math {
            *math = self.math;
            text.push('$');
        }

        if size != &self.size || *init {
            if !*init {
                for style in styles.iter().rev() {
                    if style.is_math() {
                        text.push_str("}$");
                    } else if style != &Style::Normal {
                        text.push_str("}");
                    }
                }
                if *size != Size::Normalsize {
                    text.push_str("}");
                }
            }
            styles.clear();
            *size = self.size;
            if *size != Size::Normalsize {
                text.push_str(&format!("\\{}{{", self.size));
            }
        }
        *init = false;

        let mut i = 0;
        while i < styles.len() {
            if !self.styles.contains(&styles[i]) {
                if styles[i].is_math() {
                    text.push_str("}$");
                } else if styles[i] != Style::Normal {
                    text.push_str("}");
                }
                styles.remove(i);
            } else {
                i += 1;
            }
        }

        for style in &self.styles {
            if !styles.contains(&style) {
                styles.push(*style);
                if styles[i].is_math() {
                    text.push_str(&format!("$\\{}{{", style));
                } else if styles[i] != Style::Normal {
                    text.push_str(&format!("\\{}{{", style));
                }
            }
        }

        if self.math && !*math {
            *math = self.math;
            text.push('$');
        }

        let base = self.modifiers.iter().fold(self.base.clone(), |acc, modif| {
            format!("\\{modif}{{{acc}}}")
        });
        text.push_str(&base);

        if self.base.starts_with("\\") {
            text.push(' ');
        }

        text
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

        let x = pixels.iter().map(|(x, _)| x).min().unwrap();
        let y = pixels.iter().map(|(_, y)| y).min().unwrap();
        let width = pixels.iter().map(|(px, _)| px - x + 1).max().unwrap();
        let height = pixels.iter().map(|(_, py)| py - y + 1).max().unwrap();

        let mut glyph_image = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
        for (px, py) in &pixels {
            let color = image.get_pixel(px + bounds.x, py + bounds.y).to_rgb();
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
        for nx in 0..self.rect.width {
            for ny in 0..self.rect.height {
                let v = self.image[(nx + ny * self.rect.width) as usize];
                glyph_image.put_pixel(
                    nx + (self.rect.x - x),
                    ny + (self.rect.y - y),
                    Rgb([v, v, v]),
                );
            }
        }

        for nx in 0..other.rect.width {
            for ny in 0..other.rect.height {
                let v = other.image[(nx + ny * other.rect.width) as usize];
                glyph_image.put_pixel(
                    nx + (other.rect.x - x),
                    ny + (other.rect.y - y),
                    Rgb([v, v, v]),
                );
            }
        }

        UnknownGlyph {
            rect: Rect::new(x, y, width, height),
            image: DynamicImage::ImageRgb8(glyph_image).to_luma8().into_raw(),
            dist: None,
            guess: None,
        }
    }

    pub fn try_guess(
        &mut self,
        fontbase: &FontBase,
        baseline: u32,
        aligned: bool,
        hint: Option<(Code, Size)>,
    ) {
        let (code, size) = hint.unzip();
        let mut closest = self.dist.unwrap_or(f32::MAX / 1.1) * 1.1;
        'outer: for (&key, family) in &fontbase.glyphs {
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

                            let offset = glyph.offset
                                - ((self.rect.y + self.rect.height) as i32 - baseline as i32);
                            let dist =
                                self.distance(glyph, if aligned { offset } else { 0 }, closest)
                                    + if aligned { 0 } else { offset.abs() } as f32;

                            if dist < closest {
                                closest = dist;
                                self.dist = Some(dist);
                                self.guess = Some(glyph.clone());
                            }

                            if dist < DIST_THRESHOLD {
                                continue 'outer;
                            }
                        }
                    }
                }
            }
        }

        if aligned && self.dist.unwrap_or(f32::INFINITY) > DIST_UNALIGNED_THRESHOLD {
            self.try_guess(fontbase, baseline, false, hint);
        }
    }
}
