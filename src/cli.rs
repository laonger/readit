use std::io;

use crate::{
    config::Config,
    env::Env,
    file_utils,
    ignore_rules::Ignore,
    language_extensions::LanguageExtensions,
    structs::InitStep,
};

fn init_step_init_home(_env: &mut Env){
    let (home_dir, _) = file_utils::home_dir();
    _env.home_dir = home_dir;
    file_utils::create_dir(_env.home_dir());
    let config_file = _env.home_dir().join("config.yaml");
    _env.config = Config::init_config(config_file.as_path());
    _env.config.save();
}

fn init_step_make_ignore_rules_file(_env: &mut Env){
    let ignore_rules: Ignore = Ignore::new();
    ignore_rules.write_to_file(
        _env.home_dir().join("ignore_rules.yaml").as_path()
    );
    _env.ignore = ignore_rules;
}

fn init_step_make_language_extensions_file(_env: &mut Env){
    let language_extensions: LanguageExtensions 
            = LanguageExtensions::new();
    language_extensions.write_to_file(
        _env.home_dir().join("language_extensions.yaml").as_path()
    );
    _env.language_extensions = language_extensions;
}

fn init_step_init_work_dir(_env: &mut Env){
    let temp_dir = _env.work_dir().join(".readit");
    file_utils::create_dir(&temp_dir);
}

fn init_step_ask_main_language(_env: &mut Env){
    println!("Please tell me, what language you speak? Default is English.");
    println!("Language: ");
    let mut language = String::new();
    let _ = io::stdin().read_line(&mut language);
    println!("\nThanks");
    let mut language = language.replace("\n", "").replace(" ", "");
    if language.is_empty() {
        language = "English".to_string();
    }
    _env.config.language = Some(language);
    _env.config.save();
}

fn init_step_ask_api_key(_env: &mut Env){

    let api_key = loop {
        let mut api_key = String::new();
        println!("Please tell me, what is your openai api key?");
        println!("Api Key: ");
        let _ = io::stdin().read_line(&mut api_key);
        println!("\nThanks");
        let api_key = api_key.replace("\n", "").replace(" ", "");
        if api_key.is_empty() {
            continue;
        }
        break api_key;
    };

    _env.config.openai_key = Some(api_key);
    _env.config.save();
}

fn init_step_set_api_key(_env: &mut Env){
    let api_key = _env.openai_key();
    _env.config.openai_key = Some(api_key);
    _env.config.save();
}

pub fn init(_env: &mut Env, init_step: Vec<InitStep>) -> &Env{
    for step in init_step {
        match step {
            InitStep::InitHome => {
                init_step_init_home(_env);
            },
            InitStep::MakeIgnoreFile => {
                init_step_make_ignore_rules_file(_env);
            },
            InitStep::MakeLanguageExtensionsFile => {
                init_step_make_language_extensions_file(_env);
            },
            InitStep::AskMainLanguage => {
                init_step_ask_main_language(_env);
            },
            InitStep::AskApiKey => {
                init_step_ask_api_key(_env);
            },
            InitStep::SetApiKey => {
                init_step_set_api_key(_env);
            },
            InitStep::InitWorkDir => {
                init_step_init_work_dir(_env);
            },
            _ => {
            }
        }
    };
    _env
}

