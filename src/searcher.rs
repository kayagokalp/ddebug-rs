//! A searcher over the possible source code space for finding minimal reproduction of an error.
//!
//! Currently only AST guided searcher is implemented.
//!
//! AST guided searcher roughly works as:
//!
//! 1. Build target project using `CodeBuilder` and collect error codes.
//! 2. Find which file causes the user specified error.
//! 3. Parse the file, to generate AST as a graph.
//! 4. Start doing a BFS over the graph. Remove a node and check if the `same` error code still exists.
//!    4a. If same error code still exists mark it unncessary and continue with BFS order.
//!    4b. If error changed or disappeared, start a new BFS from that node.
//! 5. Continue until all nodes are visited or removing all childs of a node changes the error.

use std::path::{Path, PathBuf};
use syn::visit::Visit;
use thiserror::Error;

use crate::{
    builder::{CodeBuilder, CodeBuilderError},
    generator::CodeGenerator,
    graph::{GraphBuilder, SyntaxTree},
    parser::AbstractSyntaxTree,
    remover::NodeRemover,
};
pub trait Search {
    fn search(self) -> Result<(), SearcherError>;
}

pub enum Target<'a> {
    Path(&'a Path),
}

impl<'a> From<Target<'a>> for CodeBuilder<'a> {
    fn from(value: Target<'a>) -> Self {
        match value {
            Target::Path(target_path) => CodeBuilder::Path(target_path),
        }
    }
}

pub struct ASTGuidedSearcher<'a> {
    target: Target<'a>,
}

impl<'a> ASTGuidedSearcher<'a> {
    pub fn new(target: Target<'a>) -> Self {
        Self { target }
    }
}

#[derive(Error, Debug)]
pub enum SearcherError {
    #[error("Error while trying to build code variant: {0}")]
    BuildOperationError(CodeBuilderError),
    #[error("Error source file is missing for error: {0}")]
    ErrorSourceFileIsMissing(String),
    #[error("Cannot find source of error file locally at: {0}")]
    ErrorSourceFileNotFound(PathBuf),
    #[error("AST seems to be missing a root node")]
    RootNodeFound,
}

impl From<CodeBuilderError> for SearcherError {
    fn from(value: CodeBuilderError) -> Self {
        Self::BuildOperationError(value)
    }
}

impl Search for ASTGuidedSearcher<'_> {
    fn search(self) -> Result<(), SearcherError> {
        let code_builder = CodeBuilder::from(self.target);
        let variant_errors = code_builder.collect_errors()?;

        // TODO: Maybe add an option for users to be able to specify this.
        let master_error = variant_errors.errors.first();

        if let Some(master_error) = master_error {
            println!("error -> {master_error:?}");
            // We are searching the root for this error.
            let root_file = master_error.source_file.as_ref().ok_or_else(|| {
                SearcherError::ErrorSourceFileIsMissing(master_error.error_src.clone())
            })?;

            let file_str = std::fs::read_to_string(root_file)
                .map_err(|_| SearcherError::ErrorSourceFileNotFound(root_file.to_path_buf()))?;
            let ast = AbstractSyntaxTree::parse(file_str);

            let file = ast.syn_file();

            let mut syntax_tree = SyntaxTree::new();
            let mut graph_builder = GraphBuilder::new(&mut syntax_tree, None, None);
            graph_builder.visit_file(&file);
            let root = graph_builder
                .root_node()
                .ok_or(SearcherError::RootNodeFound)?;

            let graph = graph_builder.syntax_tree().as_ref();
            let mut bfs = petgraph::visit::Bfs::new(graph, root);
            // Omit root node of the graph.
            let _ = bfs.next(graph);

            let mut code_generator = CodeGenerator::new();
            while let Some(node_to_check) = bfs.next(graph) {
                let mut invariant_graph = graph.clone();
                NodeRemover::remove_node(&mut invariant_graph, node_to_check);
                //println!("{:?}", petgraph::dot::Dot::new(&invariant_graph));
                let generated_code = code_generator.generate(&invariant_graph, root).unwrap();

                println!("{generated_code}");
                println!("----------------");
            }
        }
        Ok(())
    }
}
