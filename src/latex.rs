use crate::{pdf::Pdf, result::Result};
use latex::{Document, DocumentClass};
use std::{fs::File, io::Write};

pub struct Latex {
    pub document: Document,
}

impl Latex {
    pub fn from(pdf: &Pdf) -> Latex {
        let document = Document::new(DocumentClass::Article);

        Latex { document }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        let content = latex::print(&self.document).unwrap();
        file.write(content.as_bytes())?;

        Ok(())
    }
}
