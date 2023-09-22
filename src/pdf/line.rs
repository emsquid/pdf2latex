use super::Word;
use crate::fonts::FontBase;
use crate::fonts::KnownGlyph;
use crate::utils::{find_parts, most_frequent, Rect};
use image::DynamicImage;

const WORD_SPACING: u32 = 15;

/// A Line from a Page from a Pdf
pub struct Line {
    pub rect: Rect,
    pub baseline: u32,

    pub words: Vec<Word>,
}

impl Line {
    /// Create a Line from the given rect and image
    #[must_use]
    pub fn from(rect: Rect, image: &DynamicImage) -> Line {
        let words = Self::find_words(rect, image);
        let baseline = Self::find_baseline(&words);

        Line {
            rect,
            baseline,
            words,
        }
    }

    /// Find the words in a Line based on its bounds
    fn find_words(bounds: Rect, image: &DynamicImage) -> Vec<Word> {
        find_parts(&bounds.crop(image).rotate90().to_luma8(), WORD_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(bounds.x + start, bounds.y, end - start + 1, bounds.height);
                Word::from(rect, image)
            })
            .collect()
    }

    /// Find the baseline of the given words
    fn find_baseline(words: &[Word]) -> u32 {
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
    pub fn get_first_guess(&self) -> Option<KnownGlyph> {
        self.words
            .first()
            .and_then(|word| word.glyphs.first().and_then(|glyph| glyph.guess.clone()))
    }

    /// Get the guess for the last glyph in a Line
    #[must_use]
    pub fn get_last_guess(&self) -> Option<KnownGlyph> {
        self.words
            .last()
            .and_then(|word| word.glyphs.last().and_then(|glyph| glyph.guess.clone()))
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
    pub fn get_latex(&self, prev: &Option<KnownGlyph>, next: &Option<KnownGlyph>) -> String {
        self.words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let prev = self
                    .words
                    .get(i - 1)
                    .map_or(prev.clone(), Word::get_last_guess);
                let next = self
                    .words
                    .get(i + 1)
                    .map_or(next.clone(), Word::get_first_guess);

                word.get_latex(&prev, &next)
            })
            .collect::<Vec<String>>()
            .join(" ")
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
            .and_then(|word| word.glyphs.last()).map(|glyph| glyph.rect.width + glyph.rect.x)
    }

    /// Compute the relative margin of the first glyph of the line
    #[must_use]
    pub fn get_left_margin(&self) -> Option<u32> {
        self.words
            .first()
            .and_then(|word| word.glyphs.first()).map(|glyph| glyph.rect.x)
    }
}
