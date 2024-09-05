use std::fs;
use std::env;
use std::path::Path;

use serde_yml;

use crate::file_utils;
use crate::config;
use crate::ignore_rules::Ignore;
use crate::language_extensions::{
    LanguageExtensions,
    FileExtensionList,
    FileExtensionTypeMap
};

#[derive(Debug, Clone)]
pub struct Env {
    pub home_dir: String,
    pub work_dir: String,
    pub temp_dir: String,   // 项目配置，db等
    pub config: config::Config, // 全局配置
    pub ignore: Ignore,
    pub language_extensions: LanguageExtensions,
}

impl Env {
    pub fn new(path: Option<String>) -> Self {

        let (home_dir_string, home_exist) = file_utils::home_dir();
        if !home_exist {
            file_utils::init_home(&home_dir_string);
        }
        let home_dir = Path::new(&home_dir_string);

        let work_dir_string = match path {
            Some(p) => {
                p
            },
            None => {
                file_utils::init_workdir()
            }
        };
        let work_dir = Path::new(&work_dir_string);
        
        let temp_dir = work_dir.join(".readit");
        
        let config_file = home_dir.join("config.yaml");

        // TODO 挪出去
        if !config_file.exists() {
            config::Config::init_config(config_file.as_path());
        }

        
        let ignore = Ignore::new_from_path(
            home_dir.join("ignore_rules.yaml").as_path()
        );
        let language_extensions = LanguageExtensions::new_from_path(
            home_dir.join("language_extensions.yaml").as_path()
        );

        Self {
            home_dir: home_dir_string,
            work_dir: work_dir_string,
            temp_dir: temp_dir.to_str().unwrap().to_string(),
            config: config::Config::new_from_path(config_file.as_path()),
            ignore,
            language_extensions,
        }
    }

    /// new project
    pub fn is_new_project(&self) -> bool {
        if Path::new(&self.temp_dir).join("db").exists() {
            false
        } else {
            true
        }
    }

    pub fn home_dir(&self) -> &Path {
        Path::new(&self.home_dir)
    }

    pub fn work_dir(&self) -> &Path {
        Path::new(&self.work_dir)
    }

    pub fn openai_key(&self) -> String {
        match env::var("OPENAI_KEY") {
            Ok(val) => val,
            Err(_) => {
                self.config.openai_key.clone().unwrap_or("".to_string())
            }
        }
    }

    pub fn check_openai_key(&self) -> bool {
        match self.openai_key().as_str() {
            "" => false,
            _ => true,
        }
    }

    pub fn openai_base(&self) -> String {
        match env::var("OPENAI_BASE") {
            Ok(val) => val,
            Err(_) => {
                if self.config.openai_base().is_empty() {
                    panic!("openai_base is not set in the environment")
                } else {
                    self.config.openai_base()
                }
            }
        }
    }

    pub fn dim(&self) -> usize {
        if self.config.dim() == 0 {
            256
        } else {
            self.config.dim()
        }
    }

}
