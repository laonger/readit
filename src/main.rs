use std::io;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{stdout, Write, Read};
use futures::{Future, StreamExt};
//use std::env as std_env;
use std::sync::{Arc, Mutex};

use std::env as std_env;

use env_logger::Builder;


use clap::{
    Parser,
    Args,
    Subcommand,
    Command,

};

use md5;

use tokio;
use tokio::task::JoinSet;
use tokio::{runtime::Handle, task};

mod errors;

mod openai_utils;
use openai_utils::OpenAI;

mod file_utils;

mod prompt_string;
mod prompt_utils;

mod pooling;

mod embeding_utils;
use embeding_utils::Embedding;

mod structs;
use structs::InitStep;

mod ignore_rules;
mod language_extensions;

mod env;

mod config;

mod controler;

mod cli;
mod tui;

fn log_init() {
    let file = File::create("app.log").expect("Failed to create log file");
    let file = Mutex::new(file); // 使用 Mutex 保证线程安全

    // 初始化 env_logger 并设置输出目标
    Builder::new()
        .format(move |buf, record| {
            let mut file = file.lock().unwrap();
            writeln!(file, "{} - {}", record.level(), record.args())
            //writeln!(buf, "{} - {}", record.level(), record.args()) // 同时输出到控制台
        })
        .filter_level(log::LevelFilter::Info) // 设置日志级别
        .init();
}



/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {

    #[command(subcommand)]
    command: Option<Commands>,

    /// project path
    #[arg(short, long)]
    path: Option<String>,

}

#[derive(Subcommand, Debug)]
enum Commands {
    /// init
    Init,

    /// ask something
    Ask(AskArgs),
    
}

#[derive(Args, Debug)]
struct InitArgs {
    /// force init
    #[arg(short, long)]
    force: bool,
}

#[derive(Args, Debug)]
struct AskArgs {
    /// the question
    query: String,
}

// check env
fn check_and_load_env(path: Option<String>) -> (Vec<InitStep>, env::Env){

    let mut _env = env::Env::new(path.clone());
    let mut init_steps: Vec<InitStep> = Vec::new();

    let (home_dir_string, home_exist) = file_utils::home_dir();

    //let mut has_api_key = false;

    let has_api_key = match std_env::var("OPENAI_KEY") {
        Ok(_) => {
            true
        },
        Err(_) => {
            false
        }
    };

    let home_dir = Path::new(&home_dir_string);
    
    if !home_exist {
        //file_utils::init_home(&home_dir_string);
        init_steps.push(InitStep::InitHome);
        init_steps.push(InitStep::AskMainLanguage);
        if !has_api_key {
            init_steps.push(InitStep::AskApiKey);
        };
        init_steps.push(InitStep::SetApiKey);
    } else {

        let config_file = home_dir.join("config.yaml");
        let config = config::Config::new_from_path(config_file.as_path());

        _env.home_dir = home_dir_string.clone();

        if config.openai_key().is_empty() && !has_api_key {
            init_steps.push(InitStep::AskApiKey);
            init_steps.push(InitStep::SetApiKey);
        }
        if config.language().is_empty() {
            init_steps.push(InitStep::AskMainLanguage);
        }

        _env.config = config;
    }

    if !home_dir.join("ignore_rules.yaml").exists() {
        init_steps.push(InitStep::MakeIgnoreFile);
    } else {
        let ignore_rules = ignore_rules::Ignore::new_from_path(
            home_dir.join("ignore_rules.yaml").as_path()
        );
        _env.ignore = ignore_rules;
    };
    if !home_dir.join("language_extensions.yaml").exists() {
        init_steps.push(InitStep::MakeLanguageExtensionsFile);
    } else {
        let language_extensions = language_extensions::LanguageExtensions::new_from_path(
            home_dir.join("language_extensions.yaml").as_path()
        );
        _env.language_extensions = language_extensions;
    }
    let temp_dir = Path::new(&_env.temp_dir);
    if !temp_dir.exists() {
        init_steps.push(InitStep::InitWorkDir);
    };

    //env.check_openai_key;
    (init_steps, _env)
}


#[tokio::main]
async fn main() {

    log_init();

    let command = Cli::parse();
    //println!("{:?}", command);
    //

    let path = command.path;

    //let mut _env = env::Env::new(path.clone());

    //if !_env.check_openai_key() {
    //    println!("Please set openai key first, \nrun \"export OPENAI_KEY=your_openai_key\" in your shell, \nor set openai_key in $HOME/.readit/config.yaml \nyou can run \"readit -h \" for help.");
    //    return
    //}
    //

    let (init_steps, mut _env) = check_and_load_env(path.clone());
    //println!("steps: {:?}", init_steps);

    match command.command {
        Some(Commands::Init) => {
            controler::force_init(_env).await;
        },
        Some(Commands::Ask(args)) => {
            
            //controler::init(_env.clone()).await;

            cli::init(&mut _env, init_steps);

            let query = args.query.clone();

            cli::ask(_env, query).await;
            //println!("{}", res);
            //println!("tokens usage: {:?}", a_tokens+e_tokens);
        },
        None => {
            tui::run_ui(_env).await;
        }
    };
}
