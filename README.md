# pdf2latex 🔁

pdf2latex is a CLI tool to convert a PDF back to LaTeX.

## Installation

### From source (Recommended)

Prerequisites
- [Git](https://git-scm.com/downloads)
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Poppler](https://poppler.freedesktop.org)

Command line instructions
```bash
# Clone the repository
git clone https://github.com/emsquid/pdf2latex

# Build and install
cargo install --path pdf2latex

# Use freely
pdf2latex super_cool.pdf -o cooler.tex
```

## Notes 

- The project currently requires `json` font files to recognize characters, ask us! 
  These files should be placed in the following directories
  
  |Platform | Value                                           | Example                                            |
  | ------- | ----------------------------------------------- | -------------------------------------------------- |
  | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config/pdf2latex | /home/alice/.config/pdf2latex                      |
  | macOS   | `$HOME`/Library/Application Support/pdf2latex   | /Users/Alice/Library/Application Support/pdf2latex |
  | Windows | `{FOLDERID_RoamingAppData}`\pdf2latex           | C:\Users\Alice\AppData\Roaming\pdf2latex           |
 
## Command line usage

```
Usage: pdf2latex [OPTIONS] <INPUT>

Arguments:
  <INPUT>  PDF to convert

Options:
  -o, --output <OUTPUT>  Output file
  -c, --create <CREATE>  Create font files [possible values: cmr, lmr, put, qag, qcr, qcs, qpl]
  -s, --silent           Silent mode
  -h, --help             Print help
  -V, --version          Print version
```

## Progress

- Documentation
    * [ ] Write a greater README
    * [ ] Make releases/packages (publish on crates.io)
- Miscellaneous
    * [ ] Show cooler log/error messages
    * [ ] Write tests (I guess I need to do that...)
