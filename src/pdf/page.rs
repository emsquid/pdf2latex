use crate::args::MainArg;
use crate::fonts::glyph::Glyph;
use crate::fonts::FontBase;
use crate::pdf::Line;
use crate::utils::{find_parts, log, most_frequent, Rect};
use crate::vit::Model;
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
            .map(|line| line.get_content())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Get the LaTeX for a Page
    pub fn get_latex(&self) -> String {
        let right_margin_mode = self.get_right_margin_mode();
        let left_margin_mode = self.get_left_margin_mode();
        self.lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let prev = self.lines.get(i - 1).and_then(Line::get_last_guess);
                let next = self.lines.get(i + 1).and_then(Line::get_first_guess);
                let newline = if line
                    .get_right_margin()
                    .is_some_and(|margin| margin < right_margin_mode - 10)
                    && line.can_have_new_line
                {
                    if self.lines.get(i + 1).is_some_and(|line| {
                        line.get_left_margin()
                            .is_some_and(|margin| margin < left_margin_mode + 10)
                    }) {
                        "\\\\"
                    } else {
                        "\n"
                    }
                } else {
                    ""
                };
                format!("\n    {}{}", line.get_latex(&prev, &next,), newline)
            })
            .collect()
    }

    /// Show the guess on the Page's image, mostly for debugging
    pub fn debug_image(&self) -> DynamicImage {
        // idk wtf is going on here, ask Noe
        let mut copy = self.image.clone();
        let mut alt = 0;
        for line in &self.lines {
            let unlock_line = line;
            let sub =
                image::RgbaImage::from_pixel(unlock_line.rect.width, 1, Rgba([255, 0, 255, 255]));

            overlay(
                &mut copy,
                &sub,
                i64::from(unlock_line.rect.x),
                i64::from(unlock_line.baseline),
            );

            for word in &unlock_line.words {
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
                                        (unlock_line.baseline + y - glyph.rect.height)
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
                        i64::from(unlock_line.rect.y + unlock_line.rect.height + 2),
                    );
                }
                let sub =
                    image::RgbaImage::from_pixel(word.rect.width, 2, Rgba([255, 100, 100, 255]));

                overlay(
                    &mut copy,
                    &sub,
                    i64::from(word.rect.x),
                    i64::from(unlock_line.rect.y + unlock_line.rect.height + 4),
                );
            }
        }

        copy
    }

    /// Compute the average distance between glyphs and their guesses, mostly for debugging
    pub fn debug_dist_avg(&self) {
        let data = self.lines.iter().fold((0., 0), |acc, line| {
            let n1 = acc.0 + line.get_dist_sum();
            (n1, acc.1 + line.get_glyph_count())
        });
        println!("distance moyenne : {}", data.0 / data.1 as f32);
    }

    /// Clean the pdf, like removing trailing dashes
    pub fn clean(&mut self) -> Result<()> {
        for i in 0..self.lines.len() {
            let current_line = self.lines.get_mut(i).unwrap();
            let last_word = current_line.words.last();
            let last_char = last_word.map(|word| word.get_content().chars().last());
            if last_char == Some(Some('-')) {
                // TODO remove trailing dash from image
                // TODO FIX : image is not in place so newline is inserted add values to avoid this
                current_line.words.last_mut().unwrap().glyphs.pop();
                current_line.can_have_new_line = false;
                let mut next_line = self.lines.get_mut(i + 1);
                if next_line
                    .as_ref()
                    .is_some_and(|line| line.words.first().is_some())
                {
                    let word = next_line.as_mut().unwrap().words.remove(0);
                    // TODO more than just add items
                    let current_line = self.lines.get_mut(i).unwrap();
                    let last_word = current_line.words.last_mut().unwrap();
                    last_word.glyphs.extend(word.glyphs);
                }
            }

            let current_line = self.lines.get_mut(i).unwrap();
            let searched_words = current_line.search_words("=");

            if !searched_words.is_empty() {
                let margins = (self.get_left_margin_mode(), self.get_right_margin_mode());
                let (prev_line_some, next_line_some) =
                    (self.lines.get(i - 1), self.lines.get(i + 1));
                let mut y_top = prev_line_some.map(Line::get_bottom);
                let mut y_bottom = next_line_some.map(Line::get_top);

                if self.lines.get(i - 1).is_some() {
                    let prev_line = prev_line_some.unwrap();
                    if let Some(line_margin) = prev_line.get_left_margin() {
                        if (line_margin as i32 - margins.0 as i32).abs() > 10
                            && prev_line.count_glyphes() < 20
                        {
                            let _ = y_top.insert(prev_line.get_top());
                        }
                    }
                }

                if next_line_some.is_some() {
                    let next_line = next_line_some.unwrap();
                    if let Some(line_margin) = next_line.get_left_margin() {
                        if (line_margin as i32 - margins.0 as i32).abs() > 10
                            && next_line.count_glyphes() < 20
                        {
                            let _ = y_bottom.insert(next_line.get_bottom());
                        }
                    }
                }
                for words_index in searched_words {
                    if let (Some(Some(top)), Some(Some(bottom))) = (y_top, y_bottom) {
                        let current_line = self.lines.get_mut(i).unwrap();
                        let word = current_line.words.get_mut(words_index);
                        if word.is_none() {
                            continue;
                        }
                        let word = word.unwrap();
                        let rect = Rect::new(
                            word.rect.x + word.rect.width,
                            top,
                            current_line.rect.width - word.rect.x,
                            bottom - top,
                        );
                        let extracted_image = rect.crop(&self.image);
                        let latex = Model::predict(&extracted_image)?;
                        extracted_image.save(format!("test{}.png", top))?;
                        let _ = word.latex.insert("= ".to_owned() + &latex + "$");
                        current_line.pop_words_in_rect(&rect);
                        self.lines
                            .get_mut(i - 1)
                            .map(|line| line.pop_words_in_rect(&rect));
                        self.lines
                            .get_mut(i + 1)
                            .map(|line| line.pop_words_in_rect(&rect));
                    }
                }
            }
        }

        if self.lines.last().is_some_and(|line| {
            line.words.len() == 1
                && line.words[0].glyphs.len() == 1
                && line.words[0].glyphs[0]
                    .guess
                    .clone()
                    .is_some_and(|guess| guess.base.parse::<i32>().is_ok())
        }) {
            self.lines.remove(self.lines.len() - 1);
        }

        Ok(())
    }

    pub fn get_right_margin_mode(&self) -> u32 {
        let right_margins = self
            .lines
            .iter()
            .filter_map(Line::get_right_margin)
            .collect::<Vec<u32>>();
        most_frequent(&right_margins, 0).0
    }

    pub fn get_left_margin_mode(&self) -> u32 {
        let left_margins = self
            .lines
            .iter()
            .filter_map(Line::get_left_margin)
            .collect::<Vec<u32>>();
        most_frequent(&left_margins, 0).0
    }

    pub fn search_words(&self, pattern: &str) -> Vec<(usize, Vec<usize>)> {
        self.lines
            .iter()
            .enumerate()
            .map(|(i, line)| (i, line.search_words(pattern)))
            .collect()
    }
}
