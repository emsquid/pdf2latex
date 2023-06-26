use crate::{pdf::{Pdf, Page}, result::Result};
use crate::dictionary::Dictionary;
// use latex::{Document, DocumentClass};
use std::{fs::File, io::Write};

pub struct Latex {
    // pub document: Document,
    pub pdf: Pdf,
}

impl Latex {
    pub fn from(pdf: Pdf) -> Latex {
        // let document = Document::new(DocumentClass::Article);
        // Latex { document }

        Latex { pdf }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        // let mut file = File::create(path)?;
        // let content = latex::print(&self.document).unwrap();
        // file.write_all(content.as_bytes())?;

        // Ok(())


        let mut file = File::create(path)?;
        let dictionary = Dictionary::new()?;
        let margin = self.pdf.pages.iter().map(|p| p.lines.iter().map(|l| l.words[0].rect.x).min().unwrap_or(300)).min().unwrap_or(300);

        let mut content = String::from(
            "\\documentclass{article}".to_owned() +
            "\n\\title{LE TITRE !!!}" +
            "\n\\author{pdf2latex}" +
            "\n\\date{}" +
            "\n\\usepackage{geometry}" +
            "\n\\geometry{margin=" + (margin * 96 / 300).to_string().as_str() + "in, top=0.7in}" +
            "\n\\usepackage{amsmath}" +
            "\n\\begin{document}");
        for page in &self.pdf.pages { for line in &page.lines {
            content.push_str("\n    ");

            content.push_str(&String::from_iter(
                line.get_content(&dictionary)
                .char_indices()
                .map(|c| if c.1.is_ascii() { c.1 } else { '?' })));
        }}
        content.push_str("\n\\end{document}");
            
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
