use crate::args::Args;
use crate::glyph::KnownGlyph;
use crate::result::Result;
use crate::utils::log;
use bitcode::Encode;
use clap::ValueEnum;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::{time, vec};

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
const ACCENTS: &str = include_str!("data/accents.txt");
const MATH_ACCENTS: &str = include_str!("data/math_accents.txt");
const PUNCTUATIONS: &str = include_str!("data/punctuations.txt");
const LIGATURES: &str = include_str!("data/ligatures.txt");
const GREEKS: &str = include_str!("data/greeks.txt");
const HEBREWS: &str = include_str!("data/hebrews.txt");
const CONSTRUCTS: &str = include_str!("data/constructs.txt");
const OPERATIONS: &str = include_str!("data/operations.txt");
const ARROWS: &str = include_str!("data/arrows.txt");
const MISCELLANEOUS: &str = include_str!("data/miscellaneous.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Encode, bitcode::Decode)]
pub enum Code {
    Cmr,
    Lmr,
    Put,
    Qag,
    Qcr,
    Qcs,
    Qpl,
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
        ]
    }

    pub fn as_path(self) -> String {
        let config = dirs::config_dir().unwrap_or(PathBuf::from("~/.config"));
        format!("{}/pdf2latex/{self}", config.display())
    }
}

/// An enum representing the different LaTeX sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bitcode::Encode, bitcode::Decode)]
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

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Size::Tiny => "tiny",
            Size::Scriptsize => "scriptsize",
            Size::Footnotesize => "footnotesize",
            Size::Small => "small",
            Size::Normalsize => "normalsize",
            Size::Large => "large",
            Size::LLarge => "Large",
            Size::LLLarge => "LARGE",
            Size::Huge => "huge",
            Size::HHuge => "Huge",
        };
        write!(f, "{string}")
    }
}

impl Size {
    /// Create an iterator over all possible sizes
    pub fn all() -> Vec<Size> {
        vec![
            Size::Normalsize,
            Size::Small,
            Size::Large,
            Size::Footnotesize,
            Size::LLarge,
            Size::Scriptsize,
            Size::Tiny,
            Size::LLLarge,
            Size::Huge,
            Size::HHuge,
        ]
    }

    /// Convert a size to a decent file path
    pub fn as_path(self) -> String {
        match self {
            Size::Tiny => "tiny",
            Size::Scriptsize => "scriptsize",
            Size::Footnotesize => "footnotesize",
            Size::Small => "small",
            Size::Normalsize => "normalsize",
            Size::Large => "large",
            Size::LLarge => "llarge",
            Size::LLLarge => "lllarge",
            Size::Huge => "huge",
            Size::HHuge => "hhuge",
        }
        .to_string()
    }
}

/// An enum representing different LaTeX styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bitcode::Encode, bitcode::Decode)]
pub enum Style {
    Normal,
    Bold,
    Italic,
    Slanted,
    // Underlined,
    SansSerif,
    BlackBoard,
    Calligraphic,
    Fraktur,
    Script,
    EuScript,
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Style::Normal => "textnormal",
            Style::Bold => "textbf",
            Style::Italic => "textit",
            Style::Slanted => "textsl",
            Style::SansSerif => "textsf",
            Style::BlackBoard => "mathbb",
            Style::Calligraphic => "mathcal",
            Style::Fraktur => "mathfrak",
            Style::Script => "mathscr",
            Style::EuScript => "EuScript",
        };
        write!(f, "{string}")
    }
}

impl Style {
    /// Create an iterator over normal style only
    pub fn basic() -> Vec<Vec<Style>> {
        vec![vec![Style::Normal]]
    }

    /// Create an iterator over text styles
    pub fn text() -> Vec<Vec<Style>> {
        vec![
            vec![Style::Normal],
            vec![Style::Bold],
            vec![Style::Italic],
            vec![Style::Slanted],
            vec![Style::Bold, Style::Italic],
            vec![Style::Bold, Style::Slanted],
            vec![Style::SansSerif],
        ]
    }

    /// Create an iterator over math styles
    pub fn math() -> Vec<Vec<Style>> {
        vec![
            vec![Style::BlackBoard],
            vec![Style::Calligraphic],
            vec![Style::Fraktur],
            vec![Style::Script],
            vec![Style::EuScript],
        ]
    }
}

type GlyphData = (String, Vec<Vec<Style>>, Vec<String>, bool);

/// A collection containing font glyphs sorted by their family and dimensions
pub struct FontBase {
    pub glyphs: HashMap<Code, HashMap<(u32, u32), Vec<KnownGlyph>>>,
}

impl FontBase {
    /// Create an empty FontBase
    pub fn new() -> FontBase {
        FontBase {
            glyphs: HashMap::new(),
        }
    }

    /// Create a FontBase based on the given arguments
    pub fn from(args: &Args) -> Result<FontBase> {
        // Create the font family if needed
        if let Some(codes) = &args.create {
            for &code in codes {
                Self::create_family(code, args)?;
            }
        }

        let now = time::Instant::now();
        if args.verbose {
            log("LOADING FONTS\n", None, None, "1m")?;
        }

        // Load each family into the FontBase
        let mut fontbase = FontBase::new();
        for code in Code::all() {
            fontbase.glyphs.insert(code, Self::load_family(code, args)?);
        }

        let duration = now.elapsed().as_secs_f32();
        if args.verbose {
            log("LOADED FONTS", None, Some(duration), "1m")?;
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(fontbase)
    }

    /// Get the glyphs stored for the given family and size
    fn get_family(code: Code, size: Size) -> Result<Vec<KnownGlyph>> {
        if let Ok(bit) = std::fs::read(format!("{}/{}", code.as_path(), size.as_path())) {
            let glyphs: Vec<KnownGlyph> = bitcode::decode(&bit)?;

            Ok(glyphs)
        } else {
            Ok(Vec::new())
        }
    }

    /// Create and store the glyphs for the given family
    fn create_family(code: Code, args: &Args) -> Result<()> {
        if args.verbose {
            log(&format!("CREATING FONT {code}\n"), None, None, "1m")?;
        }

        std::fs::create_dir_all("temp")?;

        // We use a thread scope to ensure that variables live long enough
        std::thread::scope(|scope| -> Result<()> {
            // Get the data for all symbols to render
            let symbols = Self::generate_symbols();
            let count = symbols.iter().fold(0, |acc, data| acc + data.1.len());

            // We create a different file for each size
            for size in Size::all() {
                if args.verbose {
                    log(&size.to_string(), Some(0.), None, "s")?;
                }

                std::fs::create_dir_all(code.as_path())?;

                // Try to retrieve already created glyphs
                let mut glyphs = Self::get_family(code, size)?;
                let mut id = glyphs.len();
                // Handles to store threads
                let mut handles = Vec::new();
                for (base, styles, modifiers, math) in symbols.clone() {
                    for style in styles.clone() {
                        let data = (base.clone(), code, size, style, modifiers.clone(), math);

                        // Don't recreate glyphs with the same data
                        if glyphs.iter().any(|g| g.get_data() == data) {
                            continue;
                        }

                        // Use a thread to create several glyphs concurrently
                        handles.push(scope.spawn(move || KnownGlyph::from(data, id)));

                        // Control the number of threads created
                        if handles.len() >= args.threads {
                            let glyph = handles.remove(0).join().unwrap()?;
                            glyphs.push(glyph);

                            // Save the glyphs
                            let bit = bitcode::encode(&glyphs)?;
                            std::fs::write(format!("{}/{}", code.as_path(), size.as_path()), bit)?;
                        }

                        if args.verbose {
                            let progress = id as f32 / count as f32;
                            log(&size.to_string(), Some(progress), None, "u")?;
                        }

                        id += 1;
                    }
                }

                // Join all threads
                for handle in handles {
                    let glyph = handle.join().unwrap()?;
                    glyphs.push(glyph);
                }

                // Save the glyphs
                let bit = bitcode::encode(&glyphs)?;
                std::fs::write(format!("{}/{}", code.as_path(), size.as_path()), bit)?;

                if args.verbose {
                    log(&size.to_string(), Some(1.), None, "u")?;
                    std::io::stdout().write_all(b"\n")?;
                }
            }

            std::fs::remove_dir_all("temp")?;

            Ok(())
        })?;

        if args.verbose {
            log(&format!("CREATED FONT {code}\n"), None, None, "1m")?;
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(())
    }

    /// Load the glyphs for a family sorted by dimensions
    fn load_family(code: Code, args: &Args) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
        if args.verbose {
            log(&format!("loading font {code}"), Some(0.), None, "s")?;
        }

        // Load each glyph into the family based on its dimensions
        let mut family = HashMap::new();
        for size in Size::all() {
            for glyph in Self::get_family(code, size)? {
                family
                    .entry((glyph.rect.width, glyph.rect.height))
                    .or_insert(Vec::new())
                    .push(glyph);
            }
        }

        if args.verbose {
            log(&format!("loading font {code}"), Some(1.), None, "u")?;
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(family)
    }

    /// Generate the data needed to create alphanumeric glyphs
    fn generate_alphanumeric() -> Vec<GlyphData> {
        let mut symbols = Vec::new();
        for chr in ALPHABET.chars() {
            symbols.extend_from_slice(&[
                (chr.to_lowercase().to_string(), Style::text(), vec![], false),
                (chr.to_uppercase().to_string(), Style::math(), vec![], true),
                (chr.to_uppercase().to_string(), Style::text(), vec![], false),
                (chr.to_lowercase().to_string(), Style::basic(), vec![], true),
                (chr.to_uppercase().to_string(), Style::basic(), vec![], true),
            ]);
        }

        for n in 0..10 {
            symbols.extend_from_slice(&[
                (n.to_string(), Style::text(), vec![], false),
                (format!("^{n}"), Style::basic(), vec![], true),
                (format!("_{n}"), Style::basic(), vec![], true),
            ]);
        }

        symbols
    }

    /// Generate the data needed to create punctuations glyphs
    fn generate_punctuations() -> Vec<GlyphData> {
        PUNCTUATIONS
            .lines()
            .map(|punct| (punct.to_string(), Style::text(), vec![], false))
            .collect()
    }

    /// Generate the data needed to create ligatures glyphs
    fn generate_ligatures() -> Vec<GlyphData> {
        LIGATURES
            .lines()
            .map(|lig| (lig.to_string(), Style::text(), vec![], false))
            .collect()
    }

    /// Generate the data needed to create accents glyphs
    fn generate_accents() -> Vec<GlyphData> {
        let mut symbols = Vec::new();
        for accent in ACCENTS.lines() {
            for chr in ALPHABET.chars() {
                symbols.extend_from_slice(&[
                    (
                        chr.to_lowercase().to_string(),
                        Style::text(),
                        vec![accent.to_string()],
                        false,
                    ),
                    (
                        chr.to_uppercase().to_string(),
                        Style::text(),
                        vec![accent.to_string()],
                        false,
                    ),
                ]);
            }
        }

        symbols
    }

    /// Generate the data needed to create greeks glyphs
    fn generate_greeks() -> Vec<GlyphData> {
        GREEKS
            .lines()
            .map(|greek| (greek.to_string(), Style::basic(), vec![], true))
            .collect()
    }

    /// Generate the data needed to create hebrews glyphs
    fn generate_hebrews() -> Vec<GlyphData> {
        HEBREWS
            .lines()
            .map(|hebrew| (hebrew.to_string(), Style::basic(), vec![], true))
            .collect()
    }

    /// Generate the data needed to create math constructs glyphs
    fn generate_constructs() -> Vec<GlyphData> {
        let mut symbols = Vec::new();
        for construct in CONSTRUCTS.lines() {
            for chr in ALPHABET.chars() {
                symbols.extend_from_slice(&[
                    (
                        chr.to_lowercase().to_string(),
                        Style::basic(),
                        vec![construct.to_string()],
                        true,
                    ),
                    (
                        chr.to_uppercase().to_string(),
                        Style::basic(),
                        vec![construct.to_string()],
                        true,
                    ),
                ]);
            }
        }

        symbols
    }

    /// Generate the data needed to create operations glyphs
    fn generate_operations() -> Vec<GlyphData> {
        OPERATIONS
            .lines()
            .map(|op| (op.to_string(), Style::basic(), vec![], true))
            .collect()
    }

    /// Generate the data needed to create arrows glyphs
    fn generate_arrows() -> Vec<GlyphData> {
        ARROWS
            .lines()
            .map(|arrow| (arrow.to_string(), Style::basic(), vec![], true))
            .collect()
    }

    /// Generate the data needed to create miscellaneous math glyphs
    fn generate_misc() -> Vec<GlyphData> {
        MISCELLANEOUS
            .lines()
            .map(|misc| (misc.to_string(), Style::basic(), vec![], true))
            .collect()
    }

    /// Generate the data needed to create math accents glyphs
    fn generate_math_accents() -> Vec<GlyphData> {
        let mut symbols = Vec::new();
        for accent in MATH_ACCENTS.lines() {
            for chr in ALPHABET.chars() {
                symbols.extend_from_slice(&[
                    (
                        chr.to_lowercase().to_string(),
                        Style::basic(),
                        vec![accent.to_string()],
                        true,
                    ),
                    (
                        chr.to_uppercase().to_string(),
                        Style::basic(),
                        vec![accent.to_string()],
                        true,
                    ),
                ]);
            }
        }

        symbols
    }

    /// Generate the data needed to create all glyphs
    fn generate_symbols() -> Vec<GlyphData> {
        let mut symbols = Vec::new();

        // Text
        symbols.extend(Self::generate_alphanumeric());
        symbols.extend(Self::generate_punctuations());
        symbols.extend(Self::generate_ligatures());
        symbols.extend(Self::generate_accents());

        // Math
        symbols.extend(Self::generate_greeks());
        symbols.extend(Self::generate_hebrews());
        symbols.extend(Self::generate_constructs());
        symbols.extend(Self::generate_operations());
        symbols.extend(Self::generate_arrows());
        symbols.extend(Self::generate_misc());
        symbols.extend(Self::generate_math_accents());

        symbols
    }
}
