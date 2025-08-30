mod command;
mod settings;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// CLI wrapper for the Game Porting Toolkit
#[derive(Parser)]
#[command(arg_required_else_help = true, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Output JSON
    #[arg(global = true, long)]
    json: bool,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Add an existing prefix
    Add {
        /// Prefix name
        prefix: String,
        /// Existing prefix's directory
        #[arg(long, short)]
        dir: String,
    },
    /// Create a new prefix
    Create {
        /// Prefix name
        prefix: String,
        /// Custom directory for the prefix
        #[arg(long, short)]
        dir: Option<String>,
    },
    Install {
        path: String,
    },
    /// Manage prefixes
    Prefix {
        /// Get or set the prefix as default
        #[arg(conflicts_with_all = ["list", "settings"], long)]
        default: bool,
        /// List prefixes
        #[arg(conflicts_with = "settings", long, short)]
        list: bool,
        /// Prefix name
        #[arg(long, short)]
        prefix: Option<String>,
        /// Set a configuration value, <KEY=VALUE>
        #[arg(long = "config", short = 'c', value_parser = parse_key_value_pair)]
        settings: Vec<(String, String)>,
    },
    /// Remove a prefix
    #[clap(alias = "rm")]
    Remove {
        /// Prefix name
        prefix: String,
    },
    /// Run a command
    Run {
        command: String,
        /// Prefix name
        #[arg(long, short)]
        prefix: Option<String>,
        #[arg(last = true)]
        args: Vec<String>,
    },
}

fn parse_key_value_pair(key_value_pair: &str) -> Result<(String, String)> {
    let pos = key_value_pair.find("=").with_context(|| format!("Unable to parse setting string, expects <key>=<value>"))?;
    Ok((key_value_pair[..pos].to_string(), key_value_pair[pos + 1..].to_string()))
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Add { prefix, dir } => {
            command::add_prefix(prefix, dir)
        },
        Command::Create { prefix, dir } => {
            command::create_prefix(prefix, dir)
        },
        Command::Install { path } => {
            command::install(path)
        },
        Command::Prefix { default, list, prefix, settings } => {
            if default {
                command::default_prefix(prefix)
            } else if list {
                command::list_prefixes()
            } else {
                command::prefix_config(prefix, settings)
            }
        },
        Command::Remove { prefix } => {
            command::remove_prefix(Some(prefix))
        },
        Command::Run { command, prefix, args } => {
            command::run(command, prefix, args)
        },
    }
}
