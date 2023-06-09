use crate::result::{Error, Result};
use image::{DynamicImage, GrayImage};
use std::cmp::Ordering;
use std::io::Write;
use std::ops::AddAssign;
use std::path::Path;
use std::process::Command;
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, Debug, bitcode::Encode, bitcode::Decode)]
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

pub fn pdf_to_images(path: &Path) -> Result<Vec<DynamicImage>> {
    let output = Command::new("pdftoppm")
        .args(["-r", "512", &path.to_string_lossy()])
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
        let average = row.fold(0, |acc, line| acc + u32::from(line.2 .0[0])) / gray.width();
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

pub fn average<T: Eq + Hash>(list: Vec<T>, default: T) -> T {
    let mut count = HashMap::new();
    for key in list {
        count.entry(key).or_insert(0).add_assign(1);
    }

    count
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .unwrap_or((default, 0))
        .0
}

pub fn log(
    message: &str,
    progress: Option<f32>,
    duration: Option<f32>,
    action: &str,
) -> Result<()> {
    let mut stdout = std::io::stdout();
    stdout.write_all(format!("\x1b[{action}").as_bytes())?;

    let tab = "\t".repeat(3_usize.saturating_sub(message.len() / 8));
    match (progress, duration) {
        (Some(progress), Some(duration)) => {
            let progress = (progress * 20.) as u32;
            let bar = (0..20)
                .map(|i| match progress.cmp(&i) {
                    Ordering::Equal => '>',
                    Ordering::Greater => '=',
                    Ordering::Less => ' ',
                })
                .collect::<String>();
            stdout.write_all(format!("{message}{tab} [{bar}] in {duration}s").as_bytes())
        }
        (Some(progress), None) => {
            let percent = (progress * 100.) as u32;
            let progress = percent / 5;
            let bar = (0..20)
                .map(|i| match progress.cmp(&i) {
                    Ordering::Equal => '>',
                    Ordering::Greater => '=',
                    Ordering::Less => ' ',
                })
                .collect::<String>();
            stdout.write_all(format!("{message}{tab} [{bar}] {percent}%").as_bytes())
        }
        (None, Some(duration)) => stdout.write_all(format!("{message} in {duration}s").as_bytes()),
        (None, None) => stdout.write_all(message.as_bytes()),
    }?;
    stdout.write_all(b"\x1b[m")?;
    stdout.flush()?;

    Ok(())
}

pub fn round(value: f32, digits: u32) -> f32 {
    (value * (10.0_f32).powi(digits as i32)).round() / 10.0_f32.powi(digits as i32)
}
