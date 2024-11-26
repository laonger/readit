
use std::fmt;

use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Ai,
    Human,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Ai => write!(f, "AI"),
            Role::Human => write!(f, "User"),
        }
    }
    
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    //pub role: Role,
    pub query: String,  // user query
    pub prompt: Option<String>, // prompt
    pub answer: Option<String>, // Ai answer
    pub create_time: String,
}

impl Message {
    pub fn new(query: String) -> Self {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let create_time = duration.as_secs().to_string();

        Self {
            id: Uuid::new_v4().to_string(),
            //role,
            query,
            prompt: None,
            answer: None,
            create_time,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User: {}\nAI: {}", self.query, self.answer.clone().unwrap_or("".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct History {
    pub history: Vec<Message>,
}

impl History {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn add(&mut self, query: String) -> String {
        let m = Message::new(query);
        let id = m.id.clone();
        self.history.push(m);
        id
    }

    pub fn add_answer(&mut self, id: &String, answer: String) {
        for m in self.history.iter_mut() {
            if m.id == *id {
                m.answer = Some(answer.clone());
            }
        }
    }

    pub fn add_prompt(&mut self, id: &String, prompt: String) {
        for m in self.history.iter_mut() {
            if m.id == *id {
                m.prompt = Some(prompt.clone());
            }
        }
    }
}

