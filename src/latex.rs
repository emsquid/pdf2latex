use crate::pdf::Pdf;
use crate::result::Result;
use crate::text::Line;
use crate::utils::round;
use std::path::PathBuf;

pub struct Latex {
    pub content: String,
}

impl Latex {
    pub fn from(pdf: &Pdf) -> Latex {
        let margin = pdf.get_margin();

        let mut content = "\\documentclass{article}".to_owned()
            + "\n\\author{pdf2latex}"
            + "\n\\usepackage[margin="
            + &(round(margin, 1)).to_string()
            + "in]{geometry}"
            + "\n\\usepackage{amsmath, amssymb, amsthm}"
            + "\n\\usepackage{euscript, mathrsfs}"
            + "\n\\begin{document}";

        for page in &pdf.pages {
            for (i, line) in page.lines.iter().enumerate() {
                let prev = page.lines.get(i - 1).and_then(Line::get_last_guess);
                let next = page.lines.get(i + 1).and_then(Line::get_first_guess);

                content.push_str("\n    ");
                content.push_str(&line.get_latex(&prev, &next));
            }
        }

        content.push_str("\n\\end{document}");

        Latex { content }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        std::fs::write(path, &self.content)?;

        Ok(())
    }
}
