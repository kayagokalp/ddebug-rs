use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};
use syn::visit::{self, Visit};

use crate::parser::AstNode;

impl std::fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRoot(_) => f.write_str("root"),
            Self::Item(_) => f.write_str("item"),
            Self::ItemFn(_) => f.write_str("item fn"),
            Self::Block(_) => f.write_str("block"),
            Self::LocalStmt(_) => f.write_str("loc_stmt"),
            Self::ExprArray(_) => f.write_str("expr_array"),
            Self::ExprAssign(_) => f.write_str("expr_assign"),
            Self::ExprLet(_) => f.write_str("expr_let"),
        }
    }
}

// Define a struct to represent the syntax tree
#[derive(Debug)]
pub struct SyntaxTree<'a> {
    graph: StableDiGraph<AstNode<'a>, ()>,
    root_node: Option<NodeIndex>,
}

impl<'a> AsRef<StableDiGraph<AstNode<'a>, ()>> for SyntaxTree<'a> {
    fn as_ref(&self) -> &StableDiGraph<AstNode<'a>, ()> {
        &self.graph
    }
}
impl<'a> SyntaxTree<'a> {
    // Constructor function to create a new SyntaxTree
    pub fn new() -> Self {
        SyntaxTree {
            graph: StableDiGraph::new(),
            root_node: None,
        }
    }

    // Function to add a node to the graph
    fn add_node(&mut self, node: AstNode<'a>) -> NodeIndex {
        self.graph.add_node(node)
    }

    // Function to add an edge between two nodes in the graph
    fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
        self.graph.add_edge(source, target, ());
    }

    pub fn root_node(&self) -> Option<NodeIndex> {
        self.root_node
    }
}

// Custom visitor to traverse the syntax tree and build the graph
pub struct GraphBuilder<'a> {
    syntax_tree: &'a mut SyntaxTree<'a>,
    current_node: Option<NodeIndex>,
}

impl<'a> GraphBuilder<'a> {
    pub fn new(syntax_tree: &'a mut SyntaxTree<'a>, current_node: Option<NodeIndex>) -> Self {
        Self {
            syntax_tree,
            current_node,
        }
    }

    pub fn syntax_tree(&self) -> &SyntaxTree<'_> {
        self.syntax_tree
    }
}

/// A macro to insert current node to the graph and visit its child.
macro_rules! insert_and_visit {
    ($self:ident, $ast_node_variant:ident, $ast_node_var:ident, $visit_fn:ident) => {
        let ast_node = AstNode::$ast_node_variant($ast_node_var);
        let node_index = $self.syntax_tree.add_node(ast_node);

        let parent_node = $self.current_node;

        if let Some(parent_node) = $self.current_node {
            $self.syntax_tree.add_edge(parent_node, node_index);
        }

        $self.current_node = Some(node_index);
        visit::$visit_fn($self, $ast_node_var);
        $self.current_node = parent_node;
    };
}

impl<'a> Visit<'a> for GraphBuilder<'a> {
    fn visit_file(&mut self, file: &'a syn::File) {
        insert_and_visit!(self, SourceRoot, file, visit_file);
        // We inserted source root, the only node in the graph is the source root.
        let root_node = self.syntax_tree.graph.node_indices().next();
        self.syntax_tree.root_node = root_node;
    }
    fn visit_item(&mut self, item: &'a syn::Item) {
        insert_and_visit!(self, Item, item, visit_item);
    }

    fn visit_item_fn(&mut self, item_fn: &'a syn::ItemFn) {
        insert_and_visit!(self, ItemFn, item_fn, visit_item_fn);
    }

    fn visit_block(&mut self, block: &'a syn::Block) {
        insert_and_visit!(self, Block, block, visit_block);
    }

    fn visit_local(&mut self, local_stmt: &'a syn::Local) {
        insert_and_visit!(self, LocalStmt, local_stmt, visit_local);
    }

    fn visit_expr_array(&mut self, expr_arr: &'a syn::ExprArray) {
        insert_and_visit!(self, ExprArray, expr_arr, visit_expr_array);
    }

    fn visit_expr_assign(&mut self, expr_assign: &'a syn::ExprAssign) {
        insert_and_visit!(self, ExprAssign, expr_assign, visit_expr_assign);
    }

    fn visit_expr_let(&mut self, let_expr: &'a syn::ExprLet) {
        insert_and_visit!(self, ExprLet, let_expr, visit_expr_let);
    }
}

// TODO: Testing infra is very inefficient. Both from dev ex and performance perspectives (lots of
// unnecessary clones).
#[cfg(test)]
mod tests {
    use crate::parser::{AbstractSyntaxTree, AstNode};

    use super::{GraphBuilder, SyntaxTree};
    use syn::visit::Visit;

    #[derive(Debug, PartialEq, Eq)]
    pub enum ASTNodeType {
        SourceRoot,
        Item,
        ItemFn,
        Block,
        LocalStmt,
        ExprArray,
        ExprAssign,
        ExprLet,
    }

    impl From<AstNode<'_>> for ASTNodeType {
        fn from(value: AstNode) -> Self {
            match value {
                AstNode::SourceRoot(_) => ASTNodeType::SourceRoot,
                AstNode::Item(_) => ASTNodeType::Item,
                AstNode::ItemFn(_) => ASTNodeType::ItemFn,
                AstNode::Block(_) => ASTNodeType::Block,
                AstNode::LocalStmt(_) => ASTNodeType::LocalStmt,
                AstNode::ExprArray(_) => ASTNodeType::ExprArray,
                AstNode::ExprAssign(_) => ASTNodeType::ExprAssign,
                AstNode::ExprLet(_) => ASTNodeType::ExprLet,
            }
        }
    }

    // A test util for testing graphs that collects DFS order
    fn leaf_nodes(ast: &AbstractSyntaxTree) -> Vec<ASTNodeType> {
        use petgraph::visit::Dfs;

        let mut syntax_tree = SyntaxTree::new();
        let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None);
        let file = ast.clone().syn_file();

        // Construct the graph by visiting the entire file.
        graph_builder.visit_file(&file);
        let graph = &graph_builder.syntax_tree.graph;

        // Find SourceRoot.
        let source_root = graph
            .node_indices()
            .find(|node_ix| matches!(graph[*node_ix], AstNode::SourceRoot(_)));

        if let Some(source_root) = source_root {
            // We found the source root. Do a DFS from source root.
            //
            let mut dfs = Dfs::new(&graph, source_root);
            let mut leaf = vec![];
            while let Some(next_node) = dfs.next(graph) {
                leaf.push(graph[next_node].clone().into());
            }
            leaf
        } else {
            vec![]
        }
    }

    #[test]
    fn graph_item_item_fn_block() {
        let test_code = r#"fn main() {}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        let file = parsed_ast.clone().syn_file();

        let mut syntax_tree = SyntaxTree::new();
        let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None);
        graph_builder.visit_file(&file);

        let graph = graph_builder.syntax_tree;
        // root -> item -> item_fn -> block
        assert_eq!(graph.graph.node_count(), 4);

        let leaf_node_types = leaf_nodes(&parsed_ast);
        let expected_leaf_node_types = vec![
            ASTNodeType::SourceRoot,
            ASTNodeType::Item,
            ASTNodeType::ItemFn,
            ASTNodeType::Block,
        ];
        assert_eq!(leaf_node_types, expected_leaf_node_types)
    }

    #[test]
    fn graph_multiple_item_item_fn_block() {
        let test_code = r#"
fn test_fn() {}
fn main() {}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        let file = parsed_ast.clone().syn_file();

        let mut syntax_tree = SyntaxTree::new();
        let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None);
        graph_builder.visit_file(&file);

        let graph = graph_builder.syntax_tree;
        // root -> item -> item_fn -> block
        //     |-> item_fn -> block
        assert_eq!(graph.graph.node_count(), 7);

        let leaf_node_types = leaf_nodes(&parsed_ast);
        let expected_leaf_node_types = vec![
            ASTNodeType::SourceRoot,
            ASTNodeType::Item,
            ASTNodeType::ItemFn,
            ASTNodeType::Block,
            ASTNodeType::Item,
            ASTNodeType::ItemFn,
            ASTNodeType::Block,
        ];
        assert_eq!(leaf_node_types, expected_leaf_node_types)
    }

    #[test]
    fn graph_item_item_fn_block_locstmt() {
        let test_code = r#"
fn test_fn() {
    let b = [10, 10];
    a = 10;
}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        let file = parsed_ast.clone().syn_file();

        let mut syntax_tree = SyntaxTree::new();
        let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None);
        graph_builder.visit_file(&file);

        let graph = graph_builder.syntax_tree;
        // root->item->item_fn->block->local_stmt->expr_assign
        //                          |->expr_array
        assert_eq!(graph.graph.node_count(), 7);

        let leaf_node_types = leaf_nodes(&parsed_ast);
        let expected_leaf_node_types = vec![
            ASTNodeType::SourceRoot,
            ASTNodeType::Item,
            ASTNodeType::ItemFn,
            ASTNodeType::Block,
            ASTNodeType::LocalStmt,
            ASTNodeType::ExprArray,
            ASTNodeType::ExprAssign,
        ];
        assert_eq!(leaf_node_types, expected_leaf_node_types)
    }
}
