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
    #[must_use]
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
    #[must_use]
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
