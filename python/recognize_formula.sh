#!/bin/bash

pwd=$(pwd)
if [[ "$(basename "$pwd")" != "pdf2latex" ]];then
    echo "Error: Please go to the root dir or rename the current root dir"
    exit
fi
if [[ ! -d "python/python-env" ]]; then
    pix2tex "$1"
    echo "Error: Please setup python by running setup_python_ia.py"
    exit
fi

if test "$1" == "";then 
    echo "Error: Please provide a path to the image"
    exit
fi

if test ! -f "$1";then 
    echo "Error: Please provide a valid path"
    exit
fi

. python/python-env/bin/activate
pix2tex "$1"
