use crate::{deduplicate::DeduplicatedAstNode, ChunkedReceive, ChunkedReceiverWrapper};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    sync::mpsc::Receiver,
};

#[derive(Debug)]
pub enum CollectedAstNode {
    ModifyPointer { offset: isize },
    ModifyData { new_data: isize },
    ModifyDataOffset { new_data: isize, offset: isize },
    Output,
    Input,
    Group { contents: VecDeque<Self> },
}

pub trait Execute {
    fn execute(self, state: &mut [u8], pointer: &mut isize);
}

impl Execute for VecDeque<CollectedAstNode> {
    fn execute(self, state: &mut [u8], pointer: &mut isize) {
        CollectedAstNode::Group { contents: self }.execute(state, pointer)
    }
}

impl Execute for CollectedAstNode {
    fn execute(self, state: &mut [u8], pointer: &mut isize) {
        let Self::Group { contents } = self else {
            panic!("&self is not Self::Group(..)")
        };
        Self::execute_internal(state, pointer, &contents, false);
    }
}

impl CollectedAstNode {
    pub fn parse(
        receiver: &mut ChunkedReceiverWrapper<
            Receiver<VecDeque<DeduplicatedAstNode>>,
            DeduplicatedAstNode,
        >,
    ) -> VecDeque<Self> {
        let mut res = VecDeque::new();
        while let Some(node) = receiver.next() {
            let new_node = match node {
                DeduplicatedAstNode::ModifyPointer(val) => Self::ModifyPointer { offset: val },
                DeduplicatedAstNode::ModifyData(val) => Self::ModifyData { new_data: val },
                DeduplicatedAstNode::Output => Self::Output,
                DeduplicatedAstNode::Input => Self::Input,
                DeduplicatedAstNode::Group(receiver) => {
                    let mut receiver = ChunkedReceiverWrapper::new(receiver);
                    Self::Group {
                        contents: Self::parse(&mut receiver),
                    }
                }
            };
            res.push_back(new_node);
        }
        res
    }

    fn execute_internal(
        state: &mut [u8],
        pointer: &mut isize,
        program: &VecDeque<Self>,
        is_group: bool,
    ) {
        if is_group && state[*pointer as usize] == 0 {
            return;
        }
        loop {
            for node in program {
                match node {
                    CollectedAstNode::ModifyPointer { offset } => *pointer += *offset,
                    CollectedAstNode::ModifyData { new_data } => {
                        state[*pointer as usize] += *new_data as u8
                    }
                    CollectedAstNode::ModifyDataOffset { new_data, offset } => {
                        state[(*pointer + *offset) as usize] += *new_data as u8
                    }
                    CollectedAstNode::Output => {
                        print!("{}", state[*pointer as usize] as char);
                    }
                    CollectedAstNode::Input => {
                        std::io::stdout().flush().unwrap(); // some output text might not be flushed
                        let input = std::io::stdin().bytes().next().unwrap().unwrap();
                        state[*pointer as usize] = input;
                    }
                    CollectedAstNode::Group { contents } => {
                        Self::execute_internal(state, pointer, contents, true)
                    }
                }
            }
            if state[*pointer as usize] == 0 || !is_group {
                return;
            }
        }
    }
}
