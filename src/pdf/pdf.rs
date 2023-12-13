use super::Page;
use crate::args::MainArg;
use crate::fonts::FontBase;
use crate::utils::{log, pdf_pages_number, pdf_to_images};
use anyhow::{anyhow, Ok, Result};
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
        let mut indexes: Vec<usize> = Vec::new();
        let nb_pages = pdf_pages_number(&args.input)?;
        if let Some(pages_number) = &args.pages {
            pages_number.split(",").for_each(|s| {
                let a = s
                    .split("-")
                    .map(|v| v.trim().parse::<usize>().unwrap())
                    .collect::<Vec<usize>>();
                indexes.extend_from_slice(&match a.len() {
                    1 => a,
                    2 => (a[0]..=a[1]).collect(),
                    _ => panic!("error"),
                });
            });
            indexes.sort();
            indexes.dedup();
            if indexes
                .last()
                .is_some_and(|page_number| page_number > &nb_pages)
            {
                return Err(anyhow!("Error page number: you provided the {} page however the PDF contains {nb_pages} pages", indexes.last().unwrap()));
            }
        } else {
            indexes.extend_from_slice(&(0..nb_pages).collect::<Vec<usize>>());
        }

        // The FontBase is needed to compare glyphs
        let fontbase = FontBase::try_from(args)?;
        self.pages = Vec::with_capacity(indexes.len());

        for i in indexes {
            if args.verbose {
                log(&format!("\nPAGE {i}\n"), None, None, "1m")?;
            }

            self.pages.push(
                pdf_to_images(&args.input, Some(&[i]))?
                    .get(0)
                    .map(|v| Page::from(v, None))
                    .unwrap(),
            );
            let page = self.pages.last_mut().unwrap();

            page.guess(&fontbase, args)?;
        }
        self.verify(args, &fontbase)?;

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
                        line.words.get(0).map(|word| word.rect().x)
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

    pub fn verify(&mut self, args: &MainArg, fontbase: &FontBase) -> Result<()> {
        for page in self.pages.iter_mut() {
            page.verify(args, fontbase)?;
        }
        Ok(())
    }
}
