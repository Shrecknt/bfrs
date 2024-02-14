use crate::{ast::AstNode, ChunkedReceive, ChunkedReceiverWrapper};
use std::{
    collections::VecDeque,
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug)]
pub enum DeduplicatedAstNode {
    ModifyPointer(isize),
    ModifyData(isize),
    Output,
    Input,
    Group(Receiver<VecDeque<DeduplicatedAstNode>>),
}

impl DeduplicatedAstNode {
    pub fn parse(
        mut receiver: ChunkedReceiverWrapper<Receiver<VecDeque<AstNode>>, AstNode>,
        sender: Sender<VecDeque<Self>>,
    ) {
        let mut current_token: Option<Self> = None;
        let mut chunks = VecDeque::with_capacity(8);
        while let Some(token) = receiver.next() {
            match token {
                AstNode::Group(receiver) => {
                    let receiver = ChunkedReceiverWrapper::new(receiver);
                    if let Some(current_token_value) = current_token {
                        if current_token_value.is_nonzero() {
                            chunks.push_back(current_token_value);
                            current_token = None;
                        } else {
                            current_token = None;
                        }
                    }
                    let (s, r) = channel();
                    chunks.push_back(Self::Group(r));
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
                                chunks.push_back(current_token.unwrap());
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
                                chunks.push_back(current_token.unwrap());
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
                            chunks.push_back(current_token_value);
                        }
                        current_token = None;
                    }
                    chunks.push_back(Self::Output);
                }
                AstNode::Input => {
                    if let Some(current_token_value) = current_token {
                        if current_token_value.is_nonzero() {
                            chunks.push_back(current_token_value);
                        }
                        current_token = None;
                    }
                    chunks.push_back(Self::Input);
                }
            }

            if chunks.len() == chunks.capacity() {
                sender.send(chunks).unwrap();
                chunks = VecDeque::new();
            }
        }
        if let Some(current_token) = current_token {
            if current_token.is_nonzero() {
                chunks.push_back(current_token);
            }
        }
        if !chunks.is_empty() {
            sender.send(chunks).unwrap();
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
