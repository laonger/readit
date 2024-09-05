use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_yml;

const IGNORE_RULES: &str = &"directories:
  - .git
  - .github
  - .vscode
  - .idea
  - .gitlab
  - .readit
  - db
  - node_modules
  - __pycache__
  - dist
  - build
  - benchmark
  - bench
  - dockerfiles
  - target
directory_prefix:
  - test
  - Test
directory_posfix:
  - Test
  - test
  - .egg-info
files:
  - README.md
  - LICENSE
  - requirements.txt
  - setup.py
  - Dockerfile
  - NOTICE
  - Makefile
  - Cargo.toml
  - Cargo.lock
  - .gitignore
  - .dockerignore
  - .gitattributes
  - .editorconfig
file_prefix:
  - test
  - Test
file_posfix:
  - Test
  - test
  - .pyc
  - .pyo
  - .swp
  - .log
  - .tmp
  - .bak
  - .tmp
  - .log
  - .cache
  - .egg-info
  - .egg
  - .md
  - .json
  - .yaml
  - .toml
  - .gz
  - .zip
  - .tar.gz
  - .tar.bz2
  - .tar.xz
  - .rar
  - .7z
  - .iso
  - .bin
  - .exe
  - .dll
  - .so
  - .dylib
  - .lib
  - .a
  - .o
  - .class
  - .jar
  - .war
  - .ear
  - .apk
  - .ipa
  - .aab
  - .txt
  - .template
  - .md
  - .png
  - .jpg
  - .jpeg
  - .gif
  - .bmp
  - .tif
  - .tiff
  - .svg
  - .webp
  - .ico
  - .pdf
  - .doc
  - .docx
  - .xls
  - .xlsx
  - .ppt
  - .pptx
  - .csv
  - .tsv
  - .rst
  - .tex
  - .bib
  - .bibtex
  - .Dockerfile
";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ignore {
    pub directories: Vec<String>,
    pub directory_prefix: Vec<String>,
    pub directory_posfix: Vec<String>,
    pub files: Vec<String>,
    pub file_prefix: Vec<String>,
    pub file_posfix: Vec<String>,
}

impl Ignore {
    pub fn new() -> Self {
        serde_yml::from_str(IGNORE_RULES).unwrap()
    }

    pub fn new_from_path(path: &Path) -> Self {
        let ignore_string = fs::read_to_string(path).unwrap();
        serde_yml::from_str(&ignore_string).unwrap()
    }
    
    pub fn write_to_file(&self, path: &Path) {
        let ignore_string = serde_yml::to_string(&self).unwrap();
        fs::write(path, ignore_string).unwrap();
    }
}
