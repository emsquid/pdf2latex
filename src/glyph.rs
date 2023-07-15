use crate::font::{Code, FontBase, Size, Style};
use crate::result::Result;
use crate::utils::{find_parts, flood_fill, Rect};
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage};
use std::collections::HashMap;
use std::process::Command;

pub const DIST_UNALIGNED_THRESHOLD: f32 = 32.;
pub const DIST_THRESHOLD: f32 = 8.;
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
            f32::from(self.image()[(x as u32 + y as u32 * self.rect().width) as usize]) / 255.
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
                        if (v_g - 1.).abs() > f32::EPSILON {
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
                        if (v_g - 1.).abs() < f32::EPSILON {
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

type GlyphData = (String, Size, Vec<Style>, Vec<String>, bool);

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
    pub fn from(data: GlyphData, code: Code, id: u32) -> Result<KnownGlyph> {
        let (image, offset) = Self::render(data.clone(), code, id)?;

        Ok(KnownGlyph {
            base: data.0,
            code,
            size: data.1,
            styles: data.2,
            modifiers: data.3,
            math: data.4,
            rect: Rect::new(0, 0, image.width(), image.height()),
            image: image.to_luma8().into_raw(),
            offset,
        })
    }

    fn render(data: GlyphData, code: Code, id: u32) -> Result<(DynamicImage, i32)> {
        let latex = Self::latex(data, &None, &None, true);
        let doc = format!(
            "\\documentclass[11pt, border=4pt]{{standalone}}
            \\usepackage{{amsmath, amssymb, amsthm}}
            \\usepackage{{euscript, mathrsfs}}
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
        data: GlyphData,
        prev: &Option<GlyphData>,
        next: &Option<GlyphData>,
        end: bool,
    ) -> String {
        let mut result = String::new();
        let default = (
            String::new(),
            Size::Normalsize,
            vec![Style::Normal],
            vec![],
            false,
        );
        let (base, size, styles, modifiers, math) = data;
        let (_p_base, p_size, p_styles, _p_modifiers, p_math) =
            prev.clone().unwrap_or(default.clone());
        let (_n_base, n_size, n_styles, _n_modifiers, n_math) =
            next.clone().unwrap_or(default.clone());

        if size != p_size || (math && !p_math) || styles != p_styles {
            if size != Size::Normalsize && !math {
                result.push_str(&format!("{{\\{size} "));
            }

            if math && !p_math {
                result.push('$');
            }

            for &style in &styles {
                if style != Style::Normal {
                    result.push_str(&format!("\\{style}{{"));
                }
            }
        }

        result.push_str(
            &modifiers
                .iter()
                .fold(base.clone(), |acc, modif| format!("\\{modif}{{{acc}}}")),
        );

        if base.starts_with('\\') && n_math && !end {
            result.push(' ');
        }

        if size != n_size || (math && !n_math) || styles != n_styles {
            for &style in &styles {
                if style != Style::Normal {
                    result.push('}');
                }
            }

            if math && !n_math {
                result.push('$');
            }

            if size != Size::Normalsize && !math {
                result.push('}');
            }
        }

        result
    }

    fn find_baseline(image: &DynamicImage) -> u32 {
        let image = image.crop_imm(0, 0, 45, image.height());

        find_parts(&image.to_luma8(), 0)
            .last()
            .unwrap_or(&(0, image.height()))
            .1
    }

    fn find_glyph(image: &DynamicImage) -> (DynamicImage, i32) {
        let baseline = Self::find_baseline(image);
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

    pub fn get_data(&self) -> GlyphData {
        (
            self.base.clone(),
            self.size,
            self.styles.clone(),
            self.modifiers.clone(),
            self.math,
        )
    }

    pub fn get_latex(
        &self,
        prev: &Option<KnownGlyph>,
        next: &Option<KnownGlyph>,
        end: bool,
    ) -> String {
        Self::latex(
            self.get_data(),
            &prev.clone().map(|glyph| glyph.get_data()),
            &next.clone().map(|glyph| glyph.get_data()),
            end,
        )
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

    pub fn try_guess(&mut self, fontbase: &FontBase, baseline: u32, aligned: bool) {
        let mut closest = self.dist.unwrap_or(f32::INFINITY) * 1.5;
        'outer: for family in fontbase.glyphs.values() {
            for dw in [0, -1, 1, -2, 2] {
                for dh in [0, -1, 1, -2, 2] {
                    let width = self.rect.width.saturating_add_signed(dw);
                    let height = self.rect.height.saturating_add_signed(dh);
                    if let Some(glyphs) = family.get(&(width, height)) {
                        for glyph in glyphs {
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
    }
}
