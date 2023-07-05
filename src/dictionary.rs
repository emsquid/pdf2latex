use std::thread::sleep_ms;

use crate::result::Result;
use strsim::jaro_winkler;
use unicode_segmentation::UnicodeSegmentation;
use ucd::{UnicodeCategory, Codepoint};

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

    fn asc2ification(guess: &str) -> String {
        let mut new = String::new();
        for chr in guess.graphemes(true) {
            match chr {
                "ﬁ" => new.push_str("fi"),
                "ﬂ" => new.push_str("fl"),
                "ﬀ" => new.push_str("ff"),
                "ﬄ" => new.push_str("ffl"),
                "ﬃ" => new.push_str("ffi"),
                _ => new.push_str(chr),
            }
        }
        new
    }    
    
    fn in_dict(&self, word: &str) -> bool {
        for w in &self.words {
            if *w == word.to_string().to_lowercase() {return true};
        }
        return false
    }
    
    fn jaro_space(&self, string:&str) -> String {
        /*
        let length = string.len();
        for i in (1..length).rev() {
            let (s,e) = string.split_at(i);
            if self.in_dict(e){
                if self.in_dict(s){
                    let v = [s,e];
                    return v.join(" ")
                }
                else {
                    let mut s = self.jaro_space(s);
                    if !s.is_empty(){
                        s.push(' ');
                        s.push_str(e);
                        return s
                    }
                }
            }
        }
        
        String::new()
        */
        if self.in_dict(string) || !string.is_ascii() {
            return String::from(string)
        }
        let iter: Vec<(usize, &str)> = string.grapheme_indices(true).collect();
        for (i, _) in iter.iter().rev() {
            let (s, e) = string.split_at(*i);
            if self.in_dict(s){
                if self.in_dict(e){
                    return [s,e].join(" ")
                }
                else {
                    let ne = self.jaro_space(e);
                    if ne != e{
                        return [s, &ne].join(" ")
                    }
                }
            }
        }

        string.to_string()
    }
    /*
    
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
        }*/
    
    fn correct_word(&self, guess: &str) -> String {
        //if guess.chars().all(|chr| chr.is_ascii_alphabetic()) {
            let mut best_match: String = String::new();
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
        //} else {
        //    String::from(guess)
        //}
    }

    pub fn correct_string(&self, string: &str) -> String {
        /*
        let mut corrected: String = String::new();
        let (splitters, mut punct) = Dictionary::get_punctuation(&string);

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
        corrected*/
        let words: Vec<&str> = string.split_word_bounds().collect();
        for word in &words {
            self.correct_word(word);
        }
        words.concat()

    }

    fn correct_line(&self, line_: &String) -> String {
        //let mut strings : Vec<String> = Dictionary::asc2ification(&line).split(' ').map(|part| self.jaro_space(part)).collect::<Vec<String>>().join(" ").split(" ").map(String::from).collect();
        let mut words: Vec<String> = Vec::new();
        let mut spacing: Vec<String> = Vec::new();
        spacing.push("".to_string());

        let mut line = Dictionary::asc2ification(&line_);
        let mut in_word: bool = false;
        for char in line.graphemes(true) {
            if char.len() != 1 {
                if in_word {
                    words.last_mut().unwrap().push_str(char);
                }
                else {
                    words.push(String::from(char));
                    in_word = true;
                }
            }
            else if char == " " || PUNCTUATION.contains(&char.chars().next().unwrap().category()) {
                if !in_word {
                    spacing.last_mut().unwrap().push_str(char);
                }
                else {
                    spacing.push(String::from(char));
                    in_word = false;
                }
            }
            else {
                if in_word {
                    words.last_mut().unwrap().push_str(char);
                }
                else {
                    words.push(String::from(char));
                    in_word = true;
                }
            }
        }
        if in_word {
            spacing.push("".to_string());
        }


        let mut offset: usize = 0;
        for (i, word) in words.clone().into_iter().enumerate() {
            let new_words = self.jaro_space(&word);
            if new_words != word {
                println!("{word} -> ");
                std::thread::sleep(std::time::Duration::from_millis(50));
                let index = i + offset;
                words.remove(index);
                for new_word in new_words.rsplit(" ") {
                    words.insert(index, new_word.to_string());
                    spacing.insert(index +1, " ".to_string());
                    offset += 1;
                }
                offset -= 1;
                spacing.remove(index +1);
                println!("\t{new_words}");
            }
        }

        line = String::from("");
        for i in 0..words.len() {
            //self.correct_string(&words[i]);
            line.push_str(&spacing[i]);
            line.push_str(&words[i]);
            // println!("{}",words[i]);
        }
        line.push_str(&spacing.last().unwrap());

        //strings.retain(|part| !part.is_empty());
        // println!("--------------");
        //strings.join(" ")
        line
    }

    pub fn correct_text(&self, mut text: String) -> String {
        println!("START DEBUG");
        text = text
            .replace("‘‘", "\"")
            .replace("’’", "\"")
            .replace("·,", ";");
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        let mut cross_lines: Vec<usize> = Vec::new();
        for i in 0..lines.len() {
            cross_lines.push(0);
            if lines[i].ends_with('-') && i != lines.len() - 1 {
                lines[i].pop();
                while !lines[i + 1].starts_with(" ") {
                    let chr = lines[i + 1].remove(0);
                    lines[i].push(chr);
                    cross_lines[i] += 1;
                }
            }
            println!("LIGNE {} :", i);
            self.correct_line(&lines[i]);
            println!("FIN {i}");
        }

        for k in 0..lines.len() {
            if cross_lines[k] > 0 {
                while !cross_lines[k] != 0 {
                    let chr = lines[k].pop().unwrap();
                    lines[k+1].insert(0, chr);
                    cross_lines[k] -= 1
                }
                lines[k].push('-');
            }
        }
        println!("END DEBUG");

        lines.join("\n")
    }
}
