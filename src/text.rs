use crate::font::FontBase;
use crate::glyph::{Glyph, KnownGlyph, DIST_THRESHOLD, DIST_UNALIGNED_THRESHOLD};
use crate::glyph::{UnknownGlyph, CHAR_THRESHOLD};
use crate::utils::{average, find_parts, Rect};
use image::DynamicImage;

const WORD_SPACING: u32 = 15;

pub struct Word {
    pub rect: Rect,
    pub glyphs: Vec<UnknownGlyph>,
}

impl Word {
    fn find_glyphs(bounds: Rect, image: &DynamicImage) -> Vec<UnknownGlyph> {
        let mut gray = bounds.crop(image).to_luma8();

        let mut glyphs = Vec::new();
        let mut x = 0;
        while x < gray.width() {
            for y in 0..gray.height() {
                if gray[(x, y)].0[0] <= CHAR_THRESHOLD {
                    let glyph = UnknownGlyph::from((x, y), bounds, image);

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

    pub fn from(rect: Rect, image: &DynamicImage) -> Word {
        Word {
            rect,
            glyphs: Word::find_glyphs(rect, image),
        }
    }

    fn should_glyph_join(&self, index: usize) -> bool {
        self.glyphs[index - 1].rect.x + self.glyphs[index - 1].rect.width - (WORD_SPACING / 4)
            > self.glyphs[index].rect.x
            || self.glyphs[index].dist.unwrap_or(f32::INFINITY) > DIST_THRESHOLD
    }

    pub fn guess(&mut self, fontbase: &FontBase, baseline: u32) {
        for glyph in &mut self.glyphs {
            glyph.try_guess(fontbase, baseline, true);
        }

        let mut base_index: usize = self.glyphs.len();
        'outer: while base_index > 1 {
            base_index -= 1;

            if !self.should_glyph_join(base_index) {
                continue 'outer;
            }

            let mut dist = self.glyphs[base_index].dist.unwrap_or(f32::INFINITY);
            for collapse_length in 1..=2 {
                if base_index < collapse_length {
                    continue 'outer;
                }
                dist += self.glyphs[base_index - collapse_length]
                    .dist
                    .unwrap_or(f32::INFINITY);

                let mut joined = self.glyphs[base_index].clone();
                for i in 1..=collapse_length {
                    joined = joined.join(&self.glyphs[base_index - i]);
                }
                joined.try_guess(fontbase, baseline, true);

                if joined.dist.unwrap_or(f32::INFINITY) < dist {
                    self.glyphs.drain(base_index - collapse_length..=base_index);
                    self.glyphs.insert(base_index - collapse_length, joined);
                    base_index -= collapse_length;
                    continue 'outer;
                }
            }
        }

        for glyph in &mut self.glyphs {
            if glyph.dist.unwrap_or(f32::INFINITY) > DIST_UNALIGNED_THRESHOLD {
                glyph.try_guess(fontbase, baseline, false);
            }
        }
    }

    pub fn get_first_guess(&self) -> Option<KnownGlyph> {
        self.glyphs.first().and_then(|glyph| glyph.guess.clone())
    }

    pub fn get_last_guess(&self) -> Option<KnownGlyph> {
        self.glyphs.last().and_then(|glyph| glyph.guess.clone())
    }

    pub fn get_content(&self) -> String {
        self.glyphs
            .iter()
            .map(|glyph| match &glyph.guess {
                Some(guess) => guess.base.clone(),
                None => '\u{2584}'.to_string(),
            })
            .collect()
    }

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

    pub fn get_dist_sum(&self) -> f32 {
        self.glyphs
            .iter()
            .map(|glyph| glyph.dist.unwrap_or(0.))
            .sum()
    }
}

pub struct Line {
    pub rect: Rect,
    pub baseline: u32,
    pub words: Vec<Word>,
}

impl Line {
    fn find_words(bounds: Rect, image: &DynamicImage) -> Vec<Word> {
        find_parts(&bounds.crop(image).rotate90().to_luma8(), WORD_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(bounds.x + start, bounds.y, end - start + 1, bounds.height);
                Word::from(rect, image)
            })
            .collect()
    }

    pub fn from(rect: Rect, image: &DynamicImage) -> Line {
        let words = Self::find_words(rect, image);

        let bottoms = words
            .iter()
            .flat_map(|word| {
                word.glyphs
                    .iter()
                    .map(|glyph| glyph.rect.y + glyph.rect.height)
            })
            .collect();

        Line {
            rect,
            words,
            baseline: average(bottoms, 0),
        }
    }

    pub fn guess(&mut self, fontbase: &FontBase) {
        for word in &mut self.words {
            word.guess(fontbase, self.baseline);
        }
    }

    pub fn get_first_guess(&self) -> Option<KnownGlyph> {
        self.words
            .first()
            .and_then(|word| word.glyphs.first().and_then(|glyph| glyph.guess.clone()))
    }

    pub fn get_last_guess(&self) -> Option<KnownGlyph> {
        self.words
            .last()
            .and_then(|word| word.glyphs.last().and_then(|glyph| glyph.guess.clone()))
    }

    pub fn get_content(&self) -> String {
        self.words
            .iter()
            .map(Word::get_content)
            .collect::<Vec<String>>()
            .join(" ")
    }

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

    pub fn get_dist_sum(&self) -> f32 {
        self.words.iter().map(|word| word.get_dist_sum()).sum()
    }

    pub fn get_glyph_count(&self) -> u32 {
        self.words.iter().map(|word| word.glyphs.len() as u32).sum()
    }
}
