use crate::args::Args;
use crate::dictionary::Dictionary;
use crate::font::FontBase;
use crate::result::Result;
use crate::text::Line;
use crate::utils::{find_parts, log, pdf_to_images, Rect};
use image::imageops::overlay;
use image::{DynamicImage, Rgba};
use std::io::Write;
use std::path::Path;
use std::time;

const LINE_SPACING: u32 = 10;

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

    pub fn guess(&mut self, fontbase: &FontBase) -> Result<()> {
        std::thread::scope(|scope| -> Result<()> {
            let mut now = time::Instant::now();
            let mut progress = 0.;
            let step = 1. / self.lines.len() as f32;

            std::io::stdout().write_all(b"\n\x1b[s")?;
            std::io::stdout().flush()?;
            log("creating threads", Some(0.), None)?;

            let mut handles = Vec::new();
            for line in &mut self.lines {
                let handle = scope.spawn(move || line.guess(fontbase));
                handles.push(handle);

                progress += step;
                log("creating threads", Some(progress), None)?;
            }

            let duration = now.elapsed().as_secs_f32();
            log("creating threads", Some(1.), Some(duration))?;

            now = time::Instant::now();
            progress = 0.;

            std::io::stdout().write_all(b"\n\x1b[s")?;
            std::io::stdout().flush()?;
            log("converting text", Some(0.), None)?;

            for handle in handles {
                handle.join().unwrap();

                progress += step;
                log("converting text", Some(progress), None)?;
            }

            let duration = now.elapsed().as_secs_f32();
            log("converting text", Some(1.), Some(duration))?;
            std::io::stdout().write_all(b"\n")?;

            Ok(())
        })
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

    pub fn debug_dist_avg(&self) {
        let data = self.lines.iter().fold((0., 0), |acc, line| {
            (acc.0 + line.get_dist_sum(), acc.1 + line.get_glyph_count())
        });
        println!("distance moyenne : {}", data.0 / data.1 as f32);
    }
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &Path) -> Result<Pdf> {
        let pages = pdf_to_images(path)?.iter().map(Page::from).collect();

        Ok(Pdf { pages })
    }

    pub fn guess(&mut self, args: &Args) -> Result<()> {
        let fontbase = FontBase::new(args)?;

        for page in &mut self.pages {
            page.guess(&fontbase)?;
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

    pub fn save_content(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.get_content()?)?;

        Ok(())
    }
}
