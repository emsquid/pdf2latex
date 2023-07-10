use crate::{pdf::Pdf, result::Result};
use std::{fs::File, io::Write};

pub struct Latex {
    pub pdf: Pdf,
}

impl Latex {
    pub fn from(pdf: Pdf) -> Latex {
        Latex { pdf }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        let margin = self
            .pdf
            .pages
            .iter()
            .map(|p| {
                p.lines
                    .iter()
                    .map(|l| l.words[0].rect.x)
                    .min()
                    .unwrap_or(300)
            })
            .min()
            .unwrap_or(300);

        let mut content = String::from(
            "\\documentclass{article}".to_owned()
                + "\n\\author{pdf2latex}"
                + "\n\\date{}"
                + "\n\\usepackage{geometry}"
                + "\n\\geometry{margin="
                + (margin * 96 / 300).to_string().as_str()
                + "in, top="
                + (self.pdf.pages[0].lines[0].rect.y).to_string().as_str()
                + "0.7in}"
                + "\n\\usepackage{amsmath}"
                + "\n\\begin{document}",
        );
        for page in &self.pdf.pages {
            for line in &page.lines {
                content.push_str("\n    ");

                content.push_str(&String::from_iter(line.get_content().char_indices().map(
                    |c| {
                        if c.1.is_ascii() {
                            c.1
                        } else {
                            '?'
                        }
                    },
                )));
            }
        }
        content.push_str("\n\\end{document}");

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
