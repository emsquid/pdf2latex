from __future__ import annotations

import numpy as np
from PIL import Image, ImageOps
from pdf2image import convert_from_path

LETTER_SPACE: int = 4
LETTER_THRESHOLD: int = 180

class Letter:
    # ========================================
    left: int
    right: int

    pixels: list[tuple[int, int]]
    # ========================================
    
    def __init__(self, left: int, right: int):
        self.left = left
        self.right = right
        
    
    @staticmethod
    def flood_fill(x_pos: int, y_pos: int, array: np.ndarray) -> tuple[int, int, list[tuple[int, int]]]:
        list_pos: list[tuple(int, int)] = [(x_pos, y_pos)]
        last_index: int = 0

        left: int = x_pos
        right: int = x_pos

        while last_index < len(list_pos):
            for x_, y_ in [(x, y) for x in range(-1, 2) for y in range(-1, 2) if (x != 0 or y != 0)]:
                x_pos = list_pos[last_index][0] + x_
                y_pos = list_pos[last_index][1] + y_
                if x_pos < 0 or x_pos >= len(array) or y_pos < 0 or y_pos >= len(array[0]):
                    continue

                if array[x_pos, y_pos] <= LETTER_THRESHOLD and (x_pos, y_pos) not in list_pos:
                    list_pos.append((x_pos, y_pos))
                    right = max(right, x_pos)
            
            last_index += 1
        
        return left, right, list_pos

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
        array = array.T

        x: int = self.left
        while x < self.right:
            for y in range(len(array[0])):
                if array[x, y] <= LETTER_THRESHOLD:
                    l, r, pixels = Letter.flood_fill(x, y, array)

                    self.letters.append(Letter(l, r + 1))
                    self.letters[-1].pixels = pixels
                    
                    x = r + 1
                    break
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
        array = array[self.top: self.bottom].T

        start: int = -1
        end: int = -1
        inWord: bool = False
        for i, line in enumerate(array):
            if sum(line) / len(line) == 255: #no black pixel in column
                if inWord:
                    if end == -1:
                        end = i
                    elif (i - end >= LETTER_SPACE):
                        self.words.append(Word(start, end))
                        inWord = False
                
            else:
                end = -1
                if not(inWord):
                    start = i
                    inWord = True
    
    def set_letters(self, array: np.ndarray):
        if self.words == None:
            self.set_words(array)
        
        for word in self.words:
            word.set_letters(array[self.top: self.bottom])