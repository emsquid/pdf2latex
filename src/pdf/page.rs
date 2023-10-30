use crate::args::MainArg;
use crate::fonts::glyph::MATRIX_SPACING;
use crate::fonts::{
    glyph::{BracketData, Glyph, Matrix, SpecialFormulas},
    FontBase, UnknownGlyph, DIST_THRESHOLD,
};
use crate::pdf::{Line, Word};
use crate::utils::{find_parts, log, most_frequent, BracketType, Rect};
use crate::vit::Model;
use anyhow::{anyhow, Result};
use image::{imageops::overlay, DynamicImage, GenericImage, GenericImageView, Rgba};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time,
};

const LINE_SPACING: u32 = 10;

/// A Page from a Pdf, it holds an image and multiple lines
#[derive(Clone)]
pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    /// Create a Page from an image
    #[must_use]
    pub fn from(image: &DynamicImage, word_spacing: Option<u32>) -> Page {
        Page {
            image: image.clone(),
            lines: Page::find_lines(image, word_spacing),
        }
    }

    /// Find the different lines in an image
    fn find_lines(image: &DynamicImage, word_spacing: Option<u32>) -> Vec<Line> {
        find_parts(&image.to_luma8(), LINE_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(0, start, image.width(), end - start + 1);
                Line::from(rect, image, word_spacing)
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
            let mut handles = Vec::with_capacity(args.threads);
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
                format!(
                    "\n    {}{}",
                    line.get_latex(prev.as_ref(), next.as_ref(),),
                    newline
                )
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
    pub fn verify(&mut self, args: &MainArg, fontbase: &FontBase) -> Result<()> {
        for i in 0..self.lines.len() {
            self.handle_trailing_dash_verify(i);
        }
        self.handle_matrixes_verify(fontbase, args);
        self.handle_formulas_verify(args)?;

        // remove page number
        if self.lines.last().is_some_and(|line| line.words.len() == 1) {
            self.lines.remove(self.lines.len() - 1);
        }

        Ok(())
    }

    pub fn handle_matrix_verify(
        &self,
        fontbase: &FontBase,
        args: &MainArg,
        bracket_opening: &BracketData,
        bracket_closing: &BracketData,
    ) -> Option<Matrix> {
        let (rect_bo, rect_bc) = (bracket_opening.0.rect, bracket_closing.0.rect);
        if (rect_bo.height).abs_diff(rect_bc.height) > 10 {
            return None;
        }
        let matrix_width = rect_bc.x.abs_diff(rect_bo.x + rect_bo.width);
        let matrix_height = std::cmp::min(rect_bc.y, rect_bo.y).abs_diff(std::cmp::min(
            rect_bo.y + rect_bo.height,
            rect_bc.y + rect_bc.height,
        ));
        let image = self.image.view(
            rect_bo.x + rect_bo.width,
            std::cmp::max(rect_bo.y, rect_bc.y),
            matrix_width,
            matrix_height,
        );

        let matrix_inside_image = DynamicImage::from(image.to_image());
        // TODO if matrix in matrix
        let matrix = Matrix::from(
            &matrix_inside_image,
            bracket_opening.1.clone(),
            Some(MATRIX_SPACING),
            fontbase,
            args,
        );
        Some(matrix)
    }

    pub fn handle_matrixes_verify(&mut self, fontbase: &FontBase, args: &MainArg) -> Result<()> {
        let mut brackets: Vec<BracketData>;
        let mut matrixes_to_push: Vec<(usize, Matrix)> = Vec::new();
        let mut glyphs_to_pop: Vec<((usize, usize), (usize, usize))> = Vec::new();
        let mut c: usize;
        for li in 0..self.lines.len() {
            matrixes_to_push.clear();
            glyphs_to_pop.clear();
            let line = self.lines.get(li).unwrap();
            c = 0;
            brackets = line.get_all_brackets()?;
            println!(
                "al brackets = {:?}",
                brackets.iter().map(|v| v.1.clone()).collect::<Vec<BracketType>>()
            );
            while c < brackets.len() {
                let bc = &brackets[c];
                if let Some(opposing) = line.search_opposing_brackets(bc, &brackets[c..]) {
                    let bo = &brackets[opposing];
                    if let Some(matrix) = self.handle_matrix_verify(fontbase, args, bc, bo) {
                        println!("latex matrix = {}", matrix.get_latex());
                        matrixes_to_push.push((bc.2 + 1, matrix));
                        glyphs_to_pop.push(((bc.2, bc.3), (bo.2, bo.3)));
                    } else { //TODO handle parentheses that does not match to a matrix
                    }
                    c = opposing;
                } else {
                    c += 1;
                }
            }
            println!("to pop => {:?}", glyphs_to_pop);
            let line = self.lines.get_mut(li).unwrap();
            // TODO set rect CORRECTLY
            for ((wi_f, gi_f), (wi_l, gi_l)) in glyphs_to_pop.iter().rev() {
                // matrix has been detected in the same word
                let mat = Some(SpecialFormulas::Matrix(matrixes_to_push.pop().unwrap().1));
                if wi_f == wi_l {
                    let word = line.words.get_mut(*wi_f).unwrap();
                    // the matrix represents a full word
                    if gi_f == &0 && *gi_l == word.glyphs.len() {
                        word.glyphs.clear();
                        word.glyphs.shrink_to_fit();
                        word.special_formula = mat;
                    }
                    // the matrix is at the end of a word
                    else if gi_l + 1 == word.glyphs.len() {
                        word.glyphs.drain(gi_f + 1..);
                        let mut w = Word::default();
                        w.special_formula = mat;
                        line.words.insert(wi_f.saturating_add_signed(-1), w);
                    }
                    // the matrix is at the start of a word
                    else if gi_f == &0 {
                        word.glyphs.drain(..gi_l);
                        let mut w = Word::default();
                        w.special_formula = mat;
                        line.words.insert(*wi_f, w);
                    }
                    // matrix is between 2 words or an intersection of the 2
                    else {
                        let mut mat_added = false;
                        let word_f = line.words.get_mut(*wi_f).unwrap();
                        if *gi_f == 0 {
                            mat_added = true;
                            word_f.glyphs.clear();
                            // TODO avoid this clone
                            word_f.special_formula = mat.clone();
                        } else {
                            word_f.glyphs.drain(gi_f..);
                        }

                        let word_l = line.words.get_mut(*wi_l).unwrap();
                        if *gi_l + 1 == word_l.glyphs.len() {
                            if !mat_added {
                                // TODO avoid this clone
                                word_l.special_formula = mat.clone();
                                word_l.glyphs.clear();
                                mat_added = true;
                            } else {
                                line.words.remove(*wi_l);
                            }
                        } else {
                            word_l.glyphs.drain(..gi_l);
                        }

                        line.words.drain(wi_f + 2..*wi_l);
                        if mat_added {
                            line.words.remove(wi_f + 1);
                        } else {
                            if wi_l.abs_diff(*wi_f) == 2 {
                                let mut w = Word::default();
                                w.special_formula = mat;
                                line.words.insert(wi_f + 1, w);
                            } else {
                                let word_m = line.words.get_mut(*wi_f + 1).unwrap();
                                word_m.glyphs.clear();
                                word_m.special_formula = mat;
                            }
                        }
                    }
                }
            }
        }
        // for bracket in brackets.drain(0..) {
        // if bracket.1 == BracketType::OpeningCurly {
        // println!("maybe found system");
        // }
        Ok(())
    }

    pub fn handle_trailing_dash_verify(&mut self, line_index: usize) {
        let current_line = self.lines.get_mut(line_index).unwrap();
        let last_word = current_line.words.last();
        let last_char = last_word.map(|word| word.get_content().chars().last());
        if last_char == Some(Some('-')) {
            // TODO remove trailing dash from image
            // TODO FIX : image is not in place so newline is inserted add values to avoid this
            current_line.words.last_mut().unwrap().glyphs.pop();
            current_line.can_have_new_line = false;
            drop(current_line);
            let mut next_line = self.lines.get_mut(line_index + 1);
            if next_line
                .as_ref()
                .is_some_and(|line| line.words.first().is_some())
            {
                let word = next_line.as_mut().unwrap().words.remove(0);
                // TODO more than just add items
                let current_line = self.lines.get_mut(line_index).unwrap();
                let last_word = current_line.words.last_mut().unwrap();
                last_word.glyphs.extend(word.glyphs);
            }
        }
    }

    pub fn handle_formulas_verify(&mut self, args: &MainArg) -> Result<()> {
        let formula_indexes = self.get_middle_formula_indexes();
        if formula_indexes.len() == 00 {
            return Ok(());
        }
        let nb_formula = &formula_indexes.len().to_owned();
        let mut formula_id = 0;
        let formula_done = Arc::new(Mutex::new(0));

        let image_page = &self.image.to_owned();
        let margins = (self.get_left_margin_mode(), self.get_right_margin_mode());

        let mut lines_to_remove: Vec<u32> = Vec::new();
        let lines_to_modify: Arc<Mutex<Vec<(usize, Word)>>> = Arc::new(Mutex::new(Vec::new()));

        if args.verbose {
            log("\nGENERATING FORMULAS\n", None, None, "1m")?;
            log(
                format!("Generating Formulas ... 0/{}", nb_formula).as_str(),
                None,
                None,
                "u",
            )?;
        }
        let now = time::Instant::now();

        std::thread::scope(|scope| -> Result<()> {
            let mut handles = Vec::with_capacity(args.threads);

            for i in formula_indexes {
                // println!("handling {}", i);
                let current_line = self.lines.get(i).unwrap();
                let line_margin = (
                    current_line.get_left_margin(),
                    current_line.get_right_margin(),
                );

                if line_margin.1.is_some() && line_margin.0.is_some() {
                    // println!("passed => {}", i);
                    let (prev_line_some, next_line_some) =
                        (self.lines.get(i - 1), self.lines.get(i + 1));
                    let mut y_top = prev_line_some.map(Line::get_bottom);
                    let mut y_bottom = next_line_some.map(Line::get_top);

                    if prev_line_some.is_some() {
                        let prev_line = prev_line_some.unwrap();
                        if let Some(line_margin) = prev_line.get_left_margin() {
                            if (line_margin as i32 - margins.0 as i32).abs() > 10
                                && prev_line.count_glyphes() < 20
                            {
                                lines_to_remove.push(i as u32 - 1);
                                let _ = y_top.insert(prev_line.get_top());
                            }
                        }
                    }

                    if next_line_some.is_some() {
                        let next_line = next_line_some.unwrap();
                        if let Some(line_margin) = next_line.get_left_margin() {
                            if i + 2 != self.lines.len()
                                && (line_margin as i32 - margins.0 as i32).abs() > 10
                                && next_line.count_glyphes() < 20
                            {
                                lines_to_remove.push(i as u32 + 1);
                                let _ = y_bottom.insert(next_line.get_bottom());
                            }
                        }
                    }
                    if let (Some(Some(top)), Some(Some(bottom))) = (y_top, y_bottom) {
                        let current_left_margin = current_line.get_left_margin().unwrap_or(0);
                        let current_line_width = current_line.rect.width;

                        let lines_to_modify_cloned = lines_to_modify.clone();
                        let count_cloned = formula_id.to_owned();
                        let formula_done_cloned = formula_done.clone();
                        // println!("strating thread => {}", i);

                        let handle = scope.spawn(move || -> Result<()> {
                            let rect = Rect::new(
                                current_left_margin,
                                top,
                                current_line_width,
                                bottom - top,
                            );
                            let extracted_image = rect.crop(&image_page);
                            let latex = Model::predict(&extracted_image, Some(count_cloned))?;
                            {
                                *formula_done_cloned.lock().unwrap() += 1;
                            }
                            if args.verbose {
                                log(
                                    format!(
                                        "Generating Formulas ... {}/{}",
                                        formula_done_cloned.lock().unwrap(),
                                        nb_formula
                                    )
                                    .as_str(),
                                    None,
                                    None,
                                    "u",
                                )?;
                            }
                            extracted_image.save(format!("test{}.png", top))?;
                            let mut word = Word::from(rect, &image_page);
                            let _ = word
                                .special_formula
                                .insert(SpecialFormulas::GivenIaFormula(latex));
                            lines_to_modify_cloned.lock().unwrap().push((i, word));
                            Ok(())
                        });
                        formula_id += 1;

                        handles.push(handle);
                        if handles.len() >= 2 {
                            // if handles.len() >= args.threads {
                            handles.remove(0).join().unwrap().unwrap();
                        }
                    }
                }
            }
            for handle in handles {
                handle.join().unwrap().unwrap();
            }
            Ok(())
        })?;

        if args.verbose {
            log(
                format!("Generating Formulas ... {}/{}", nb_formula, nb_formula).as_str(),
                None,
                None,
                "u",
            )?;
            let duration = now.elapsed().as_secs_f32();
            std::io::stdout().write(&[10])?;
            log("GENERATED FORMULAS", None, Some(duration), "1m")?;
            std::io::stdout().write(&[10, 10])?;
            std::io::stdout().flush()?;
        }

        let len = lines_to_modify.lock().unwrap().len();
        for (line_index, word) in lines_to_modify.lock().unwrap().iter().take(len) {
            let line = self.lines.get_mut(*line_index).unwrap();
            line.words.clear();
            line.words.push(word.to_owned());
        }

        lines_to_remove.reverse();
        for index in lines_to_remove {
            self.lines.remove(index as usize);
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

    pub fn get_middle_formula_indexes(&self) -> Vec<usize> {
        let mut lines_vec = Vec::new();
        let margins = (self.get_left_margin_mode(), self.get_right_margin_mode());
        for (i, line) in self.lines.iter().enumerate() {
            let line_margin = (line.get_left_margin(), line.get_right_margin());
            if let (Some(left_margin), Some(right_margin)) = line_margin {
                if margins.1 - right_margin < left_margin - margins.0 + 25
                    && line.get_dist_sum() / (line.count_glyphes() as f32) > 10.
                    && !line.is_full_line(margins)
                {
                    lines_vec.push(i);
                }
            }
        }
        return lines_vec;
    }
}
