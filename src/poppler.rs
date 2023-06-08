use image;
use std::process::Command;

fn split(buffer: &[u8], delimiter: u8) -> Vec<&[u8]> {
    buffer.split(|&b| b == delimiter).collect()
}

fn parse_to_usize(buffer: &[u8]) -> usize {
    String::from_utf8_lossy(buffer).parse::<usize>().unwrap()
}

fn parse_to_ppm(buffer: &[u8]) -> Vec<image::DynamicImage> {
    let mut images = Vec::new();
    let mut start = 0;

    while start < buffer.len() {
        let infos = split(&buffer[start..start + 40], b'\n');
        let (code, size, rgb) = (infos[0], split(infos[1], b' '), infos[2]);
        let (width, height) = (parse_to_usize(size[0]), parse_to_usize(size[1]));
        let end = code.len() + size.len() + rgb.len() + 10 + width * height * 3;

        images.push(image::load_from_memory(&buffer[start..start + end]).unwrap());

        start += end;
    }

    images
}

pub fn pdf_to_images(path: &str, resolution: i32) -> Vec<image::DynamicImage> {
    let output = Command::new("pdftoppm")
        .args(["-r", &resolution.to_string(), path])
        .output()
        .unwrap();
    parse_to_ppm(&output.stdout)
}
