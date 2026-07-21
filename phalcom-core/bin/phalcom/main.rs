//! The Phalcom programming language compiler and virtual machine driver.

pub mod cli;
pub mod disasm;

use crate::cli::{cmd_check, cmd_disasm, cmd_parse, cmd_run, cmd_tokenize, cmd_version, Cli, Commands};
use anyhow::Result;
use clap::Parser;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{fmt, Layer};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build a per-target filter: only enable `DEBUG` on the specific targets
    // requested via `--trace=<targets>`. A global `LevelFilter::DEBUG` would
    // also enable the compiler's own debug instrumentation, flooding stderr.
    let targets_filter = cli.trace.iter()
        .filter(|t| !t.is_empty())
        .fold(
            Targets::new().with_default(tracing_subscriber::filter::LevelFilter::OFF),
            |acc, t| acc.with_target(t.as_str(), tracing::Level::DEBUG),
        );

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_level(false)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(stderr_layer.with_filter(targets_filter))
        .init();

    // Resolves `--color`/`--plain` once, up front, and installs it for every diagnostic
    // renderer in `phalcom-core` to read (IS §3.2). See
    // `phalcom_core::diagnostics::RENDER_CONFIG`'s docs for why this is a `OnceLock` bridge
    // rather than an explicitly threaded parameter today.
    phalcom_core::diagnostics::install_render_config(cli.render_config());

    let result = match cli.command {
        None => cmd_run(cli),
        Some(Commands::Tokenize(args)) => cmd_tokenize(args),
        Some(Commands::Parse(args)) => cmd_parse(args),
        Some(Commands::Disasm(args)) => cmd_disasm(args),
        Some(Commands::Check(args)) => cmd_check(args),
        Some(Commands::Version) => cmd_version(),
    };

    // Dumped after the command, including on the error path: a program that
    // throws still retired the instructions it retired, and the histogram is a
    // measurement, not a success report. Goes to stderr so the golden corpus and
    // the Wren stdout diff stay byte-exact (`opcode_stats::dump`).
    #[cfg(feature = "opcode-histogram")]
    phalcom_core::opcode_stats::dump();

    result
}
