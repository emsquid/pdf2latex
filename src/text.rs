use image::{DynamicImage, GrayImage};

const GLYPH_SPACING: u32 = 5;
const GLYPH_THRESHOLD: u8 = 180;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn crop(self, image: DynamicImage) -> DynamicImage {
        image.crop_imm(self.x, self.y, self.width, self.height)
    }
}

pub struct Char {
    pub rect: Rect,
}

impl Char {
    pub fn new(rect: Rect) -> Char {
        Char { rect }
    }

    pub fn flood_fill(gray: GrayImage, pos: (u32, u32)) -> u32 {
        let mut pixels = vec![pos];
        let mut index = 0;
        let mut width = 0;

        while index < pixels.len() {
            let (x, y) = pixels[index];
            for dx in -1..2 {
                for dy in -1..2 {
                    let nx = x.saturating_add_signed(dx);
                    let ny = y.saturating_add_signed(dy);

                    if nx < gray.width()
                        && ny < gray.height()
                        && !pixels.contains(&(nx, ny))
                        && gray[(nx, ny)].0[0] <= GLYPH_THRESHOLD
                    {
                        pixels.push((nx, ny));
                        width = std::cmp::max(nx.saturating_sub(pos.0), width);
                    }
                }
            }
            index += 1;
        }

        width
    }
}

pub struct Word {
    pub rect: Rect,
    pub chars: Vec<Char>,
}

impl Word {
    pub fn new(rect: Rect, image: DynamicImage) -> Word {
        Word {
            rect,
            chars: Word::get_chars(rect, image),
        }
    }

    fn get_chars(bound: Rect, image: DynamicImage) -> Vec<Char> {
        let gray = image.to_luma8();

        let mut chars = Vec::new();
        let mut x = 0;

        while x < gray.width() {
            for y in 0..gray.height() {
                if gray[(x, y)].0[0] <= GLYPH_THRESHOLD {
                    let width = Char::flood_fill(gray.clone(), (x, y));
                    let rect = Rect::new(bound.x + x, bound.y, width + 1, bound.height);
                    chars.push(Char::new(rect));
                    x += rect.width;
                    break;
                }
            }
            x += 1;
        }

        chars
    }
}

pub struct Line {
    pub rect: Rect,
    pub words: Vec<Word>,
}

impl Line {
    pub fn new(rect: Rect, image: DynamicImage) -> Line {
        Line {
            rect,
            words: Line::get_words(rect, image),
        }
    }

    fn get_words(bound: Rect, image: DynamicImage) -> Vec<Word> {
        let mut words = Vec::new();
        let mut x = 0;
        let mut width = 0;

        for (i, column) in image.rotate90().to_luma8().enumerate_rows() {
            let average = column.map(|l| u32::from(l.2 .0[0])).sum::<u32>() / image.height();
            if x != 0 && average == 255 {
                if width == 0 {
                    width = i - x;
                } else if i - (x + width) >= GLYPH_SPACING {
                    let rect = Rect::new(bound.x + x, bound.y, width, bound.height);
                    let cropped = image.crop_imm(x, 0, rect.width, rect.height);
                    words.push(Word::new(rect, cropped));
                    x = 0;
                }
            } else if average != 255 {
                width = 0;
                if x == 0 {
                    x = i;
                }
            }
        }

        words
    }
}
