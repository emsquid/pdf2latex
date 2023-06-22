use crate::dictionary::Dictionary;
use crate::font::FontBase;
use crate::result::Result;
use crate::text::Line;
use crate::utils::{find_parts, pdf_to_images, Rect};
use image::imageops::overlay;
use image::{DynamicImage, Rgba};
use std::io::Write;
use std::time;

const LINE_SPACING: u32 = 5;

pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    fn find_lines(image: &DynamicImage) -> Vec<Line> {
        find_parts(&image.to_luma8(), LINE_SPACING)
            .into_iter()
            .map(|(start, end)| {
                let rect = Rect::new(0, start, image.width(), end - start + 1);
                Line::from(rect, image)
            })
            .collect()
    }

    pub fn from(image: &DynamicImage) -> Page {
        Page {
            image: image.clone(),
            lines: Page::find_lines(image),
        }
    }

    pub fn guess(&mut self, fontbase: &FontBase) {
        std::thread::scope(|scope| {
            let mut progress = 0.;
            let progress_step = 1. / self.lines.len() as f32;

            let mut stdout = std::io::stdout();
            let mut now = time::Instant::now();
            
            stdout.write_all(
                format!("\n\x1b[screating threads \t[{}] 0%               ",
                (0..21).map(|_| " ").collect::<String>()
            ).as_bytes()).unwrap();
            stdout.flush().unwrap();

            let mut handles = Vec::new();
            for line in &mut self.lines {
                let handle = scope.spawn(move || line.guess(fontbase));
                handles.push(handle);
                
                // ======================== progress bar ==========================
                progress += progress_step * 21.;
                if ((progress - progress_step) * 100. / 21.).floor() != (progress * 100. / 21.).floor() {
                    let length = progress.floor() as u32;
                    
                    stdout.write_all((
                        format!("\x1b[ucreating threads \t[{}{}] {}%               ",
                        (0..length).map(|_| "=").collect::<String>(),
                        (length..20).map(|_| " ").collect::<String>(),
                        (progress * 100. / 21.).round())
                    ).as_bytes()).unwrap();
                    stdout.flush().unwrap();
                }
                // =================================================================
            }
            stdout.write_all(
                format!("\x1b[ucreating threads \t[{}] {}s               ",
                (0..21).map(|_| "=").collect::<String>(),
                now.elapsed().as_secs_f32()
            ).as_bytes()).unwrap();
            stdout.flush().unwrap();

            progress = 0.;
            
            stdout.write_all(
                format!("\n\x1b[sconverting text \t[{}] 0%               ",
                (0..21).map(|_| " ").collect::<String>()
            ).as_bytes()).unwrap();
            stdout.flush().unwrap();
            now = time::Instant::now();
            
            for handle in handles {
                // ======================== progress bar ==========================
                progress += progress_step * 21.;
                if ((progress - progress_step) * 100. / 21.).floor() != (progress * 100. / 21.).floor() {
                    let length = progress.floor() as u32;
                    
                    stdout.write_all((
                        format!("\x1b[uconverting text \t[{}{}] {}%               ",
                        (0..length).map(|_| "=").collect::<String>(),
                        (length..20).map(|_| " ").collect::<String>(),
                        (progress * 100. / 21.).round())
                    ).as_bytes()).unwrap();
                    stdout.flush().unwrap();
                }
                // =================================================================

                handle.join().unwrap();
            }
            
            stdout.write_all(
                format!("\x1b[uconverting text \t[{}] {}s               \n",
                (0..21).map(|_| "=").collect::<String>(),
                now.elapsed().as_secs_f32()
            ).as_bytes()).unwrap();
            stdout.flush().unwrap();
        });
    }

    pub fn get_content(&self, dictionary: &Dictionary) -> String {
        let mut content = String::new();
        for line in &self.lines {
            content.push_str(&line.get_content(dictionary));
            content.push('\n');
        }

        content
    }

    pub fn debug(&self) -> DynamicImage {
        let mut copy = self.image.clone();
        let mut alt = true;
        for line in &self.lines {
            for word in &line.words {
                for glyph in &word.glyphs {
                    alt = !alt;
                    let color = if alt {
                        Rgba([0, 0, 255, 255])
                    } else {
                        Rgba([0, 255, 0, 255])
                    };
                    let sub = image::RgbaImage::from_pixel(glyph.rect.width, 2, color);

                    overlay(
                        &mut copy,
                        &sub,
                        i64::from(glyph.rect.x),
                        i64::from(line.rect.y + line.rect.height + 1),
                    );
                }
                let sub = image::RgbaImage::from_pixel(word.rect.width, 2, Rgba([255, 100, 100, 255]));

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
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &str) -> Result<Pdf> {
        let pages = pdf_to_images(path)?.iter().map(Page::from).collect();

        Ok(Pdf { pages })
    }

    pub fn guess(&mut self) -> Result<()> {
        let fontbase = FontBase::new()?;

        for page in &mut self.pages {
            page.guess(&fontbase);
        }

        Ok(())
    }

    pub fn get_content(&self) -> Result<String> {
        let dictionary = Dictionary::new()?;

        let mut content = String::new();
        for page in &self.pages {
            content.push_str(&page.get_content(&dictionary));
            content.push('\n');
        }

        Ok(content)
    }
}
