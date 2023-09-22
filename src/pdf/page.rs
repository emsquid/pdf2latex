use crate::args::MainArg;
use crate::fonts::glyph::Glyph;
use crate::fonts::FontBase;
use crate::pdf::Line;
use crate::utils::{find_parts, log, most_frequent, Rect};
use anyhow::Result;
use image::{imageops::overlay, DynamicImage, GenericImage, Rgba};
use std::{io::Write, time};

const LINE_SPACING: u32 = 10;

/// A Page from a Pdf, it holds an image and multiple lines
pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    /// Create a Page from an image
    #[must_use]
    pub fn from(image: &DynamicImage) -> Page {
        Page {
            image: image.clone(),
            lines: Page::find_lines(image),
        }
    }

    /// Find the different lines in an image
    fn find_lines(image: &DynamicImage) -> Vec<Line> {
        find_parts(&image.to_luma8(), LINE_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(0, start, image.width(), end - start + 1);
                Line::from(rect, image)
            })
            .collect()
    }

    /// Guess the content of a Page
    ///
    /// # Errors
    /// Fails if cannot log or cannot write into stdout
    /// # Panics
    /// Fails if cannot join correctly the threads
    pub fn guess(&mut self, fontbase: &FontBase, args: &MainArg) -> Result<()> {
        // We use a thread scope to ensure that variables live long enough
        std::thread::scope(|scope| -> Result<()> {
            let now = time::Instant::now();
            let mut progress = 0.;
            let step = 1. / self.lines.len() as f32;
            if args.verbose {
                log("converting text", Some(0.), None, "s")?;
            }

            // Handles to store threads
            let mut handles = Vec::new();
            for line in &mut self.lines {
                // Use a thread to guess the content of several lines concurrently
                let handle = scope.spawn(move || line.guess(fontbase));
                handles.push(handle);

                // Control the number of threads created
                if handles.len() >= args.threads {
                    handles.remove(0).join().unwrap();
                }

                progress += step;
                if args.verbose {
                    log("converting text", Some(progress), None, "u")?;
                }
            }

            // Join all threads
            for handle in handles {
                handle.join().unwrap();
            }

            let duration = now.elapsed().as_secs_f32();
            if args.verbose {
                log("converting text", Some(1.), Some(duration), "u")?;
                std::io::stdout().write_all(b"\n")?;
            }

            Ok(())
        })
    }

    /// Get the content of a Page, mostly for debugging
    pub fn get_content(&self) -> String {
        self.lines
            .iter()
            .map(Line::get_content)
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Get the LaTeX for a Page
    pub fn get_latex(&self) -> String {
        let right_margins = self
            .lines
            .iter()
            .filter_map(Line::get_right_margin)
            .collect::<Vec<u32>>();
        let right_margin_mode = most_frequent(&right_margins, 0).0;

        self.lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let prev = self.lines.get(i - 1).and_then(Line::get_last_guess);
                let next = self.lines.get(i + 1).and_then(Line::get_first_guess);
                let newline = if line
                    .get_right_margin()
                    .is_some_and(|margin| margin < right_margin_mode - 10)
                {
                    "\n"
                } else {
                    ""
                };
                format!("\n    {}{}", line.get_latex(&prev, &next), newline)
            })
            .collect()
    }

    /// Show the guess on the Page's image, mostly for debugging
    pub fn debug_image(&self) -> DynamicImage {
        // idk wtf is going on here, ask Noe
        let mut copy = self.image.clone();
        let mut alt = 0;
        for line in &self.lines {
            let sub = image::RgbaImage::from_pixel(line.rect.width, 1, Rgba([255, 0, 255, 255]));

            overlay(
                &mut copy,
                &sub,
                i64::from(line.rect.x),
                i64::from(line.baseline),
            );

            for word in &line.words {
                for glyph in &word.glyphs {
                    alt = (alt + 1) % 4;
                    let color = match alt {
                        0 => Rgba([255, 0, 0, 255]),
                        1 => Rgba([0, 255, 0, 255]),
                        2 => Rgba([0, 0, 255, 255]),
                        3 => Rgba([255, 255, 0, 255]),
                        _ => Rgba([0, 255, 255, 255]),
                    };
                    let sub = image::RgbaImage::from_pixel(glyph.rect.width, 2, color);

                    if let Some(guess) = &glyph.guess {
                        for x in 0..guess.rect.width {
                            for y in 0..guess.rect.height {
                                if guess.get_pixel(x, y) < 0.9 {
                                    let v = (255. * guess.get_pixel(x, y)) as u8;
                                    let c = match alt {
                                        0 => Rgba([255, v, v, 255]),
                                        1 => Rgba([v, 255, v, 255]),
                                        2 => Rgba([v, v, 255, 255]),
                                        3 => Rgba([255, 255, v, 255]),
                                        _ => Rgba([v, 255, 255, 255]),
                                    };
                                    copy.put_pixel(
                                        glyph.rect.x + x,
                                        (line.baseline + y - glyph.rect.height)
                                            .saturating_add_signed(guess.offset),
                                        c,
                                    );
                                }
                            }
                        }
                    }

                    overlay(
                        &mut copy,
                        &sub,
                        i64::from(glyph.rect.x),
                        i64::from(line.rect.y + line.rect.height + 2),
                    );
                }
                let sub =
                    image::RgbaImage::from_pixel(word.rect.width, 2, Rgba([255, 100, 100, 255]));

                overlay(
                    &mut copy,
                    &sub,
                    i64::from(word.rect.x),
                    i64::from(line.rect.y + line.rect.height + 4),
                );
            }
        }

        copy
    }

    /// Compute the average distance between glyphs and their guesses, mostly for debugging
    pub fn debug_dist_avg(&self) {
        let data = self.lines.iter().fold((0., 0), |acc, line| {
            (acc.0 + line.get_dist_sum(), acc.1 + line.get_glyph_count())
        });
        println!("distance moyenne : {}", data.0 / data.1 as f32);
    }
}
