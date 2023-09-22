use std::path::PathBuf;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum, bitcode::Encode, bitcode::Decode,
)]
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
