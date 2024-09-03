use std::fs;
use std::env;
use std::path::Path;

use serde_yml;

use crate::file_utils;
use crate::structs;

pub struct Env {
    pub home_dir: String,
    pub work_dir: String,
    pub config: structs::Config,
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

        
        let config_file = Path::new(&home_dir_string).join("config.yaml");
        if !config_file.exists() {
            Self::init_config(config_file.as_path());
        }

        Self {
            home_dir: home_dir_string,
            work_dir: work_dir_string,
            config: file_utils::read_config_file(config_file.as_path()),
        }
    }

    pub fn init_config(file_path: &Path) {
        let config = structs::Config {
            openai_key: "".to_string(),
            openai_base: "https://api.openai.com/v1".to_string(),
            chat_model: "gpt-4o".to_string(),
            analyse_model: "gpt-4o".to_string(),
            embedding_model: "text-embedding-3-large".to_string(),
            dim: 256,
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
                if self.config.openai_key.is_empty() {
                    panic!("OPENAI_KEY is not set in the environment")
                } else {
                    self.config.openai_key.clone()
                }
            }
        }
    }

    pub fn openai_base(&self) -> String {
        match env::var("OPENAI_BASE") {
            Ok(val) => val,
            Err(_) => {
                if self.config.openai_base.is_empty() {
                    panic!("openai_base is not set in the environment")
                } else {
                    self.config.openai_base.clone()
                }
            }
        }
    }

    pub fn dim(&self) -> usize {
        if self.config.dim == 0 {
            256
        } else {
            self.config.dim
        }
    }

}
