use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::env;

use home;

use serde::{Deserialize, Serialize};
use serde_yml;

use crate::structs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ignore {
    directories: Vec<String>,
    directory_prefix: Vec<String>,
    directory_posfix: Vec<String>,
    files: Vec<String>,
    file_prefix: Vec<String>,
    file_posfix: Vec<String>,
}

pub type FileExtension = Vec<String>;
pub type FileExtensionMap = HashMap<String, String>;

pub fn read_ignore_file(path: &Path) -> Ignore {
    let file = File::open(path).unwrap();
    let ignore: Ignore = serde_yml::from_reader(file).unwrap();
    ignore
}

pub fn read_file_extension(path: &Path) -> (FileExtension, FileExtensionMap){
    let file = File::open(path).unwrap();
    let _file_extension: HashMap<String, Vec<String>> 
        = serde_yml::from_reader(file).unwrap();

    let mut file_extension_map: FileExtensionMap = HashMap::new();
    for (k, v) in _file_extension.iter() {
        for i in v.iter() {
            file_extension_map.insert(i.clone(), k.clone());
        }
    }
    (
        file_extension_map.clone().into_keys().collect::<FileExtension>(),
        file_extension_map
    )
}

fn file_filter(file: &PathBuf, ignore: &Ignore, file_ext_list: &FileExtension) -> bool {
    let file_name = file.file_name().unwrap().to_str().unwrap();
    if ignore.directories.contains(&file_name.to_string()) {
        //println!("1, file_name: {:?}", file_name);
        return false;
    }
    if ignore.directories.iter().any(|x| {
        let p = Path::new(x);
        match p.canonicalize() {
            Ok(p) => file.canonicalize().unwrap() == p,
            Err(_) => false
        }
    }) {
        //println!("1.1, file_name: {:?}", file_name);
        return false;
    }
    if ignore.directory_prefix.iter().any(|x| file_name.starts_with(x)) {
        //println!("2, file_name: {:?}", file_name);
        return false;
    }
    if ignore.directory_posfix.iter().any(|x| file_name.ends_with(x)) {
        //println!("3, file_name: {:?}", file_name);
        return false;
    }

    if ignore.files.contains(&file_name.to_string()) {
        //println!("4, file_name: {:?}", file_name);
        return false;
    }
    if ignore.files.iter().any(|x| {
        let p = Path::new(x);
        match p.canonicalize() {
            Ok(p) => file.canonicalize().unwrap() == p,
            Err(_) => false
        }
    }) {
        //println!("4.1, file_name: {:?}", file_name);
        return false;
    }
    if ignore.file_prefix.iter().any(|x| file_name.starts_with(x)) {
        //println!("5, file_name: {:?}", file_name);
        return false;
    }
    if ignore.file_posfix.iter().any(|x| file_name.ends_with(x)) {
        //println!("6, file_name: {:?}", file_name);
        return false;
    }


    if file.is_dir() {
        //println!("7, file_name: {:?}", file_name);
        return true;
    }
    //println!("8, file_name: {:?}", file_name);
    match file.extension() {
        None => return false,
        Some(ext) => {
            let ext = &ext.to_str().unwrap().to_string();
            let ext = format!(".{}", ext);
            //println!("9, file_name: {:?}", ext);
            if file_ext_list.contains(&ext) {
                //println!("ext: {:?}", ext);
                return true;
            }
        }
    }
    false
}

pub fn read_config_file(path: &Path) -> structs::Config {
    let file = File::open(path).unwrap();
    let config: structs::Config = serde_yml::from_reader(file).unwrap();
    config
}

fn source_language(file: &PathBuf, file_extension_map: &FileExtensionMap) -> String {
    let ext = file.extension().unwrap().to_str().unwrap();
    let ext = format!(".{}", ext);
    match file_extension_map.get(&ext) {
        None => "unknown".to_string(),
        Some(lang) => lang.clone()
    }
}

pub fn list_path(
    path: &Path,
    file_list: &mut Vec<(PathBuf, String)>,
    ignore: &Ignore,
    file_extension: &FileExtension,
    file_extension_map: &FileExtensionMap
) {
    match fs::read_dir(path) {
        Err(e) => println!("Error: {}", e),
        Ok(paths) => {
            for p in paths {
                let p = p.unwrap().path();
                if !file_filter(&p, ignore, file_extension) {
                    continue;
                }
                if p.is_dir() {
                    list_path(&p, file_list, ignore, file_extension, file_extension_map);
                } else {
                    let lang = source_language(&p, &file_extension_map);
                    file_list.push((p, lang));
                }
            }
        }
    }
}

pub fn init_home() -> String {
    let _home_dir_p = home::home_dir().unwrap();
    let home_dir = _home_dir_p.as_path().join(".readit");
    if !home_dir.exists() {
        fs::create_dir(home_dir.clone()).unwrap();
    }
    if !home_dir.join("ignore_rules.yaml").exists() {
        // TODO
        fs::copy(
            "/Users/laonger/Work/self/readit/ignore_rules.yaml",
            home_dir.join("ignore_rules.yaml")
        ).unwrap();
    }
    if !home_dir.join("language_extensions.yaml").exists() {
        // TODO
        fs::copy(
            "/Users/laonger/Work/self/readit/language_extensions.yaml",
            home_dir.join("language_extensions.yaml")
        ).unwrap();
    };
    home_dir.to_str().unwrap().to_string()
}

pub fn init_workdir() -> String {

    let _work_dir_p = env::current_dir().unwrap();
    let work_dir = _work_dir_p.as_path();
    let _work_dir = work_dir.join(".readit");
    if !_work_dir.exists() {
        fs::create_dir(_work_dir).unwrap();
    };
    work_dir.to_str().unwrap().to_string()
}
