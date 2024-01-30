mod builder;
mod command;
mod generator;
mod graph;
mod parser;
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
