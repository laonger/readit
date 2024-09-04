
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yml;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDescription {
    pub name: String,
    pub md5: Option<String>,
    pub source_code: String,
    pub purpose: String,
    pub lang: Option<String>,
    pub file: Option<String>,
    pub code_type: Option<String>, // "file", "class", "function"
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPTResponse {
    pub purpose: String,
    pub classes: Vec<CodeDescription>,
    pub functions: Vec<CodeDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSplit {
    pub name: String,
    pub source_code: String,
    pub code_type: Option<String>, // "file", "class", "function"
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPTCodeSplitResponse {
    pub classes: Vec<CodeSplit>,
    pub functions: Vec<CodeSplit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub openai_key: Option<String>,
    pub openai_base: Option<String>,
    pub chat_model: Option<String>,
    pub analyse_model: Option<String>,
    pub embedding_model: Option<String>,
    pub dim: Option<usize>,
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
        self.dim.unwrap_or(265)
    }
}
