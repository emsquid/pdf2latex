use crate::{pdf::Pdf, result::Result};
use std::{fs::File, io::Write};
use crate::utils::round;

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
                + &(round(margin as f32 /512.,2)).to_string()
                + "in, top="
                // + &(self.pdf.pages[0].lines[0].rect.y).to_string()
                + "0.7in}"
                + "\n\\usepackage{amsmath}"
                + "\n\\begin{document}",
        );
        content.push_str(&self.pdf.get_content());
        content.push_str("\n\\end{document}");

        println!("{content}");

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
