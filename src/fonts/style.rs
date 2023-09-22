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
    #[must_use]
    pub fn basic() -> Vec<Vec<Style>> {
        vec![vec![Style::Normal]]
    }

    /// Create an iterator over text styles
    #[must_use]
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
    #[must_use]
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
