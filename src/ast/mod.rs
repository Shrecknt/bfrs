use crate::{parse::Token, ChunkedReceive, ChunkedReceiverWrapper};
use std::{
    collections::VecDeque,
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug)]
pub enum AstNode {
    IncrementPointer,
    DecrementPointer,
    IncrementData,
    DecrementData,
    Output,
    Input,
    Group(Receiver<Vec<Self>>),
}

impl AstNode {
    pub fn parse(
        receiver: &mut ChunkedReceiverWrapper<Receiver<VecDeque<Token>>, Token>,
        sender: Sender<Vec<Self>>,
        chunk_size: usize,
    ) {
        let mut chunks = Vec::with_capacity(chunk_size);
        while let Some(token) = receiver.next() {
            match token {
                Token::LeftBracket => {
                    let (s, r) = channel();
                    chunks.push_within_capacity(Self::Group(r)).unwrap();
                    Self::parse(receiver, s, chunk_size);
                }
                Token::RightBracket => {
                    sender.send(chunks).unwrap();
                    return;
                }
                Token::IncrementPointer => {
                    chunks.push_within_capacity(Self::IncrementPointer).unwrap()
                }
                Token::DecrementPointer => {
                    chunks.push_within_capacity(Self::DecrementPointer).unwrap()
                }
                Token::IncrementData => chunks.push_within_capacity(Self::IncrementData).unwrap(),
                Token::DecrementData => chunks.push_within_capacity(Self::DecrementData).unwrap(),
                Token::Output => chunks.push_within_capacity(Self::Output).unwrap(),
                Token::Input => chunks.push_within_capacity(Self::Input).unwrap(),
            }

            if chunks.len() == chunks.capacity() {
                sender.send(chunks).unwrap();
                chunks = Vec::with_capacity(chunk_size);
            }
        }
        if chunks.len() != 0 {
            sender.send(chunks).unwrap();
        }
    }
}
