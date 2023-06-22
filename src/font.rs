use crate::glyph::KnownGlyph;
use crate::result::Result;
use ab_glyph::{Font, FontVec};
use std::collections::HashMap;
use ucd::{Codepoint, Script, UnicodeBlock, UnicodeCategory};

const WHITELIST_SCRIPT: &[Script] = &[
    Script::Common,
    Script::Cuneiform,
    Script::Gothic,
    Script::Greek,
    Script::Hebrew,
    Script::Latin,
];

const WHITELIST_BLOCK: &[UnicodeBlock] = &[
    UnicodeBlock::BasicLatin,
    UnicodeBlock::Latin1Supplement,
    UnicodeBlock::GreekandCoptic,
    UnicodeBlock::Hebrew,
    UnicodeBlock::GeneralPunctuation,
    UnicodeBlock::SuperscriptsandSubscripts,
    UnicodeBlock::LetterlikeSymbols,
    UnicodeBlock::Arrows,
    UnicodeBlock::MathematicalOperators,
    UnicodeBlock::MiscellaneousMathematicalSymbolsA,
    UnicodeBlock::SupplementalArrowsA,
    UnicodeBlock::SupplementalArrowsB,
    UnicodeBlock::MiscellaneousMathematicalSymbolsB,
    UnicodeBlock::SupplementalMathematicalOperators,
    UnicodeBlock::AlphabeticPresentationForms,
    UnicodeBlock::Gothic,
    UnicodeBlock::CuneiformNumbersandPunctuation,
    UnicodeBlock::MathematicalAlphanumericSymbols,
    UnicodeBlock::GeometricShapes,
];

const WHITELIST_CATEGORY: &[UnicodeCategory] = &[
    UnicodeCategory::LowercaseLetter,
    UnicodeCategory::ModifierLetter,
    UnicodeCategory::OtherLetter,
    UnicodeCategory::UppercaseLetter,
    UnicodeCategory::EnclosingMark,
    UnicodeCategory::DecimalNumber,
    UnicodeCategory::LetterNumber,
    UnicodeCategory::ConnectorPunctuation,
    UnicodeCategory::DashPunctuation,
    UnicodeCategory::OpenPunctuation,
    UnicodeCategory::ClosePunctuation,
    UnicodeCategory::InitialPunctuation,
    UnicodeCategory::FinalPunctuation,
    UnicodeCategory::OtherPunctuation,
    UnicodeCategory::CurrencySymbol,
    UnicodeCategory::MathSymbol,
    UnicodeCategory::OtherSymbol,
];

const BLACKLIST: &[char] = &['·'];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    Cmr,
    Lmr,
    Put,
    Qag,
    Qcr,
    Qcs,
    Qpl,
    // Xits,
}

impl Code {
    pub fn all() -> Vec<Code> {
        vec![
            Code::Cmr,
            Code::Lmr,
            Code::Put,
            Code::Qag,
            Code::Qcr,
            Code::Qcs,
            Code::Qpl,
            // Code::Xits,
        ]
    }

    pub fn to_string(&self) -> String {
        match self {
            Code::Cmr => "cmr",
            Code::Lmr => "lmr",
            Code::Put => "put",
            Code::Qag => "qag",
            Code::Qcr => "qcr",
            Code::Qcs => "qcs",
            Code::Qpl => "qpl",
            // Code::Xits => "xits",
        }
        .to_string()
    }

    pub fn as_path(&self) -> String {
        format!("fonts/{}", self.to_string())
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
        // considering size is 11pt
        let delta = match self {
            Size::Tiny => 6.,
            Size::Scriptsize => 8.,
            Size::Footnotesize => 9.,
            Size::Small => 10.,
            Size::Normalsize => 10.95,
            Size::Large => 12.,
            Size::LLarge => 14.4,
            Size::LLLarge => 17.28,
            Size::Huge => 20.74,
            Size::HHuge => 24.88,
        };

        delta
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
                if let (Some(script), Some(block), category) =
                    (chr.script(), chr.block(), chr.category())
                {
                    if !WHITELIST_SCRIPT.contains(&script)
                        || !WHITELIST_BLOCK.contains(&block)
                        || !WHITELIST_CATEGORY.contains(&category)
                        || BLACKLIST.contains(&chr)
                    {
                        continue;
                    }
                    if let Some(glyph) = KnownGlyph::try_from(&font, id, chr, code, size, &styles) {
                        let key = (glyph.rect.width, glyph.rect.height);
                        glyphs.entry(key).or_insert(Vec::new()).push(glyph);
                    }
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
