use crate::parse::Token;
use std::sync::mpsc::{channel, Receiver, Sender};

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
    pub fn parse(receiver: &Receiver<Vec<Token>>, sender: Sender<Vec<Self>>) -> Vec<Self> {
        while let Ok(tokens) = receiver.recv() {
            let mut chunks = Vec::with_capacity(tokens.len());
            for token in tokens {
                match token {
                    Token::LeftBracket => {
                        let (s, r) = channel();
                        chunks.push_within_capacity(Self::Group(r)).unwrap();
                        let remaining_chunk = Self::parse(receiver, s);
                        todo!();
                    }
                    Token::RightBracket => {
                        sender.send(chunks).unwrap();
                        todo!()
                    }
                    Token::IncrementPointer => {
                        chunks.push_within_capacity(Self::IncrementPointer).unwrap()
                    }
                    Token::DecrementPointer => {
                        chunks.push_within_capacity(Self::DecrementPointer).unwrap()
                    }
                    Token::IncrementData => {
                        chunks.push_within_capacity(Self::IncrementData).unwrap()
                    }
                    Token::DecrementData => {
                        chunks.push_within_capacity(Self::DecrementData).unwrap()
                    }
                    Token::Output => chunks.push_within_capacity(Self::Output).unwrap(),
                    Token::Input => chunks.push_within_capacity(Self::Input).unwrap(),
                }
            }
            sender.send(chunks).unwrap();
        }
        Vec::new()
    }
}
