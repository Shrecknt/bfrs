use std::collections::VecDeque;

use crate::collect::CollectedAstNode;

pub fn collapse_unbounded(nodes: VecDeque<CollectedAstNode>) -> VecDeque<CollectedAstNode> {
    nodes
}
