#!/bin/bash

pwd=$(pwd)
base="."
if [[ "$(basename "$pwd")" != "python" ]];then
    base="python"
fi
pythonEnv="$base/python-env"

if [[ ! -d "$pythonEnv" ]]; then
    echo "Creating env directory..."
    python3 -m venv "$pythonEnv"
fi
. "$pythonEnv/bin/activate"
python3 -m pip install -r requirements.txt
