use image;
use std::process::Command;

fn split(buffer: &[u8], delimiter: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut start = 0;

    for i in 0..buffer.len() - delimiter.len() {
        if &buffer[i..i + delimiter.len()] == delimiter {
            result.push(buffer[start..i + delimiter.len()].to_vec());
            start = i + delimiter.len();
        }
    }

    if start < buffer.len() {
        result.push(buffer[start..].to_vec());
    }

    result
}

fn parse_to_jpeg(buffer: &[u8]) -> Vec<image::DynamicImage> {
    let mut images = Vec::new();

    for data in split(buffer, b"\xff\xd9") {
        images.push(image::load_from_memory(&data).unwrap())
    }

    images
}

pub fn pdf_to_images(path: &str) -> Vec<image::DynamicImage> {
    let output = Command::new("pdftoppm")
        .args(["-r", "200", "-jpeg", path])
        .output()
        .unwrap();
    parse_to_jpeg(&output.stdout)
}
