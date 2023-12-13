use anyhow::{anyhow, Result};
use image::{imageops::FilterType, DynamicImage};
use std::{path::Path, process::Command};

const PYTHON_FILE_NAME: &str = "python/recognize_formula.sh";

pub struct Model {}

impl Model {
    pub fn predict(image: &DynamicImage, image_id: Option<usize>) -> Result<String, anyhow::Error> {
        if !Path::new(PYTHON_FILE_NAME).exists() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
        }
        let image_name = format!("temp-{}.png", image_id.unwrap_or(0));
        image
            .resize(image.width() / 2, image.height() / 2, FilterType::Nearest)
            .save(&image_name)?;

        let mut cmd = Command::new("bash");
        cmd.args([PYTHON_FILE_NAME, &image_name]);

        let output = &cmd.output()?.stdout;
        let binding = String::from_utf8_lossy(output);
        let result = match binding.split(":").nth(1) {
            Some(e) => e,
            None => return Err(anyhow!("The IA did shit !")),
        }
        .trim();

        // std::fs::remove_file(image_name)?;
        Ok(result.to_string())
    }
}
