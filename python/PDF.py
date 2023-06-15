from __future__ import annotations

import numpy as np
import numexpr as ne
from PIL import Image, ImageOps
from pdf2image import convert_from_path
import multiprocessing

from TextZone import Line, Word, LETTER_THRESHOLD


class Page:
    # ========================================
    image: Image.Image
    array: np.ndarray
    width: int
    height: int

    lines: list[Line] = None
    # ========================================

    def __init__(self, image: Image.Image):
        self.image = image
        self.width = image.width
        self.height = image.height
        self.array: np.ndarray = np.array(image)

    @staticmethod
    def from_array(array: np.ndarray) -> Page:
        return Page(Image.fromarray(array.astype(np.uint8)))

    @property
    def array_rgb(self) -> np.ndarray:
        if self.array.ndim == 3:
            return self.array.copy()
        else:
            return np.dstack((self.array, self.array, self.array))

    def grayscaled(self) -> Page:
        return Page(ImageOps.grayscale(self.image))

    def thresholded(self) -> Page:
        threshold: int = LETTER_THRESHOLD
        array = self.grayscaled().array
        return Page.from_array(ne.evaluate("where(arrat <= threshold, 0, 255)"))

    def set_lines(self):
        if self.lines is not None:
            return

        self.lines = []
        start: int = -1
        inLine: bool = False
        for i, line in enumerate(self.grayscaled().array):
            if inLine and sum(line) / self.width == 255:
                self.lines.append(Line(start, i))
                inLine = False
            elif not inLine and sum(line) / self.width != 255:
                start = i
                inLine = True

    def set_words(self):
        self.set_lines()

        array = self.grayscaled().array
        for line in self.lines:
            line.set_words(array[line.top : line.bottom])

    def set_words_threaded(self):
        self.set_lines()

        array = self.grayscaled().array
        with multiprocessing.Pool(multiprocessing.cpu_count()) as pool:
            args = [
                (line, array[line.top : line.bottom], l)
                for l, line in enumerate(self.lines)
            ]

            for words, l in pool.starmap(Line.get_words, args):
                self.lines[l].words = words

    def set_letters(self):
        self.set_words()

        array = self.grayscaled().array
        for line in self.lines:
            for word in line.words:
                word.set_letters(array[line.top : line.bottom])

    def set_letters_threaded(self):
        self.set_words_threaded()

        array = self.grayscaled().array
        with multiprocessing.Pool(multiprocessing.cpu_count()) as pool:
            args = [
                (word, array[line.top : line.bottom], l, w)
                for l, line in enumerate(self.lines)
                for w, word in enumerate(line.words)
            ]

            for letters, l, w in pool.starmap(Word.get_letters, args):
                self.lines[l].words[w].letters = letters

    def show(self):
        self.image.show()

    def show_lines(self):
        self.set_lines()

        parsed = self.array_rgb
        for line in self.lines:
            parsed[line.top - 1] = [0, 255, 0]
            parsed[line.bottom] = [255, 0, 0]

        return Page.from_array(np.array(parsed)).show()

    def show_words(self):
        self.set_words()

        parsed = self.array_rgb
        for line in self.lines:
            for word in line.words:
                parsed[line.top - 1, word.left : word.right + 1] = [0, 255, 0]
                parsed[line.bottom, word.left : word.right + 1] = [255, 0, 0]

        return Page.from_array(np.array(parsed)).show()

    def show_letters(self):
        self.set_letters_threaded()

        parsed = self.array_rgb
        for line in self.lines:
            for word in line.words:
                parsed[line.top - 1, word.left : word.right + 1] = [0, 255, 0]
                parsed[line.bottom, word.left : word.right + 1] = [255, 0, 0]

                alternate: bool = False

                for letter in word.letters:
                    parsed[
                        line.bottom : line.bottom + 3,
                        letter.left : letter.right,
                        2,
                    ] -= (
                        100 if alternate else 200
                    )

                    for pixel in letter.pixels:
                        parsed[line.top + pixel[0][0]][pixel[0][1]] = (
                            [min(255, pixel[1] + 50), min(255, pixel[1] + 50), 255]
                              if alternate else 
                            [min(255, pixel[1] + 50), 255, min(255, pixel[1] + 50)])

                    alternate = not alternate

        return Page.from_array(parsed).show()


class PDF:
    # ========================================
    pages: list[Page]
    # ========================================

    def __init__(self, path: str):
        ext: str = path.split(".")[-1]

        if ext == "pdf":
            self.pages: list[Page] = [Page(image) for image in convert_from_path(path, 300)]
        elif ext == "png":
            self.pages: list[Page] = [Page(Image.open(path).convert("RGB"))]
        else:
            raise TypeError("Wrong extension")
