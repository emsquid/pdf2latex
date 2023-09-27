use crate::pdf::Word;
use anyhow::Result;
use std::process::Command;

pub struct Model {}

impl Model {
    pub fn predict(word: &Word) -> Result<String> {
        word.save("temp.png")?;
        let output = Command::new("pix2tex").arg("temp.png").output()?.stdout;
        let binding = String::from_utf8(output)?;
        let result = binding.split(":").nth(1).unwrap().trim();
        std::fs::remove_file("temp.png")?;
        Ok(result.to_string())
    }
}
