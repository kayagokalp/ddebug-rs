mod builder;
mod graph;
mod parser;
mod searcher;
mod command;

use std::env::current_dir;

use command::Args;
use clap::Parser;
use searcher::{ASTGuidedSearcher, Search};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let current_dir = current_dir()?;
    let target_path = args.path.unwrap_or(current_dir);

    let searcher = ASTGuidedSearcher::new(searcher::Target::Path(&target_path)); 
    searcher.search()?;

    Ok(())
}
