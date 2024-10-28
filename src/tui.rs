use std::env as std_env;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::{stdout, Write, Read};
use std::sync::{Arc, Mutex, MutexGuard};

use futures::{
    Future,
    future::FutureExt,
    //select,
    StreamExt
};

use log::{info, warn, error};

use tokio;
use tokio::{
    sync::mpsc,
    select,
};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use ratatui::layout::{
    Constraint,
    Direction,
    Layout,
    Rect,
};
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

use crate::errors::Result;

use crate::env;
use crate::openai_utils::OpenAI;
use crate::embeding_utils::Embedding;

enum Message {
    AswerContent(String),
    AswerEnd,
    NoticeContent(String),
    NoticeEnd,
}

async fn _ask<'a>(_env: env::Env, query: String, area_tx: mpsc::Sender<Message>) {

    let client = OpenAI::new(&_env);

    if _env.is_new_project() {
        println!("Please run init command first, you can run \"readit -h \" for help.");
        return
    }

    let embedding_obj = match Embedding::new(
        &_env, &client
    ).await {
        Ok(o) => o,
        Err(err) => {
            error!("embedding error: {}", err);
            return;
        }
    };

    let (code_list, e_tokens) = match embedding_obj.search(query.clone()).await {
        Ok(o) => o,
        Err(err) => {
            error!("embedding search error: {}", err);
            return;
        }
    };

    let mut res = match client.ask(query, code_list, _env.config.language()).await {
        Ok(o) => o,
        Err(err) => {
            error!("ask error: {}", err);
            return;
        }
    };
    
    while let Some(result) = res.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                   if let Some(ref content) = chat_choice.delta.content {
                        let content = content.to_string().replace("\n", "\n    ");
                        area_tx.send(Message::AswerContent(content)).await;
                    }
                };
            }
            Err(err) => {
                //ui.send_content_to_answer_area();
                let content = err.to_string().replace("\n", "\n    ");
                //ui.send_content_to_answer_area(content.to_string());
                area_tx.send(Message::AswerContent(content)).await;
            }
        }
        //stdout().flush().unwrap();
    }
    area_tx.send(Message::AswerEnd).await;
}

async fn ask<'a>(_env: env::Env, query: String, ui: &UI<'a>) {

    let tx = ui.answer_tx.clone();
    tokio::spawn(async move {
        _ask(_env, query, tx).await;
    });
}



struct UI<'a>{
    question_area: TextArea<'a>,
    answer_area: TextArea<'a>,
    answer_tx: mpsc::Sender<Message>,
    answer_rx: mpsc::Receiver<Message>,
    layout: Layout,
}

impl <'a> UI<'a> {
    fn new() -> Self {
        let mut question_area = TextArea::default();
        question_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Question: ( Ctrl-s to send query to AI, Ctrl-q to quit )")
        );
        let mut answer_area = TextArea::default();
        answer_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Answer")
        );
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
        ;
        let (answer_tx, answer_rx) = mpsc::channel(102400);
        Self {
            question_area,
            answer_area,
            answer_tx,
            answer_rx,
            layout,
        }
    }

    fn send_content_to_answer_area(&mut self, content: String) {
        let content = content.replace("\n", "\n    ");
        self.answer_area.set_yank_text(content);
        self.answer_area.move_cursor(CursorMove::End);
        self.answer_area.paste();
    }

    fn question_end(&mut self) {
        self.answer_area.insert_newline();
    }

    fn answer_end(&mut self) {
        self.answer_area.insert_newline();
        self.answer_area.insert_newline();
        self.answer_area.insert_newline();
    }

    fn clean_answer_area(&mut self) {
        let mut answer_area = TextArea::default();
        answer_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Answer")
        );
        self.answer_area = answer_area;
    }

    fn clean_question_area(&mut self) {
        info!("clean question area");
        let mut question_area = TextArea::default();
        question_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Question")
        );
        self.question_area = question_area;
    }
    
}

pub async fn run_ui(_env: env::Env) -> Result<()> {

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut ui = UI::new();

    let lock = Mutex::new(());

    let mut reader = EventStream::new();

    loop {
        term.draw(|f| {
            let chunks = ui.layout.split(f.area());
            f.render_widget(&ui.question_area, chunks[0]);
            f.render_widget(&ui.answer_area, chunks[1]);
        })?;
        
        let mut event = reader.next().fuse();

        select! {
            Some(Ok(ev)) = event => {
                match ev.into() {
                    Input { key: Key::Char('q'), ctrl: true, .. } => {
                        break;
                    },
                    Input { key: Key::Char('s'), ctrl: true, .. } => {
                        let _lock = match lock.lock() {
                            Ok(_) => {
                                let content = ui.question_area.lines().join("\n");
                                ui.clean_question_area();
                                let user_query = format!("USER: {}", content.clone());
                                ui.send_content_to_answer_area(user_query);
                                ui.question_end();
                                ui.send_content_to_answer_area("AI: ".to_string());

                                ask(_env.clone(), content, &mut ui).await;
                                //ui.send_content_to_answer_area(content);
                            },
                            Err(_) => {
                                //continue;
                            }
                        };
                    },
                    Input { key: Key::MouseScrollUp,  .. } => {
                        ui.answer_area.scroll((-1, 0));
                    },
                    Input { key: Key::MouseScrollDown,  .. } => {
                        ui.answer_area.scroll((1, 0));
                    },
                    //Event::Resize(_, _) => {
                    //    term.autoresize()?;
                    //}
                    input => {
                        ui.question_area.input(input);
                    }
                }
            },
            Some(content) = ui.answer_rx.recv() => {
                match content {
                    Message::AswerContent(content) => {
                        ui.send_content_to_answer_area(content);
                    },
                    Message::AswerEnd => {
                        ui.answer_end();
                    },
                    _ => {
                    }
                }
            },
        }

        //match crossterm::event::read()?.into() {
        //    Input { key: Key::Char('q'), ctrl: true, .. } => {
        //        break;
        //    },
        //    Input { key: Key::Char('s'), ctrl: true, .. } => {
        //        let _lock = match lock.lock() {
        //            Ok(_) => {
        //                let content = ui.question_area.lines().join("\n");
        //                ui.clean_question_area();
        //                let user_query = format!("USER: {}", content.clone());
        //                ui.send_content_to_answer_area(user_query);
        //                ui.question_end();
        //                ui.send_content_to_answer_area("AI: ".to_string());
        //                ask(_env.clone(), content, &mut ui).await;
        //                //ui.send_content_to_answer_area(content);
        //            },
        //            Err(_) => {
        //                continue;
        //            }
        //        };
        //    },
        //    Input { key: Key::MouseScrollUp,  .. } => {
        //        ui.answer_area.scroll((-1, 0));
        //    },
        //    Input { key: Key::MouseScrollDown,  .. } => {
        //        ui.answer_area.scroll((1, 0));
        //    },
        //    input => {
        //        ui.question_area.input(input);
        //    }
        //};
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    println!("quit");

    //println!("Lines: {:?}", textareas[0].lines());

    Ok(())
}
