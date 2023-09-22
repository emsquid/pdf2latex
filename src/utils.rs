use anyhow::{anyhow, Result};
use image::{DynamicImage, GrayImage};
use std::cmp::Ordering;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{collections::HashMap, hash::Hash};

/// A Rectangle in 2D
#[derive(Clone, Copy, Debug, bitcode::Encode, bitcode::Decode)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// Create a new Rect with the given coordinates and dimensions
    #[must_use]
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Crop an image with the Rect
    #[must_use]
    pub fn crop(&self, image: &DynamicImage) -> DynamicImage {
        image.crop_imm(self.x, self.y, self.width, self.height)
    }
}

/// Split a slice based on a delimiter
fn split<'a, T: PartialEq>(buffer: &'a [T], delimiter: &'a T) -> Vec<&'a [T]> {
    buffer.split(|b| b == delimiter).collect()
}

/// Convert a slice of u8 to an usize
fn buffer_to_usize(buffer: &[u8]) -> Result<usize> {
    let result = String::from_utf8_lossy(buffer).parse()?;

    Ok(result)
}

/// Convert a slice of u8 to ppm images
/// (Need to be reworked)
fn buffer_to_ppm(buffer: &[u8]) -> Result<Vec<DynamicImage>> {
    let mut images = Vec::new();
    let mut start = 0;

    while start < buffer.len() {
        let infos = split(&buffer[start..start + 40], &b'\n');
        let (code, size, rgb) = (infos[0], split(infos[1], &b' '), infos[2]);
        let (width, height) = (buffer_to_usize(size[0])?, buffer_to_usize(size[1])?);
        let size = code.len() + size.len() + rgb.len() + 10 + width * height * 3;

        images.push(image::load_from_memory(&buffer[start..start + size])?);

        start += size;
    }

    Ok(images)
}

/// Convert a pdf to images
///
/// # Errors
/// Fails if the command pdftoppm is not executed correcly
pub fn pdf_to_images(path: &Path) -> Result<Vec<DynamicImage>> {
    let output = Command::new("pdftoppm")
        .args(["-r", "512", &path.to_string_lossy()])
        .output()?;
    match output.stderr.len() {
        0 => buffer_to_ppm(&output.stdout),
        _ => Err(anyhow!("Format error: This is not a PDF")),
    }
}

/// Find the different black parts in an image with the given spacing
#[must_use]
pub fn find_parts(gray: &GrayImage, spacing: u32) -> Vec<(u32, u32)> {
    let mut parts = Vec::new();

    let mut start = 0;
    let mut end = 0;
    let mut in_part = false;

    for (i, row) in gray.enumerate_rows() {
        // Compute the average grayscale of the row
        let average = row.fold(0, |acc, line| acc + u32::from(line.2 .0[0])) / gray.width();
        if in_part && average == 255 {
            // If we are in a part and the row is white
            // Set the end of the part if not set
            if end == 0 {
                end = i;
            }
            // Check if the end of the part is spaced enough
            if i - (end) >= spacing {
                parts.push((start, end - 1));
                in_part = false;
            }
        } else if average != 255 {
            // Else reset the end
            end = 0;
            // And start a new part if not started
            if !in_part {
                start = i;
                in_part = true;
            }
        }
    }

    // Add the last part if needed
    if in_part {
        parts.push((start, gray.height()));
    }

    parts
}

/// Compute a flood fill from start with the given threshold
#[must_use]
pub fn flood_fill(start: Vec<(u32, u32)>, gray: &GrayImage, threshold: u8) -> Vec<(u32, u32)> {
    let mut pixels = start;
    let mut index = 0;

    while index < pixels.len() {
        let (x, y) = pixels[index];
        // Extend the flood fill from this pixel if it passes the threshold
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

/// return a tuple (count, value) where count represent the number of times that value is present
/// in `array`
pub fn most_frequent<T: Hash + Eq + Copy>(array: &[T], default: T) -> (T, i32) {
    let mut hash_map = HashMap::new();
    for value in array {
        hash_map.entry(value).and_modify(|v| *v += 1).or_insert(0);
    }

    let (mut mode, mut max): (T, i32) = (default, 0);
    for (&value, count) in hash_map {
        if count > max {
            mode = value;
            max = count;
        }
    }

    (mode, max)
}

/// Round a value to a certain number of digits
#[must_use]
pub fn round(value: f32, digits: u32) -> f32 {      
    (value * (10.0_f32).powi(digits as i32)).round() / 10.0_f32.powi(digits as i32)
}

/// Print a logging message to stdout
///
/// # Errors
/// Fails if it was impossible de to print in stdout
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
