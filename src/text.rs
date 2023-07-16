use crate::font::FontBase;
use crate::glyph::{Glyph, KnownGlyph, DIST_THRESHOLD, DIST_UNALIGNED_THRESHOLD};
use crate::glyph::{UnknownGlyph, CHAR_THRESHOLD};
use crate::utils::{find_parts, mode, Rect};
use image::DynamicImage;

const WORD_SPACING: u32 = 15;

/// A word from a Line from a Page from a Pdf
pub struct Word {
    pub rect: Rect,
    pub glyphs: Vec<UnknownGlyph>,
}

impl Word {
    /// Create a word from the given rect and image
    pub fn from(rect: Rect, image: &DynamicImage) -> Word {
        Word {
            rect,
            glyphs: Word::find_glyphs(rect, image),
        }
    }

    /// Find the glyphs in a Word based on its bounds
    fn find_glyphs(bounds: Rect, image: &DynamicImage) -> Vec<UnknownGlyph> {
        let mut gray = bounds.crop(image).to_luma8();

        let mut glyphs = Vec::new();
        let mut x = 0;
        while x < gray.width() {
            for y in 0..gray.height() {
                // Check if there is a glyph at (x, y)
                if gray[(x, y)].0[0] <= CHAR_THRESHOLD {
                    let glyph = UnknownGlyph::from((x, y), bounds, image);
                    // Remove black pixel which belongs to the glyph from the image
                    for nx in 0..glyph.rect.width {
                        for ny in 0..glyph.rect.height {
                            if glyph.get_pixel(nx, ny) < 1. {
                                gray.put_pixel(
                                    nx + glyph.rect.x - bounds.x,
                                    ny + glyph.rect.y - bounds.y,
                                    image::Luma([255]),
                                );
                            }
                        }
                    }
                    glyphs.push(glyph);
                }
            }
            x += 1;
        }

        glyphs
    }

    /// Check if a glyph should be joined with others
    fn should_glyph_join(&self, index: usize) -> bool {
        self.glyphs[index - 1].rect.x + self.glyphs[index - 1].rect.width - (WORD_SPACING / 4)
            > self.glyphs[index].rect.x
            || self.glyphs[index].dist.unwrap_or(f32::INFINITY) > DIST_THRESHOLD
    }

    /// Guess the content of a Word
    pub fn guess(&mut self, fontbase: &FontBase, baseline: u32) {
        // Try to guess normally
        for glyph in &mut self.glyphs {
            glyph.try_guess(fontbase, baseline, true);
        }

        // Join glyphs that were poorly recognized
        let mut base_index: usize = self.glyphs.len();
        'outer: while base_index > 1 {
            base_index -= 1;

            if !self.should_glyph_join(base_index) {
                continue 'outer;
            }

            let mut joined = self.glyphs[base_index].clone();
            let mut dist = self.glyphs[base_index].dist.unwrap_or(f32::INFINITY);
            for collapse_length in 1..=2 {
                if base_index < collapse_length {
                    continue 'outer;
                }

                dist = dist.max(
                    self.glyphs[base_index - collapse_length]
                        .dist
                        .unwrap_or(f32::INFINITY),
                );

                // Join a glyph with 1/2 other glyphs and try to guess it
                joined = joined.join(&self.glyphs[base_index - collapse_length]);
                joined.try_guess(fontbase, baseline, true);

                // If it's better replace the bad ones with the new one
                if joined.dist.unwrap_or(f32::INFINITY) < dist {
                    self.glyphs.drain(base_index - collapse_length..=base_index);
                    self.glyphs.insert(base_index - collapse_length, joined);
                    base_index -= collapse_length;
                    continue 'outer;
                }
            }
        }

        // The worst one are checked without paying attention to their offset
        for glyph in &mut self.glyphs {
            if glyph.dist.unwrap_or(f32::INFINITY) > DIST_UNALIGNED_THRESHOLD {
                glyph.try_guess(fontbase, baseline, false);
            }
        }
    }

    /// Get the guess for the first glyph in a Word
    pub fn get_first_guess(&self) -> Option<KnownGlyph> {
        self.glyphs.first().and_then(|glyph| glyph.guess.clone())
    }

    /// Get the guess for the last glyph in a Word
    pub fn get_last_guess(&self) -> Option<KnownGlyph> {
        self.glyphs.last().and_then(|glyph| glyph.guess.clone())
    }

    /// Get the content of a Word, mostly for debugging
    pub fn get_content(&self) -> String {
        self.glyphs
            .iter()
            .map(|glyph| match &glyph.guess {
                Some(guess) => guess.base.clone(),
                None => '\u{2584}'.to_string(),
            })
            .collect()
    }

    /// Get the LaTeX for a Word
    pub fn get_latex(&self, prev: &Option<KnownGlyph>, next: &Option<KnownGlyph>) -> String {
        self.glyphs
            .iter()
            .enumerate()
            .map(|(i, glyph)| {
                let prev = self.glyphs.get(i - 1).map_or(prev, |g| &g.guess);
                let next = self.glyphs.get(i + 1).map_or(next, |g| &g.guess);

                glyph.guess.clone().map_or(String::from("?"), |g| {
                    g.get_latex(prev, next, i == self.glyphs.len() - 1)
                })
            })
            .collect()
    }

    /// Compute the sum of the distance of each Glyph in the Word
    pub fn get_dist_sum(&self) -> f32 {
        self.glyphs
            .iter()
            .map(|glyph| glyph.dist.unwrap_or(0.))
            .sum()
    }
}

/// A Line from a Page from a Pdf
pub struct Line {
    pub rect: Rect,
    pub baseline: u32,

    pub words: Vec<Word>,
}

impl Line {
    /// Create a Line from the given rect and image
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
    fn find_baseline(words: &Vec<Word>) -> u32 {
        let bottoms = words
            .iter()
            .flat_map(|word| {
                word.glyphs
                    .iter()
                    .map(|glyph| glyph.rect.y + glyph.rect.height)
            })
            .collect();

        mode(bottoms, 0)
    }

    /// Guess the content of a Line
    pub fn guess(&mut self, fontbase: &FontBase) {
        for word in &mut self.words {
            word.guess(fontbase, self.baseline);
        }
    }

    /// Get the guess for the first glyph in a Line
    pub fn get_first_guess(&self) -> Option<KnownGlyph> {
        self.words
            .first()
            .and_then(|word| word.glyphs.first().and_then(|glyph| glyph.guess.clone()))
    }

    /// Get the guess for the last glyph in a Line
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
        self.words.iter().map(|word| word.get_dist_sum()).sum()
    }

    /// Compute the number of glyph in a Line
    pub fn get_glyph_count(&self) -> u32 {
        self.words.iter().map(|word| word.glyphs.len() as u32).sum()
    }
}
