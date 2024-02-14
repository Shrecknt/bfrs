use crate::{ChunkedReceive, ChunkedReceiverWrapper};
use std::{
    collections::VecDeque,
    io::{Cursor, Read},
    sync::mpsc::{Receiver, Sender},
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

impl<T> ChunkedReceive<T> for ChunkedReceiverWrapper<Receiver<VecDeque<T>>, T> {
    fn next(&mut self) -> Option<T> {
        if !self.chunk.is_empty() {
            return self.chunk.pop_front();
        }
        if let Ok(chunk) = self.receiver.recv() {
            self.chunk = chunk;
            return self.next();
        }
        None
    }
}

impl Token {
    pub fn parse(cursor: &mut Cursor<&[u8]>, sender: Sender<VecDeque<Token>>, chunk_size: usize) {
        let mut chunks = VecDeque::with_capacity(chunk_size);
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
            chunks.push_back(token);
            if chunks.capacity() == chunks.len() {
                sender.send(chunks).unwrap();
                chunks = VecDeque::with_capacity(chunk_size);
            }
        }
        if !chunks.is_empty() {
            sender.send(chunks).unwrap();
        }
    }
}
