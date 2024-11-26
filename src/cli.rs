use std::io;

use crate::{
    controler,
    env::Env,
};

use crate::init::{
    InitStep,
    init_step_init_home,
    init_step_make_ignore_rules_file,
    init_step_make_language_extensions_file,
    init_step_init_work_dir,
    init_step_set_api_key,
};


async fn init_step_ask_main_language(_env: &mut Env){
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

async fn init_step_ask_api_key(_env: &mut Env){

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

async fn init_step_embedding(_env: &mut Env){
    let _ = controler::init((*_env).clone()).await;
}


pub async fn init(_env: &mut Env, init_step: Vec<InitStep>) -> &Env{
    println!("init, {:?}", init_step);
    for step in init_step {
        let _ = match step {
            InitStep::InitHome => {
                init_step_init_home(_env).await;
            },
            InitStep::MakeIgnoreFile => {
                init_step_make_ignore_rules_file(_env).await;
            },
            InitStep::MakeLanguageExtensionsFile => {
                init_step_make_language_extensions_file(_env).await;
            },
            InitStep::AskMainLanguage => {
                init_step_ask_main_language(_env).await;
            },
            InitStep::AskApiKey => {
                init_step_ask_api_key(_env).await;
            },
            InitStep::SetApiKey => {
                init_step_set_api_key(_env).await;
            },
            InitStep::InitWorkDir => {
                init_step_init_work_dir(_env).await;
            },
            InitStep::Embedding => {
                init_step_embedding(_env).await;
            },
            _ => {
            }
        };
    };
    _env
}

pub async fn ask(_env: Env, query: String) {
    let _ = controler::ask(_env, query).await;
}
