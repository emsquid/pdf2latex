use crate::args::Args;
use crate::dictionary::Dictionary;
use crate::font::FontBase;
use crate::glyph::Glyph;
use crate::result::Result;
use crate::text::Line;
use crate::utils::{find_parts, log, pdf_to_images, Rect};
use image::imageops::overlay;
use image::{DynamicImage, GenericImage, Rgba};
use std::collections::btree_map::Iter;
use std::io::Write;
use std::path::Path;
use std::{default, time};

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

    pub fn guess(&mut self, fontbase: &FontBase, args: &Args) -> Result<()> {
        std::thread::scope(|scope| -> Result<()> {
            let mut now = time::Instant::now();
            let mut progress = 0.;
            let step = 1. / self.lines.len() as f32;

            if !args.silent {
                log("creating threads", Some(0.), None, "s")?;
            }

            let mut handles = Vec::new();
            for line in &mut self.lines {
                let handle = scope.spawn(move || line.guess(fontbase));
                handles.push(handle);

                progress += step;
                if !args.silent {
                    log("creating threads", Some(progress), None, "u")?;
                }
            }

            let duration = now.elapsed().as_secs_f32();
            if !args.silent {
                log("creating threads", Some(1.), Some(duration), "u")?;
            }

            now = time::Instant::now();
            progress = 0.;

            if !args.silent {
                std::io::stdout().write_all(b"\n\x1b[s")?;
                log("converting text", Some(0.), None, "u")?;
            }

            for handle in handles {
                handle.join().unwrap();

                progress += step;
                if !args.silent {
                    log("converting text", Some(progress), None, "u")?;
                }
            }

            let duration = now.elapsed().as_secs_f32();
            if !args.silent {
                log("converting text", Some(1.), Some(duration), "u")?;
                std::io::stdout().write_all(b"\n")?;
            }

            Ok(())
        })
    }

    pub fn get_content(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.get_content())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn debug_content(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.debug_content())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn debug_image(&self) -> DynamicImage {
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
                                        (line.baseline + y - glyph.rect.height).saturating_add_signed(guess.offset),
                                        c,
                                    )
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

        for (i, page) in self.pages.iter_mut().enumerate() {
            if !args.silent {
                log(&format!("\nPAGE {i}\n"), None, None, "1m")?;
            }

            page.guess(&fontbase, args)?;
        }

        Ok(())
    }

    pub fn get_content(&self) -> Result<String> {
        let dictionary = Dictionary::new()?;
        let content = self
            .pages
            .iter()
            .map(|page| page.get_content())
            .collect::<Vec<String>>()
            .join("\n");

        //Ok(dictionary.correct_text(content))
        Ok(content)
    }

    pub fn debug_content(&self) -> Result<String> {
        Ok(self
            .pages
            .iter()
            .map(|page| page.debug_content())
            .collect::<Vec<String>>()
            .join("\n"))
    }

    pub fn save_content(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.get_content()?)?;

        Ok(())
    }
}
