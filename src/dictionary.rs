use strsim::jaro_winkler;
use ucd::{Codepoint, UnicodeCategory};

use crate::result::Result;

const PUNCTUATION: &[UnicodeCategory] = &[
    UnicodeCategory::ConnectorPunctuation,
    UnicodeCategory::DashPunctuation,
    UnicodeCategory::ClosePunctuation,
    UnicodeCategory::FinalPunctuation,
    UnicodeCategory::InitialPunctuation,
    UnicodeCategory::OtherPunctuation,
    UnicodeCategory::OpenPunctuation
];

pub struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Result<Dictionary> {
        let file = std::fs::read_to_string("words.txt")?;
        let words = file.split('\n').map(String::from).collect();

        Ok(Dictionary { words })
    }

    fn get_punct_to_split_on(&self, guess: &str) -> (Vec<char>, Vec<String>){
        let mut char_sequence: Vec<String> = Vec::new();
        let mut chars_to_split: Vec<char> = Vec::new();
        let mut was_last_ponct: bool = false;
        for chr in guess.chars(){
            if PUNCTUATION.contains(&chr.category()){
                if was_last_ponct {
                    char_sequence.last_mut().unwrap().push(chr);
                }
                else {
                    char_sequence.push(chr.to_string());
                }
                was_last_ponct = true;
                chars_to_split.push(chr);
            }
            else {
                was_last_ponct = false;
            }
        }
        (chars_to_split, char_sequence)
    }

    fn correct_word(&self,guess: &str) -> String{
        let mut best_word: String = String::new();
        if guess.chars().all(|chr| chr.is_ascii()){
            let mut dist_max: f64 = 0.0;
            let mut uppercases: Vec<bool> = vec![false; guess.len()];

            for(i, chr) in guess.char_indices(){
                if chr.is_uppercase() {uppercases[i] = true;}
            }

            for word in &self.words{
                if word.len() == guess.len() && dist_max != 1.0{
                    let dist: f64 = jaro_winkler(&guess.to_ascii_lowercase(), &word);
                    if dist > dist_max {
                        dist_max = dist;
                        best_word = word.clone();
                    }
                }
            }

            for i in 0..uppercases.len(){
                if uppercases[i]{
                    best_word.get_mut(i..=i).map(|chr| {chr.make_ascii_uppercase();});
                }
            } 
            return best_word
        }
        else{
            return guess.to_string()
        }
    }

    pub fn correct_guess(&self, guess: &str) -> String {
        let mut corrected: String = String::new();
        let (spliters, mut punct) = self.get_punct_to_split_on(guess);

        let mut arrays: Vec<&str> = guess.split(spliters.as_slice()).collect();
        arrays = arrays.iter().map(|x|*x).filter(|x| !x.is_empty()).collect();

        let mut is_punct_turn: bool = PUNCTUATION.contains(&guess.chars().next().unwrap().category());

        while !arrays.is_empty() || !punct.is_empty(){
            if arrays.is_empty(){
                corrected.push_str(&punct.remove(0));
            }
            else if punct.is_empty() {
                corrected.push_str(&self.correct_word(arrays.remove(0)));
            }
            else{
                if is_punct_turn{
                    corrected.push_str(&punct.remove(0));
                }
                else{
                    corrected.push_str(&self.correct_word(arrays.remove(0)));
                }
                is_punct_turn = !is_punct_turn;
            }
        }
        corrected
    }
}
