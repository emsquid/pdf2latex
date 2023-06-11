from __future__ import annotations

import os
import argparse
import PyPDF2
from PyPDF2.generic import DecodedStreamObject, EncodedStreamObject, NameObject
import random, string

def replace_text(content: str, replacements = dict()) -> str:
    lines = content.splitlines()

    result = ""
    in_text = False

    for line in lines:
        if line == "BT":
            in_text = True

        elif line == "ET":
            in_text = False

        elif in_text:
            cmd = line[-2:]
            if cmd.lower() == 'tj':
                replaced_line = line
                for k, v in replacements.items():
                    replaced_line = replaced_line.replace(k, v)
                result += replaced_line + "\n"
            else:
                result += line + "\n"
            continue

        result += line + "\n"

    return result

def process_data(object: DecodedStreamObject | EncodedStreamObject, replacements):
    data = object.get_data()
    decoded_data = data.decode('utf-8')

    replaced_data = replace_text(decoded_data, replacements)

    encoded_data = replaced_data.encode('utf-8')
    if object.decoded_self is not None:
        object.decoded_self.set_data(encoded_data)
    else:
        object.set_data(encoded_data)

def randomPDF(parent_path: str):
    pdf = PyPDF2.PdfReader(parent_path)
    writer = PyPDF2.PdfWriter()

    replacements = {}
    for word in pdf.pages[0].extract_text().replace("\n", " ").split(" "):
        if len(word) <= 3 or not word.isalpha():
            continue
        replacements[word] = "".join([random.choice(string.ascii_letters) for i in range(len(word))])

    for page_number in range(0, len(pdf.pages)):

        page = pdf.pages[page_number]
        contents = page.get_contents()

        if isinstance(contents, DecodedStreamObject) or isinstance(contents, EncodedStreamObject):
            process_data(contents, replacements)
        elif len(contents) > 0:
            for obj in contents:
                if isinstance(obj, DecodedStreamObject) or isinstance(obj, EncodedStreamObject):
                    streamObj = obj.getObject()
                    process_data(streamObj, replacements)

        page[NameObject("/Contents")] = contents.decoded_self
        writer.add_page(page)

    with open(os.path.dirname(os.path.realpath(__file__)) + "/temp/random.pdf", 'wb') as out_file:
        writer.write(out_file)