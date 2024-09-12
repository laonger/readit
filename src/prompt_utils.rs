use minijinja::{Environment, context};

use crate::prompt_string;

pub fn analyse_source_file_prompt(programming_language: String, code: String, language: String) -> String {

    let mut env = Environment::new();
    env.add_template("t", prompt_string::ANALYSE_SOURCE_FILE).unwrap();
    let tmpl = env.get_template("t").unwrap();
    let p = tmpl.render(context!(
        programming_language => programming_language,
        code => code,
    )).unwrap();

    format!("{}\n\nMake sure all the output contents are in {}.", p, language)
}

pub fn split_source_file_prompt(programming_language: String, code: String, language: String) -> String {
    let mut env = Environment::new();
    env.add_template("t", prompt_string::SPLIT_SOURCE_FILE).unwrap();
    let tmpl = env.get_template("t").unwrap();
    let p = tmpl.render(context!(
        programming_language => programming_language,
        code => code,
    )).unwrap();

    format!("{}\n\nMake sure all the output contents are in {}.", p, language)
}

pub fn ask_prompt(query: String, code_list: Vec<String>, language: String) -> String {
    let mut env = Environment::new();
    env.add_template("t", prompt_string::CHAT_WITH_RELATED_SOURCE_FILES).unwrap();
    let tmpl = env.get_template("t").unwrap();
    let p = tmpl.render(context!(
        query => query,
        content_list => code_list,
    )).unwrap();

    format!("{}\n\nMake sure all the output contents are in {}.", p, language)
}

pub fn summarize_prompt(query: String, language: String) -> String{
    let mut env = Environment::new();
    env.add_template("t", prompt_string::SUMMARIZE_PROJECT).unwrap();
    let tmpl = env.get_template("t").unwrap();
    let p = tmpl.render(context!(
        query => query,
    )).unwrap();

    format!("{}\n\nMake sure all the output contents are in {}.", p, language)
    
}
