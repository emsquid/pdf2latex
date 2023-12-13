use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView};

use crate::{
    args::MainArg,
    fonts::FontBase,
    utils::{find_parts, BracketType, Rect},
};

use super::{word::BracketData, Page, Word};

pub const MATRIX_SPACING: u32 = 70;

#[derive(Clone)]
pub struct Matrix {
    pub page: Page,
    pub bracket_opening: (BracketType, Rect),
    pub bracket_closing: (BracketType, Rect),
    pub image: DynamicImage, // the image of the matrix with the brackets
    pub rect: Rect,
}

impl Matrix {
    /// generate a matrix from the given`image`, that muss include the given `brackets`, the column
    /// will be parsed by the `matrix_spacing`
    ///
    /// # Error
    ///
    /// return an error if the page guess fails or if the the matrix is a one by one
    pub fn try_from(
        image: &DynamicImage,
        (bracket_opening, bracket_closing): (&BracketData, &BracketData),
        matrix_spacing: Option<u32>,
        fontbase: &FontBase,
        args: &MainArg,
    ) -> Result<Matrix> {
        let (rect_bo, rect_bc) = (bracket_opening.0.rect, bracket_closing.0.rect);

        // compute the whole (with brackets) matrix inside to pe parsed
        let matrix_width = rect_bo.x.abs_diff(rect_bc.x + rect_bc.width);
        let matrix_height = std::cmp::min(rect_bc.y, rect_bo.y).abs_diff(std::cmp::max(
            rect_bo.y + rect_bo.height,
            rect_bc.y + rect_bc.height,
        ));
        let matrix_rect = Rect::new(
            rect_bo.x,
            std::cmp::min(rect_bo.y, rect_bc.y),
            matrix_width,
            matrix_height,
        );
        let matrix_image_view =
            image.view(matrix_rect.x, matrix_rect.y, matrix_width, matrix_height);

        let mut matrix = Matrix {
            page: Page::default(),
            bracket_opening: (bracket_opening.1.to_owned(), bracket_opening.0.rect),
            bracket_closing: (bracket_closing.1.to_owned(), bracket_closing.0.rect),
            image: DynamicImage::from(matrix_image_view.to_image()),
            rect: matrix_rect,
        };

        // compute the inside of the matrix (without brackets)
        let matrix_inside_rect = matrix.get_inside_rect();
        let matrix_inside_image_view = image.view(
            matrix_inside_rect.x,
            matrix_inside_rect.y,
            matrix_inside_rect.width,
            matrix_inside_rect.height,
        );
        let matrix_inside_image = DynamicImage::from(matrix_inside_image_view.to_image());

        // find columns and lines
        let cols_indexes = find_parts(
            &matrix_inside_image.rotate90().to_luma8(),
            matrix_spacing.unwrap_or(MATRIX_SPACING),
        );
        let lines_indexes = find_parts(
            &matrix_inside_image.rotate90().to_luma8(),
            matrix_spacing.unwrap_or(MATRIX_SPACING),
        );

        if lines_indexes.len() == 1 && cols_indexes.len() == 1 {
            return Err(anyhow!("matrix of one item"));
        }

        // TODO some matrix are not rightly parsed... some lines a not parsed and seen as one word
        let mut page = Page::from(&matrix_inside_image, matrix_spacing);
        let mut args = args.to_owned();
        args.verbose = false;
        page.guess(fontbase, &args)?;
        page.verify(&args, fontbase)?;

        let mut empty_words_to_push: Vec<usize> = Vec::new();
        let mut first_of_part;
        let mut wi;
        let mut word_in_col;
        // collapse divided columns
        for li in 0..page.lines.len() {
            let line = page.lines.get_mut(li).unwrap();
            empty_words_to_push.clear();
            wi = 0;
            for i in 0..cols_indexes.len() {
                first_of_part = true;
                word_in_col = false;
                let col = cols_indexes.get(i).unwrap();
                // find the first and the next ones of a column
                while line.words.get(wi).is_some_and(|word| word.is_between(&col)) {
                    // join words they are at the same column
                    if !first_of_part {
                        let (inf, sup) = line.words.split_at_mut(wi);
                        if let Some(last) = inf.last_mut() {
                            last.join(sup.first().unwrap());
                            line.words.remove(wi);
                        }
                        // indexes_to_pop.push(wi);
                    } else {
                        wi += 1;
                    }
                }
                // did not found any word in the column so add a new word with the rect of the
                // column in it
                if !word_in_col {
                    line.words.insert(
                        i,
                        Word::new(
                            Rect::new(
                                line.rect.x + col.0,
                                line.rect.y,
                                col.1 - col.0,
                                line.rect.height,
                            ),
                            vec![],
                            None,
                        ),
                    );
                }
            }
        }
        matrix.page = page;
        Ok(matrix)
    }

    /// generate the latex equivalent of this matrix in latex (word are written as get_content()
    /// and not get_latex())
    pub fn get_latex(&self) -> String {
        let mut str = String::from("\\begin{pmatrix}\n");

        str += &self
            .page
            .lines
            .iter()
            .map(|line| {
                println!("len = {}", line.words.len());
                line.words
                    .iter()
                    .map(|word| match &word.special_formula {
                        Some(s) => s.get_latex(),
                        None => word.get_content(),
                    })
                    .collect::<Vec<String>>()
                    .join(" & ")
            })
            .collect::<Vec<String>>()
            .join("\\\\\n");

        str += "\n\\end{pmatrix}";
        str
    }

    /// return the inside rect of the matrix, without the brackets
    pub fn get_inside_rect(&self) -> Rect {
        let matrix_inside_width = self
            .bracket_closing
            .1
            .x
            .abs_diff(self.bracket_opening.1.x + self.bracket_opening.1.width);
        let matrix_inside_height =
            std::cmp::min(self.bracket_closing.1.y, self.bracket_opening.1.y).abs_diff(
                std::cmp::min(
                    self.bracket_opening.1.y + self.bracket_opening.1.height,
                    self.bracket_closing.1.y + self.bracket_closing.1.height,
                ),
            );

        Rect::new(
            self.bracket_opening.1.x + self.bracket_opening.1.width,
            std::cmp::min(self.bracket_opening.1.y, self.bracket_closing.1.y),
            matrix_inside_width,
            matrix_inside_height,
        )
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }
}
