use crate::{result::Result, utils::Rect};
use ab_glyph::{Font, FontVec, GlyphId};
use image::{DynamicImage, Rgb, RgbImage};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Code {
    Cmr,
    Lmr,
    Qag,
    Qcr,
    Qpl,
    Xits,
}

fn code_to_path(code: Code) -> &'static str {
    match code {
        Code::Lmr => "fonts/lmr",
        Code::Cmr => "fonts/cmr",
        Code::Qag => "fonts/qag",
        Code::Qcr => "fonts/qcr",
        Code::Qpl => "fonts/qpl",
        Code::Xits => "fonts/xits",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Tiny,
    Scriptsize,
    Footnotesize,
    Small,
    Normalsize,
    Large,
    LLarge,
    LLLarge,
    Huge,
    HHuge,
}

fn size_to_pt(size: Size) -> f32 {
    match size {
        Size::Tiny => 5.0,
        Size::Scriptsize => 7.0,
        Size::Footnotesize => 8.0,
        Size::Small => 9.0,
        Size::Normalsize => 10.0,
        Size::Large => 12.0,
        Size::LLarge => 14.4,
        Size::LLLarge => 17.28,
        Size::Huge => 20.74,
        Size::HHuge => 24.88,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Bold,
    Italic,
    Slanted,
    // Underlined,
}

fn path_to_styles(path: &str) -> Vec<Style> {
    let mut styles = Vec::new();

    if path.contains("bold") {
        styles.push(Style::Bold)
    }
    if path.contains("italic") {
        styles.push(Style::Italic)
    }
    if path.contains("slant") {
        styles.push(Style::Slanted)
    }

    styles
}

#[derive(Clone)]
pub struct FontGlyph {
    pub chr: char,
    pub code: Code,
    pub size: Size,
    pub styles: Vec<Style>,
    pub rect: Rect,
    pub image: Vec<u8>,
}

impl FontGlyph {
    pub fn from(
        font: &FontVec,
        id: GlyphId,
        chr: char,
        code: Code,
        size: Size,
        styles: Vec<Style>,
    ) -> Option<FontGlyph> {
        // TODO: improve scale
        let pt = size_to_pt(size);
        let scale = font.pt_to_px_scale(pt * 300.0 / 96.0 + 3.5).unwrap();
        let glyph = id.with_scale(scale);

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let point = bounds.max - bounds.min;
            let rect = Rect::new(0, 0, point.x as u32, point.y as u32);

            let mut image = RgbImage::from_pixel(rect.width, rect.height, Rgb([255, 255, 255]));
            outlined.draw(|x, y, v| {
                let c = (255.0 - v * 255.0) as u8;
                image.put_pixel(x, y, Rgb([c, c, c]))
            });

            Some(FontGlyph {
                chr,
                code,
                size,
                styles,
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

pub struct FontBase {
    pub glyphs: HashMap<Code, HashMap<(u32, u32), Vec<FontGlyph>>>,
}

impl FontBase {
    fn load_file(path: &str, code: Code) -> Result<HashMap<(u32, u32), Vec<FontGlyph>>> {
        let font = FontVec::try_from_vec(std::fs::read(path)?)?;
        let sizes = [
            Size::Tiny,
            Size::Scriptsize,
            Size::Footnotesize,
            Size::Small,
            Size::Normalsize,
            Size::Large,
            Size::LLarge,
            Size::LLLarge,
            Size::Huge,
            Size::HHuge,
        ];
        let styles = path_to_styles(path);

        let mut glyphs = HashMap::new();
        for size in sizes {
            for (id, chr) in font.codepoint_ids() {
                if let Some(glyph) = FontGlyph::from(&font, id, chr, code, size, styles.clone()) {
                    let key = (glyph.rect.width, glyph.rect.height);
                    glyphs.entry(key).or_insert(Vec::new()).push(glyph);
                }
            }
        }

        Ok(glyphs)
    }

    fn load_family(code: Code) -> Result<HashMap<(u32, u32), Vec<FontGlyph>>> {
        let files = std::fs::read_dir(code_to_path(code))?;

        let mut family = HashMap::new();
        for file in files {
            let path = file?.path();
            for (key, glyphs) in FontBase::load_file(&path.to_string_lossy(), code)? {
                family.entry(key).or_insert(Vec::new()).extend(glyphs);
            }
        }

        Ok(family)
    }

    pub fn new() -> Result<FontBase> {
        let codes = [
            Code::Cmr,
            Code::Lmr,
            Code::Qag,
            Code::Qcr,
            Code::Qpl,
            Code::Xits,
        ];

        let mut glyphs = HashMap::new();
        for code in codes {
            glyphs.insert(code, FontBase::load_family(code)?);
        }

        Ok(FontBase { glyphs })
    }
}
