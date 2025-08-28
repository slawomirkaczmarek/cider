mod command;
mod settings;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// CLI wrapper for the Game Porting Toolkit
#[derive(Parser)]
#[command(arg_required_else_help = true, version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

// TODO: document the CLI

#[derive(Subcommand, Clone)]
enum Command {
    Add {
        prefix: String,
        #[arg(long, short)]
        dir: String,
    },
    Create {
        prefix: String,
        #[arg(long, short)]
        dir: Option<String>,
    },
    Install {
        path: String,
    },
    Prefix {
        #[arg(action, conflicts_with_all = ["list", "settings"], long)]
        default: bool,
        #[arg(action, conflicts_with = "settings", long, short)]
        list: bool,
        #[arg(long, short)]
        prefix: Option<String>,
        #[arg(long = "config", short = 'c', value_parser = parse_key_value_pair)]
        settings: Vec<(String, String)>,
    },
    #[clap(alias = "rm")]
    Remove {
        prefix: String,
    },
    Run {
        command: String,
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
        Some(Command::Add { prefix, dir }) => {
            command::add_prefix(prefix, dir)
        },
        Some(Command::Create { prefix, dir }) => {
            command::create_prefix(prefix, dir)
        },
        Some(Command::Install { path }) => {
            command::install(path)
        },
        Some(Command::Prefix { default, list, prefix, settings }) => {
            if default {
                command::default_prefix(prefix)
            } else if list {
                command::list_prefixes()
            } else {
                command::prefix_config(prefix, settings)
            }
        },
        Some(Command::Remove { prefix }) => {
            command::remove_prefix(Some(prefix))
        },
        Some(Command::Run { command, prefix, args }) => {
            command::run(command, prefix, args)
        },
        None => { Ok(()) },
    }
}
