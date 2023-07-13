use crate::args::Args;
use crate::glyph::KnownGlyph;
use crate::result::Result;
use crate::utils::log;
use clap::ValueEnum;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::time;

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, bitcode::Encode, bitcode::Decode)]
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

    pub fn as_path(&self) -> String {
        let config = dirs::config_dir().unwrap_or(PathBuf::from("~/.config"));
        format!("{}/pdf2latex/{self}", config.display())
    }
}

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

    pub fn as_path(&self) -> String {
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

    pub fn apply(&self, symbol: String) -> String {
        format!("\\{self}{{{symbol}}}")
    }
}

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
            Style::EuScript => "EuScript",
        };
        write!(f, "{string}")
    }
}

impl Style {
    pub fn all() -> Vec<Vec<Style>> {
        vec![
            vec![Style::Normal],
            vec![Style::Bold],
            vec![Style::Italic],
            vec![Style::Bold, Style::Italic],
            vec![Style::SansSerif],
            vec![Style::BlackBoard],
            vec![Style::Calligraphic],
            vec![Style::Fraktur],
            vec![Style::EuScript],
        ]
    }

    pub fn text() -> Vec<Vec<Style>> {
        vec![
            vec![Style::Normal],
            vec![Style::Bold],
            vec![Style::Italic],
            vec![Style::Bold, Style::Italic],
            vec![Style::Slanted],
            vec![Style::SansSerif],
        ]
    }

    pub fn math() -> Vec<Vec<Style>> {
        vec![
            vec![Style::BlackBoard],
            vec![Style::Calligraphic],
            vec![Style::Fraktur],
            vec![Style::EuScript],
        ]
    }

    pub fn is_math(&self) -> bool {
        Self::math().iter().any(|style| style.contains(self))
    }

    pub fn apply(&self, base: String) -> String {
        if self == &Self::Normal {
            base
        } else {
            match Self::math().iter().any(|style| style.contains(self)) {
                true => {
                    let base = base.replace("$", "");
                    format!("$\\{self}{{{base}}}$")
                }
                false => format!("\\{self}{{{base}}}"),
            }
        }
    }
}

pub struct FontBase {
    pub glyphs: HashMap<Code, HashMap<(u32, u32), Vec<KnownGlyph>>>,
}

impl FontBase {
    pub fn new(args: &Args) -> Result<FontBase> {
        let now = time::Instant::now();

        if args.verbose {
            log("LOADING FONTS\n", None, None, "1m")?;
        }

        if let Some(code) = args.create {
            Self::create_family(code, args)?;
        }

        let mut glyphs = HashMap::new();
        for code in Code::all() {
            glyphs.insert(code, Self::load_family(code, args)?);
        }

        let duration = now.elapsed().as_secs_f32();
        if args.verbose {
            log("LOADED FONTS", None, Some(duration), "1m")?;
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(FontBase { glyphs })
    }

    fn get_family(code: Code, size: Size) -> Result<Vec<KnownGlyph>> {
        if let Ok(bit) = std::fs::read(format!("{}/{}", code.as_path(), size.as_path())) {
            let glyphs: Vec<KnownGlyph> = bitcode::decode(&bit)?;

            Ok(glyphs)
        } else {
            Ok(Vec::new())
        }
    }

    fn create_family(code: Code, args: &Args) -> Result<()> {
        std::thread::scope(|scope| -> Result<()> {
            std::fs::create_dir_all("temp")?;

            if args.verbose {
                log(&format!("creating font {code}"), Some(0.), None, "s")?;
            }

            let mut id = 0;
            let (symbols, count) = Self::generate_symbols();
            for size in Size::all() {
                std::fs::create_dir_all(code.as_path())?;

                let mut glyphs = Self::get_family(code, size)?;

                for (i, glyph) in glyphs.clone().iter().enumerate().rev() {
                    if glyph.math {
                        glyphs.remove(i);
                    }
                }

                let mut handles = Vec::new();
                id += glyphs.len() as u32;
                for (base, styles, modifiers, math) in symbols.clone() {
                    for style in styles.clone() {
                        if glyphs.iter().any(|g| {
                            g.base == base
                                && g.size == size
                                && g.styles == style
                                && g.modifiers == modifiers
                                && g.math == math
                        }) {
                            continue;
                        }

                        let t_base = base.clone();
                        let t_modifiers = modifiers.clone();
                        handles.push(scope.spawn(move || {
                            KnownGlyph::from(&t_base, code, size, style, t_modifiers, math, id)
                        }));

                        if handles.len() >= 3 {
                            let glyph = handles.remove(0).join().unwrap()?;
                            glyphs.push(glyph);

                            let bit = bitcode::encode(&glyphs)?;
                            std::fs::write(format!("{}/{}", code.as_path(), size.as_path()), bit)?;
                        }

                        if args.verbose {
                            log(
                                &format!("creating font {code}"),
                                Some(id as f32 / count as f32),
                                None,
                                "u",
                            )?;
                        }
                        id += 1;
                    }
                }

                for handle in handles {
                    let glyph = handle.join().unwrap()?;
                    glyphs.push(glyph);
                }

                let bit = bitcode::encode(&glyphs)?;
                std::fs::write(format!("{}/{}", code.as_path(), size.as_path()), bit)?;
            }

            if args.verbose {
                log(&format!("created font {code}"), Some(1.), None, "u")?;
                std::io::stdout().write_all(b"\n")?;
            }

            std::fs::remove_dir_all("temp")?;

            Ok(())
        })?;

        Ok(())
    }

    fn load_family(code: Code, args: &Args) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
        if args.verbose {
            log(&format!("loading font {code}"), Some(0.), None, "s")?;
        }

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
            log(&format!("loaded font {code}"), Some(1.), None, "u")?;
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(family)
    }

    fn generate_alphanumeric() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        let mut symbols = Vec::new();
        for chr in ALPHABET.chars() {
            symbols.push((chr.to_lowercase().to_string(), Style::text(), vec![], false));
            symbols.push((chr.to_uppercase().to_string(), Style::all(), vec![], false));
        }
        for n in 0..10 {
            symbols.push((n.to_string(), Style::text(), vec![], false));
        }

        symbols
    }

    fn generate_accents() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        let mut symbols = Vec::new();
        for accent in include_str!("data/accents.txt").lines() {
            for chr in ALPHABET.chars() {
                symbols.push((
                    chr.to_lowercase().to_string(),
                    Style::text(),
                    vec![accent.to_string()],
                    false,
                ));
                symbols.push((
                    chr.to_uppercase().to_string(),
                    Style::text(),
                    vec![accent.to_string()],
                    false,
                ));
            }
        }

        symbols
    }

    fn generate_math_accents() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        let mut symbols = Vec::new();
        for accent in include_str!("data/math_accents.txt").lines() {
            for chr in ALPHABET.chars() {
                symbols.push((
                    chr.to_lowercase().to_string(),
                    vec![vec![Style::Normal]],
                    vec![accent.to_string()],
                    true,
                ));
                symbols.push((
                    chr.to_uppercase().to_string(),
                    vec![vec![Style::Normal]],
                    vec![accent.to_string()],
                    true,
                ));
            }
        }

        symbols
    }

    fn generate_punctuations() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        include_str!("data/punctuations.txt")
            .lines()
            .map(|punct| (punct.to_string(), Style::text(), vec![], false))
            .collect()
    }

    fn generate_ligatures() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        include_str!("data/ligatures.txt")
            .lines()
            .map(|lig| (lig.to_string(), Style::text(), vec![], false))
            .collect()
    }

    fn generate_greeks() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        include_str!("data/greeks.txt")
            .lines()
            .map(|greek| (greek.to_string(), vec![vec![Style::Normal]], vec![], true))
            .collect()
    }

    fn generate_constructs() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        let mut symbols = Vec::new();
        for construct in include_str!("data/constructs.txt").lines() {
            for chr in ALPHABET.chars() {
                symbols.push((
                    chr.to_lowercase().to_string(),
                    vec![vec![Style::Normal]],
                    vec![construct.to_string()],
                    true,
                ));
                symbols.push((
                    chr.to_uppercase().to_string(),
                    vec![vec![Style::Normal]],
                    vec![construct.to_string()],
                    true,
                ));
            }
        }

        symbols
    }

    fn generate_arrows() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        include_str!("data/arrows.txt")
            .lines()
            .map(|arrow| (arrow.to_string(), vec![vec![Style::Normal]], vec![], true))
            .collect()
    }

    fn generate_operations() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        include_str!("data/operations.txt")
            .lines()
            .map(|op| (op.to_string(), vec![vec![Style::Normal]], vec![], true))
            .collect()
    }

    fn generate_misc() -> Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)> {
        include_str!("data/miscellaneous.txt")
            .lines()
            .map(|misc| (misc.to_string(), vec![vec![Style::Normal]], vec![], true))
            .collect()
    }

    fn generate_symbols() -> (Vec<(String, Vec<Vec<Style>>, Vec<String>, bool)>, usize) {
        let mut symbols = Vec::new();

        symbols.extend(Self::generate_alphanumeric());
        symbols.extend(Self::generate_punctuations());
        symbols.extend(Self::generate_ligatures());
        symbols.extend(Self::generate_accents());

        symbols.extend(Self::generate_greeks());
        symbols.extend(Self::generate_constructs());
        symbols.extend(Self::generate_operations());
        symbols.extend(Self::generate_arrows());
        symbols.extend(Self::generate_misc());
        symbols.extend(Self::generate_math_accents());

        let count = symbols.iter().map(|d| d.1.len() * Size::all().len()).sum();
        (symbols, count)
    }
}
