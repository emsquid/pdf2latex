use super::Word;
use super::{word::BracketData, Page};
use crate::{
    fonts::{FontBase, Glyph, KnownGlyph, DIST_THRESHOLD},
    utils::{find_parts, most_frequent, Rect},
};
use anyhow::Result;
use image::DynamicImage;

const WORD_SPACING: u32 = 15;

pub struct LineData {
    pub is_math_middle_line: bool,
}

/// A Line from a Page from a Pdf
#[derive(Clone)]
pub struct Line {
    pub rect: Rect,
    pub baseline: u32,
    pub can_have_new_line: bool,

    pub words: Vec<Word>,
}

impl Line {
    /// Create a Line from the given rect and image
    #[must_use]
    pub fn from(rect: Rect, image: &DynamicImage, word_spacing: Option<u32>) -> Line {
        let words = Self::find_words(rect, image, word_spacing);
        let baseline = Self::find_baseline(&words);

        Line {
            rect,
            baseline,
            can_have_new_line: true,
            words,
        }
    }

    /// Find the words in a Line based on its bounds
    fn find_words(bounds: Rect, image: &DynamicImage, word_spacing: Option<u32>) -> Vec<Word> {
        find_parts(
            &bounds.crop(image).rotate90().to_luma8(),
            word_spacing.unwrap_or(WORD_SPACING),
        )
        .into_iter()
        .map(|(start, end)| {
            let rect = Rect::new(bounds.x + start, bounds.y, end - start + 1, bounds.height);
            Word::from(rect, image)
        })
        .collect()
    }

    /// Find the baseline of the given words
    pub fn find_baseline(words: &[Word]) -> u32 {
        let bottoms = words
            .iter()
            .flat_map(|word| {
                word.glyphs
                    .iter()
                    .map(|glyph| glyph.rect.y + glyph.rect.height)
            })
            .collect::<Vec<u32>>();

        most_frequent(&bottoms, 0).0
    }

    /// Guess the content of a Line
    pub fn guess(&mut self, fontbase: &FontBase) {
        for word in &mut self.words {
            word.guess(fontbase, self.baseline);
        }
    }

    /// Get the guess for the first glyph in a Line
    #[must_use]
    pub fn get_first_guess(&self) -> Option<&KnownGlyph> {
        self.words.first().and_then(|word| word.get_first_guess())
    }

    /// Get the guess for the last glyph in a Line
    #[must_use]
    pub fn get_last_guess(&self) -> Option<&KnownGlyph> {
        self.words.last().and_then(|word| word.get_last_guess())
    }

    /// Get the content of a Line, mostly for debugging
    pub fn get_content(&self) -> String {
        self.words
            .iter()
            .map(Word::get_content)
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Get the LaTeX for a Line
    #[must_use]
    pub fn get_latex(
        &self,
        page_owner: &Page,
        prev: Option<&KnownGlyph>,
        next: Option<&KnownGlyph>,
    ) -> String {
        let line_data = self.into_line_data(&page_owner.get_margins());
        let mut latex = self
            .words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let prev = self.words.get(i - 1).map_or(prev, |w| w.get_last_guess());
                let next = self.words.get(i + 1).map_or(next, |w| w.get_first_guess());

                word.get_latex(&line_data, prev, next)
            })
            .collect::<Vec<String>>()
            .join(" ");
        if self.words.len() != 1 && self.is_middle_line(&page_owner.get_margins()) {
            latex = "$$".to_string() + &latex + "$$";
        }
        latex
    }

    /// Compute the sum of the distance of each Word in the Line
    pub fn get_dist_sum(&self) -> f32 {
        self.words.iter().map(Word::get_dist_sum).sum()
    }

    /// Compute the number of glyph in a Line
    #[must_use]
    pub fn get_glyph_count(&self) -> u32 {
        self.words.iter().map(|word| word.glyphs.len() as u32).sum()
    }

    #[must_use]
    pub fn get_margins(&self) -> (Option<u32>, Option<u32>) {
        (self.get_left_margin(), self.get_right_margin())
    }

    /// Compute the relative margin of the last glyph of the line
    #[must_use]
    pub fn get_right_margin(&self) -> Option<u32> {
        self.words
            .last()
            .and_then(|word| word.glyphs.last())
            .map(|glyph| glyph.rect.width + glyph.rect.x)
    }

    /// Compute the relative margin of the first glyph of the line
    #[must_use]
    pub fn get_left_margin(&self) -> Option<u32> {
        self.words
            .first()
            .and_then(|word| word.glyphs.first())
            .map(|glyph| glyph.rect.x)
    }

    pub fn get_bottom(&self) -> Option<u32> {
        self.words
            .iter()
            .map(|word| word.rect().y + word.rect().height)
            .min()
    }

    pub fn get_top(&self) -> Option<u32> {
        self.words.iter().map(|word| word.rect().y).max()
    }

    pub fn search_words(&self, pattern: &str) -> Vec<usize> {
        self.words
            .iter()
            .enumerate()
            .flat_map(|(i, word)| {
                if word.get_content() == pattern {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn pop_words_in_rect(&mut self, rect: &Rect) {
        let mut index_to_pop = Vec::new();
        for (i, word) in self.words.iter_mut().enumerate() {
            if rect.contains(word.rect()) {
                index_to_pop.push(i);
            }
        }

        for i in (0..index_to_pop.len()).rev() {
            self.words.remove(*index_to_pop.get(i).unwrap());
        }
    }

    pub fn count_glyphes(&self) -> usize {
        self.words.iter().map(|word| word.glyphs.len()).sum()
    }

    pub fn is_full_line(&self, page_margins: &(u32, u32)) -> bool {
        if let (Some(left_margin), Some(right_margin)) = self.get_margins() {
            return left_margin.abs_diff(page_margins.0) < 50
                && right_margin.abs_diff(page_margins.1) < 50;
        }
        return false;
    }

    /// return a Vec with values ordered by (glyph, bracket_type, word_index, glyph_index)
    pub fn get_all_brackets(&self) -> Result<Vec<BracketData>> {
        let mut brackets: Vec<BracketData> = Vec::new();
        for (wi, word) in self.words.iter().enumerate() {
            for (gi, glyph) in word.glyphs.iter().enumerate() {
                if glyph.dist.is_some_and(|v| v > DIST_THRESHOLD) {
                    if let Ok(Some(bracket_type)) = glyph.get_bracket_type() {
                        // TODO avoid clone image
                        brackets.push((glyph.clone(), bracket_type, wi, gi));
                    }
                }
            }
        }
        Ok(brackets)
    }

    pub fn search_opposing_brackets(
        &self,
        data: &BracketData,
        brackets: &[BracketData],
    ) -> Option<usize> {
        let dr = &data.0.rect;
        let mut br: &Rect;
        let opposing = data.1.get_opposit();
        for (i, bracket) in brackets.iter().enumerate() {
            br = &bracket.0.rect;
            if bracket.1 == opposing
                && br.height.abs_diff(dr.height) <= 10
                && br.y.abs_diff(dr.y) <= 10
            {
                return Some(i);
            }
        }
        None
    }

    pub fn is_middle_line(&self, margins: &(u32, u32)) -> bool {
        let line_margin = (self.get_left_margin(), self.get_right_margin());
        if let (Some(left_margin), Some(right_margin)) = line_margin {
            if ((margins.1).abs_diff(right_margin)).abs_diff((left_margin).abs_diff(margins.0)) < 25
                && !self.is_full_line(margins)
            {
                return true;
            }
        }
        false
    }

    pub fn is_math_middle_line_undetected(&self) -> bool {
        self.get_dist_sum() / (self.count_glyphes() as f32) > 20.
    }

    pub fn into_line_data(&self, margins: &(u32, u32)) -> LineData {
        LineData {
            is_math_middle_line: self.is_middle_line(margins),
        }
    }
}
