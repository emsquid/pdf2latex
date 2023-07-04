use crate::font::{Code, FontBase, Size, Style};
use crate::glyph::{UnknownGlyph, CHAR_THRESHOLD};
use std::ops::AddAssign;
use std::process::exit;

use crate::glyph::{Glyph, DIST_THRESHOLD, DIST_UNALIGNED_THRESHOLD};
use crate::utils::{average, find_parts, Rect};
use image::DynamicImage;
use std::collections::HashMap;

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

                    for _x in 0..glyph.rect.width {
                        for _y in 0..glyph.rect.height {
                            if glyph.get_pixel(_x, _y) < 1. {
                                gray.put_pixel(
                                    _x + glyph.rect.x - bounds.x,
                                    _y + glyph.rect.y - bounds.y,
                                    image::Luma([255]),
                                )
                            }
                        }
                    }
                    // gray.save(format!("./test/debug{}.png", glyphs.len())).unwrap();
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

    fn can_glyph_join(&self, index: usize) -> bool {
        if self.glyphs[index - 1].rect.x + self.glyphs[index - 1].rect.width - (WORD_SPACING / 4)
            > self.glyphs[index].rect.x
        {
            return true;
        }

        self.glyphs[index].dist.unwrap_or(f32::INFINITY) > DIST_THRESHOLD
    }
    pub fn guess(&mut self, fontbase: &FontBase, baseline: u32) {
        let length = self.glyphs.len();

        for glyph in &mut self.glyphs {
            glyph.try_guess(baseline, fontbase, length, None, true);
        }

        let mut base_index: usize = self.glyphs.len();
        'outer: while base_index > 1 {
            base_index -= 1;
            let mut dist = self.glyphs[base_index].dist.unwrap_or(f32::INFINITY);
            if !self.can_glyph_join(base_index) {
                continue 'outer;
            }

            for collapse_length in 1..=2 {
                if base_index < collapse_length {
                    continue 'outer;
                }
                dist += self.glyphs[base_index - collapse_length]
                    .dist
                    .unwrap_or(f32::INFINITY);

                let mut joined = self.glyphs[base_index].join(&self.glyphs[base_index - 1]);
                for i in 2..=collapse_length {
                    joined = joined.join(&self.glyphs[base_index - i]);
                }
                joined.try_guess(baseline, fontbase, length, None, true);

                if joined.dist.unwrap_or(f32::INFINITY) < dist {
                    for _ in 0..(collapse_length + 1) {
                        self.glyphs.remove(base_index - collapse_length);
                    }
                    self.glyphs.insert(base_index - collapse_length, joined);
                    base_index -= collapse_length;

                    continue 'outer;
                }
            }
        }

        for glyph in &mut self.glyphs {
            if glyph.dist.unwrap_or(f32::INFINITY) > DIST_UNALIGNED_THRESHOLD {
                glyph.try_guess(baseline, fontbase, length, None, false);
            }
        }

        // let hint = Option::zip(self.get_code(), self.get_size());
        // for glyph in &mut self.glyphs {
        // glyph.try_guess(fontbase, length, hint);
        // }
    }
    pub fn correct_font_guess(&mut self, fontbase: &FontBase, baseline: u32, code: Code) {
        let length = self.glyphs.len();

        for glyph in &mut self.glyphs {
            glyph.try_guess(
                baseline,
                fontbase,
                length,
                Some((code, Size::Normalsize)),
                true,
            );
        }
    }

    pub fn get_code(&self) -> Option<Code> {
        let codes = self
            .glyphs
            .iter()
            .map(|glyph| glyph.guess.clone().map(|guess| guess.code))
            .collect();

        average(codes)
    }

    pub fn get_size(&self) -> Option<Size> {
        let sizes = self
            .glyphs
            .iter()
            .map(|glyph| glyph.guess.clone().map(|guess| guess.size))
            .collect();

        average(sizes)
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

    pub fn debug_content(&self) -> String {
        self.glyphs
            .iter()
            .map(|glyph| {
                let mut content = String::new();
                if let Some(guess) = &glyph.guess {
                    if !guess.base.is_ascii() {
                        content.push_str("\x1b[31m");
                    }
                    if guess.style == Style::Bold {
                        content.push_str("\x1b[1;32m");
                    }
                    if guess.style == Style::Italic {
                        content.push_str("\x1b[3;34m");
                    }
                    if guess.style == Style::Slanted {
                        content.push_str("\x1b[3;35m");
                    }
                    content.push_str(&guess.base);
                } else {
                    content.push_str("\x1b[33m");
                    content.push('\u{2584}');
                }
                content.push_str("\x1b[0m");
                content
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
        let words = Line::find_words(rect, image);

        let mut bottoms = HashMap::new();
        for word in words.as_slice() {
            for glyph in word.glyphs.as_slice() {
                bottoms
                    .entry(glyph.rect.y + glyph.rect.height)
                    .or_insert(0)
                    .add_assign(1);
            }
        }

        let baseline = bottoms.into_iter().max_by_key(|&(_, c)| c).unwrap().0;

        Line {
            rect,
            words,
            baseline,
        }
    }

    pub fn get_code(&self) -> Code {
        let mut count = HashMap::new();
        for w in self.words.iter() {
            for g in w.glyphs.iter() {
                if g.guess.is_some() {
                    count
                        .entry(g.guess.as_ref().unwrap().code)
                        .or_insert(0)
                        .add_assign(1);
                }
            }
        }

        count
            .into_iter()
            .max_by_key(|&(_, c)| c)
            .unwrap_or((Code::Lmr, 0))
            .0
    }

    pub fn guess(&mut self, fontbase: &FontBase) {
        for word in &mut self.words {
            word.guess(fontbase, self.baseline);
        }
    }

    pub fn get_content(&self) -> String {
        self.words
            .iter()
            .map(|word| word.get_content())
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn debug_content(&self) -> String {
        self.words
            .iter()
            .map(|word| word.debug_content())
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
