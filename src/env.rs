use std::env;
use std::path::Path;


use crate::file_utils;
use crate::config;
use crate::ignore_rules::Ignore;
use crate::language_extensions::LanguageExtensions;
use crate::history::History;

#[derive(Debug, Clone)]
pub struct Env {
    pub home_dir: String,
    pub work_dir: String,
    pub temp_dir: String,   // 项目配置，db等
    pub history: History,
    pub config: config::Config, // 全局配置
    pub ignore: Ignore,
    pub language_extensions: LanguageExtensions,
}

impl Env {
    pub fn new(path: Option<String>) -> Self {

        let work_dir_string = match path {
            Some(p) => {
                p
            },
            None => {
                file_utils::get_workdir()
            }
        };
        let work_dir = Path::new(&work_dir_string);
        let temp_dir = work_dir.join(".readit");


        Self {
            home_dir: String::from(""),
            work_dir: work_dir_string,
            temp_dir: temp_dir.to_str().unwrap().to_string(),
            history : History::new(),
            config  : config::Config::new(),
            ignore  : Ignore::new(),
            language_extensions: LanguageExtensions::new(),
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

    // TODO
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

}
