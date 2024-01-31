//! Remove a specified node from given syntax tree.
use crate::parser::AstNode;
use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};

pub struct NodeRemover;

/// Removes node from given syntax tree.
impl NodeRemover {
    pub fn remove_node(
        graph: &mut StableDiGraph<AstNode<'_>, ()>,
        node_ix: NodeIndex,
    ) -> Vec<NodeIndex> {
        let mut removed_nodes = vec![];
        let mut bfs = petgraph::visit::Bfs::new(&*graph, node_ix);
        while let Some(connected_node) = bfs.next(&*graph) {
            removed_nodes.push(connected_node);
            graph.remove_node(connected_node);
        }
        removed_nodes
    }
}
