from __future__ import annotations

import numpy as np
from utils import transpose

LETTER_SPACING: int = 5
LETTER_THRESHOLD: int = 180


class Letter:
    # ========================================
    left: int
    right: int
    # ========================================

    def __init__(self, left: int, right: int):
        self.left = left
        self.right = right

    @staticmethod
    def from_flood_fill(x: int, y: int, array: np.ndarray) -> Letter:
        pixels: list[tuple[int, int]] = [(y, x)]
        index: int = 0

        left: int = x
        right: int = x
        while index < len(pixels):
            for dx, dy in [(dx, dy) for dx in range(-1, 2) for dy in range(-1, 2)]:
                x = pixels[index][1] + dx
                y = pixels[index][0] + dy
                if x < 0 or x >= len(array[0]) or y < 0 or y >= len(array):
                    continue

                if (y, x) not in pixels and array[y, x] <= LETTER_THRESHOLD:
                    pixels.append((y, x))
                    right = max(right, x)
            index += 1
        return Letter(left, right + 1)


class Word:
    # ========================================
    left: int
    right: int

    letters: list[Letter] = None
    # ========================================

    def __init__(self, left: int, right: int):
        self.left = left
        self.right = right

    def set_letters(self, array: np.ndarray):
        if self.letters is not None:
            return

        self.letters = []
        x: int = self.left
        while x < self.right:
            for y in range(len(array)):
                if array[y, x] <= LETTER_THRESHOLD:
                    self.letters.append(Letter.from_flood_fill(x, y, array))
                    x = self.letters[-1].right
                    break
            x += 1

    def get_letters(self, array: np.ndarray, line_index: int = 0, word_index: int = 0):
        letters = []
        x: int = self.left
        while x < self.right:
            for y in range(len(array)):
                if array[y, x] <= LETTER_THRESHOLD:
                    letters.append(Letter.from_flood_fill(x, y, array))
                    x = letters[-1].right
                    break
            x += 1

        return (letters, line_index, word_index)


class Line:
    # ========================================
    top: int
    bottom: int

    words: list[Word] = None
    # ========================================

    def __init__(self, top: int, bottom: int):
        self.top = top
        self.bottom = bottom

    def set_words(self, array: np.ndarray):
        if self.words is not None:
            return

        self.words = []
        left: int = -1
        right: int = -1
        inWord: bool = False
        for i, column in enumerate(transpose(array)):
            if sum(column) / len(column) == 255:
                if inWord:
                    if right == -1:
                        right = i
                    elif i - right >= LETTER_SPACING:
                        self.words.append(Word(left, right))
                        inWord = False
            else:
                right = -1
                if not inWord:
                    left = i
                    inWord = True
