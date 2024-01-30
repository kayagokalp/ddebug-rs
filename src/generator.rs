//! Code generation from given `AbstractSyntaxTree`.

use std::collections::HashMap;

use petgraph::{
    prelude::NodeIndex,
    stable_graph::StableDiGraph,
    visit::{EdgeRef, Walker},
    Direction,
};
use syn::{Block, Expr, ExprArray, ExprAssign, ExprLet, File, Item, ItemFn, Local, Stmt};
use thiserror::Error;

use crate::parser::AstNode;

/// Code generation from the `SyntaxTree`.
pub struct CodeGenerator {
    ix_to_ast_node: HashMap<NodeIndex, GeneratedASTNode>,
}

#[derive(Debug, Error)]
pub enum CodeGeneratorError {
    #[error("Missing root node in the syntax tree")]
    RootNodeMissingInSyntaxTree,
    #[error("File structure could not be generated")]
    FileNotGeneratedFromTree,
    #[error("Mismatch conversion attempted, tried to convert {0} to {1}")]
    MismatchedASTConversion(String, String),
    #[error("SourceRoot does not have an item child")]
    SourceRootDoesNotHaveItemChild,
}

#[derive(Clone)]
pub enum GeneratedASTNode {
    SourceRoot(File),
    Item(Item),
    ItemFn(ItemFn),
    Block(Block),
    LocalStmt(Local),
    ExprArray(ExprArray),
    ExprAssign(ExprAssign),
    ExprLet(ExprLet),
}

impl std::fmt::Debug for GeneratedASTNode {
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

impl From<AstNode<'_>> for GeneratedASTNode {
    fn from(value: AstNode<'_>) -> Self {
        match value {
            AstNode::SourceRoot(source_root) => GeneratedASTNode::SourceRoot(source_root.clone()),
            AstNode::Item(item) => GeneratedASTNode::Item(item.clone()),
            AstNode::ItemFn(item_fn) => GeneratedASTNode::ItemFn(item_fn.clone()),
            AstNode::Block(block) => GeneratedASTNode::Block(block.clone()),
            AstNode::LocalStmt(local_stmt) => GeneratedASTNode::LocalStmt(local_stmt.clone()),
            AstNode::ExprArray(expr_array) => GeneratedASTNode::ExprArray(expr_array.clone()),
            AstNode::ExprAssign(expr_assign) => GeneratedASTNode::ExprAssign(expr_assign.clone()),
            AstNode::ExprLet(expr_let) => GeneratedASTNode::ExprLet(expr_let.clone()),
        }
    }
}

impl TryFrom<GeneratedASTNode> for Stmt {
    type Error = CodeGeneratorError;

    fn try_from(value: GeneratedASTNode) -> Result<Self, Self::Error> {
        match value {
            GeneratedASTNode::LocalStmt(local_stmt) => Ok(Stmt::Local(local_stmt)),
            GeneratedASTNode::ExprArray(expr_arr) => {
                let expr = Expr::Array(expr_arr);
                // TODO: look into this `,` being none.
                Ok(Stmt::Expr(expr, None))
            }
            GeneratedASTNode::ExprAssign(expr_assign) => {
                let expr = Expr::Assign(expr_assign);
                // TODO: look into this `,` being none.
                Ok(Stmt::Expr(expr, None))
            }
            GeneratedASTNode::ExprLet(expr_let) => {
                let expr = Expr::Let(expr_let);
                // TODO: look into this `,` being none.
                Ok(Stmt::Expr(expr, None))
            }
            other => Err(Self::Error::MismatchedASTConversion(
                format!("{other:?}"),
                "stmt".to_owned(),
            )),
        }
    }
}

impl TryFrom<GeneratedASTNode> for Block {
    type Error = CodeGeneratorError;

    fn try_from(value: GeneratedASTNode) -> Result<Self, Self::Error> {
        match value {
            GeneratedASTNode::Block(block) => Ok(block),
            other => Err(Self::Error::MismatchedASTConversion(
                format!("{other:?}"),
                "block".to_owned(),
            )),
        }
    }
}

impl TryFrom<GeneratedASTNode> for ItemFn {
    type Error = CodeGeneratorError;

    fn try_from(value: GeneratedASTNode) -> Result<Self, Self::Error> {
        match value {
            GeneratedASTNode::ItemFn(item_fn) => Ok(item_fn),
            other => Err(Self::Error::MismatchedASTConversion(
                format!("{other:?}"),
                "item_fn".to_owned(),
            )),
        }
    }
}

impl TryFrom<GeneratedASTNode> for Item {
    type Error = CodeGeneratorError;

    fn try_from(value: GeneratedASTNode) -> Result<Self, Self::Error> {
        match value {
            GeneratedASTNode::Item(item) => Ok(item),
            other => Err(Self::Error::MismatchedASTConversion(
                format!("{other:?}"),
                "item".to_owned(),
            )),
        }
    }
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            ix_to_ast_node: HashMap::new(),
        }
    }

    pub fn generate(
        &mut self,
        graph: &StableDiGraph<AstNode<'_>, ()>,
        root_node_ix: NodeIndex,
    ) -> Result<String, CodeGeneratorError> {
        // Get the source root.
        let bfs = petgraph::visit::Bfs::new(graph, root_node_ix);

        let mut order: Vec<_> = bfs.iter(graph).collect();
        order.reverse();

        let mut file = None;

        for node_ix in order {
            let node = &graph[node_ix];
            match node {
                AstNode::SourceRoot(root) => {
                    let items = graph
                        .edges_directed(node_ix, Direction::Outgoing)
                        .map(|edge| edge.target())
                        .map(|target_ix| self.ix_to_ast_node[&target_ix].clone())
                        .map(Item::try_from)
                        .collect::<Result<Vec<Item>, _>>()?;

                    file = Some(File {
                        shebang: root.shebang.clone(),
                        attrs: root.attrs.clone(),
                        items,
                    });
                    break;
                }
                AstNode::Item(_) => {
                    let item_fn = graph
                        .edges_directed(node_ix, Direction::Outgoing)
                        .map(|edge| edge.target())
                        .map(|target_ix| self.ix_to_ast_node[&target_ix].clone())
                        .map(ItemFn::try_from)
                        .find_map(Result::ok);

                    if let Some(item_fn) = item_fn {
                        let item = Item::Fn(item_fn);
                        self.ix_to_ast_node
                            .insert(node_ix, GeneratedASTNode::Item(item));
                    }
                }
                AstNode::ItemFn(item_fn) => {
                    let block = graph
                        .edges_directed(node_ix, Direction::Outgoing)
                        .map(|edge| edge.target())
                        .map(|target_ix| self.ix_to_ast_node[&target_ix].clone())
                        .map(Block::try_from)
                        .find_map(Result::ok)
                        .unwrap_or_else(|| Block {
                            brace_token: Default::default(),
                            stmts: vec![],
                        });

                    let item_fn = ItemFn {
                        attrs: item_fn.attrs.clone(),
                        vis: item_fn.vis.clone(),
                        sig: item_fn.sig.clone(),
                        block: Box::new(block),
                    };

                    self.ix_to_ast_node
                        .insert(node_ix, GeneratedASTNode::ItemFn(item_fn));
                }
                AstNode::Block(block) => {
                    let mut child_stmnts = graph
                        .edges_directed(node_ix, Direction::Outgoing)
                        .map(|edge| edge.target())
                        .map(|target_ix| self.ix_to_ast_node[&target_ix].clone())
                        .map(Stmt::try_from)
                        .collect::<Result<Vec<Stmt>, _>>()?;

                    child_stmnts.reverse();

                    let block = Block {
                        brace_token: block.brace_token,
                        stmts: child_stmnts,
                    };

                    self.ix_to_ast_node
                        .insert(node_ix, GeneratedASTNode::Block(block));
                }
                _ => {
                    // this is a leaf node.
                    self.ix_to_ast_node
                        .insert(node_ix, GeneratedASTNode::from(node.clone()));
                }
            }
        }

        if let Some(file) = file {
            Ok(prettyplease::unparse(&file))
        } else {
            Err(CodeGeneratorError::FileNotGeneratedFromTree)
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::visit::Visit;

    use crate::{
        graph::{GraphBuilder, SyntaxTree},
        parser::AbstractSyntaxTree,
    };

    use super::CodeGenerator;

    #[test]
    fn parse_unparse_parse() {
        let test_code = r#"
fn test_fn() {}
fn main() {}"#;
        let parsed_ast = AbstractSyntaxTree::parse(test_code);
        let file = parsed_ast.clone().syn_file();

        let mut syntax_tree = SyntaxTree::new();
        let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None);
        graph_builder.visit_file(&file);

        let root_node = syntax_tree.root_node();
        let graph = syntax_tree.as_ref();
        let code_generator = CodeGenerator::new(graph, root_node);
        let generated_code = code_generator.generate().unwrap();

        println!("---");
        println!("{generated_code}");

        let reparsed_ast = AbstractSyntaxTree::parse(generated_code);

        assert_eq!(parsed_ast, reparsed_ast)
    }
}
