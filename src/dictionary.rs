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
        let words = file.lines().map(String::from).collect();
        Ok(Dictionary { words })
    }

    fn get_punctuation(word: &str) -> (Vec<char>, Vec<String>) {
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

    fn asc2ification(guess: &str) -> String {
        let mut new = String::new();
        for chr in guess.chars() {
            match chr {
                'ﬁ' => new.push_str("fi"),
                'ﬂ' => new.push_str("fl"),
                'ﬀ' => new.push_str("ff"),
                'ﬄ' => new.push_str("ffl"),
                'ﬃ' => new.push_str("ffi"),
                _ => new.push(chr),
            }
        }
        new
    }

    fn correct_word(&self, guess: &str) -> String {
        let guess = Dictionary::asc2ification(guess);
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
            guess
        }
    }

    pub fn correct_string(&self, string: &str) -> String {
        let mut corrected: String = String::new();
        let (splitters, mut punct) = Dictionary::get_punctuation(string);

        let mut words: Vec<&str> = string.split(splitters.as_slice()).collect();
        words.retain(|part| !part.is_empty());

        let mut is_punct_turn: bool =
            PUNCTUATION.contains(&string.chars().next().unwrap().category());

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

    fn correct_line(&self, line: &String) -> String {
        let mut strings: Vec<String> = line.split(' ').map(String::from).collect();
        strings.retain(|part| !part.is_empty());

        for string in &strings {
            self.correct_string(&string);
        }

        strings.join(" ")
    }

    pub fn correct_text(&self, mut text: String) -> String {
        text = text
            .replace("‘‘", "\"")
            .replace("’’", "\"")
            .replace("·,", ";");
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        let mut cross_lines: Vec<bool> = Vec::new();
        for i in 0..lines.len() {
            if lines[i].ends_with('-') && lines.last().unwrap() != &lines[i] {
                while !lines[i + 1].starts_with(" ") {
                    let chr = lines[i + 1].remove(0);
                    lines[i].push(chr);
                }
                self.correct_line(&lines[i]);
                cross_lines.push(true);
            } else {
                self.correct_line(&lines[i]);
                cross_lines.push(false);
            }
        }

        for k in 0..lines.len() {
            if cross_lines[k] {
                while !lines[k].ends_with("-") {
                    let chr = lines[k].pop().unwrap();
                    lines[k+1].insert(0, chr);
                }
            }
        }

        lines.join("\n")
    }
}
