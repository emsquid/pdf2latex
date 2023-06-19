use crate::font::{Code, FontBase, Size};
use crate::glyph;
use crate::utils::{average, find_parts, Rect};
use image::DynamicImage;

const WORD_SPACING: u32 = 12;

pub struct Word {
    pub rect: Rect,
    pub glyphs: Vec<glyph::Unknown>,
}

impl Word {
    fn find_glyphs(bounds: Rect, image: &DynamicImage) -> Vec<glyph::Unknown> {
        let gray = bounds.crop(image).to_luma8();

        let mut glyphs = Vec::new();
        let mut x = 0;
        'outer: while x < gray.width() {
            for y in 0..gray.height() {
                if gray[(x, y)].0[0] <= glyph::CHAR_THRESHOLD {
                    let glyph = glyph::Unknown::from((x, y), bounds, image);
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
        for glyph in &mut self.glyphs {
            glyph.guess(fontbase, None);
        }

        let codes = self
            .glyphs
            .iter()
            .map(|glyph| glyph.guess.clone().unwrap().code)
            .collect::<Vec<Code>>();
        let sizes = self
            .glyphs
            .iter()
            .map(|glyph| glyph.guess.clone().unwrap().size)
            .collect::<Vec<Size>>();
        let (a_code, a_size) = (average(codes), average(sizes));

        for glyph in &mut self.glyphs {
            glyph.guess(fontbase, Some((a_code, a_size)));
        }
    }

    pub fn get_content(&self) -> String {
        let mut content = String::new();
        for glyph in &self.glyphs {
            if let Some(guess) = &glyph.guess {
                content.push(guess.chr);
            } else {
                content.push(' ');
            }
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

    pub fn get_content(&self) -> String {
        let mut content = String::new();
        for word in &self.words {
            content.push_str(&word.get_content());
            content.push(' ');
        }

        content
    }
}
