use crate::{pdf::Pdf, result::Result, font::Size};
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
        for page in &self.pdf.pages {
            let mut init = true;
            let mut math = true;
            let mut current_size = Size::Normalsize;
            let mut current_styles = Vec::new();
            for line in &page.lines {
                content.push_str("\n    ");

                content.push_str(&line.get_latex(&mut current_size, &mut current_styles, &mut math, &mut init));
            }
        }
        content.push_str("\n\\end{document}");

        println!("{content}");

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
