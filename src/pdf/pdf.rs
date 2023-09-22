use super::Page;
use crate::args::Args;
use crate::fonts::FontBase;
use crate::utils::{log, pdf_to_images};
use anyhow::Result;
use std::{io::Write, path::Path};

/// A Pdf document represented as multiple pages
pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    /// Load a Pdf from the given path
    pub fn load(path: &Path) -> Result<Pdf> {
        let pages = pdf_to_images(path)?.iter().map(Page::from).collect();

        Ok(Pdf { pages })
    }

    /// Guess the content of a Pdf
    pub fn guess(&mut self, args: &Args) -> Result<()> {
        // The FontBase is needed to compare glyphs
        let fontbase = FontBase::try_from(args)?;

        for (i, page) in self.pages.iter_mut().enumerate() {
            if args.verbose() {
                log(&format!("\nPAGE {i}\n"), None, None, "1m")?;
            }

            page.guess(&fontbase, args)?;
        }

        if args.verbose() {
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(())
    }

    /// Compute the overall margin of a Pdf
    pub fn get_margin(&self) -> f32 {
        self.pages
            .iter()
            .map(|page| {
                page.lines
                    .iter()
                    .map(|line| line.words[0].rect.x)
                    .min()
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0) as f32
            / 512.
    }

    /// Get the content of a Pdf, mostly for debugging
    pub fn get_content(&self) -> String {
        self.pages
            .iter()
            .map(Page::get_content)
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Get the LateX of a Pdf
    pub fn get_latex(&self) -> String {
        self.pages.iter().map(Page::get_latex).collect()
    }
}
