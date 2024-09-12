use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_yml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub openai_key: Option<String>,
    pub openai_base: Option<String>,
    pub chat_model: Option<String>,
    pub analyse_model: Option<String>,
    pub embedding_model: Option<String>,
    pub dim: Option<usize>,
    pub language: Option<String>,
}

impl Config {
    pub fn openai_key(&self) -> String {
        self.openai_key.clone().unwrap()
    }
    pub fn openai_base(&self) -> String {
        self.openai_base.clone().unwrap_or("https://api.openai.com/v1".to_string())
    }
    pub fn chat_model(&self) -> String {
        self.chat_model.clone().unwrap_or("gpt-4o".to_string())
    }
    pub fn analyse_model(&self) -> String {
        self.analyse_model.clone().unwrap_or("gpt-4o".to_string())
    }
    pub fn embedding_model(&self) -> String {
        self.embedding_model.clone().unwrap_or("text-embedding-3-large".to_string())
    }
    pub fn dim(&self) -> usize {
        //self.dim.unwrap_or(265)
        self.dim.unwrap_or(1024)
    }

    pub fn language(&self) -> String {
        self.language.clone().unwrap_or("".to_string())
    }

    pub fn new_from_path(path: &Path) -> Self {
        let file = fs::read_to_string(path).unwrap();
        let config:Self  = serde_yml::from_str(&file).unwrap();
        config
    }

    pub fn save(&self, path: &Path) {
        let config_string = serde_yml::to_string(&self).unwrap();
        fs::write(path, config_string).unwrap();
    }

    pub fn init_config(file_path: &Path) -> Self {
        let config = Self {
            openai_key      : Some("".to_string()),
            openai_base     : Some("https://api.openai.com/v1".to_string()),
            chat_model      : Some("gpt-4o".to_string()),
            analyse_model   : Some("gpt-4o".to_string()),
            embedding_model : Some("text-embedding-3-large".to_string()),
            dim             : Some(256),
            language        : Some("".to_string()),
        };
        let config_string = serde_yml::to_string(&config).unwrap();
        fs::write(file_path, config_string).unwrap();
        config
    }
}

