use crate::result::Result;
use edit_distance::edit_distance;

pub struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Result<Dictionary> {
        let file = std::fs::read_to_string("words.txt")?;
        let words = file.split('\n').map(String::from).collect();

        Ok(Dictionary { words })
    }

    pub fn correct(&self, guess: String) -> String {
        self.words
            .iter()
            .min_by_key(|word| edit_distance(&guess, &word))
            .unwrap()
            .clone()
    }
}
