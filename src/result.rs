#[derive(Debug)]
pub enum Error {
    /// Input/Output error
    Io(std::io::Error),
    /// Parsing error
    Parsing(std::num::ParseIntError),
    /// Image error
    Image(image::error::ImageError),
    /// Font error
    Font(ab_glyph::InvalidFont),
    /// Serde error
    Serde(serde_json::Error),
    /// Custom error
    Custom(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "Io error: {err}"),
            Error::Parsing(err) => write!(f, "Parsing error: {err}"),
            Error::Image(err) => write!(f, "Image error: {err}"),
            Error::Font(err) => write!(f, "Font error: {err}"),
            Error::Serde(err) => write!(f, "Serde error: {err}"),
            Error::Custom(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::Parsing(err)
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::Image(err)
    }
}

impl From<ab_glyph::InvalidFont> for Error {
    fn from(err: ab_glyph::InvalidFont) -> Self {
        Error::Font(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde(err)
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;
