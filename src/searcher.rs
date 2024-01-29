//! A searcher over the possible source code space for finding minimal reproduction of an error.
//!
//! Currently only AST guided searcher is implemented.
//!
//! AST guided searcher roughly works as:
//!
//! 1. Build target project using `CodeBuilder` and collect error codes.
//! 2. Find which file causes the user specified error.
//! 3. Parse the file, to generate AST as a graph.
//! 4. Start doing a BFS over the graph. Remove a node check if the `same` error code still exists.
//!    4a. If same error code still exists continue with BFS order.
//!    4b. If error changed or disappeared, start a new BFS from that node and mark the node as the
//!    last solution.
//! 5. Continue until removing a child of the current solution does not removes the error or there
//!    is no un-visited node in the graph.

use std::path::Path;

use crate::builder::CodeBuilder;
pub trait Search {
    fn search(self);
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

impl Search for ASTGuidedSearcher<'_> {
    fn search(self) {
        let code_builder = CodeBuilder::from(self.target);
    }
}
