//! texforge — Self-contained LaTeX to PDF compiler CLI.
//!
//! A command-line tool that compiles LaTeX documents to PDF without requiring
//! TeX Live, MiKTeX, or any external LaTeX distribution.

mod cli;
mod commands;
mod compiler;
mod domain;
mod error;
mod formatter;
mod linter;
mod utils;

use anyhow::Result;
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
