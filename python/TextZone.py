from __future__ import annotations

import numpy as np
from utils import transpose

LETTER_SPACING: int = 5
LETTER_THRESHOLD: int = 180


class Letter:
    # ========================================
    left: int
    right: int
    pixels: list[tuple[tuple[int, int], float]]
    # ========================================

    def __init__(self, left: int, right: int, pixels: list[tuple[tuple[int, int], float]]):
        self.left = left
        self.right = right
        self.pixels = pixels

    @staticmethod
    def from_flood_fill(x: int, y: int, array: np.ndarray) -> Letter:
        pixels: list[tuple[tuple[int, int], float]] = [((y, x), array[y, x])]
        index: int = 0

        left: int = x
        right: int = x
        while index < len(pixels):
            for dx, dy in [(dx, dy) for dx in range(-1, 2) for dy in range(-1, 2)]:
                x = pixels[index][0][1] + dx
                y = pixels[index][0][0] + dy
                if x < 0 or x >= len(array[0]) or y < 0 or y >= len(array):
                    continue
                if ((y, x), array[y, x]) in pixels or array[y, x] >= 255:
                    continue

                if array[y, x] <= LETTER_THRESHOLD:
                    pixels.append(((y, x), array[y, x]))
                else:
                    pixels.insert(0, ((y, x), array[y, x]))
                    index += 1
                
                left = min(left, x)
                right = max(right, x)

            index += 1
        return Letter(left, right + 1, pixels)
    
    def clean_up_pixels(self, array: np.ndarray) -> Letter:
        outside = [((y, x), array[y, x]) for x in [self.left - 2, self.right + 3] for y in range(0, len(array)) if array[y, x] <= LETTER_THRESHOLD and x >= 0 and x < len(array[0])]
        index = 0
        while index < len(outside):
            for dx, dy in [(dx, dy) for dx in range(-1, 2) for dy in range(-1, 2)]:
                x = outside[index][0][1] + dx
                y = outside[index][0][0] + dy
                if x < self.left - 2 or x > self.right + 2 or y < 0 or y >= len(array):
                    continue
                if ((y, x), array[y, x]) in outside or array[y, x] >= 255:
                    continue

                if array[y, x] <= LETTER_THRESHOLD:
                    outside.append(((y, x), array[y, x]))
                else:
                    outside.insert(0, ((y, x), array[y, x]))
                    index += 1

            index += 1
            
        
        self.pixels = [
            ((y, x), array[y, x]) for x in range(self.left, self.right + 1) for y in range(0, len(array)) if array[y, x] < 255 and ((y, x), array[y, x]) not in outside
        ]


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
                    self.letters[-1].clean_up_pixels(array)
                    x = self.letters[-1].right
                    break
            else:
                x += 1

    def get_letters(self, array: np.ndarray, line_index: int = 0, word_index: int = 0):
        self.set_letters(array)
        return (self.letters, line_index, word_index)


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

    def get_words(self, array: np.ndarray, line_index: int = 0):
        self.set_words(array)
        return (self.words, line_index)
