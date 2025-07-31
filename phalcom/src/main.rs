pub mod cli;
pub mod disasm;

use crate::cli::{cmd_disasm, cmd_parse, cmd_run, cmd_tokenize, cmd_version, Cli, Commands};
use anyhow::Result;
use clap::Parser;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, fmt, Layer};

fn main() -> Result<()> {
    let stdout_log = fmt::layer().pretty();

    tracing_subscriber::registry().with(stdout_log.with_filter(filter::LevelFilter::OFF)).init();

    let cli = Cli::parse();

    match cli.command {
        None => cmd_run(cli),
        Some(Commands::Tokenize(args)) => cmd_tokenize(args),
        Some(Commands::Parse(args)) => cmd_parse(args),
        Some(Commands::Disasm(args)) => cmd_disasm(args),
        Some(Commands::Version) => cmd_version(),
    }
}
