use anyhow::Result;
use image::{imageops::FilterType, DynamicImage};
use std::process::Command;

pub struct Model {}

impl Model {
    pub fn predict(image: &DynamicImage) -> Result<String> {
        image
            .resize(image.width() / 2, image.height() / 2, FilterType::Nearest)
            .save("temp.png")?;

        let mut cmd = Command::new("bash");
        cmd.args(["python/recognize_formula.sh", "temp.png"]);

        let output = &cmd.output()?.stdout;
        let binding = String::from_utf8_lossy(output);
        let result = binding.split(":").nth(1).unwrap().trim();

        std::fs::remove_file("temp.png")?;
        Ok(result.to_string())
    }
}
