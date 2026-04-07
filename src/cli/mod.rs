//! CLI argument parsing and command dispatch.

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;

/// Self-contained LaTeX to PDF compiler
#[derive(Parser)]
#[command(name = "texforge", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Remove build artifacts
    Clean,
    /// Initialize a texforge project in the current directory
    Init,
    /// Create a new project from a template
    New {
        /// Project name
        name: String,
        /// Template name (default: basic)
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Compile project to PDF
    Build {
        /// Watch for file changes and rebuild automatically
        #[arg(long)]
        watch: bool,
        /// Debounce delay in seconds before rebuilding (default: 10)
        #[arg(long, default_value = "2")]
        delay: u64,
    },
    /// Format .tex files
    Fmt {
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },
    /// Lint project without compiling
    Check,
    /// Manage templates
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    /// Manage global configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// List available templates
    List {
        /// Also show templates available in the remote registry
        #[arg(long)]
        all: bool,
    },
    /// Add a template from URL or registry
    Add { source: String },
    /// Remove a template
    Remove { name: String },
    /// Validate template compatibility
    Validate { name: String },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Get a config value
    Get { key: String },
    /// Set a config value
    Set { key: String, value: String },
    /// List all config values
    List,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        match self.command {
            Commands::Clean => commands::clean::execute(),
            Commands::Init => commands::init::execute(),
            Commands::New { name, template } => commands::new::execute(&name, template.as_deref()),
            Commands::Build { watch, delay } => {
                if watch {
                    commands::build::watch(delay)
                } else {
                    commands::build::execute()
                }
            }
            Commands::Fmt { check } => commands::fmt::execute(check),
            Commands::Check => commands::check::execute(),
            Commands::Template { action } => match action {
                TemplateAction::List { all } => commands::template::list(all),
                TemplateAction::Add { source } => commands::template::add(&source),
                TemplateAction::Remove { name } => commands::template::remove(&name),
                TemplateAction::Validate { name } => commands::template::validate(&name),
            },
            Commands::Config { action } => match action {
                ConfigAction::Get { key } => commands::config::get(&key),
                ConfigAction::Set { key, value } => commands::config::set(&key, &value),
                ConfigAction::List => commands::config::list(),
            },
        }
    }
}
