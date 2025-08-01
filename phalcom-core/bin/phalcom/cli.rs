use crate::disasm;
use anyhow::{bail, Context, Result};
use clap::{arg, command, Args, Parser, Subcommand, ValueHint};
use phalcom_core::compiler::lib::{compile, CompilerError};
use phalcom_core::vm::VM;
use std::{fs, path::PathBuf};

/// Run, tokenize, parse, or disassemble phalcom source.
#[derive(Parser)]
#[command(author, about, long_about = None)]
pub struct Cli {
    /// Path to a `.ph` file
    #[arg(value_name = "path", value_hint = ValueHint::FilePath, conflicts_with = "source")]
    pub(crate) path: Option<PathBuf>,

    /// Provide source inline instead of a file
    #[arg(short = 'i', long, value_name = "source", conflicts_with = "path")]
    pub(crate) source: Option<String>,

    /// Sub-command to execute
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tokenize phalcom source and print tokens
    Tokenize(TokenizeArgs),

    /// Parse phalcom source and print the AST
    Parse(ParseArgs),

    /// Print disassembled phalcom bytecode
    Disasm(DisasmArgs),

    /// Print version
    Version,
}

/// Tokenize phalcom code
#[derive(Args)]
pub struct TokenizeArgs {
    /// Path to a `.ph` file
    #[arg(value_name = "path", value_hint = clap::ValueHint::FilePath, conflicts_with = "source")]
    path: Option<PathBuf>,

    /// Provide source inline instead of a file
    #[arg(short, long, value_name = "source", conflicts_with = "path")]
    source: Option<String>,
}

/// Parse phalcom code
#[derive(Args)]
pub struct ParseArgs {
    /// Path to a `.ph` file
    #[arg(value_name = "path", value_hint = clap::ValueHint::FilePath, conflicts_with = "source")]
    path: Option<PathBuf>,

    /// Provide source inline instead of a file
    #[arg(short, long, value_name = "source", conflicts_with = "path")]
    source: Option<String>,
}

/// Disassemble phalcom code
#[derive(Args)]
pub struct DisasmArgs {
    /// Path to a `.ph` file
    #[arg(value_name = "path", value_hint = ValueHint::FilePath, conflicts_with = "source")]
    path: Option<PathBuf>,

    /// Provide source inline instead of a file
    #[arg(short, long, value_name = "source", conflicts_with = "path")]
    source: Option<String>,
}

pub fn cmd_run(cli: Cli) -> Result<()> {
    let source = read_source(cli.path, cli.source)?;
    let mut vm = VM::new();
    let closure = compile(&mut vm, &source)?;
    let module = vm.create_module_from_str("<main>", &source);
    match vm.run_module(module, closure) {
        Ok(value) => println!("{value}"),
        Err(e) => eprintln!("{e}"),
    }
    Ok(())
}

pub fn cmd_tokenize(args: TokenizeArgs) -> Result<()> {
    let source = read_source(args.path, args.source)?;
    let lexer = phalcom_ast::lexer::Lexer::new(&source);
    for token in lexer {
        let token = token.unwrap();
        println!("{token:?}");
    }
    Ok(())
}

pub fn cmd_parse(args: ParseArgs) -> Result<()> {
    let source = read_source(args.path, args.source)?;
    let parser = phalcom_ast::parser::ProgramParser::new();
    let lexer = phalcom_ast::lexer::Lexer::new(&source);
    let program = parser.parse(lexer).map_err(CompilerError::from)?;
    println!("{program:#?}");
    Ok(())
}

pub fn cmd_disasm(args: DisasmArgs) -> Result<()> {
    let source = read_source(args.path, args.source)?;
    disasm::disassemble_source(&source)?;
    Ok(())
}

pub fn cmd_version() -> Result<()> {
    println!("Phalcom {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Reads source either from a path or an inline string. Enforces that exactly
/// one of the two options is provided.
fn read_source(path: Option<PathBuf>, source: Option<String>) -> Result<String> {
    match (path, source) {
        (Some(p), None) => fs::read_to_string(&p).with_context(|| format!("Failed to read file {}", p.display())),
        (None, Some(s)) => Ok(s),
        _ => bail!("Must provide either a path or --source/-s"),
    }
}
