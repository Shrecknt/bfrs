use crate::deduplicate::DeduplicatedAstNode;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum CollectedAstNode {
    ModifyPointer(isize),
    ModifyData(isize),
    Output,
    Input,
    Group(Vec<Self>),
}

impl CollectedAstNode {
    pub fn parse(receiver: &Receiver<DeduplicatedAstNode>) -> Vec<Self> {
        let mut res = Vec::new();
        while let Ok(node) = receiver.recv() {
            let new_node = match &node {
                DeduplicatedAstNode::ModifyPointer(val) => Self::ModifyPointer(*val),
                DeduplicatedAstNode::ModifyData(val) => Self::ModifyData(*val),
                DeduplicatedAstNode::Output => Self::Output,
                DeduplicatedAstNode::Input => Self::Input,
                DeduplicatedAstNode::Group(receiver) => Self::Group(Self::parse(receiver)),
            };
            res.push(new_node);
        }
        res
    }
}
