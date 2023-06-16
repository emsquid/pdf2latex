use crate::{
    font::FontGlyph,
    result::{Error, Result},
    text::UnknownGlyph,
};
use image::{DynamicImage, GrayImage};
use std::process::Command;

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

fn split(buffer: &[u8], delimiter: u8) -> Vec<&[u8]> {
    buffer.split(|&b| b == delimiter).collect()
}

fn parse_to_usize(buffer: &[u8]) -> Result<usize> {
    let result = String::from_utf8_lossy(buffer).parse()?;
    Ok(result)
}

fn parse_to_ppm(buffer: &[u8]) -> Result<Vec<DynamicImage>> {
    let mut images = Vec::new();
    let mut start = 0;

    while start < buffer.len() {
        let infos = split(&buffer[start..start + 40], b'\n');
        let (code, size, rgb) = (infos[0], split(infos[1], b' '), infos[2]);
        let (width, height) = (parse_to_usize(size[0])?, parse_to_usize(size[1])?);
        let size = code.len() + size.len() + rgb.len() + 10 + width * height * 3;

        images.push(image::load_from_memory(&buffer[start..start + size])?);

        start += size;
    }

    Ok(images)
}

pub fn pdf_to_images(path: &str, resolution: u32) -> Result<Vec<DynamicImage>> {
    let output = Command::new("pdftoppm")
        .args(["-r", &resolution.to_string(), path])
        .output()?;
    match output.stderr.len() {
        0 => parse_to_ppm(&output.stdout),
        _ => Err(Error::Custom("Format error: This is not a PDF".to_string())),
    }
}

pub fn find_parts(gray: GrayImage, spacing: u32) -> Vec<(u32, u32)> {
    let mut parts = Vec::new();

    let mut start = 0;
    let mut end = 0;

    for (i, row) in gray.enumerate_rows() {
        let average = row.map(|l| u32::from(l.2 .0[0])).sum::<u32>() / gray.width();
        if start != 0 && average == 255 {
            if end == 0 {
                end = i;
            }
            if i - (end) >= spacing {
                parts.push((start, end - 1));
                start = 0;
            }
        } else if average != 255 {
            end = 0;
            if start == 0 {
                start = i;
            }
        }
    }

    if start != 0 {
        parts.push((start, gray.height()))
    }

    parts
}

pub fn flood_fill(start: Vec<(u32, u32)>, gray: &GrayImage, threshold: u8) -> Vec<(u32, u32)> {
    let mut pixels = start;
    let mut index = 0;

    while index < pixels.len() {
        let (x, y) = pixels[index];

        if gray[(x, y)].0[0] <= threshold {
            for dx in -1..2 {
                for dy in -1..2 {
                    let nx = x.saturating_add_signed(dx);
                    let ny = y.saturating_add_signed(dy);

                    if nx < gray.width()
                        && ny < gray.height()
                        && !pixels.contains(&(nx, ny))
                        && gray[(nx, ny)].0[0] < 255
                    {
                        pixels.push((nx, ny));
                    }
                }
            }
        }

        index += 1;
    }

    pixels
}

pub fn distance(glyph: &UnknownGlyph, other: &FontGlyph) -> u32 {
    let (g_rect, o_rect) = (glyph.rect, other.rect);
    let width = u32::max(g_rect.width, o_rect.width);
    let height = u32::max(g_rect.height, o_rect.height);

    let mut dist = 0;
    for x in 0..width {
        for y in 0..height {
            if x < g_rect.width && y < g_rect.height && x < o_rect.width && y < o_rect.height {
                let v_g = u32::from(glyph.image[(x + y * g_rect.width) as usize]);
                let v_o = u32::from(other.image[(x + y * o_rect.width) as usize]);
                dist += (v_g - v_o).pow(2);
            } else if x < g_rect.width && y < g_rect.height {
                let v_g = u32::from(glyph.image[(x + y * g_rect.width) as usize]);
                dist += (255 - v_g).pow(2);
            } else if x < o_rect.width && y < o_rect.height {
                let v_o = u32::from(other.image[(x + y * o_rect.width) as usize]);
                dist += (255 - v_o).pow(2);
            }
        }
    }

    dist
}
