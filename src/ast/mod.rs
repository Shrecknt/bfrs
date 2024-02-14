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
    Group(Receiver<VecDeque<Self>>),
}

impl AstNode {
    pub fn parse(
        receiver: &mut ChunkedReceiverWrapper<Receiver<VecDeque<Token>>, Token>,
        sender: Sender<VecDeque<Self>>,
        chunk_size: usize,
    ) {
        let mut chunks = VecDeque::with_capacity(chunk_size);
        while let Some(token) = receiver.next() {
            match token {
                Token::LeftBracket => {
                    let (s, r) = channel();
                    chunks.push_back(Self::Group(r));
                    Self::parse(receiver, s, chunk_size);
                }
                Token::RightBracket => {
                    sender.send(chunks).unwrap();
                    return;
                }
                Token::IncrementPointer => chunks.push_back(Self::IncrementPointer),
                Token::DecrementPointer => chunks.push_back(Self::DecrementPointer),
                Token::IncrementData => chunks.push_back(Self::IncrementData),
                Token::DecrementData => chunks.push_back(Self::DecrementData),
                Token::Output => chunks.push_back(Self::Output),
                Token::Input => chunks.push_back(Self::Input),
            }

            if chunks.len() == chunks.capacity() {
                sender.send(chunks).unwrap();
                chunks = VecDeque::with_capacity(chunk_size);
            }
        }
        if chunks.len() != 0 {
            sender.send(chunks).unwrap();
        }
    }
}
