use crate::result::{Error, Result};
use image::{DynamicImage, GrayImage};
use std::{collections::HashMap, hash::Hash, ops::AddAssign, process::Command};

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
    
    pub fn join(&mut self, rect: Rect) {
        self.width = self.x.min(rect.x).abs_diff((rect.x + rect.width).max(self.x + self.width));
        self.height = self.y.min(rect.y).abs_diff((rect.y + rect.height).max(self.y + self.height));
        self.x = self.x.min(rect.x);
        self.y = self.y.min(rect.y);
    }

    pub fn crop(&self, image: &DynamicImage) -> DynamicImage {
        image.crop_imm(self.x, self.y, self.width, self.height)
    }
}

fn split(buffer: &[u8], delimiter: u8) -> Vec<&[u8]> {
    buffer.split(|&b| b == delimiter).collect()
}

fn buffer_to_usize(buffer: &[u8]) -> Result<usize> {
    let result = String::from_utf8_lossy(buffer).parse()?;
    Ok(result)
}

fn buffer_to_ppm(buffer: &[u8]) -> Result<Vec<DynamicImage>> {
    let mut images = Vec::new();
    let mut start = 0;

    while start < buffer.len() {
        let infos = split(&buffer[start..start + 40], b'\n');
        let (code, size, rgb) = (infos[0], split(infos[1], b' '), infos[2]);
        let (width, height) = (buffer_to_usize(size[0])?, buffer_to_usize(size[1])?);
        let size = code.len() + size.len() + rgb.len() + 10 + width * height * 3;

        images.push(image::load_from_memory(&buffer[start..start + size])?);

        start += size;
    }

    Ok(images)
}

pub fn pdf_to_images(path: &str) -> Result<Vec<DynamicImage>> {
    let output = Command::new("pdftoppm")
        .args(["-r", "512", path])
        .output()?;
    match output.stderr.len() {
        0 => buffer_to_ppm(&output.stdout),
        _ => Err(Error::Custom("Format error: This is not a PDF")),
    }
}

pub fn find_parts(gray: &GrayImage, spacing: u32) -> Vec<(u32, u32)> {
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
        parts.push((start, gray.height()));
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

pub fn average<T: Eq + Hash>(list: Vec<T>) -> T {
    let mut count = HashMap::new();
    for key in list {
        count.entry(key).or_insert(0).add_assign(1);
    }

    count.into_iter().max_by_key(|&(_, c)| c).unwrap().0
}
