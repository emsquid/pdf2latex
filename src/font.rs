use crate::glyph::KnownGlyph;
use crate::result::Result;
use ab_glyph::{Font, FontVec};
use std::collections::HashMap;
use ucd::{Codepoint, Script, UnicodeBlock, UnicodeCategory};
use std::io::Write;
use std::time;

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
    Xits,
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
            Code::Xits,
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
            Code::Xits => "xits",
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
        let files_count = std::fs::read_dir(code.as_path())?.count();
        let files = std::fs::read_dir(code.as_path())?;
        
        let now = time::Instant::now();
        let mut stdout = std::io::stdout();
        let mut progress = 0.;
        let progress_step = 1. / (files_count) as f32;
        stdout.write_all(
            format!("\n\x1b[sloading font {}\t[{}] 0%               ",
            code.to_string(),
            (0..21).map(|_| " ").collect::<String>()
        ).as_bytes()).unwrap();
        stdout.flush().unwrap();

        let mut family = HashMap::new();
        for file in files {
            // ======================== progress bar ==========================
            progress += progress_step * 21.;
            if ((progress - progress_step) * 100. / 21.).floor() != (progress * 100. / 21.).floor() {
                let length = progress.floor() as u32;
                
                stdout.write_all((
                    format!("\x1b[uloading font {}\t[{}{}] {}%               ",
                    code.to_string(),
                    (0..length).map(|_| "=").collect::<String>(),
                    (length..20).map(|_| " ").collect::<String>(),
                    (progress * 100. / 21.).round())
                ).as_bytes()).unwrap();
                stdout.flush().unwrap();
            }
            // =================================================================

            let path = file?.path();
            for (key, glyphs) in FontBase::load_font(&path.to_string_lossy(), code)? {
                family.entry(key).or_insert(Vec::new()).extend(glyphs);
            }
        }
        stdout.write_all(
            format!("\x1b[uloading font {}\t[{}] {}s               ",
            code.to_string(),
            (0..21).map(|_| "=").collect::<String>(),
            now.elapsed().as_secs_f32()
        ).as_bytes()).unwrap();
        stdout.flush().unwrap();

        Ok(family)
    }

    pub fn new() -> Result<FontBase> {
        let now = time::Instant::now();
        let mut stdout = std::io::stdout();
        stdout.write_all(b"LOADING FONTS").unwrap();
        stdout.flush().unwrap();

        let mut glyphs = HashMap::new();
        for code in Code::all() {
            glyphs.insert(code, FontBase::load_family(code)?);
        }
        
        stdout.write_all(format!("\n{} FONTS LOADED IN {}s\n", Code::all().len(), now.elapsed().as_secs_f32()).as_bytes()).unwrap();
        stdout.flush().unwrap();

        Ok(FontBase { glyphs })
    }
}
