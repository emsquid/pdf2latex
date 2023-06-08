from PDF import PDF

if __name__ == "__main__":
    test = PDF("test/test_1_toLatex.pdf")
    test.pages[0].show_letters()
