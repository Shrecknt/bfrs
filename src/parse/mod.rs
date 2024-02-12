use std::{
    io::{Cursor, Read},
    sync::mpsc::Sender,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    IncrementPointer,
    DecrementPointer,
    IncrementData,
    DecrementData,
    Output,
    Input,
    LeftBracket,
    RightBracket,
}

impl Token {
    pub fn parse(cursor: &mut Cursor<&[u8]>, sender: Sender<Token>) {
        let mut buf = [0u8; 1];
        while let Ok(_) = cursor.read_exact(&mut buf) {
            let token = match buf[0] {
                b'>' => Token::IncrementPointer,
                b'<' => Token::DecrementPointer,
                b'+' => Token::IncrementData,
                b'-' => Token::DecrementData,
                b'.' => Token::Output,
                b',' => Token::Input,
                b'[' => Token::LeftBracket,
                b']' => Token::RightBracket,
                _ => continue,
            };
            sender.send(token).unwrap();
        }
    }
}
