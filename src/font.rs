use crate::result::Result;
use ab_glyph::{Font, FontVec, GlyphId};
use image::{DynamicImage, Rgb};

#[derive(Clone, Copy)]
pub enum FontCode {
    Lmr,
}

fn code_to_path(code: FontCode) -> &'static str {
    match code {
        FontCode::Lmr => "fonts/lmr",
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum Style {
    Bold,
    Italic,
    Slanted,
    Underlined,
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
pub struct Glyph {
    pub chr: char,
    pub size: Size,
    pub styles: Vec<Style>,
    pub image: Vec<u8>,
}

impl Glyph {
    pub fn from(font: &FontVec, id: GlyphId, chr: char, size: Size, styles: Vec<Style>) -> Glyph {
        let mut image = image::RgbImage::from_pixel(32, 32, Rgb([255, 255, 255]));

        let pt = size_to_pt(size);
        let scale = font.pt_to_px_scale(pt * 300.0 / 96.0 + 3.0).unwrap();
        let glyph = id.with_scale(scale);

        if let Some(outlined) = font.outline_glyph(glyph) {
            outlined.draw(|x, y, v| {
                if x < 32 && y < 32 {
                    let c = (255.0 - v * 255.0) as u8;
                    image.put_pixel(x, y, Rgb([c, c, c]))
                }
            })
        }

        Glyph {
            chr,
            size,
            styles,
            image: DynamicImage::ImageRgb8(image).to_luma8().into_raw(),
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        image::save_buffer_with_format(
            path,
            &self.image,
            32,
            32,
            image::ColorType::L8,
            image::ImageFormat::Png,
        )?;

        Ok(())
    }
}

pub struct FontFamily {
    pub code: FontCode,
    pub glyphs: Vec<Glyph>,
}

impl FontFamily {
    fn load_font(path: &str) -> Result<Vec<Glyph>> {
        let mut glyphs = Vec::new();

        let font = FontVec::try_from_vec(std::fs::read(path)?)?;
        let sizes = [Size::Small, Size::Normalsize, Size::Large, Size::Huge];
        let styles = path_to_styles(path);

        for size in sizes {
            glyphs.extend(
                font.codepoint_ids()
                    .map(|(id, chr)| Glyph::from(&font, id, chr, size.clone(), styles.clone())),
            );
        }

        Ok(glyphs)
    }

    pub fn from_code(code: FontCode) -> Result<FontFamily> {
        let mut glyphs = Vec::new();

        let fonts = std::fs::read_dir(code_to_path(code))?;

        for font in fonts {
            let path = font?.path();
            glyphs.extend(FontFamily::load_font(path.to_str().unwrap())?);
        }

        Ok(FontFamily { code, glyphs })
    }
}
