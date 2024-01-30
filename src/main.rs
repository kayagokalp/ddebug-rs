/// Code builder, builds the code using rust compiler.
mod builder;
/// Command definining the CLI for ddebug-rs.
mod command;
/// Code generator, generates the code from syntax tree.
mod generator;
/// Graph generator, generates a (pet)graph (`SyntaxTree`) from the parsed AST.
mod graph;
/// Rust parser interface, using `syn` crate parse rust code into AST nodes.
mod parser;
/// A node remover for the syntax tree.
mod remover;
/// Actual searcher which searches input program space for unnecessary statements.
mod searcher;

use std::env::current_dir;

use clap::Parser;
use command::Args;
use searcher::{ASTGuidedSearcher, Search};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let current_dir = current_dir()?;
    let target_path = args.path.unwrap_or(current_dir);

    let searcher = ASTGuidedSearcher::new(searcher::Target::Path(&target_path));
    searcher.search()?;

    Ok(())
}
