use anyhow::Result;
use image::{imageops::FilterType, DynamicImage};
use std::process::Command;

pub struct Model {}

impl Model {
    pub fn predict(image: &DynamicImage, image_id: Option<usize>) -> Result<String> {
        let image_name = format!("temp-{}.png", image_id.unwrap_or(0));
        image
            .resize(image.width() / 2, image.height() / 2, FilterType::Nearest)
            .save(&image_name)?;

        let mut cmd = Command::new("bash");
        cmd.args(["python/recognize_formula.sh", &image_name]);

        let output = &cmd.output()?.stdout;
        let binding = String::from_utf8_lossy(output);
        let result = binding.split(":").nth(1).unwrap().trim();

        std::fs::remove_file(image_name)?;
        Ok(result.to_string())
    }
}
