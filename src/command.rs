use std::path::PathBuf;

use clap::Parser;

/// A delta debugger tool for finding minimally reproducable versions of programs.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path of the target project.
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}
