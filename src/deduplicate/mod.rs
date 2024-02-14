use crate::ast::AstNode;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub enum DeduplicatedAstNode {
    ModifyPointer(isize),
    ModifyData(isize),
    Output,
    Input,
    Group(Receiver<Vec<DeduplicatedAstNode>>),
}

impl DeduplicatedAstNode {
    pub fn parse(receiver: Receiver<Vec<AstNode>>, sender: Sender<Vec<Self>>) {
        let mut current_token: Option<Self> = None;
        while let Ok(tokens) = receiver.recv() {
            let mut chunks = Vec::with_capacity(tokens.len());
            for token in tokens {
                match token {
                    AstNode::Group(receiver) => {
                        if let Some(current_token_value) = current_token {
                            if current_token_value.is_nonzero() {
                                chunks.push_within_capacity(current_token_value).unwrap();
                                current_token = None;
                            } else {
                                current_token = None;
                            }
                        }
                        let (s, r) = channel();
                        chunks.push_within_capacity(Self::Group(r)).unwrap();
                        Self::parse(receiver, s);
                    }
                    AstNode::IncrementPointer | AstNode::DecrementPointer => {
                        let modifier = match token {
                            AstNode::IncrementPointer => 1,
                            AstNode::DecrementPointer => -1,
                            _ => unreachable!(),
                        };
                        if let Some(current_token_value) = &mut current_token {
                            if let Self::ModifyPointer(pointer_value) = current_token_value {
                                *pointer_value += modifier;
                            } else {
                                if current_token_value.is_nonzero() {
                                    chunks.push_within_capacity(current_token.unwrap()).unwrap();
                                }
                                current_token = Some(Self::ModifyPointer(modifier));
                            }
                        } else {
                            current_token = Some(Self::ModifyPointer(modifier));
                        }
                    }
                    AstNode::IncrementData | AstNode::DecrementData => {
                        let modifier = match token {
                            AstNode::IncrementData => 1,
                            AstNode::DecrementData => -1,
                            _ => unreachable!(),
                        };
                        if let Some(current_token_value) = &mut current_token {
                            if let Self::ModifyData(data_value) = current_token_value {
                                *data_value += modifier;
                            } else {
                                if current_token_value.is_nonzero() {
                                    chunks.push_within_capacity(current_token.unwrap()).unwrap();
                                }
                                current_token = Some(Self::ModifyData(modifier));
                            }
                        } else {
                            current_token = Some(Self::ModifyData(modifier));
                        }
                    }
                    AstNode::Output => {
                        if let Some(current_token_value) = current_token {
                            if current_token_value.is_nonzero() {
                                chunks.push_within_capacity(current_token_value).unwrap();
                            }
                            current_token = None;
                        }
                        chunks.push_within_capacity(Self::Output).unwrap();
                    }
                    AstNode::Input => {
                        if let Some(current_token_value) = current_token {
                            if current_token_value.is_nonzero() {
                                chunks.push_within_capacity(current_token_value).unwrap();
                            }
                            current_token = None;
                        }
                        chunks.push_within_capacity(Self::Input).unwrap();
                    }
                }
            }
            sender.send(chunks).unwrap();
        }
        if let Some(current_token) = current_token {
            if current_token.is_nonzero() {
                sender.send(vec![current_token]).unwrap();
            }
        }
    }
    fn is_nonzero(&self) -> bool {
        match self {
            Self::ModifyPointer(pointer_value) => *pointer_value != 0,
            Self::ModifyData(data_value) => *data_value != 0,
            _ => true,
        }
    }
}
