use std::io;
use std::fs::File;
use std::path::PathBuf;
use std::io::{stdout, Write, Read};
use futures::StreamExt;
//use std::env as std_env;

use log::info;

use md5;

use tokio;
use tokio::task::JoinSet;

use crate::errors::Result;

use crate::openai_utils::OpenAI;
use crate::file_utils;
use crate::embeding_utils::Embedding;
use crate::structs;
use crate::env;



pub async fn embedding_file(
    env: env::Env,
    f_name: String,
    lang: String,
    f_path: String,
    md5_value: String,
    file_content: String,
    is_update: Option<bool>,
) {
    let client = OpenAI::new(&env);
    let embedding_obj = Embedding::new(
        &env, &client
    ).await.unwrap();

    //println!("f: {:?}", f_name);

    let (response, a_tockens) = client.analyse_source(
        file_content.clone(), lang.to_string(), env.config.language().to_string()
    ).await.unwrap();

    let file_des = structs::CodeDescription {
        //line_number: 0,
        //lines: 0,
        file: Some(f_path.clone()),
        md5: Some(md5_value.clone()),
        code_type: Some("file".to_string()),
        lang: Some(lang.to_string()),
        name: f_name,
        purpose: response.clone().purpose,
        source_code: file_content,
    };

    if is_update == Some(true) {
        embedding_obj.delete_file(file_des.clone()).await.unwrap();
    }

    let mut e_tokens = embedding_obj.add_data(file_des).await.unwrap();
    for c in response.classes {
        //println!("c: {:?}", c);
        let data = structs::CodeDescription {
            file: Some(f_path.clone()),
            md5: Some(md5_value.clone()),
            code_type: Some("class".to_string()),
            lang: Some(lang.to_string()),
            name: c.name,
            purpose: c.purpose,
            source_code: c.source_code,
        };
        e_tokens += embedding_obj.add_data(data).await.unwrap();
    };
    for c in response.functions {
        //println!("c: {:?}", c);
        let data = structs::CodeDescription {
            file: Some(f_path.clone()),
            md5: Some(md5_value.clone()),
            code_type: Some("function".to_string()),
            lang: Some(lang.to_string()),
            name: c.name,
            purpose: c.purpose,
            source_code: c.source_code,
        };
        e_tokens += embedding_obj.add_data(data).await.unwrap();
    };
    println!(
        "{}  analysing use tokens: {:?}    embedding use tokens: {:?}",
        f_path, a_tockens, e_tokens
    );
}

pub async fn force_init(env: env::Env, ) -> Result<()> {

    let path = env.work_dir();

    let client = OpenAI::new(&env);
    let embedding_obj = Embedding::new(
        &env, &client
    ).await?;

    embedding_obj.clean_all().await?;

    let mut file_list: Vec<(PathBuf, String)> = Vec::new();
    file_utils::list_path(
        path, &mut file_list, &env.ignore, &env.language_extensions
    );

    let mut job_set = JoinSet::new();
    for (f, programming_lang) in file_list.iter() {

        let f_path = f.canonicalize().unwrap().to_str().unwrap().to_string().clone();
        let f_name = f.to_str().unwrap().to_string().clone();
        let programming_lang = programming_lang.clone();


        println!("analyse: {:?}", f_path);
        //let (f, programming_lang) = file_list.get(0).unwrap();
        let mut code = String::new();
        let _ = File::open(f).unwrap().read_to_string(&mut code);

        let md5_value = format!("{:x}", md5::compute(code.clone()));

        let _env = env.clone();
        
        job_set.spawn(async move {
            embedding_file(
                _env, f_name, 
                programming_lang, f_path, 
                md5_value, code, None
            ).await;
        });
        //embedding_file(
        //    _env, f_name, 
        //    programming_lang, f_path, 
        //    md5_value, code
        //).await;
    };
    let mut seen = vec![];
    while let Some(res) = job_set.join_next().await {
        res.unwrap();
        seen.push(true);
    }

    if seen.len() == file_list.len(){
        println!("Embedding Done");
        let tokens = embedding_obj.update_summary(env.config.language()).await?;
        println!(
            "projedct summary embedding use tokens: {:?}",
            tokens
        );
    };
    Ok(())
}

pub async fn init(env: env::Env ) -> Result<()> {

    let client = OpenAI::new(&env);
    let embedding_obj = Embedding::new(
        &env, &client
    ).await?;

    let path = env.work_dir();

    let mut file_list: Vec<(PathBuf, String)> = Vec::new();
    file_utils::list_path(
        path, &mut file_list, &env.ignore, &env.language_extensions
    );

    let mut _file_list:Vec<(PathBuf, String, String, String, String)> = Vec::new();
    for (f, programming_lang) in file_list.iter() {

        let f_path = f.canonicalize().unwrap().to_str().unwrap().to_string().clone();
        //let f_name = f.to_str().unwrap().to_string().clone();
        //let programming_lang = programming_lang.clone();

        //let (f, programming_lang) = file_list.get(0).unwrap();
        let mut code = String::new();
        let _ = File::open(f).unwrap().read_to_string(&mut code);

        let md5_value = format!("{:x}", md5::compute(code.clone()));

        if embedding_obj.is_file_change(&f_path, &md5_value).await? {
            _file_list.push((
                f.clone(),
                f_path.clone(),
                programming_lang.clone(),
                code,
                md5_value
            ));
        }
    };

    if _file_list.is_empty() {
        return Ok(())
    }

    println!("these files is changed, would you want to re-embedding them?");
    for i in _file_list.clone() {
        println!("    {}", i.1)
    };

    println!("Yes(default)/No: ");

    let mut y_n = String::new();
    let _ = io::stdin().read_line(&mut y_n);
    println!("");
    y_n = y_n.replace("\n", "").replace(" ", "").replace("\r", "");
    if y_n == "No".to_string() || y_n == "no" {
        println!("....");
        return Ok(())
    }
    println!("analysing....");
    let mut job_set = JoinSet::new();
    for (f, f_path, programming_lang, code, md5_value) in _file_list.clone() {

        println!("analyse: {:?}", f_path);
        let _env = env.clone();
        let f_name = f.to_str().unwrap().to_string().clone();
        let programming_lang = programming_lang.clone();
        
        job_set.spawn(async move {
            embedding_file(
                _env, f_name, 
                programming_lang, f_path, 
                md5_value, code, Some(true)
            ).await;
        });

    };

    let mut seen = vec![];
    while let Some(res) = job_set.join_next().await {
        res.unwrap();
        seen.push(true);
    }

    if seen.len() == _file_list.len(){
        println!("Embedding Done");

        let client = OpenAI::new(&env);
        let embedding_obj = Embedding::new(
            &env, &client
        ).await?;
        let tokens = embedding_obj.update_summary(env.config.language()).await;

        println!(
            "project summary embedding use tokens: {:?}",
            tokens
        );
    }

    println!("Embedding Done");
    Ok(())
}

pub async fn ask(mut _env: env::Env, query: String, ) -> Result<()>{
    let client = OpenAI::new(&_env);

    if _env.is_new_project() {
        println!("Please run init command first, you can run \"readit -h \" for help.");
        return Ok(())
    }

    let embedding_obj = Embedding::new(
        &_env, &client
    ).await?;

    let (code_list, e_tokens) = embedding_obj.search(query.clone()).await?;

    let (_, mut res) = client.ask(query, code_list).await?;
    
    let mut lock = stdout().lock();
    while let Some(result) = res.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    //println!("{:?}", chat_choice);
                    if let Some(ref content) = chat_choice.delta.content {
                        write!(lock, "{}", content).unwrap();
                    }
                });
            }
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        stdout().flush().unwrap();
    };
    Ok(())
}

