# Better typing
from __future__ import annotations

import numpy as np
from PIL import Image, ImageOps
from pdf2image import convert_from_path


class Page:
    def __init__(self, image: Image.Image):
        self.image: Image.Image = image
        self.array: np.ndarray = np.array(image)
        self.width: int = image.width
        self.height: int = image.height

    @staticmethod
    def from_array(array: np.ndarray) -> Page:
        return Page(Image.fromarray(array))

    def show(self):
        self.image.show()

    def grayscaled(self) -> Page:
        return Page(ImageOps.grayscale(self.image))

    def lines(self) -> Page:
        parsed = self.array.copy()
        for i, line in enumerate(self.grayscaled().array):
            if sum(line) / self.width == 255:
                parsed[i] = [[255, 0, 0] for _ in range(self.width)]
        return Page.from_array(np.array(parsed))

    def columns(self) -> Page:
        parsed = self.array.transpose(1, 0, 2)
        for j, column in enumerate(self.grayscaled().array.transpose()):
            if sum(column) / self.height == 255:
                parsed[j] = [[255, 0, 0] for _ in range(self.height)]
        return Page.from_array(np.array(parsed.transpose(1, 0, 2)))


class PDF:
    def __init__(self, path: str):
        self.pages: list[Page] = [Page(image) for image in convert_from_path(path)]


test_1 = PDF("test/test_1_toLatex.pdf")
test_1.pages[0].lines().show()
test_1.pages[0].columns().show()
test_1.pages[0].show()
