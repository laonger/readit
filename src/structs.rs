
use serde::{Deserialize, Serialize};
use serde_json;


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

