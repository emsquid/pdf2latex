from PDF import PDF
from RandomPDF import randomPDF
import os

if __name__ == "__main__":
    # randomPDF("test/test_2_toLatex.pdf")
    # test = PDF(os.path.dirname(os.path.realpath(__file__)) + "/temp/random.pdf")
    test = PDF("test/test_1_toLatex.pdf")
    test.pages[0].show_letters()