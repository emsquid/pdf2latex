use crate::result::Result;
use strsim::jaro_winkler;
use ucd::{Codepoint, UnicodeCategory};

const PUNCTUATION: &[UnicodeCategory] = &[
    UnicodeCategory::ConnectorPunctuation,
    UnicodeCategory::DashPunctuation,
    UnicodeCategory::ClosePunctuation,
    UnicodeCategory::FinalPunctuation,
    UnicodeCategory::InitialPunctuation,
    UnicodeCategory::OtherPunctuation,
    UnicodeCategory::OpenPunctuation,
];

pub struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Result<Dictionary> {
        let file = std::fs::read_to_string("words.txt")?;
        let mut words = file
            .split(['\r', '\n'])
            .map(String::from)
            .collect::<Vec<String>>();
        words.retain(|word| !word.is_empty());

        Ok(Dictionary { words })
    }

    fn get_punctuation(&self, word: &str) -> (Vec<char>, Vec<String>) {
        let mut splitters: Vec<char> = Vec::new();
        let mut sequences: Vec<String> = Vec::new();
        let mut was_last_ponct: bool = false;
        for chr in word.chars() {
            if PUNCTUATION.contains(&chr.category()) {
                if was_last_ponct {
                    sequences.last_mut().unwrap().push(chr);
                } else {
                    sequences.push(chr.to_string());
                }
                was_last_ponct = true;
                splitters.push(chr);
            } else {
                was_last_ponct = false;
            }
        }

        (splitters, sequences)
    }

    fn correct_word(&self, guess: &str) -> String {
        let mut best_match: String = String::new();
        if guess.chars().all(|chr| chr.is_ascii_alphabetic()) {
            let mut best_dist: f64 = 0.;
            for word in &self.words {
                if (best_dist - 1.0).abs() > f64::EPSILON && word.len() == guess.len() {
                    let dist: f64 = jaro_winkler(&guess.to_ascii_lowercase(), word);
                    if dist > best_dist {
                        best_dist = dist;
                        best_match = word.clone();
                    }
                }
            }

            let iter = guess.chars().zip(best_match.chars());
            iter.map(|(original, new)| {
                if original.is_uppercase() {
                    new.to_ascii_uppercase()
                } else {
                    new
                }
            })
            .collect()
        } else {
            guess.to_string()
        }
    }

    pub fn correct_guess(&self, guess: &str) -> String {
        let mut corrected: String = String::new();
        let (splitters, mut punct) = self.get_punctuation(guess);

        let mut words: Vec<&str> = guess.split(splitters.as_slice()).collect();
        words.retain(|part| !part.is_empty());

        let mut is_punct_turn: bool =
            PUNCTUATION.contains(&guess.chars().next().unwrap().category());

        while !words.is_empty() || !punct.is_empty() {
            if words.is_empty() {
                corrected.push_str(&punct.remove(0));
            } else if punct.is_empty() {
                corrected.push_str(&self.correct_word(words.remove(0)));
            } else {
                if is_punct_turn {
                    corrected.push_str(&punct.remove(0));
                } else {
                    corrected.push_str(&self.correct_word(words.remove(0)));
                }
                is_punct_turn = !is_punct_turn;
            }
        }
        corrected
    }
}
