use super::Page;
use crate::args::MainArg;
use crate::fonts::FontBase;
use crate::utils::{log, pdf_pages_number, pdf_to_images};
use anyhow::Result;
use std::io::Write;

/// A Pdf document represented as multiple pages
#[derive(Default)]
pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    /// Load a Pdf from the given path
    /// Guess the content of a Pdf
    ///
    /// # Errors
    /// Fails if cannot convert the PDF into an image
    /// Fails if cannot write into stdout or log
    pub fn guess(&mut self, args: &MainArg) -> Result<()> {
        // The FontBase is needed to compare glyphs
        let fontbase = FontBase::try_from(args)?;
        let nb_pages = pdf_pages_number(&args.input)?;
        self.pages = Vec::with_capacity(nb_pages);

        for i in 0..nb_pages {
            self.pages.push(
                pdf_to_images(&args.input, Some(&[i + 1]))?
                    .get(0)
                    .map(Page::from)
                    .unwrap(),
            );
            let page = self.pages.get_mut(i).unwrap();
            if args.verbose {
                log(&format!("\nPAGE {i}\n"), None, None, "1m")?;
            }

            page.guess(&fontbase, args)?;
        }
        self.clean(args)?;

        if args.verbose {
            std::io::stdout().write_all(b"\n")?;
        }

        Ok(())
    }

    /// Compute the overall margin of a Pdf
    /// TODO: Change this to something better, maybe return (f32, f32, f32, f32)
    #[must_use]
    pub fn get_margin(&self) -> f32 {
        let mut i = 0;
        self.pages
            .iter()
            .map(|page| {
                page.lines
                    .iter()
                    .flat_map(|line| {
                        i += 1;
                        line.words.get(0).map(|word| word.rect.x)
                    })
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

    /// Get the `LateX` of a Pdf
    pub fn get_latex(&self) -> String {
        self.pages.iter().map(Page::get_latex).collect()
    }

    pub fn clean(&mut self, args: &MainArg) -> Result<()> {
        for page in self.pages.iter_mut() {
            page.clean(args)?;
        }
        Ok(())
    }
}
