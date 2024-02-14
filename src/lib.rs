#![feature(vec_push_within_capacity)]

use std::collections::VecDeque;

pub mod ast;
pub mod collect;
pub mod deduplicate;
pub mod optimizations;
pub mod parse;

pub struct ChunkedReceiverWrapper<T, U> {
    receiver: T,
    chunk: VecDeque<U>,
}

impl<T, U> ChunkedReceiverWrapper<T, U> {
    pub fn new(receiver: T) -> Self {
        Self {
            receiver,
            chunk: VecDeque::new(),
        }
    }
}

pub trait ChunkedReceive<T> {
    fn next(&mut self) -> Option<T>;
}
