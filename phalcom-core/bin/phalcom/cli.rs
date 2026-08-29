use crate::disasm;
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueHint};
use phalcom_core::compiler::attributes::CompileMode;
use phalcom_core::diagnostics::style::{ColorMode, RenderConfig};
use phalcom_core::vm::VM;
use serde::Serialize;
use std::sync::Arc;
use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum DiagnosticDetailArg {
    Compact,
    Explain,
    Trace,
}

impl From<DiagnosticDetailArg> for phalcom_semantic::DiagnosticDetail {
    fn from(value: DiagnosticDetailArg) -> Self {
        match value {
            DiagnosticDetailArg::Compact => Self::Compact,
            DiagnosticDetailArg::Explain => Self::Explain,
            DiagnosticDetailArg::Trace => Self::Trace,
        }
    }
}

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

    /// Semantic diagnostic detail: compact, explain (default), or trace.
    #[arg(long, value_enum, default_value_t = DiagnosticDetailArg::Explain, global = true)]
    pub(crate) diagnostic_detail: DiagnosticDetailArg,

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
                if let Some(snapshot) = diags.snapshot.as_deref() {
                    let detail: phalcom_semantic::DiagnosticDetail = cli.diagnostic_detail.into();
                    let config = cli.render_config();
                    for (_, module_diags) in diags.iter() {
                        for diag in module_diags {
                            print_rich_semantic_text(diag, snapshot, snapshot.sources().as_ref(), detail, &config);
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

#[derive(Serialize)]
struct SemanticJsonPosition {
    line: usize,
    column: usize,
}

#[derive(Serialize)]
struct SemanticJsonRange {
    start: SemanticJsonPosition,
    end: SemanticJsonPosition,
}

#[derive(Serialize)]
struct SemanticJsonLabel {
    module: String,
    source: Option<String>,
    range: SemanticJsonRange,
    message: String,
}

#[derive(Serialize)]
struct SemanticJsonFix {
    message: String,
    range: Option<SemanticJsonRange>,
    replacement: Option<String>,
}

#[derive(Serialize)]
struct SemanticJsonDiagnostic {
    severity: &'static str,
    code: String,
    message: String,
    module: String,
    source: Option<String>,
    range: SemanticJsonRange,
    labels: Vec<SemanticJsonLabel>,
    notes: Vec<String>,
    helps: Vec<String>,
    explanations: Vec<String>,
    fixes: Vec<SemanticJsonFix>,
    root_cause: Option<String>,
}

fn semantic_severity_name(severity: phalcom_semantic::DiagnosticSeverity) -> &'static str {
    match severity {
        phalcom_semantic::DiagnosticSeverity::Error => "error",
        phalcom_semantic::DiagnosticSeverity::Warning => "warning",
        phalcom_semantic::DiagnosticSeverity::Information => "information",
        phalcom_semantic::DiagnosticSeverity::Hint => "hint",
    }
}

fn semantic_json_range(source: Option<&str>, range: std::ops::Range<usize>) -> SemanticJsonRange {
    let (start_line, start_col) = source.map_or_else(|| byte_offset_to_line_col("", range.start), |text| byte_offset_to_line_col(text, range.start));
    let (end_line, end_col) = source.map_or_else(|| byte_offset_to_line_col("", range.end), |text| byte_offset_to_line_col(text, range.end));
    SemanticJsonRange {
        start: SemanticJsonPosition {
            line: start_line,
            column: start_col,
        },
        end: SemanticJsonPosition {
            line: end_line,
            column: end_col,
        },
    }
}

fn semantic_source<'a>(
    sources: &'a std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    module: &phalcom_modules::identity::ModuleId,
) -> (Option<&'a str>, Option<String>) {
    let Some(unit) = sources.get(module) else {
        return (None, None);
    };
    let path = unit.source.as_ref().map(|source| source.display_path.display().to_string());
    (Some(unit.text.as_ref()), path)
}

fn semantic_report_severity(severity: phalcom_semantic::DiagnosticSeverity) -> phalcom_diagnostics::Severity {
    match severity {
        phalcom_semantic::DiagnosticSeverity::Error => phalcom_diagnostics::Severity::Error,
        phalcom_semantic::DiagnosticSeverity::Warning => phalcom_diagnostics::Severity::Warning,
        phalcom_semantic::DiagnosticSeverity::Information => phalcom_diagnostics::Severity::Information,
        phalcom_semantic::DiagnosticSeverity::Hint => phalcom_diagnostics::Severity::Hint,
    }
}

fn presented_semantic_text(
    presented: &phalcom_semantic::PresentedDiagnostic,
    sources: &std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    config: &RenderConfig,
) -> String {
    let (source, path) = semantic_source(sources, &presented.primary.module);
    let mut snippet_labels = Vec::new();
    if source.is_some() {
        snippet_labels.push(phalcom_diagnostics::Label {
            span: presented.primary.range,
            text: presented.headline.as_str(),
            kind: phalcom_diagnostics::LabelKind::Primary,
        });
        for label in presented.labels.iter().filter(|label| label.span.module == presented.primary.module) {
            snippet_labels.push(phalcom_diagnostics::Label {
                span: label.span.range,
                text: label.message.as_str(),
                kind: phalcom_diagnostics::LabelKind::Secondary,
            });
        }
    }
    let snippets = source
        .map(|source| {
            vec![phalcom_diagnostics::SourceSnippet {
                file: path,
                source,
                labels: snippet_labels,
            }]
        })
        .unwrap_or_default();
    let mut sections = Vec::new();
    sections.push(phalcom_diagnostics::ReportSection {
        kind: phalcom_diagnostics::ReportSectionKind::Explanation,
        lines: presented.explanation.iter().map(|line| line.text.clone()).collect(),
    });
    sections.push(phalcom_diagnostics::ReportSection {
        kind: phalcom_diagnostics::ReportSectionKind::Guidance,
        lines: presented.guidance.iter().map(|line| line.text.clone()).collect(),
    });
    sections.push(phalcom_diagnostics::ReportSection {
        kind: phalcom_diagnostics::ReportSectionKind::Context,
        lines: presented.context.iter().map(|line| line.text.clone()).collect(),
    });
    sections.push(phalcom_diagnostics::ReportSection {
        kind: phalcom_diagnostics::ReportSectionKind::Trace,
        lines: presented
            .trace
            .iter()
            .map(|node| {
                format!(
                    "[e{}] {:?} {:?}/{:?}  {}",
                    node.reference.explanation.0, node.rule, node.status, node.origin, node.text
                )
            })
            .collect(),
    });
    phalcom_diagnostics::format_diagnostic(
        Some(presented.code.as_str()),
        semantic_report_severity(presented.severity),
        &presented.headline,
        &snippets,
        &[],
        &sections,
        config,
    )
}

fn guidance_kind(guidance: &phalcom_semantic::DiagnosticGuidance) -> &'static str {
    match guidance {
        phalcom_semantic::DiagnosticGuidance::ChangeAnnotation { .. } => "change_annotation",
        phalcom_semantic::DiagnosticGuidance::SupplyAssignableValue { .. } => "supply_assignable_value",
        phalcom_semantic::DiagnosticGuidance::UseCallableShape { .. } => "use_callable_shape",
        phalcom_semantic::DiagnosticGuidance::EstablishTypeEvidence { .. } => "establish_type_evidence",
        phalcom_semantic::DiagnosticGuidance::ResolveGenericParameter { .. } => "resolve_generic_parameter",
    }
}

fn print_rich_semantic_json(
    diag: &phalcom_semantic::SemanticDiagnostic,
    snapshot: &phalcom_semantic::SemanticSnapshot,
    sources: &std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    detail: phalcom_semantic::DiagnosticDetail,
) {
    let presenter = phalcom_semantic::DiagnosticPresenter::new(snapshot);
    let presented = presenter.present(diag, detail);
    let full = presenter.present(diag, phalcom_semantic::DiagnosticDetail::Trace);
    let (primary_source, primary_path) = semantic_source(sources, &diag.primary.module);
    let explanation = presented
        .explanation
        .iter()
        .map(|line| {
            let metadata = full.trace.iter().find(|node| node.text == line.text);
            serde_json::json!({
                "rule": metadata.map(|node| node.rule.as_str()),
                "status": metadata.map(|node| node.status.as_str()),
                "origin": metadata.map(|node| node.origin.as_str()),
                "message": line.text,
            })
        })
        .collect::<Vec<_>>();
    let guidance = diag
        .guidance
        .iter()
        .zip(presented.guidance.iter())
        .map(|(guidance, line)| serde_json::json!({ "kind": guidance_kind(guidance), "message": line.text }))
        .collect::<Vec<_>>();
    let trace = presented
        .trace
        .iter()
        .map(|node| {
            serde_json::json!({
                "rule": node.rule.as_str(),
                "status": node.status.as_str(),
                "origin": node.origin.as_str(),
                "message": node.text,
            })
        })
        .collect::<Vec<_>>();
    let labels = presented
        .labels
        .iter()
        .map(|label| {
            let (label_source, label_path) = semantic_source(sources, &label.span.module);
            serde_json::json!({
                "module": label.span.module.to_string(),
                "source": label_path,
                "range": semantic_json_range(label_source.or(primary_source), label.span.range.start..label.span.range.end),
                "message": label.message,
                "role": label.role.as_str(),
            })
        })
        .collect::<Vec<_>>();
    let value = serde_json::json!({
        "severity": semantic_severity_name(presented.severity),
        "code": presented.code.as_str(),
        "message": presented.headline,
        "module": presented.primary.module.to_string(),
        "source": primary_path,
        "range": semantic_json_range(primary_source, presented.primary.range.start..presented.primary.range.end),
        "labels": labels,
        "explanation": explanation,
        "guidance": guidance,
        "context": presented.context.iter().map(|line| line.text.clone()).collect::<Vec<_>>(),
        "trace": trace,
        "fixes": presented.fixes.iter().map(|fix| fix.message.clone()).collect::<Vec<_>>(),
    });
    match serde_json::to_string(&value) {
        Ok(json) => println!("{json}"),
        Err(error) => eprintln!("failed to serialize semantic diagnostic: {error}"),
    }
}

fn print_rich_semantic_text(
    diag: &phalcom_semantic::SemanticDiagnostic,
    snapshot: &phalcom_semantic::SemanticSnapshot,
    sources: &std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    detail: phalcom_semantic::DiagnosticDetail,
    config: &RenderConfig,
) {
    let presented = phalcom_semantic::DiagnosticPresenter::new(snapshot).present(diag, detail);
    eprint!("{}", presented_semantic_text(&presented, sources, config));
    for label in presented.labels.iter().filter(|label| label.span.module != presented.primary.module) {
        eprintln!("related [{}] {}: {}", label.span.module, label.span.range.start, label.message);
    }
}

fn semantic_json_value(
    diag: &phalcom_semantic::SemanticDiagnostic,
    sources: &std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
) -> SemanticJsonDiagnostic {
    let (primary_source, primary_path) = semantic_source(sources, &diag.primary.module);
    let labels = diag
        .labels
        .iter()
        .map(|label| {
            let (label_source, label_path) = semantic_source(sources, &label.span.module);
            SemanticJsonLabel {
                module: label.span.module.to_string(),
                source: label_path,
                range: semantic_json_range(label_source.or(primary_source), label.range.start..label.range.end),
                message: label.message.clone(),
            }
        })
        .collect();
    let fixes = diag
        .fixes
        .iter()
        .map(|fix| {
            let (range, replacement) = match &fix.replacement {
                Some((range, text)) => (Some(semantic_json_range(primary_source, range.start..range.end)), Some(text.clone())),
                None => (None, None),
            };
            SemanticJsonFix {
                message: fix.message.clone(),
                range,
                replacement,
            }
        })
        .collect();
    SemanticJsonDiagnostic {
        severity: semantic_severity_name(diag.severity),
        code: diag.code.as_str().to_string(),
        message: diag.message.clone(),
        module: diag.primary.module.to_string(),
        source: primary_path,
        range: semantic_json_range(primary_source, diag.primary_range.start..diag.primary_range.end),
        labels,
        notes: diag.notes.clone(),
        helps: diag.helps.clone(),
        explanations: diag.explanations.iter().map(|id| format!("{id:?}")).collect(),
        fixes,
        root_cause: diag.root_cause.map(|id| format!("{id:?}")),
    }
}

fn print_semantic_json(
    diag: &phalcom_semantic::SemanticDiagnostic,
    sources: &std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
) {
    let value = semantic_json_value(diag, sources);
    match serde_json::to_string(&value) {
        Ok(json) => println!("{json}"),
        Err(error) => eprintln!("failed to serialize semantic diagnostic: {error}"),
    }
}

fn print_semantic_text(
    diag: &phalcom_semantic::SemanticDiagnostic,
    sources: &std::collections::BTreeMap<phalcom_modules::identity::ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
) {
    let (source, path) = semantic_source(sources, &diag.primary.module);
    eprintln!("{} [{}]: {}", semantic_severity_name(diag.severity), diag.code, diag.message);
    eprint!("{}", diag.render(source, path.as_deref()));
    for label in diag.labels.iter().filter(|label| label.span.module != diag.primary.module) {
        eprintln!("related [{}] {}: {}", label.span.module, label.range.start, label.message);
    }
    for note in &diag.notes {
        eprintln!("note: {note}");
    }
    for help in &diag.helps {
        eprintln!("help: {help}");
    }
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
pub fn cmd_check(args: CheckArgs, detail: DiagnosticDetailArg, render_config: RenderConfig) -> Result<()> {
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
                for diag in analyzed.semantic.all_diagnostics() {
                    if args.format == "json" {
                        print_rich_semantic_json(diag, &analyzed.semantic, &analyzed.sources, detail.into());
                    } else {
                        print_rich_semantic_text(diag, &analyzed.semantic, &analyzed.sources, detail.into(), &render_config);
                    }
                }
                std::process::exit(65);
            }
            Ok(())
        }
        Err(err) => match err {
            phalcom_core::modules::compile::ProgramCompileError::Semantic(diags) => {
                let sources = std::collections::BTreeMap::new();
                for (_, module_diags) in diags.iter() {
                    for diag in module_diags {
                        if args.format == "json" {
                            print_semantic_json(diag, &sources);
                        } else {
                            print_semantic_text(diag, &sources);
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
        },
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
