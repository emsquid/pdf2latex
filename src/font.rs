use crate::glyph::KnownGlyph;
use crate::result::Result;
use ab_glyph::{Font, FontVec};
use ucd::{Script,UnicodeBlock,UnicodeCategory};
use std::collections::HashMap;
use unicode_general_category::{get_general_category, GeneralCategory};

const BLACKLIST: &[GeneralCategory] = &[
    GeneralCategory::Control,
    GeneralCategory::Format,
    GeneralCategory::SpacingMark,
    GeneralCategory::NonspacingMark,
    GeneralCategory::LineSeparator,
    GeneralCategory::ParagraphSeparator,
    GeneralCategory::SpaceSeparator,
];

const WHITELIST_SCRIPT: &[Script] = &[

];

const WHITELIST_BLOCK: &[UnicodeBlock] = &[
    UnicodeBlock::BasicLatin,
    UnicodeBlock::Latin1Supplement,
    UnicodeBlock::LatinExtendedA,
    UnicodeBlock::LatinExtendedB, //80%
    UnicodeBlock::IPAExtensions, //80%
    UnicodeBlock::GreekandCoptic,
    UnicodeBlock::Hebrew, //Lots of strange thingys
    UnicodeBlock::LatinExtendedAdditional, //40%
    UnicodeBlock::GeneralPunctuation, //90%
    UnicodeBlock::SuperscriptsandSubscripts, //60%
    UnicodeBlock::LetterlikeSymbols, //Ensemble maths
    UnicodeBlock::Arrows,
    UnicodeBlock::MathematicalOperators,
    UnicodeBlock::MiscellaneousMathematicalSymbolsA,
    UnicodeBlock::SupplementalArrowsA,
    UnicodeBlock::SupplementalArrowsB,
    UnicodeBlock::MiscellaneousMathematicalSymbolsB,
    UnicodeBlock::SupplementalMathematicalOperators,
    UnicodeBlock::LatinExtendedC,
    UnicodeBlock::AlphabeticPresentationForms,
    UnicodeBlock::Gothic,
    UnicodeBlock::CuneiformNumbersandPunctuation,
    UnicodeBlock::MathematicalAlphanumericSymbols
];

const WHITELIST_CATEGORY: &[UnicodeCategory] = &[

];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    Cmr,
    Lmr,
    Qag,
    Qcr,
    Qpl,
    Xits,
}

impl Code {
    pub fn all() -> Vec<Code> {
        vec![
            Code::Cmr,
            Code::Lmr,
            Code::Qag,
            Code::Qcr,
            Code::Qpl,
            Code::Xits,
        ]
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Code::Cmr => "cmr",
            Code::Lmr => "lmr",
            Code::Qag => "qag",
            Code::Qcr => "qcr",
            Code::Qpl => "qpl",
            Code::Xits => "xits",
        }
    }

    pub fn as_path(&self) -> &'static str {
        match self {
            Code::Cmr => "fonts/cmr",
            Code::Lmr => "fonts/lmr",
            Code::Qag => "fonts/qag",
            Code::Qcr => "fonts/qcr",
            Code::Qpl => "fonts/qpl",
            Code::Xits => "fonts/xits",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Size {
    Tiny,
    Scriptsize,
    Footnotesize,
    Small,
    Normalsize,
    Large,
    LLarge,
    LLLarge,
    Huge,
    HHuge,
}

impl Size {
    pub fn all() -> Vec<Size> {
        vec![
            Size::Tiny,
            Size::Scriptsize,
            Size::Footnotesize,
            Size::Small,
            Size::Normalsize,
            Size::Large,
            Size::LLarge,
            Size::LLLarge,
            Size::Huge,
            Size::HHuge,
        ]
    }

    pub fn as_pt(&self) -> f32 {
        let base = 12.0;
        let delta = match self {
            Size::Tiny => -5.0,
            Size::Scriptsize => -3.25,
            Size::Footnotesize => -2.0,
            Size::Small => -1.0,
            Size::Normalsize => 0.0,
            Size::Large => 2.0,
            Size::LLarge => 4.4,
            Size::LLLarge => 7.28,
            Size::Huge => 10.74,
            Size::HHuge => 14.88,
        };

        base + delta
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    Bold,
    Italic,
    Slanted,
    // Underlined,
}

impl Style {
    pub fn from(path: &str) -> Vec<Style> {
        let mut styles = Vec::new();

        if path.contains("bold") {
            styles.push(Style::Bold);
        }
        if path.contains("italic") {
            styles.push(Style::Italic);
        }
        if path.contains("slant") {
            styles.push(Style::Slanted);
        }

        styles
    }
}

pub struct FontBase {
    pub glyphs: HashMap<Code, HashMap<(u32, u32), Vec<KnownGlyph>>>,
}

impl FontBase {
    fn load_font(path: &str, code: Code) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
        let font = FontVec::try_from_vec(std::fs::read(path)?)?;
        let styles = Style::from(path);

        let mut glyphs = HashMap::new();
        for size in Size::all() {
            for (id, chr) in font.codepoint_ids() {
                if BLACKLIST.contains(&get_general_category(chr)) {
                    continue;
                }
                if let Some(glyph) = KnownGlyph::try_from(&font, id, chr, code, size, &styles) {
                    let key = (glyph.rect.width, glyph.rect.height);
                    glyphs.entry(key).or_insert(Vec::new()).push(glyph);
                }
            }
        }

        Ok(glyphs)
    }

    fn load_family(code: Code) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
        let files = std::fs::read_dir(code.as_path())?;

        let mut family = HashMap::new();
        for file in files {
            let path = file?.path();
            for (key, glyphs) in FontBase::load_font(&path.to_string_lossy(), code)? {
                family.entry(key).or_insert(Vec::new()).extend(glyphs);
            }
        }

        Ok(family)
    }

    pub fn new() -> Result<FontBase> {
        let mut glyphs = HashMap::new();
        for code in Code::all() {
            glyphs.insert(code, FontBase::load_family(code)?);
        }

        Ok(FontBase { glyphs })
    }
}
