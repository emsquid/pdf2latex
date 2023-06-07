use crate::{
    poppler::pdf_to_images,
    text::{Line, Rect},
};
use image::DynamicImage;

pub struct Page {
    pub image: DynamicImage,
    pub lines: Vec<Line>,
}

impl Page {
    pub fn from(image: DynamicImage) -> Page {
        Page {
            image: image.clone(),
            lines: Page::get_lines(image),
        }
    }

    pub fn get_lines(image: DynamicImage) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut y = 0;

        for (i, row) in image.to_luma8().enumerate_rows() {
            let average = row.map(|l| u32::from(l.2 .0[0])).sum::<u32>() / image.width();
            if y == 0 && average != 255 {
                y = i;
            } else if y != 0 && average == 255 {
                let rect = Rect::new(0, y, image.width(), i - y);
                let cropped = image.crop_imm(rect.x, rect.y, rect.width, rect.height);
                lines.push(Line::new(rect, cropped));
                y = 0;
            }
        }

        lines
    }
}

pub struct Pdf {
    pub pages: Vec<Page>,
}

impl Pdf {
    pub fn load(path: &str) -> Pdf {
        Pdf {
            pages: pdf_to_images(path, 200)
                .iter()
                .map(|image| Page::from(image.clone()))
                .collect(),
        }
    }
}
