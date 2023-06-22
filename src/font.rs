use crate::glyph::KnownGlyph;
use crate::result::Result;
use crate::{args::Args, utils::log};
use ab_glyph::{Font, FontVec};
use std::collections::HashMap;
use std::io::Write;
use std::time;
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

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = match self {
            Code::Cmr => "cmr",
            Code::Lmr => "lmr",
            Code::Put => "put",
            Code::Qag => "qag",
            Code::Qcr => "qcr",
            Code::Qcs => "qcs",
            Code::Qpl => "qpl",
            // Code::Xits => "xits",
        };
        write!(f, "{string}")
    }
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

    pub fn as_path(&self) -> String {
        format!("fonts/{}", self.to_string())
    }

    pub fn len() -> usize {
        Code::all().len()
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

    pub fn as_pt(&self, base: f32) -> f32 {
        if (base - 10.0).abs() < f32::EPSILON {
            match self {
                Size::Tiny => 5.,
                Size::Scriptsize => 7.,
                Size::Footnotesize => 8.,
                Size::Small => 9.,
                Size::Normalsize => 10.,
                Size::Large => 12.,
                Size::LLarge => 14.4,
                Size::LLLarge => 17.28,
                Size::Huge => 20.74,
                Size::HHuge => 24.88,
            }
        } else if (base - 11.0).abs() < f32::EPSILON {
            match self {
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
            }
        } else if (base - 12.0).abs() < f32::EPSILON {
            match self {
                Size::Tiny => 6.,
                Size::Scriptsize => 8.,
                Size::Footnotesize => 10.,
                Size::Small => 10.95,
                Size::Normalsize => 12.,
                Size::Large => 14.4,
                Size::LLarge => 17.28,
                Size::LLLarge => 20.74,
                Size::Huge | Size::HHuge => 24.88,
            }
        } else {
            0.
        }
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
    fn load_font(
        path: &str,
        code: Code,
        args: &Args,
    ) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
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
                    if let Some(glyph) =
                        KnownGlyph::try_from(&font, id, chr, code, size, &styles, args)
                    {
                        let key = (glyph.rect.width, glyph.rect.height);
                        glyphs.entry(key).or_insert(Vec::new()).push(glyph);
                    }
                }
            }
        }

        Ok(glyphs)
    }

    fn load_family(code: Code, args: &Args) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
        let files = std::fs::read_dir(code.as_path())?;
        let step = 1. / std::fs::read_dir(code.as_path())?.count() as f32;

        let now = time::Instant::now();
        let mut progress = 0.;

        std::io::stdout().write_all(b"\x1b[s")?;
        log(&format!("loading font {code}"), Some(0.), None)?;

        let mut family = HashMap::new();
        for file in files {
            let path = file?.path();
            for (key, glyphs) in FontBase::load_font(&path.to_string_lossy(), code, args)? {
                family.entry(key).or_insert(Vec::new()).extend(glyphs);
            }

            progress += step;
            log(&format!("loading font {code}"), Some(progress), None)?;
        }

        let duration = now.elapsed().as_secs_f32();
        log(&format!("loading font {code}"), Some(1.), Some(duration))?;
        std::io::stdout().write_all(b"\n")?;

        Ok(family)
    }

    pub fn new(args: &Args) -> Result<FontBase> {
        let now = time::Instant::now();
        log("LOADING FONTS\n", None, None)?;

        let mut glyphs = HashMap::new();
        for code in Code::all() {
            glyphs.insert(code, FontBase::load_family(code, args)?);
        }

        let duration = now.elapsed().as_secs_f32();
        log(&format!("{} LOADED", Code::len()), None, Some(duration))?;
        std::io::stdout().write_all(b"\n")?;

        Ok(FontBase { glyphs })
    }
}
