use crate::font::Style;
use crate::utils::round;
use crate::{font::Size, pdf::Pdf, result::Result};
use std::path::PathBuf;

pub struct Latex {
    pub content: String,
}

impl Latex {
    pub fn from(pdf: &Pdf) -> Latex {
        let margin = pdf.get_margin();

        let mut content = String::from(
            "\\documentclass{article}".to_owned()
                + "\n\\author{pdf2latex}"
                + "\n\\usepackage[margin="
                + &(round(margin, 1)).to_string()
                + "in]{geometry}"
                + "\n\\usepackage{{amsmath, amssymb, amsthm}}"
                + "\n\\usepackage{{euscript}}"
                + "\n\\begin{document}",
        );

        for page in &pdf.pages {
            let mut init = true;
            let mut math = false;
            let mut current_size = Size::Normalsize;
            let mut current_styles = Vec::new();

            for line in &page.lines {
                content.push_str("\n    ");
                content.push_str(&line.get_latex(
                    &mut current_size,
                    &mut current_styles,
                    &mut math,
                    &mut init,
                ));
            }

            for style in current_styles {
                if style.is_math() {
                    content.push_str("}$");
                } else if style != Style::Normal {
                    content.push_str("}");
                }
            }

            if math {
                content.push('$');
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
