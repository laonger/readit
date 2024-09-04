use std::fs;
use std::env;
use std::path::Path;

use serde_yml;

use crate::file_utils;
use crate::structs;

pub struct Env {
    pub home_dir: String,
    pub work_dir: String,
    pub temp_dir: String,   // 项目配置，db等
    pub config: structs::Config, // 全局配置
}

impl Env {
    pub fn new(path: Option<String>) -> Self {

        let home_dir_string = file_utils::init_home();
        let work_dir_string = match path {
            Some(p) => {
                p
            },
            None => {
                file_utils::init_workdir()
            }
        };
        
        let temp_dir = Path::new(&work_dir_string).join(".readit");
        
        let config_file = Path::new(&home_dir_string).join("config.yaml");
        if !config_file.exists() {
            Self::init_config(config_file.as_path());
        }

        Self {
            home_dir: home_dir_string,
            work_dir: work_dir_string,
            temp_dir: temp_dir.to_str().unwrap().to_string(),
            config: file_utils::read_config_file(config_file.as_path()),
        }
    }

    pub fn is_new(&self) -> bool {
        if Path::new(&self.temp_dir).join("db").exists() {
            false
        } else {
            true
        }
    }

    pub fn init_config(file_path: &Path) {
        let config = structs::Config {
            openai_key      : Some("".to_string()),
            openai_base     : Some("https://api.openai.com/v1".to_string()),
            chat_model      : Some("gpt-4o".to_string()),
            analyse_model   : Some("gpt-4o".to_string()),
            embedding_model : Some("text-embedding-3-large".to_string()),
            dim             : Some(256),
        };
        let config_string = serde_yml::to_string(&config).unwrap();
        fs::write(file_path, config_string).unwrap();
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
