use crate::result::Result;

pub struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Result<Dictionary> {
        let file = std::fs::read_to_string("words.txt")?;
        let words = file.split('\n').map(String::from).collect();

        Ok(Dictionary { words })
    }

    pub fn correct(&self, guess: &str) -> String {
        self.words
            .iter()
            .min_by_key(|word| 1) // find the distance between word and guess
            .unwrap()
            .clone()
    }
}
