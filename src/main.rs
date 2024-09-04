use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
//use std::env as std_env;

use clap::{
    Parser,
    Args,
    Subcommand,
    Command,

};

use md5;

use tokio;

mod openai_utils;
use openai_utils::OpenAI;

mod file_utils;

mod prompt_string;
mod prompt_utils;

mod embeding_utils;
use embeding_utils::Embedding;

mod structs;
mod ignore_rules;
mod language_extensions;

mod env;
mod config;


async fn init<'a> (
    env: &'a env::Env,
    client: &'a OpenAI,
    force: Option<bool>,
) -> Embedding <'a> {

    let embedding_obj = Embedding::new(
        env, client
    ).await.unwrap();

    let path = env.work_dir();

    let mut file_list: Vec<(PathBuf, String)> = Vec::new();
    file_utils::list_path(
        path, &mut file_list, &env.ignore, &env.language_extensions
    );

    for (f, programming_lang) in file_list.iter() {

        let f_path = f.canonicalize().unwrap().to_str().unwrap().to_string();

        println!("f: {:?}", f_path);

        //let (f, programming_lang) = file_list.get(0).unwrap();
        let mut code = String::new();
        let _ = File::open(f).unwrap().read_to_string(&mut code);

        let md5_value = format!("{:x}", md5::compute(code.clone()));

        if !force.unwrap_or(false) {

            if !embedding_obj.is_file_change(&f_path, &md5_value).await.unwrap() {
                continue;
            }
        }

        let (response, a_tockens) = client.analyse_source(
            code.clone(), programming_lang.to_string(), "中文".to_string()
        ).await.unwrap();

        let mut e_tokens = embedding_obj.add_data(structs::CodeDescription {
            //line_number: 0,
            //lines: 0,
            file: Some(f_path.clone()),
            md5: Some(md5_value.clone()),
            code_type: Some("file".to_string()),
            lang: Some(programming_lang.to_string()),
            name: f.to_str().unwrap().to_string(),
            purpose: response.clone().purpose,
            source_code: code,
        }).await.unwrap();

        for c in response.classes {
            //println!("c: {:?}", c);
            e_tokens += embedding_obj.add_data(structs::CodeDescription {
                file: Some(f_path.clone()),
                md5: Some(md5_value.clone()),
                code_type: Some("class".to_string()),
                lang: Some(programming_lang.to_string()),
                name: c.name,
                purpose: c.purpose,
                source_code: c.source_code,
            }).await.unwrap();
        };
        for c in response.functions {
            //println!("c: {:?}", c);
            e_tokens += embedding_obj.add_data(structs::CodeDescription {
                file: Some(f_path.clone()),
                md5: Some(md5_value.clone()),
                code_type: Some("function".to_string()),
                lang: Some(programming_lang.to_string()),
                name: c.name,
                purpose: c.purpose,
                source_code: c.source_code,
            }).await.unwrap();
        };
        println!(
            "file: {}
            analysing use tokens: {:?}    embedding use tokens: {:?}",
            f_path, a_tockens, e_tokens
        );

    }
    embedding_obj
    
}


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {

    #[command(subcommand)]
    command: Commands,

    /// project path
    #[arg(short, long)]
    path: Option<String>,

}

#[derive(Subcommand, Debug)]
enum Commands {
    /// init
    Init(InitArgs),

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



#[tokio::main]
async fn main() {

    let command = Cli::parse();
    //println!("{:?}", command);

    let _env = env::Env::new(command.path);


    let path = _env.work_dir();
    let home_dir = _env.home_dir();

    if !_env.check_openai_key() {
        println!("Please set openai key first, \nrun \"export OPENAI_KEY=your_openai_key\" in your shell, \nor set openai_key in $HOME/.readit/config.yaml \nyou can run \"readit -h \" for help.");
        return;
    }

    let client = OpenAI::new(&_env);

    match command.command {
        Commands::Init(args) => {
            init(&_env, &client, Some(args.force)).await;
        },
        Commands::Ask(args) => {
            
            if _env.is_new_project() {
                println!("Please run init command first, you can run \"readit -h \" for help.");
                return;
            }

            let embedding_obj = Embedding::new(
                &_env, &client
            ).await.unwrap();
            let query = args.query.clone();
            let (code_list, e_tokens) = embedding_obj.search(query.clone()).await.unwrap();
            let (res, a_tokens) = client.ask(query, code_list, "中文".to_string()).await.unwrap();
            println!("{}", res);
            println!("tokens usage: {:?}", a_tokens+e_tokens);
        }
    }

}
