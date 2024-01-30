//! Remove a specified node from given syntax tree.
use crate::parser::AstNode;
use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};

pub struct NodeRemover;

/// Removes node from given syntax tree.
impl NodeRemover {
    pub fn remove_node(graph: &mut StableDiGraph<AstNode<'_>, ()>, node_ix: NodeIndex) {
        graph.remove_node(node_ix);
    }
}
