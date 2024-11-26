use std::io;
use std::io::BufRead;
use std::io::{Write, Read};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::{
    future::FutureExt,
    //select,
    StreamExt
};

use log::info;

use tokio;
use tokio::{
    sync::mpsc,
    select,
    sync::Mutex,
};


use crossterm::{
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    event::{
        KeyCode,
        KeyEvent,
        KeyModifiers,
        DisableMouseCapture,
        Event,
        EventStream,
    },
};

use ratatui::backend::CrosstermBackend;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use ratatui::layout::{
    Constraint,
    Direction,
    Layout,
    Rect,
};
use tui_textarea::{
    CursorMove, TextArea,
};

mod utils;
use utils::{ChannelMessage, send_notice};

mod history;
use history::{clear_cursor, next, previous};

use crate::history::History;

use crate::errors::Result;

use crate::env;
use crate::openai_utils::OpenAI;

struct UI<'a>{
    question_area: TextArea<'a>,
    answer_area: TextArea<'a>,
    notice_area: TextArea<'a>,
    question_rect: Rect,
    answer_rect: Rect,
    notice_rect: Rect,
    answer_tx: mpsc::Sender<ChannelMessage>,
    answer_rx: mpsc::Receiver<ChannelMessage>,
    notice_tx: mpsc::Sender<ChannelMessage>,
    notice_rx: mpsc::Receiver<ChannelMessage>,
    layout: Layout,
    activte_window_num: u8,
}

impl <'a> UI<'a> {

    fn _question_area(activte_window_num: u8) -> TextArea<'a> {
        let mut area = TextArea::default();
        if activte_window_num != 0 {
            area.set_cursor_line_style(Style::default());
            area.set_cursor_style(Style::default());
        } else {
            area.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        }
        area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Question: ( Ctrl-g to send query to AI, Ctrl-q to quit, CCtrl-w to change focus )")
        );
        area
    }

    fn _answer_area(activte_window_num: u8) -> TextArea<'a> {
        let mut area = TextArea::default();
        if activte_window_num != 1 {
            area.set_cursor_line_style(Style::default());
            area.set_cursor_style(Style::default());
        } else {
            area.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        }
        area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Answer: ( Ctrl-o to page down, Ctrl-u to page up )")
        );
        area
    }

    fn _notice_area(activte_window_num: u8) -> TextArea<'a> {
        let mut area = TextArea::default();
        if activte_window_num != 2 {
            area.set_cursor_line_style(Style::default());
            area.set_cursor_style(Style::default());
        } else {
            area.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        }
        area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Notice")
        );
        area
    }

    fn new() -> Self {

        let activte_window_num:u8 = 0;
        
        let question_area = Self::_question_area(activte_window_num);
        let question_rect = Rect::default();

        let answer_area = Self::_answer_area(activte_window_num);
        let answer_rect = Rect::default();
        let (answer_tx, answer_rx) = mpsc::channel(102400);

        let notice_area = Self::_notice_area(activte_window_num);
        let notice_rect = Rect::default();
        let (notice_tx, notice_rx) = mpsc::channel(102400);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(3)])
        ;
        Self {
            question_area,
            answer_area,
            notice_area,
            question_rect,
            answer_rect,
            notice_rect,
            answer_tx,
            answer_rx,
            notice_tx,
            notice_rx,
            layout,
            activte_window_num,
        }

    }

    fn send_content_to_question_area(&mut self, content: String) {
        for ch in content.chars() {
            self.question_area.move_cursor(CursorMove::End);
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            self.question_area.input(key_event);
        }
    }

    fn send_content_to_answer_area(&mut self, content: String) {
        for ch in content.chars() {
            self.answer_area.move_cursor(CursorMove::End);
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            self.answer_area.input(key_event);
        }
    }

    fn send_content_to_notice_area(&mut self, content: String) {
        for ch in content.chars() {
            self.notice_area.move_cursor(CursorMove::End);
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            self.notice_area.input(key_event);
        }
    }

    fn question_end(&mut self) {
        self.answer_area.insert_newline();
    }

    fn answer_end(&mut self) {
        self.answer_area.insert_newline();
        self.answer_area.insert_newline();
        self.answer_area.insert_newline();
    }

    fn notice_end(&mut self) {
        self.notice_area.insert_newline();
    }

    fn clean_answer_area(&mut self) {
        self.answer_area = UI::_answer_area(self.activte_window_num);
    }

    fn clean_question_area(&mut self) {
        self.question_area = UI::_question_area(self.activte_window_num);
    }

    fn clean_notice_area(&mut self) {
        self.notice_area = UI::_notice_area(self.activte_window_num);
    }

    fn change_focus(&mut self){
        self.activte_window_num = (self.activte_window_num + 1) % 3;

        for (n, area) in [
            (0, &mut self.question_area),
            (1, &mut self.answer_area),
            (2, &mut self.notice_area)
        ].iter_mut() {
            if *n == self.activte_window_num {
                area.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
                area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            } else {
                area.set_cursor_line_style(Style::default());
                area.set_cursor_style(Style::default());
            };
        };
    }

    fn actived_window(&self) -> &TextArea<'a> {
        match self.activte_window_num {
            0 => &self.question_area,
            1 => &self.answer_area,
            2 => &self.notice_area,
            _ => &self.question_area,
        }
        
    }
    
}


pub async fn run_ui(_env: env::Env) -> Result<()> {

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        //EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut ui = UI::new();

    let mut reader = EventStream::new();

    let mut last_exit_attempt: Option<Instant> = None; // 退出的二次确认时间

    let mut history = History::new();
    let mut history_cursor = clear_cursor();

    let (history_tx, mut history_rx) = mpsc::channel(102400);

    let answer_tx_lock = Arc::new(Mutex::new(ui.answer_tx.clone()));

    let notice_tx_lock = Arc::new(Mutex::new(ui.notice_tx.clone()));

    let client = OpenAI::new(&_env);

    loop {
        term.draw(|f| {
            let chunks = ui.layout.split(f.area());
            ui.question_rect = chunks[0];
            ui.answer_rect = chunks[1];
            ui.notice_rect = chunks[2];
            f.render_widget(&ui.question_area, ui.question_rect);
            f.render_widget(&ui.answer_area, ui.answer_rect);
            f.render_widget(&ui.notice_area, ui.notice_rect);
        })?;
        
        let event = reader.next().fuse();

        select! {
            Some(Ok(ev)) = event => {
                //info!("ev: {:?}", ev);
                // Event::Mouse(MouseEvent { kind: Up(Left), column: 98, row: 22, modifiers: KeyModifiers(0x0) })
                // Event::Key(KeyEvent { code: Char('j'), modifiers: KeyModifiers(CONTROL), kind: Press, state: KeyEventState(0x0) })
                // Event::Key(KeyEvent { code: Char('a'), modifiers: KeyModifiers(0x0), kind: Press, state: KeyEventState(0x0) })
                match ev {

                    Event::Key(KeyEvent{
                        code: KeyCode::Char('q'),
                        modifiers:KeyModifiers::CONTROL,
                        .. 
                    }) => {
                        let l = notice_tx_lock.clone();
                        let now = Instant::now();

                        if let Some(last_attempt) = last_exit_attempt {
                            if now.duration_since(last_attempt) < Duration::from_secs(2) {
                                send_notice(l, "Exiting...".to_string()).await;
                                break;
                            } else {
                                send_notice(
                                    l,
                                    "Press again within 2 seconds to confirm exit.".to_string()
                                ).await;
                            };
                        } else {
                            send_notice(
                                l,
                                "Press again within 2 seconds to confirm exit.".to_string()
                            ).await;
                        };
                        last_exit_attempt = Some(now);
                    },

                    Event::Key(KeyEvent{
                        code: KeyCode::Char('w'),
                        modifiers:KeyModifiers::CONTROL,
                        .. 
                    })=> {
                        ui.change_focus();
                    },

                    Event::Key(KeyEvent{
                        code: KeyCode::Char('o'),
                        modifiers:KeyModifiers::CONTROL,
                        .. 
                    }) => {
                        let row_count:i16 = ui.answer_rect.rows().count() as i16;
                        ui.answer_area.scroll((row_count/2, 0));
                    },

                    Event::Key(KeyEvent{
                        code: KeyCode::Char('u'),
                        modifiers:KeyModifiers::CONTROL,
                        .. 
                    }) => {
                        let row_count:i16 = ui.answer_rect.rows().count() as i16;
                        ui.answer_area.scroll((-row_count/2, 0));
                    },


                    // 上一条历史记录
                    Event::Key(KeyEvent{
                        code: KeyCode::Char('p'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        match previous(&history, history_cursor).await {
                            Some((cursor, message)) => {
                                let content = message.query;
                                ui.clean_question_area();
                                ui.send_content_to_question_area(content);
                                history_cursor = cursor;
                            },
                            None => {}
                        };
                    },
                    // 下一条历史记录
                    Event::Key(KeyEvent{
                        code: KeyCode::Char('n'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        match next(&history, history_cursor).await {
                            Some((cursor, message)) => {
                                let content = message.query;
                                ui.clean_question_area();
                                ui.send_content_to_question_area(content);
                                history_cursor = cursor;
                            },
                            None => {}
                        };
                    },

                    Event::Key(KeyEvent{
                        code: KeyCode::Char('s'),
                        modifiers:KeyModifiers::CONTROL,
                        .. 
                    }) => {

                        let (content, record_id) = {
                            let answer_tx = match answer_tx_lock.try_lock() {
                                Ok(tx) => tx,
                                Err(_) => {
                                    continue;
                                }
                            };

                            history_cursor = clear_cursor();

                            let content = ui.question_area.lines().join("\n");
                            ui.clean_question_area();

                            let record_id = history.add(content.clone());

                            let user_query = format!("USER: {}", content);

                            let _ = answer_tx.send(ChannelMessage::AswerContent(user_query)).await;
                            let _ = answer_tx.send(ChannelMessage::AswerEnd).await;
                            let _ = answer_tx.send(ChannelMessage::AswerContent("AI: ".to_string())).await;
                            (content, record_id)
                        };

                        let _client = client.clone();
                        let h_tx = history_tx.clone();

                        let _answer_tx_lock = Arc::clone(&answer_tx_lock);

                        let _notice_tx_lock = Arc::clone(&notice_tx_lock);

                        tokio::spawn(async move {
                            info!("send query to AI: ");
                            let tx = match _answer_tx_lock.try_lock() {
                                Ok(tx) => tx,
                                Err(_) => {
                                    return;
                                }
                            };
                            utils::ask(_client, record_id, content, tx, _notice_tx_lock, h_tx).await;
                        });
                    },

                    (Event::Key(KeyEvent{
                        code: KeyCode::Char(_),
                        .. 
                    }) | Event::Key(KeyEvent{
                        code: KeyCode::Enter,
                        .. 
                    }) | Event::Key(KeyEvent{
                        code: KeyCode::Backspace,
                        .. 
                    }) | Event::Key(KeyEvent{
                        code: KeyCode::Delete,
                        .. 
                    
                    }) | Event::Key(KeyEvent{
                        code: KeyCode::Insert,
                        .. 
                    
                    }) | Event::Key(KeyEvent{
                        code: KeyCode::Tab,
                        .. 
                    
                    }) | Event::Key(KeyEvent{
                        code: KeyCode::BackTab,
                        .. 
                    
                    })) if ui.activte_window_num != 0 => {
                        continue
                    },

                    input => {
                        last_exit_attempt = match last_exit_attempt {
                            Some(_) => {
                                send_notice(notice_tx_lock.clone(), "".to_string()).await;
                                None
                            },
                            None => None
                        };
                        if ui.activte_window_num == 0 {
                            ui.question_area.input(input);
                        } else if ui.activte_window_num == 1 {
                            let _answer_tx = match answer_tx_lock.try_lock() {
                                Ok(tx) => tx,
                                Err(_) => {
                                    continue;
                                }
                            };
                            ui.answer_area.input(input);
                        } else {
                            let _notice_tx = match notice_tx_lock.try_lock() {
                                Ok(tx) => tx,
                                Err(_) => {
                                    continue;
                                }
                            };
                            ui.notice_area.input(input);
                        }
                        //ui.question_area.input(input);
                        history_cursor = clear_cursor();
                    }
                }
            },
            Some(content) = ui.answer_rx.recv() => {
                match content {
                    ChannelMessage::AswerContent(content) => {
                        ui.send_content_to_answer_area(content);
                    },
                    ChannelMessage::AswerEnd => {
                        ui.answer_end();
                    },
                    _ => {
                    }
                };
            },
            Some(content) = ui.notice_rx.recv() => {
                match content {
                    ChannelMessage::NoticeContent(content) => {
                        ui.send_content_to_notice_area(content);
                    },
                    ChannelMessage::NoticeEnd => {
                        ui.notice_end();
                    },
                    _ => {
                    }
                };
            },
            Some(content) = history_rx.recv() => {
                match content {
                    ChannelMessage::HistoryPrompt((record_id, prompt)) => {
                        history.add_prompt(&record_id, prompt);
                    },
                    ChannelMessage::HistoryAnswer((record_id, answer)) => {
                        history.add_answer(&record_id, answer);
                    },
                    _ => {
                    }
                };
            },
        }
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
