pub mod args;
pub mod latex;
pub mod utils;

pub mod pdf {
    pub mod line;
    pub mod matrix;
    pub mod page;
    pub mod pdf;
    pub mod word;
    // Reexport struct
    pub use line::Line;
    pub use matrix::Matrix;
    pub use page::Page;
    pub use pdf::Pdf;
    pub use word::Word;
}

pub mod fonts {
    pub mod code;
    pub mod fonts;
    pub mod glyph;
    pub mod size;
    pub mod style;
    // Reexport struct
    pub use code::Code;
    pub use fonts::FontBase;
    pub use glyph::{
        Glyph, KnownGlyph, UnknownGlyph, CHAR_THRESHOLD, DIST_THRESHOLD, DIST_UNALIGNED_THRESHOLD,
    };
    pub use size::Size;
    pub use style::Style;
}

pub mod vit {
    pub mod model;
    // Reexport struct
    pub use model::Model;
}
