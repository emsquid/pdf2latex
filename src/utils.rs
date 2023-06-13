use crate::result::{Error, Result};
use ab_glyph::{Font, FontVec, GlyphId};
use image::{DynamicImage, GrayImage, Rgb};
use std::process::Command;

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

pub fn flood_fill(start: (u32, u32), gray: GrayImage, threshold: u8) -> Vec<(u32, u32)> {
    let mut pixels = vec![start];
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

pub fn raster_glyph(font: &FontVec, id: GlyphId) -> Vec<u8> {
    let mut image = image::RgbImage::from_pixel(32, 32, Rgb([255, 255, 255]));
    let scale = font.pt_to_px_scale(10.0 * 300.0 / 96.0 + 3.0).unwrap();
    let glyph = id.with_scale(scale);

    if let Some(outlined) = font.outline_glyph(glyph) {
        outlined.draw(|x, y, v| {
            if x < 32 && y < 32 {
                let c = (255.0 - v * 255.0) as u8;
                image.put_pixel(x, y, Rgb([c, c, c]))
            }
        })
    }

    DynamicImage::ImageRgb8(image).to_luma8().into_raw()
}

pub fn save_glyph(path: &str, glyph: Vec<u8>) -> Result<()> {
    image::save_buffer_with_format(
        path,
        &glyph,
        32,
        32,
        image::ColorType::L8,
        image::ImageFormat::Png,
    )?;

    Ok(())
}

pub fn get_rasterized_glyphs(fontpath: &str) -> Result<Vec<(char, Vec<u8>)>> {
    let font = FontVec::try_from_vec(std::fs::read(fontpath)?)?;

    save_glyph("test/a2.png", raster_glyph(&font, font.glyph_id('a')))?;

    let glyphs = font
        .codepoint_ids()
        .map(|(id, chr)| (chr, raster_glyph(&font, id)))
        .collect();

    Ok(glyphs)
}

pub fn squared_distance(reference: &[u8], other: &[u8]) -> f32 {
    let mut dist = 0.0;
    for i in 0..reference.len() {
        let (x_1, x_2) = (i16::from(other[i]), i16::from(reference[i]));
        dist += (f32::from(x_1 - x_2) / 255.0).powf(2.0);
    }
    dist
}
