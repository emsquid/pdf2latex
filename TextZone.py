from __future__ import annotations

import numpy as np
from utils import transpose

LETTER_SPACING: int = 4
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
    def flood_fill_right(
        x_pos: int, y_pos: int, y_min: int, y_max: int, array: np.ndarray
    ) -> tuple[int, int]:
        list_pos: list[tuple(int, int)] = [(x_pos, y_pos)]
        row_empty: bool = False

        y_pos: int = y_max - 1
        left: int = x_pos
        right: int = x_pos

        positiveStep: bool = False

        while not (row_empty) or y_pos < y_max or not (positiveStep):
            if y_pos >= y_max and positiveStep:
                y_pos = y_max - 1
                x_pos += 1
                row_empty = True
                positiveStep = False
            elif y_pos <= y_min and not (positiveStep):
                y_pos = y_min
                positiveStep = True

            if array[x_pos, y_pos] <= LETTER_THRESHOLD:
                connected: bool = (
                    (x_pos, y_pos + 1) in list_pos
                    or (x_pos, y_pos - 1) in list_pos
                    or (x_pos + 1, y_pos) in list_pos
                    or (x_pos - 1, y_pos) in list_pos
                    or (x_pos + 1, y_pos + 1) in list_pos
                    or (x_pos - 1, y_pos + 1) in list_pos
                    or (x_pos + 1, y_pos - 1) in list_pos
                    or (x_pos - 1, y_pos - 1) in list_pos
                )

                if connected:
                    row_empty = False
                    list_pos.append((x_pos, y_pos))
                    right = max(right, x_pos)

            y_pos += 1 if positiveStep else -1

        return left, right


class Word:
    # ========================================
    left: int
    right: int

    letters: list[Letter]
    # ========================================

    def __init__(self, left: int, right: int):
        self.left = left
        self.right = right

    def set_letters(self, array: np.ndarray):
        self.letters = []
        array = transpose(array)

        x: int = self.left
        y: int = 0
        while x < self.right:
            if array[x, y] <= LETTER_THRESHOLD:
                l, r = Letter.flood_fill_right(x, y, 0, len(array[0]), array)
                if l != r:
                    self.letters.append(Letter(l, r + 1))
                x = r + 1
                y = 0
            else:
                y += 1
                if y >= len(array[0]):
                    y = 0
                    x += 1


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
        self.words = []
        array = array[self.top : self.bottom].T

        left: int = -1
        right: int = -1
        inWord: bool = False
        for i, line in enumerate(array):
            if sum(line) / len(line) == 255:
                if inWord:
                    if right == -1:
                        right = i
                    elif i - right >= LETTER_SPACING:
                        self.words.append(Word(left, right))
                        inWord = False

            else:
                right = -1
                if not (inWord):
                    left = i
                    inWord = True

    def set_letters(self, array: np.ndarray):
        if self.words == None:
            self.set_words(array)

        for word in self.words:
            word.set_letters(array[self.top : self.bottom])
