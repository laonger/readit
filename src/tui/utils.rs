use futures::StreamExt;

use tokio;
use tokio::{
    sync::mpsc,
    sync::Mutex,
    sync::MutexGuard,
};

use std;
use std::sync::{
    Arc
};

use log::{info, error};

use crate::openai_utils::OpenAI;
use crate::embeding_utils::Embedding;

fn print_type_of<T>(_: &T) {
    info!("{}", std::any::type_name::<T>());
}

pub enum ChannelMessage {
    AswerContent(String),
    AswerEnd,
    NoticeContent(String),
    NoticeEnd,
    HistoryPrompt((String, String)), // record_id, prompt
    HistoryAnswer((String, String)), // record_id, answer
}

pub async fn send_notice(
    notice_tx: Arc<Mutex<mpsc::Sender<ChannelMessage>>>,
    content: String
) {
    let n_tx = notice_tx.lock().await;
    let _ = n_tx.send(ChannelMessage::NoticeEnd).await;
    let _ = n_tx.send(ChannelMessage::NoticeContent(content)).await;
}

pub async fn ask(
    client: OpenAI, 
    record_id: String,
    query: String,
    //area_tx: mpsc::Sender<ChannelMessage>,
    area_tx: MutexGuard<'_, mpsc::Sender<ChannelMessage>>,
    notice_tx: Arc<Mutex<mpsc::Sender<ChannelMessage>>>,
    history_tx: mpsc::Sender<ChannelMessage>,
    //lock: MutexGuard<'_, ()>
) {

    let _client = client;
    if _client.env.is_new_project() {
        println!("Please run init command first, you can run \"readit -h \" for help.");
        return;
    }

    let embedding_obj = match Embedding::new(
        &_client.env, &_client
    ).await {
        Ok(o) => o,
        Err(err) => {
            error!("embedding error: {}", err);
            return;
        }
    };

    let code_list = match embedding_obj.search(query.clone()).await {
        Ok((o, t)) => {
            let s = format!("find {} clues by spending {} tokens. Now waiting for Ai's reply.", o.len(), t);
            send_notice(notice_tx.clone(), s).await;
            o
        },
        Err(err) => {
            error!("embedding search error: {}", err);
            return;
        }
    };


    let (prompt, mut res) = match _client.ask(query.clone(), code_list).await {
        Ok(o) => o,
        Err(err) => {
            error!("ask error: {}", err);
            return;
        }
    };

    let _ = history_tx.send(ChannelMessage::HistoryPrompt((record_id.clone(), prompt.clone()))).await;

    let mut res_message: String = "".to_string();
    while let Some(result) = res.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                   if let Some(ref content) = chat_choice.delta.content {
                        let content = content.clone().replace("\n", "\n    ");
                        res_message.push_str(&content);
                        let _ = area_tx.send(ChannelMessage::AswerContent(content)).await;
                    }
                };
            }
            Err(err) => {
                let content = err.to_string().replace("\n", "\n    ");
                let _ = area_tx.send(ChannelMessage::AswerContent(content)).await;
            }
        }
    }
    let _ = area_tx.send(ChannelMessage::AswerEnd).await;
    let _ = history_tx.send(ChannelMessage::HistoryAnswer((record_id, res_message))).await;
    //drop(lock);
}

