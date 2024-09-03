
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
    pub openai_key: String,
    pub openai_base: String,
    pub chat_model: String,
    pub analyse_model: String,
    pub embedding_model: String,
    pub dim: usize,
}
