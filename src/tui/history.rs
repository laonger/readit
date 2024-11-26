
use crate::history::{History, Message};


pub async fn add(history: &mut History, query: String) -> String{
    history.add(query)
}

// step == 1: walk forward
// step == -1: walk backward
async fn _walk(history: &History, step: i16, cursor: i16) -> Option<(i16, Message)> {
    if history.history.is_empty() {
        return None;
    }

    let mut cursor = cursor;

    cursor = cursor + step;

    if cursor < 0 {
        cursor = (history.history.len() - 1) as i16;
    }
    if cursor >= history.history.len() as i16 {
        cursor = 0;
    }

    return Some((cursor, history.history[cursor as usize].clone()))
}

pub async fn previous(history: &History, cursor: i16) -> Option<(i16, Message)> {
    _walk(history, -1, cursor).await
}

pub async fn next(history: &History, cursor: i16) -> Option<(i16, Message)> {
    _walk(history, -1, cursor).await
}


pub fn clear_cursor() -> i16 {
    0
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn next_test() {
//        let mut history = History::new();
//        history.add(Role::Ai,    Some("ai1".to_string()),    "".to_string());
//        history.add(Role::Human, Some("human1".to_string()), "".to_string());
//        history.add(Role::Ai,    Some("ai2".to_string()),    "".to_string());
//        history.add(Role::Human, Some("human2".to_string()), "".to_string());
//        history.add(Role::Ai,    Some("ai3".to_string()),    "".to_string());
//        //history.add(Role::Human, "human3".to_string());
//
//        assert_eq!(history.next().unwrap().query, Some("human1".to_string()));
//        assert_eq!(history.next().unwrap().query, Some("human2".to_string()));
//        assert_eq!(history.next().unwrap().query, Some("human1".to_string()));
//    }
//
//    #[test]
//    fn previous_test() {
//        let mut history = History::new();
//        history.add(Role::Ai,    Some("ai1".to_string()),    "".to_string());
//        history.add(Role::Human, Some("human1".to_string()), "".to_string());
//        history.add(Role::Ai,    Some("ai2".to_string()),    "".to_string());
//        history.add(Role::Human, Some("human2".to_string()), "".to_string());
//        history.add(Role::Ai,    Some("ai3".to_string()),    "".to_string());
//
//        assert_eq!(history.previous().unwrap().query, Some("human2".to_string()));
//        assert_eq!(history.previous().unwrap().query, Some("human1".to_string()));
//        assert_eq!(history.previous().unwrap().query, Some("human2".to_string()));
//
//    }
//}
//
