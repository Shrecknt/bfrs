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
    Group(Receiver<Self>),
}

impl AstNode {
    pub fn parse(receiver: &Receiver<Token>, sender: Sender<Self>) {
        while let Ok(token) = receiver.recv() {
            match token {
                Token::LeftBracket => {
                    let (s, r) = channel();
                    sender.send(Self::Group(r)).unwrap();
                    Self::parse(receiver, s);
                }
                Token::RightBracket => return,
                Token::IncrementPointer => sender.send(Self::IncrementPointer).unwrap(),
                Token::DecrementPointer => sender.send(Self::DecrementPointer).unwrap(),
                Token::IncrementData => sender.send(Self::IncrementData).unwrap(),
                Token::DecrementData => sender.send(Self::DecrementData).unwrap(),
                Token::Output => sender.send(Self::Output).unwrap(),
                Token::Input => sender.send(Self::Input).unwrap(),
            }
        }
    }
}
