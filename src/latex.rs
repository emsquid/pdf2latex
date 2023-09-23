use crate::pdf::Pdf;
use crate::utils::round;
use anyhow::Result;
use std::path::PathBuf;

/// A LaTeX document represented in a String
pub struct LaTeX {
    pub content: String,
}

impl LaTeX {
    /// Create a LaTeX document from a PDF
    #[must_use]
    pub fn from(pdf: &Pdf) -> LaTeX {
        let margin = pdf.get_margin();

        let content = "\\documentclass{article}".to_owned()
            + "\n\\author{pdf2latex}"
            + "\n\\usepackage[margin="
            + &(round(margin, 1)).to_string()
            + "in]{geometry}"
            + "\n\\usepackage{amsmath, amssymb, amsthm}"
            + "\n\\usepackage{euscript, mathrsfs}"
            + "\n\\begin{document}"
            + &pdf.get_latex()
            + "\n\\end{document}";

        LaTeX { content }
    }

    /// Save a LaTeX document at a given path
    ///
    /// # Errors
    /// Fails if cannot write into the file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        Ok(std::fs::write(path, &self.content)?)
    }
}
