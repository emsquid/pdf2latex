use crate::args::Args;
use crate::glyph::KnownGlyph;
use crate::result::Result;
use crate::utils::log;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::time;

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
const PUNCTUATIONS: &[&str] = &[
    ".", ",", ";", ":", "!", "?", "'", "\"", "-", "--", "---", "(", ")", "\\{", "\\}", "[", "]",
];
const LIGATURES: &[&str] = &["ff", "fi", "fl", "ffi", "ffl", "ae"];
const ACCENTS: &[&str] = &["overline", "vec", "overrightarrow", "widehat", "widetilde"];
const GREEKS: &[&str] = &[
    "alpha",
    "beta",
    "gamma",
    "Gamma",
    "delta",
    "Delta",
    "epsilon",
    "varepsilon",
    "zeta",
    "eta",
    "theta",
    "vartheta",
    "Theta",
    "iota",
    "kappa",
    "lambda",
    "Lambda",
    "mu",
    "nu",
    "xi",
    "Xi",
    "pi",
    "Pi",
    "rho",
    "varrho",
    "sigma",
    "Sigma",
    "tau",
    "upsilon",
    "Upsilon",
    "phi",
    "varphi",
    "Phi",
    "chi",
    "psi",
    "Psi",
    "omega",
    "Omega",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
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
            // Code::Qcs,
            Code::Qpl,
        ]
    }

    pub fn as_path(&self) -> String {
        format!("fonts/{self}.json")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    pub fn apply(&self, symbol: String) -> String {
        format!("\\{self}{{{symbol}}}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn all() -> Vec<Style> {
        vec![
            Style::Normal,
            Style::Bold,
            Style::Italic,
            Style::Slanted,
            Style::SansSerif,
            Style::BlackBoard,
            Style::Calligraphic,
            Style::Fraktur,
            Style::EuScript,
        ]
    }

    pub fn text() -> Vec<Style> {
        vec![
            Style::Normal,
            Style::Bold,
            Style::Italic,
            Style::Slanted,
            Style::SansSerif,
        ]
    }

    pub fn math() -> Vec<Style> {
        vec![
            Style::BlackBoard,
            Style::Calligraphic,
            Style::Fraktur,
            Style::EuScript,
        ]
    }

    pub fn apply(&self, base: String) -> String {
        match Self::math().contains(self) {
            true => {
                let base = base.replace("$", "");
                format!("$\\{self}{{{base}}}$")
            }
            false => format!("\\{self}{{{base}}}"),
        }
    }
}

pub struct FontBase {
    pub glyphs: HashMap<Code, HashMap<(u32, u32), Vec<KnownGlyph>>>,
}

impl FontBase {
    pub fn new(args: &Args) -> Result<FontBase> {
        let now = time::Instant::now();

        if !args.silent {
            log("LOADING FONTS\n", None, None, "1m")?;
        }

        if let Some(codes) = &args.create {
            for code in codes {
                Self::create_family(*code)?;
            }
        }

        let mut glyphs = HashMap::new();
        for code in Code::all() {
            glyphs.insert(code, Self::load_family(code)?);
        }

        let duration = now.elapsed().as_secs_f32();
        if !args.silent {
            log("LOADED FONTS", None, Some(duration), "1m")?;
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(FontBase { glyphs })
    }

    fn get_family(code: Code) -> Result<Vec<KnownGlyph>> {
        if let Ok(json) = std::fs::read_to_string(code.as_path()) {
            let glyphs: Vec<KnownGlyph> = serde_json::from_str(&json)?;

            Ok(glyphs)
        } else {
            Ok(Vec::new())
        }
    }

    fn create_family(code: Code) -> Result<()> {
        std::thread::scope(|scope| -> Result<()> {
            std::fs::create_dir_all("temp")?;

            log(&format!("creating font {code}"), Some(0.), None, "s")?;

            let mut id = 0;
            let mut handles = Vec::new();
            let mut glyphs = Self::get_family(code)?;
            let (symbols, count) = Self::generate_symbols();
            for (base, sizes, styles, modifiers, math) in symbols {
                for style in styles.clone() {
                    for size in sizes.clone() {
                        if glyphs.iter().any(|g| {
                            g.base == base
                                && g.size == size
                                && g.style == style
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

                        if handles.len() >= 8 {
                            let glyph = handles.remove(0).join().unwrap()?;
                            glyphs.push(glyph);
                        }

                        log(
                            &format!("creating font {code}"),
                            Some(id as f32 / count as f32),
                            None,
                            "u",
                        )?;
                        id += 1;
                    }
                }
            }

            for handle in handles {
                let glyph = handle.join().unwrap()?;
                glyphs.push(glyph);
            }

            log(&format!("created font {code}"), Some(1.), None, "u")?;
            std::io::stdout().write_all(b"\n")?;

            let json = serde_json::to_string(&glyphs)?;
            std::fs::write(code.as_path(), json)?;
            std::fs::remove_dir_all("temp")?;

            Ok(())
        })?;

        Ok(())
    }

    fn load_family(code: Code) -> Result<HashMap<(u32, u32), Vec<KnownGlyph>>> {
        log(&format!("loading font {code}"), Some(0.), None, "s")?;

        let mut family = HashMap::new();
        for glyph in Self::get_family(code)? {
            family
                .entry((glyph.rect.width, glyph.rect.height))
                .or_insert(Vec::new())
                .push(glyph);
        }

        log(&format!("loaded font {code}"), Some(1.), None, "u")?;
        std::io::stdout().write_all(b"\n")?;

        Ok(family)
    }

    fn generate_alphanumeric() -> Vec<(String, Vec<Size>, Vec<Style>, Vec<String>, bool)> {
        let mut symbols = Vec::new();
        for chr in ALPHABET.chars() {
            symbols.push((
                chr.to_lowercase().to_string(),
                Size::all(),
                Style::text(),
                vec![],
                false,
            ));
            symbols.push((
                chr.to_uppercase().to_string(),
                Size::all(),
                Style::all(),
                vec![],
                false,
            ));
        }
        for n in 0..10 {
            symbols.push((n.to_string(), Size::all(), Style::text(), vec![], false));
        }

        symbols
    }

    fn generate_accents() -> Vec<(String, Vec<Size>, Vec<Style>, Vec<String>, bool)> {
        let mut symbols = Vec::new();
        for accent in ACCENTS {
            for chr in ALPHABET.chars() {
                symbols.push((
                    chr.to_lowercase().to_string(),
                    Size::all(),
                    Style::text(),
                    vec![accent.to_string()],
                    true,
                ));
                symbols.push((
                    chr.to_uppercase().to_string(),
                    Size::all(),
                    Style::all(),
                    vec![accent.to_string()],
                    true,
                ));
            }
        }

        symbols
    }

    fn generate_ligatures() -> Vec<(String, Vec<Size>, Vec<Style>, Vec<String>, bool)> {
        LIGATURES
            .into_iter()
            .map(|lig| (lig.to_string(), Size::all(), Style::text(), vec![], false))
            .collect()
    }

    fn generate_punctuations() -> Vec<(String, Vec<Size>, Vec<Style>, Vec<String>, bool)> {
        PUNCTUATIONS
            .into_iter()
            .map(|punct| (punct.to_string(), Size::all(), Style::text(), vec![], false))
            .collect()
    }

    fn generate_greek() -> Vec<(String, Vec<Size>, Vec<Style>, Vec<String>, bool)> {
        GREEKS
            .into_iter()
            .map(|greek| {
                (
                    format!("\\{}", greek),
                    Size::all(),
                    vec![Style::Normal],
                    vec![],
                    true,
                )
            })
            .collect()
    }

    fn generate_symbols() -> (
        Vec<(String, Vec<Size>, Vec<Style>, Vec<String>, bool)>,
        usize,
    ) {
        let mut symbols = Vec::new();

        symbols.extend(Self::generate_alphanumeric());
        symbols.extend(Self::generate_ligatures());
        symbols.extend(Self::generate_punctuations());
        symbols.extend(Self::generate_accents());
        symbols.extend(Self::generate_greek());

        let count = symbols.iter().map(|d| d.1.len() * d.2.len()).sum();
        (symbols, count)
    }
}
