use crate::disasm;
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueHint};
use phalcom_core::compiler::attributes::CompileMode;
use phalcom_core::diagnostics::style::{ColorMode, RenderConfig};
use phalcom_core::vm::VM;
use std::sync::Arc;
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

    /// Compile contracts (`@requires`/`@ensures`/`@invariant`) in release mode
    /// (U-ANNOT-CONTRACTS plan §3.6): `@requires`'s guard stays woven,
    /// `@ensures`/`@invariant`'s guards are stripped (no-op weave), and
    /// reflectable contract metadata (`MethodObject::contracts`) is retained
    /// by default — pass `--strip-contract-metadata` alongside this flag to
    /// also strip it. Conflicts with `--unchecked`. Only affects the default
    /// (no-subcommand) run path.
    #[arg(long, conflicts_with = "unchecked")]
    pub(crate) release: bool,

    /// Compile contracts in unchecked mode (plan §3.6): every
    /// `@requires`/`@ensures`/`@invariant` guard is stripped (no-op weave)
    /// and reflectable contract metadata is stripped by default. Conflicts
    /// with `--release`. Only affects the default (no-subcommand) run path.
    #[arg(long, conflicts_with = "release")]
    pub(crate) unchecked: bool,

    /// Forces reflectable contract metadata (`MethodObject::contracts`) to be
    /// stripped even under `--release`, where it is otherwise retained by
    /// default (plan §3.6). Has no additional effect under `--unchecked`
    /// (metadata there is already stripped by default) or under the default
    /// debug mode (metadata is always retained in debug).
    #[arg(long)]
    pub(crate) strip_contract_metadata: bool,

    /// Colorize diagnostic output: `auto` (the default — color iff stderr is a TTY and
    /// `NO_COLOR` is unset), `always`, or `never` (traceback spec IS §3.2, PDR-0014).
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    pub(crate) color: ColorMode,

    /// ASCII-only diagnostic rendering: forces the ASCII glyph set and, unless
    /// `--color=always` is also given, disables color too (IS §3.3 — the two axes stay
    /// separable, so `--color=always --plain` yields ASCII glyphs *with* color).
    #[arg(long)]
    pub(crate) plain: bool,

    /// Show core library frames in tracebacks
    #[arg(long)]
    pub(crate) trace_core: bool,

    /// Trace format: `text` or `json`
    #[arg(long, default_value = "text")]
    pub(crate) trace_format: String,

    /// Comma-separated list of trace targets (e.g. `fibers`, `dispatch`)
    #[arg(long, value_delimiter = ',', default_value = "")]
    pub(crate) trace: Vec<String>,

    /// Sub-command to execute
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

impl Cli {
    /// Resolves the `--release`/`--unchecked` flags (mutually exclusive via
    /// `clap`'s `conflicts_with`, so at most one is set) into a
    /// [`CompileMode`], defaulting to [`CompileMode::Debug`] when neither is
    /// passed (U-ANNOT-CONTRACTS plan §3.6).
    pub(crate) fn compile_mode(&self) -> CompileMode {
        if self.unchecked {
            CompileMode::Unchecked
        } else if self.release {
            CompileMode::Release
        } else {
            CompileMode::Debug
        }
    }

    /// Resolves `--color`/`--plain` (plus the real process environment — `NO_COLOR`, whether
    /// stderr is a TTY) into a [`RenderConfig`] (IS §3.2, §3.3).
    pub(crate) fn render_config(&self) -> RenderConfig {
        RenderConfig::from_env(self.color, self.plain)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tokenize phalcom source and print tokens
    Tokenize(TokenizeArgs),

    /// Parse phalcom source and print the AST
    Parse(ParseArgs),

    /// Print disassembled phalcom bytecode
    Disasm(DisasmArgs),

    /// Lex and parse phalcom source, reporting syntax diagnostics without compiling or running
    Check(CheckArgs),

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

/// Lex and parse phalcom code, reporting syntax diagnostics only
#[derive(Args)]
pub struct CheckArgs {
    /// Path to a `.ph` file
    #[arg(value_name = "path", value_hint = clap::ValueHint::FilePath, conflicts_with = "source")]
    path: Option<PathBuf>,

    /// Provide source inline instead of a file
    #[arg(short, long, value_name = "source", conflicts_with = "path")]
    source: Option<String>,

    /// Output format: `text` (human-readable, default) or `json` (machine-readable, one diagnostic per syntax error)
    #[arg(long, value_name = "format", default_value = "text")]
    format: String,
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
    if let Some(p) = &cli.path {
        if !p.exists() {
            eprintln!("Error: File {} does not exist", p.display());
            std::process::exit(66);
        }
    }
    let compile_mode = cli.compile_mode();
    let strip_contract_metadata = cli.strip_contract_metadata;

    let selection = if let Some(src) = &cli.source {
        phalcom_core::modules::compile::EntrySelection::Inline(src.as_str().into())
    } else if let Some(path) = &cli.path {
        let canonical = fs::canonicalize(path).with_context(|| format!("Failed to resolve path {}", path.display()))?;
        if canonical.is_dir() {
            if canonical.join("project.toml").is_file() {
                phalcom_core::modules::compile::EntrySelection::Project(canonical)
            } else if canonical.join("package.ph").is_file() {
                phalcom_core::modules::compile::EntrySelection::Package(canonical)
            } else {
                bail!(
                    "directory '{}' is neither a Project (project.toml) nor a Package (package.ph)",
                    canonical.display()
                );
            }
        } else {
            phalcom_core::modules::compile::EntrySelection::Module(canonical)
        }
    } else {
        let source = match read_source(cli.path.clone(), cli.source.clone()) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(74);
            }
        };
        phalcom_core::modules::compile::EntrySelection::Inline(source.as_str().into())
    };

    let mut vm = VM::new();
    vm.compile_mode = compile_mode;
    vm.strip_contract_metadata = strip_contract_metadata;
    vm.trace_core = cli.trace_core;
    vm.trace_format_json = cli.trace_format == "json";
    vm.trace_fibers = cli.trace.iter().any(|t| t == "fibers");
    if cli.trace.iter().any(|t| t == "dispatch") && !cfg!(feature = "vm-trace") {
        eprintln!("warning: --trace=dispatch requested but the 'vm-trace' cargo feature is not enabled");
    }

    let program_res = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(selection);
    let run_res = match program_res {
        Ok(program) => vm.run_compiled(&program),
        Err(err) => {
            // Structured diagnostics are formatted here, at the user-facing boundary; never reparse formatted errors.
            if let phalcom_core::modules::compile::ProgramCompileError::ModuleLoad(phalcom_modules::ModuleLoadError::Parse { source, error, .. }) = &err {
                if let Ok(source_text) = fs::read_to_string(source) {
                    let path = source.display().to_string();
                    phalcom_core::diagnostics::print_parse(&source_text, Some(&path), &error.kind.to_string(), error.range.clone());
                    std::process::exit(65);
                }
            }
            if let Some(p) = &cli.path {
                if p.is_file() {
                    if let Ok(source) = fs::read_to_string(p) {
                        if let Err(parse_err) = phalcom_ast::parse_source(&source, 0) {
                            let message = parse_err.kind.to_string();
                            let path_str = fs::canonicalize(p).ok().map(|p| p.display().to_string());
                            phalcom_core::diagnostics::print_parse(&source, path_str.as_deref(), &message, parse_err.range);
                            std::process::exit(65);
                        }
                    }
                }
            } else if let Some(src) = &cli.source {
                if let Err(parse_err) = phalcom_ast::parse_source(src, 0) {
                    let message = parse_err.kind.to_string();
                    phalcom_core::diagnostics::print_parse(src, None, &message, parse_err.range);
                    std::process::exit(65);
                }
            }
            if let phalcom_core::modules::compile::ProgramCompileError::Semantic(diags) = &err {
                for (module, module_diags) in diags.iter() {
                    for diag in module_diags {
                        eprintln!("Semantic error in {module} [{}]: {}", diag.code, diag.message);
                    }
                }
                std::process::exit(65);
            }
            eprintln!("Compile error: {err}");
            std::process::exit(65);
        }
    };

    let leaks = vm.resources.leaks();
    if !leaks.is_empty() {
        for (kind, site) in &leaks {
            let site_str = site.map(|s| format!("{}:{}", s.start, s.end)).unwrap_or_else(|| "unknown".to_string());
            eprintln!("Unclosed resource kind: {} opened at {}", kind, site_str);
            if kind.to_string() == "BufferedWriter" {
                eprintln!("Note: pending bytes may be lost for unclosed BufferedWriter");
            }
        }
        if vm.strict_resources {
            std::process::exit(70);
        }
    }

    if let Err(err) = run_res {
        match err {
            phalcom_core::error::PhError::Compile(_) | phalcom_core::error::PhError::Parse(_) => {
                std::process::exit(65);
            }
            phalcom_core::error::PhError::Runtime(_) | phalcom_core::error::PhError::ModuleInitialization(_) => {
                std::process::exit(70);
            }
            phalcom_core::error::PhError::Io(_) => {
                std::process::exit(74);
            }
            _ => {
                std::process::exit(1);
            }
        }
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
    let program = phalcom_ast::parse_source(&source, 0)?;
    println!("{program:#?}");
    Ok(())
}

/// Lexes and parses source only (no compile, no run) and reports syntax
/// diagnostics.
///
/// Exits `0` with no output on a clean parse; exits `65` and prints exactly one
/// diagnostic otherwise (the front end currently stops at the first syntax
/// error). `--format json` emits a single-line, machine-readable object
/// intended for editor tooling (e.g. an LSP-less VS Code extension shelling
/// out per save); `--format text` (default) reuses the existing span-aware
/// renderer.
pub fn cmd_check(args: CheckArgs) -> Result<()> {
    let selection = if let Some(p) = args.path.clone() {
        if !p.exists() {
            eprintln!("Error: File {} does not exist", p.display());
            std::process::exit(66);
        }
        if p.is_dir() {
            if p.join("project.toml").exists() {
                phalcom_core::modules::compile::EntrySelection::Project(p)
            } else {
                phalcom_core::modules::compile::EntrySelection::Package(p)
            }
        } else {
            phalcom_core::modules::compile::EntrySelection::Module(p)
        }
    } else if let Some(source) = args.source.clone() {
        phalcom_core::modules::compile::EntrySelection::Inline(Arc::from(source))
    } else {
        eprintln!("Error: Either a path or inline source must be provided to check");
        std::process::exit(64);
    };

    match phalcom_core::modules::compile::ProgramAnalyzer::analyze_entry_selection(selection) {
        Ok(analyzed) => {
            if analyzed.semantic.has_errors() {
                for (module, diags) in analyzed.semantic.diagnostics.iter() {
                    let source_text = analyzed
                        .sources
                        .get(module)
                        .map(|u| u.text.clone())
                        .unwrap_or_else(|| Arc::from(""));
                    let path_str = analyzed
                        .sources
                        .get(module)
                        .and_then(|u| u.source.as_ref())
                        .map(|s| s.display_path.display().to_string());

                    if args.format == "json" {
                        for diag in diags.iter() {
                            let (start_line, start_col) =
                                byte_offset_to_line_col(&source_text, diag.primary_range.start);
                            let (end_line, end_col) =
                                byte_offset_to_line_col(&source_text, diag.primary_range.end);
                            println!(
                                "{{\"severity\":\"error\",\"code\":{},\"message\":{},\"module\":{},\"range\":{{\"start\":{{\"line\":{},\"column\":{}}},\"end\":{{\"line\":{},\"column\":{}}}}}}}",
                                json_escape(diag.code.as_str()),
                                json_escape(&diag.message),
                                json_escape(&module.to_string()),
                                start_line,
                                start_col,
                                end_line,
                                end_col
                            );
                        }
                    } else {
                        for diag in diags.iter() {
                            let range = diag.primary_range.start..diag.primary_range.end;
                            phalcom_core::diagnostics::print_parse(
                                &source_text,
                                path_str.as_deref(),
                                &format!("{}: {}", diag.code, diag.message),
                                range,
                            );
                        }
                    }
                }
                std::process::exit(65);
            }
            Ok(())
        }
        Err(err) => {
            match err {
                phalcom_core::modules::compile::ProgramCompileError::Semantic(diags) => {
                    if args.format == "json" {
                        for (module, module_diags) in diags.iter() {
                            for diag in module_diags {
                                println!(
                                    "{{\"severity\":\"error\",\"code\":{},\"message\":{},\"module\":{}}}",
                                    json_escape(diag.code.as_str()),
                                    json_escape(&diag.message),
                                    json_escape(&module.to_string()),
                                );
                            }
                        }
                    } else {
                        for (module, module_diags) in diags.iter() {
                            for diag in module_diags {
                                eprintln!("Semantic error in {module} [{}]: {}", diag.code, diag.message);
                            }
                        }
                    }
                    std::process::exit(65);
                }
                phalcom_core::modules::compile::ProgramCompileError::ModuleLoad(phalcom_modules::ModuleLoadError::Parse { source, error, .. }) => {
                    let message = error.kind.to_string();
                    let source_text = fs::read_to_string(&source).unwrap_or_default();
                    let path_display = source.display().to_string();
                    if args.format == "json" {
                        let (start_line, start_col) = byte_offset_to_line_col(&source_text, error.range.start);
                        let (end_line, end_col) = byte_offset_to_line_col(&source_text, error.range.end);
                        println!(
                            "{{\"severity\":\"error\",\"code\":\"SyntaxError\",\"message\":{},\"range\":{{\"start\":{{\"line\":{},\"column\":{}}},\"end\":{{\"line\":{},\"column\":{}}}}}}}",
                            json_escape(&message),
                            start_line,
                            start_col,
                            end_line,
                            end_col
                        );
                    } else {
                        phalcom_core::diagnostics::print_parse(&source_text, Some(&path_display), &message, error.range.clone());
                    }
                    std::process::exit(65);
                }
                phalcom_core::modules::compile::ProgramCompileError::Parse(parse_err) => {
                    let message = parse_err.kind.to_string();
                    let source_text = if let Some(ref p) = args.path {
                        fs::read_to_string(p).unwrap_or_default()
                    } else {
                        args.source.clone().unwrap_or_default()
                    };
                    let path_display = args
                        .path
                        .as_ref()
                        .and_then(|p| fs::canonicalize(p).ok())
                        .map(|p| p.display().to_string())
                        .or_else(|| args.path.as_ref().map(|p| p.display().to_string()));
                    if args.format == "json" {
                        let (start_line, start_col) = byte_offset_to_line_col(&source_text, parse_err.range.start);
                        let (end_line, end_col) = byte_offset_to_line_col(&source_text, parse_err.range.end);
                        println!(
                            "{{\"severity\":\"error\",\"code\":\"SyntaxError\",\"message\":{},\"range\":{{\"start\":{{\"line\":{},\"column\":{}}},\"end\":{{\"line\":{},\"column\":{}}}}}}}",
                            json_escape(&message),
                            start_line,
                            start_col,
                            end_line,
                            end_col
                        );
                    } else {
                        phalcom_core::diagnostics::print_parse(&source_text, path_display.as_deref(), &message, parse_err.range.clone());
                    }
                    std::process::exit(65);
                }
                _ => {
                    eprintln!("Check error: {err}");
                    std::process::exit(65);
                }
            }
        }
    }
}

/// Converts a 0-based UTF-8 byte offset into a 1-based `(line, column)` pair.
///
/// Both line and column count from `1`, matching editor conventions (LSP
/// positions are 0-based instead, but this CLI's only consumer today is the
/// `--format json` diagnostic above, which a client re-bases as needed).
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for ch in source[..offset.min(source.len())].chars() {
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Escapes a string as a JSON string literal (quotes included).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
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
