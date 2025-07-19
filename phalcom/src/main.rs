pub mod cli;
pub mod disasm;

use crate::cli::{cmd_disasm, cmd_parse, cmd_run, cmd_tokenize, cmd_version, Cli, Commands};
use anyhow::{Context, Result};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => cmd_run(cli),
        Some(Commands::Tokenize(args)) => cmd_tokenize(args),
        Some(Commands::Parse(args)) => cmd_parse(args),
        Some(Commands::Disasm(args)) => cmd_disasm(args),
        Some(Commands::Version) => cmd_version(),
    }
}
