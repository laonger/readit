use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::env;

use home;

use serde::{Deserialize, Serialize};
use serde_yml;

use crate::ignore_rules::Ignore;
use crate::language_extensions::{
    LanguageExtensions,
    FileExtensionList,
    FileExtensionTypeMap
};


fn file_filter(file: &PathBuf, ignore: &Ignore, file_ext_list: &FileExtensionList) -> bool {
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


fn source_language(file: &PathBuf, file_extension_map: &FileExtensionTypeMap) -> String {
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
    language_extensions: &LanguageExtensions,
) {
    match fs::read_dir(path) {
        Err(e) => println!("Error: {}", e),
        Ok(paths) => {
            for p in paths {
                let p = p.unwrap().path();
                if !file_filter(&p, ignore, &language_extensions.ext_list) {
                    continue;
                }
                if p.is_dir() {
                    list_path(&p, file_list, ignore, language_extensions);
                } else {
                    let lang = source_language(&p, &language_extensions.ext_type_map);
                    file_list.push((p, lang));
                }
            }
        }
    }
}

pub fn home_dir() -> (String, bool) {
    let _home_dir_p = home::home_dir().unwrap();
    let home_dir = _home_dir_p.as_path().join(".readit");
    (
        home_dir.to_str().unwrap().to_string(),
        home_dir.as_path().exists()
    )
}

pub fn init_home(
    home_dir: &String,
) {

    let ignore_rules: Ignore = Ignore::new();
    let language_extensions: LanguageExtensions = LanguageExtensions::new();

    let home_dir = Path::new(home_dir);

    if !home_dir.exists() {
        fs::create_dir(home_dir).unwrap();
    }
    if !home_dir.join("ignore_rules.yaml").exists() {
        ignore_rules.write_to_file(
            home_dir.join("ignore_rules.yaml").as_path()
        )
            
    }
    if !home_dir.join("language_extensions.yaml").exists() {
        // TODO
        language_extensions.write_to_file(
            home_dir.join("language_extensions.yaml").as_path()
        );
    };
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
