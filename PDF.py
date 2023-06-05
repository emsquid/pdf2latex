from __future__ import annotations

import numpy as np
import numexpr as ne
from PIL import Image, ImageOps
from pdf2image import convert_from_path
from TextZone import Line, LETTER_THRESHOLD

class Page:
    # ========================================
    image: Image.Image
    array: np.ndarray
    width: int
    height: int

    lines: list[Line] = None
    # ========================================
    
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
    def show_threshold(self):
        arr = self.grayscaled().array
        tr = LETTER_THRESHOLD
        Page.from_array(ne.evaluate("where(arr <= tr, 0, 255)")).show()

    def set_lines(self):
        self.lines = []

        start: int = -1
        inLine: bool = False
        for i, line in enumerate(self.grayscaled().array):
            if inLine and sum(line) / self.width == 255:
                self.lines.append(Line(start, i))
                inLine = False
            elif not(inLine) and sum(line) / self.width != 255:
                start = i
                inLine = True

    def show_lines(self):
        if self.lines == None:
            self.set_lines()
        
        parsed = self.array.copy()
        for line in self.lines:
            parsed[line.top] = [[0, 255, 0] for _ in range(self.width)]
            parsed[line.bottom] = [[255, 0, 0] for _ in range(self.width)]
        return Page.from_array(np.array(parsed)).show()
    def show_words(self):
        if self.lines == None:
            self.set_lines()
        for line in self.lines:
            line.set_words(self.grayscaled().array)
        
        parsed = self.array.copy()
        for line in self.lines:
            for word in line.words:
                parsed[line.top][word.left: word.right] = [0, 255, 0]
                parsed[line.bottom][word.left: word.right] = [255, 0, 0]
        return Page.from_array(np.array(parsed)).show()
    def show_letters(self):
        if self.lines == None:
            self.set_lines()
        
        for line in self.lines:
            line.set_letters(self.grayscaled().array)
        # self.lines[0].set_letters(self.grayscaled().array)
        # self.lines = [self.lines[0]]
        
        parsed = self.array.copy()
        for line in self.lines:
            for word in line.words:
                parsed[line.top, word.left: word.right] = [0, 255, 0]
                parsed[line.bottom, word.left: word.right] = [255, 0, 0]

                alternate: bool = False
                for letter in word.letters:
                    parsed[line.bottom: line.bottom + 5, letter.left: letter.right, 2] -= 25 if alternate else 75
                    alternate = not(alternate)
        return Page.from_array(np.array(parsed)).show()
    def show_columns(self):
        parsed = self.array.transpose(1, 0, 2)
        for j, column in enumerate(self.grayscaled().array.transpose()):
            if sum(column) / self.height == 255:
                parsed[j] = [[255, 0, 0] for _ in range(self.height)]
        return Page.from_array(np.array(parsed.transpose(1, 0, 2))).show()


class PDF:
    # ========================================
    pages: list[Page]
    # ========================================

    def __init__(self, path: str):
        ext: str = path.split('.')[-1]

        if ext == "pdf":
            self.pages: list[Page] = [Page(image) for image in convert_from_path(path)]
        elif ext == "png":
            self.pages: list[Page] = [Page(Image.open(path))]
        else:
            raise TypeError("wrong extention")