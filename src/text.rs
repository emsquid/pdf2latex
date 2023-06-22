use crate::dictionary::Dictionary;
use crate::font::{Code, FontBase, Size, Style};
use crate::glyph::{UnknownGlyph, CHAR_THRESHOLD, self};
use crate::utils::{average, find_parts, Rect};
use image::DynamicImage;

const WORD_SPACING: u32 = 15;

pub struct Word {
    pub rect: Rect,
    pub glyphs: Vec<UnknownGlyph>,
}

impl Word {
    fn find_glyphs(bounds: Rect, image: &DynamicImage) -> Vec<UnknownGlyph> {
        let gray = bounds.crop(image).to_luma8();

        let mut glyphs = Vec::new();
        let mut x = 0;
        'outer: while x < gray.width() {
            for y in 0..gray.height() {
                if gray[(x, y)].0[0] <= CHAR_THRESHOLD {
                    let glyph = UnknownGlyph::from((x, y), bounds, image);
                    x = glyph.rect.x - bounds.x + glyph.rect.width;
                    glyphs.push(glyph);
                    continue 'outer;
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

    pub fn guess(&mut self, fontbase: &FontBase) {
        let length = self.glyphs.len();

        for glyph in &mut self.glyphs {
            glyph.try_guess(fontbase, length, None);
        }

        let hint = Option::zip(self.get_code(), self.get_size());
        for glyph in &mut self.glyphs {
            glyph.try_guess(fontbase, length, hint);
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

    pub fn get_content(&self, _dictionary: &Dictionary) -> String {
        let mut content = String::new();
        for glyph in &self.glyphs {
            if let Some(guess) = &glyph.guess {
                if !guess.chr.is_ascii() {
                    content.push_str("\x1b[31m");
                }
                if guess.styles.contains(&Style::Bold) {
                    content.push_str("\x1b[1m");
                }
                if guess.styles.contains(&Style::Italic) {
                    content.push_str("\x1b[3m");
                }
                if guess.styles.contains(&Style::Slanted) {
                    content.push_str("\x1b[3;4m");
                }
                content.push(guess.chr);
            } else {
                content.push_str("\x1b[33m");
                content.push('\u{2584}');
            }
            content.push_str("\x1b[0m");
        }

        content
    }
}

pub struct Line {
    pub rect: Rect,
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
        
        // let letters = Word::find_glyphs(bounds, image);
        // let mut words = Vec::new();
        // if letters.len() == 0 {
        //     return words;
        // }

        // words.push(Word{
        //     rect: letters[0].rect,
        //     glyphs: Vec::new(),
        // });
        // words[0].glyphs.push(letters[0].clone());
        // for glyph in &letters[1..]
        // {
        //     if glyph.distance_between(words.last().unwrap().glyphs.last().unwrap().clone()) < WORD_SPACING as f32
        //     {
        //         words.last_mut().unwrap().rect.join(glyph.rect);
        //         words.last_mut().unwrap().glyphs.push(glyph.clone());
        //     }
        //     else
        //     {
        //         words.push(Word{
        //             rect: glyph.rect,
        //             glyphs: Vec::new(),
        //         });
        //         words.last_mut().unwrap().glyphs.push(glyph.clone());
        //     }
        // };
        // return words;
    }

    pub fn from(rect: Rect, image: &DynamicImage) -> Line {
        Line {
            rect,
            words: Line::find_words(rect, image),
        }
    }

    pub fn guess(&mut self, fontbase: &FontBase) {
        for word in &mut self.words {
            word.guess(fontbase);
        }
    }

    pub fn get_content(&self, dictionary: &Dictionary) -> String {
        let mut content = String::new();
        for word in &self.words {
            content.push_str(&word.get_content(dictionary));
            content.push(' ');
        }

        content
    }
}