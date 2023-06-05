from PDF import Page, PDF

test_1 = PDF("test/test_1_toLatex.png")
test_1.pages[0].show_threshold()
test_1.pages[0].show_letters()
# test_1.pages[0].show_columns()
# test_1.pages[0].show()