mod compare;
mod env;
mod measure;
mod model;
mod store;
mod suite;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};

use model::*;
use suite::Lane;

#[derive(Parser)]
#[command(name = "phalcom-perf", about = "Phalcom VM reproducible benchmark and performance tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Backward compat: Run only corpus
    #[arg(long)]
    corpus_only: bool,

    /// Backward compat: Run only benchmarks
    #[arg(long)]
    bench_only: bool,

    /// Backward compat: Include pending cases
    #[arg(long)]
    pending: bool,

    /// Backward compat: Filter label
    #[arg(long)]
    label: Option<String>,

    /// Backward compat: Force debug binary
    #[arg(long)]
    debug: bool,

    /// Backward compat: Top N cases
    #[arg(long, default_value_t = 20)]
    top: usize,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run default or specified suite; optionally record
    Run(RunArgs),
    /// Guarded alternating A/B between two binaries
    Ab(AbArgs),
    /// Compare two stored runs
    Compare(CompareArgs),
    /// Display one stored run
    Show(ShowArgs),
    /// List stored runs
    List(ListArgs),
    /// Print representation sizes (no exec)
    Layout(LayoutArgs),
    /// Manage baseline runs
    Baseline(BaselineArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    #[arg(long)]
    suite: Option<String>,

    #[arg(long)]
    case: Option<String>,

    #[arg(long, default_value_t = 5)]
    samples: usize,

    #[arg(long, default_value_t = 1)]
    warmup: usize,

    #[arg(long)]
    heavy: bool,

    #[arg(long)]
    binary: Option<PathBuf>,

    #[arg(long)]
    record: bool,

    #[arg(long)]
    name: Option<String>,

    #[arg(long, default_value_t = 20)]
    top: usize,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    corpus_only: bool,

    #[arg(long)]
    bench_only: bool,

    #[arg(long)]
    pending: bool,

    #[arg(long)]
    label: Option<String>,

    #[arg(long)]
    debug: bool,
}

#[derive(Parser, Debug)]
struct AbArgs {
    #[arg(long)]
    baseline_bin: PathBuf,

    #[arg(long)]
    candidate_bin: PathBuf,

    #[arg(long, default_value = "representation")]
    suite: String,

    #[arg(long, default_value_t = 5)]
    pairs: usize,

    #[arg(long)]
    quick: bool,

    #[arg(long)]
    record: bool,

    #[arg(long)]
    name: Option<String>,

    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct CompareArgs {
    baseline_ref: String,
    candidate_ref: String,

    #[arg(long)]
    gate: bool,

    #[arg(long)]
    allow_host_mismatch: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct ShowArgs {
    run_ref: String,

    #[arg(long)]
    json: bool,

    #[arg(long, default_value_t = 20)]
    top: usize,
}

#[derive(Parser, Debug)]
struct ListArgs {
    #[arg(long, default_value_t = 20)]
    limit: usize,

    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct LayoutArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct BaselineArgs {
    #[command(subcommand)]
    command: BaselineSubcommands,
}

#[derive(Subcommand, Debug)]
enum BaselineSubcommands {
    List {
        #[arg(long)]
        json: bool,
    },
    Promote {
        run_id: String,
        #[arg(long)]
        name: String,
    },
}

fn workspace_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    root.canonicalize().unwrap_or(root)
}

fn find_phalcom_binary(force_debug: bool) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("phalcom");
            if candidate.exists() {
                return candidate;
            }
        }
    }
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target"));
    let release = target.join("release/phalcom");
    let debug = target.join("debug/phalcom");

    if !force_debug && release.exists() {
        return release;
    }
    if debug.exists() {
        if !force_debug {
            eprintln!("warning: no release binary at {} — using debug (timings not representative)", release.display());
        }
        return debug;
    }
    eprintln!(
        "error: no phalcom binary found at {} or {}\nbuild one first: cargo build -r -p phalcom-core --bin phalcom",
        release.display(),
        debug.display()
    );
    std::process::exit(2);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Layout(args)) => handle_layout(args),
        Some(Commands::Baseline(args)) => handle_baseline(args),
        Some(Commands::List(args)) => handle_list(args),
        Some(Commands::Show(args)) => handle_show(args),
        Some(Commands::Compare(args)) => handle_compare(args),
        Some(Commands::Run(args)) => handle_run(args),
        Some(Commands::Ab(args)) => handle_ab(args),
        None => {
            // Backward compatibility fallback to `run`
            let args = RunArgs {
                suite: None,
                case: None,
                samples: 1,
                warmup: 0,
                heavy: false,
                binary: None,
                record: false,
                name: None,
                top: cli.top,
                json: false,
                corpus_only: cli.corpus_only,
                bench_only: cli.bench_only,
                pending: cli.pending,
                label: cli.label,
                debug: cli.debug,
            };
            handle_run(args);
        }
    }
}

fn handle_layout(args: LayoutArgs) {
    let layouts = env::capture_layouts();
    if args.json {
        println!("{}", serde_json::to_string_pretty(&layouts).unwrap());
    } else {
        println!("Type Layouts (representation sizes):");
        println!("{:<25} {:>10} {:>10}", "Type Name", "Size (B)", "Align (B)");
        println!("{}", "-".repeat(47));
        for (name, layout) in layouts {
            println!("{:<25} {:>10} {:>10}", name, layout.size_bytes, layout.align_bytes);
        }
    }
}

fn handle_baseline(args: BaselineArgs) {
    let ws = workspace_root();
    match args.command {
        BaselineSubcommands::List { json } => {
            let index = store::load_baseline_index(&ws).unwrap_or_default();
            if json {
                println!("{}", serde_json::to_string_pretty(&index).unwrap());
            } else {
                println!("Baselines:");
                for (name, run_id) in index.baselines {
                    println!("  {:<15} -> {run_id}", name);
                }
            }
        }
        BaselineSubcommands::Promote { run_id, name } => match store::promote(&ws, &run_id, &name) {
            Ok(path) => {
                println!("Promoted run '{run_id}' to baseline '{name}' ({})", path.display());
            }
            Err(err) => {
                eprintln!("error promoting run: {err}");
                std::process::exit(2);
            }
        },
    }
}

fn handle_list(args: ListArgs) {
    let ws = workspace_root();
    let runs = store::list_runs(&ws, args.limit);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&runs).unwrap());
    } else {
        println!("{:<30} {:<15} {:<10} {:<8}", "Run ID", "Timestamp", "Suite", "Cases");
        println!("{}", "-".repeat(65));
        for (run_id, ts, suite_name, cases) in runs {
            println!("{:<30} {:<15} {:<10} {:<8}", run_id, ts, suite_name, cases);
        }
    }
}

fn handle_show(args: ShowArgs) {
    let ws = workspace_root();
    match store::load_run(&ws, &args.run_ref) {
        Ok(run) => {
            if args.json {
                println!("{}", run.to_json_string().unwrap());
            } else {
                println!("Run ID: {}", run.run_id);
                println!("Suite: {} ({})", run.suite.name, run.suite.path);
                println!("Git SHA: {} ({})", run.git.short_sha, run.git.branch);
                println!("Host: {}", run.host.host_key);
                println!("Resource Quality: {:?}", run.resource_quality);
                println!("\nCases ({} total):", run.cases.len());
                for case in run.cases.iter().take(args.top) {
                    if let Some(agg) = &case.aggregate {
                        println!("  {:<30} {:>8.2} ms wall (mad {:>.2})", case.id, agg.wall.median, agg.wall.mad);
                    } else {
                        println!("  {:<30} verification failure / process error", case.id);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    }
}

fn handle_compare(args: CompareArgs) {
    let ws = workspace_root();
    let base_run = match store::load_run(&ws, &args.baseline_ref) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error loading baseline '{}': {e}", args.baseline_ref);
            std::process::exit(2);
        }
    };
    let cand_run = match store::load_run(&ws, &args.candidate_ref) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error loading candidate '{}': {e}", args.candidate_ref);
            std::process::exit(2);
        }
    };

    let comp = compare::compare_runs(&base_run, &cand_run, args.allow_host_mismatch, args.gate);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&comp).unwrap());
    } else {
        println!("Comparison: {} vs {}", base_run.run_id, cand_run.run_id);
        println!("Compatible: {} (reasons: {:?})", comp.compatible, comp.incompatibility_reasons);
        if !comp.layout_delta.is_empty() {
            println!("Layout Deltas:");
            for (type_name, (b, c)) in &comp.layout_delta {
                println!("  {type_name}: {} B -> {} B", b.size_bytes, c.size_bytes);
            }
        }
        println!("\n{:<30} {:>10} {:>10} {:>10} {:>8}", "Case ID", "Base (ms)", "Cand (ms)", "Delta %", "Gate");
        println!("{}", "-".repeat(72));
        for case in &comp.cases {
            let base_ms = case.baseline_agg.as_ref().map(|a| a.wall.median).unwrap_or(0.0);
            let cand_ms = case.candidate_agg.as_ref().map(|a| a.wall.median).unwrap_or(0.0);
            let delta_pct = case.delta_pct.unwrap_or(0.0);
            println!(
                "{:<30} {:>10.2} {:>10.2} {:>+9.2}% {:>8}",
                case.id, base_ms, cand_ms, delta_pct, case.gate_result
            );
        }
        println!("\nVerdict: {:?}", comp.verdict);
    }

    if args.gate && comp.verdict == Verdict::Regressions {
        std::process::exit(1);
    }
}

fn handle_run(args: RunArgs) {
    let ws = workspace_root();
    let bin = args.binary.unwrap_or_else(|| find_phalcom_binary(args.debug));

    let meter = measure::ResourceMeter::detect();
    let git = env::capture_git();
    let build = env::capture_build(&bin);
    let host = env::capture_host();
    let layouts = env::capture_layouts();

    let now_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let run_id = args.name.clone().unwrap_or_else(|| format!("{}_{}", now_ts, git.short_sha));

    let suite_name = args.suite.clone().unwrap_or_else(|| "default".into());
    let mut case_results = Vec::new();
    let mut passed_cnt = 0;
    let mut failed_cnt = 0;

    if let Ok((_manifest, suite_cases)) = suite::load_suite(&ws, &suite_name) {
        for case_spec in suite_cases {
            if let Some(ref filter) = args.case {
                if !case_spec.id.contains(filter) {
                    continue;
                }
            }
            if case_spec.heavy && !args.heavy && args.case.is_none() {
                continue;
            }

            let num_samples = if args.samples > 1 { args.samples } else { case_spec.samples };

            // Warmup iterations
            for _ in 0..case_spec.warmup {
                let _ = measure::measure_sample(meter, &bin, &case_spec.path, &case_spec.verification, 0, None);
            }

            let mut samples = Vec::new();
            for s_idx in 0..num_samples {
                let (sample, _note, _stdout) = measure::measure_sample(meter, &bin, &case_spec.path, &case_spec.verification, s_idx, None);
                samples.push(sample);
            }

            let aggregate = compare::compute_case_aggregate(&case_spec.id, &case_spec.tag, &samples);
            let ok = samples.iter().all(|s| s.status == SampleStatus::Ok);
            if ok {
                passed_cnt += 1;
            } else {
                failed_cnt += 1;
            }

            case_results.push(CaseResult {
                id: case_spec.id.clone(),
                path: case_spec.path.to_string_lossy().to_string(),
                verification_result: if ok { "ok".into() } else { "failed".into() },
                samples,
                aggregate,
            });
        }
    } else {
        // Legacy corpus/bench walk
        let run_corpus = !args.bench_only;
        let run_bench = !args.corpus_only;

        if run_corpus {
            let root = ws.join("phalcom-core/tests/fixtures/language");
            for path in suite::collect_corpus_cases(&root) {
                let lane = suite::classify_lane(&path, &root);
                if lane == Lane::Pending && !args.pending {
                    continue;
                }
                let label = suite::label_of(&path, &root);
                if let Some(filter) = &args.label {
                    if &label != filter {
                        continue;
                    }
                }
                let name = suite::case_name(&path, &root);
                let verification = match lane {
                    Lane::Negative => {
                        let expected = fs::read_to_string(path.with_extension("expected")).unwrap_or_default();
                        suite::CaseVerification::NegativeDiagnostic {
                            substring: expected.trim().to_string(),
                        }
                    }
                    _ => suite::CaseVerification::SidecarExpected,
                };
                let (sample, note, _stdout) = measure::measure_sample(meter, &bin, &path, &verification, 0, None);
                let ok = sample.status == SampleStatus::Ok;
                if ok {
                    passed_cnt += 1;
                } else {
                    failed_cnt += 1;
                }

                let aggregate = compare::compute_case_aggregate(&name, &label, std::slice::from_ref(&sample));
                case_results.push(CaseResult {
                    id: name,
                    path: path.to_string_lossy().to_string(),
                    verification_result: if ok { "ok".into() } else { note },
                    samples: vec![sample],
                    aggregate,
                });
            }
        }

        if run_bench {
            let root = ws.join("benchmarks");
            for path in suite::collect_bench_cases(&root) {
                let label = suite::label_of(&path, &root);
                if let Some(filter) = &args.label {
                    if &label != filter {
                        continue;
                    }
                }
                let name = suite::case_name(&path, &root);
                let verification = suite::CaseVerification::ExitZeroOnly;
                let (sample, note, _stdout) = measure::measure_sample(meter, &bin, &path, &verification, 0, None);
                let ok = sample.status == SampleStatus::Ok;
                if ok {
                    passed_cnt += 1;
                } else {
                    failed_cnt += 1;
                }

                let aggregate = compare::compute_case_aggregate(&name, &label, std::slice::from_ref(&sample));
                case_results.push(CaseResult {
                    id: name,
                    path: path.to_string_lossy().to_string(),
                    verification_result: if ok { "ok".into() } else { note },
                    samples: vec![sample],
                    aggregate,
                });
            }
        }
    }

    let summary = RunSummary {
        total_cases: case_results.len(),
        passed: passed_cnt,
        failed: failed_cnt,
        contaminated: 0,
    };

    let run = BenchmarkRun {
        schema_version: 1,
        run_id: run_id.clone(),
        timestamp: now_ts,
        git,
        build,
        host,
        layouts,
        command: RunCommandMetadata {
            subcommand: "run".into(),
            args_summary: format!("suite={suite_name} samples={}", args.samples),
        },
        suite: SuiteMetadata {
            name: suite_name,
            path: "".into(),
            case_count: case_results.len(),
        },
        resource_quality: meter.quality(),
        cases: case_results,
        summary,
    };

    if args.record || args.suite.is_some() {
        if let Err(err) = store::save_local(&ws, &run, None) {
            eprintln!("warning: failed to save run record: {err}");
        }
    }

    if args.json {
        println!("{}", run.to_json_string().unwrap());
    } else {
        println!("Run ID: {}", run.run_id);
        println!("Cases: {} (passed: {}, failed: {})", run.cases.len(), passed_cnt, failed_cnt);
        for c in run.cases.iter().take(args.top) {
            if let Some(agg) = &c.aggregate {
                println!("  {:<35} {:>8.2} ms", c.id, agg.wall.median);
            }
        }
    }

    if failed_cnt > 0 {
        std::process::exit(2);
    }
}

fn handle_ab(args: AbArgs) {
    let ws = workspace_root();

    if !args.baseline_bin.exists() {
        eprintln!("error: baseline binary does not exist at {}", args.baseline_bin.display());
        std::process::exit(2);
    }
    if !args.candidate_bin.exists() {
        eprintln!("error: candidate binary does not exist at {}", args.candidate_bin.display());
        std::process::exit(2);
    }

    // Preflight quiet check
    if !run_quiet_guard(&ws, "preflight") {
        eprintln!("benchmark aborted: machine became busy; no performance verdict recorded");
        record_contaminated_run(&ws, &args, "preflight");
        std::process::exit(3);
    }

    let (_manifest, suite_cases) = match suite::load_suite(&ws, &args.suite) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("error loading suite '{}': {err}", args.suite);
            std::process::exit(2);
        }
    };

    let meter = measure::ResourceMeter::detect();
    let git = env::capture_git();
    let base_build = env::capture_build(&args.baseline_bin);
    let cand_build = env::capture_build(&args.candidate_bin);
    let host = env::capture_host();
    let layouts = env::capture_layouts();
    let now_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let run_id = args.name.clone().unwrap_or_else(|| format!("ab_{}_{}", now_ts, git.short_sha));

    let pairs_count = if args.quick { 3 } else { args.pairs };

    let mut base_case_results = Vec::new();
    let mut cand_case_results = Vec::new();

    for case_spec in &suite_cases {
        if case_spec.heavy && args.quick {
            continue;
        }

        let mut base_samples = Vec::new();
        let mut cand_samples = Vec::new();

        for pair_idx in 0..pairs_count {
            if !run_quiet_guard(&ws, &format!("pair_{pair_idx}_{}", case_spec.id)) {
                eprintln!("benchmark aborted: machine became busy; no performance verdict recorded");
                record_contaminated_run(&ws, &args, "mid_run");
                std::process::exit(3);
            }

            let (order_base, order_cand) = if pair_idx % 2 == 0 {
                (SampleOrder::BaselineThenCandidate, SampleOrder::BaselineThenCandidate)
            } else {
                (SampleOrder::CandidateThenBaseline, SampleOrder::CandidateThenBaseline)
            };

            if pair_idx % 2 == 0 {
                let (b_sample, _, _) = measure::measure_sample(meter, &args.baseline_bin, &case_spec.path, &case_spec.verification, pair_idx, Some(order_base));
                let (c_sample, _, _) =
                    measure::measure_sample(meter, &args.candidate_bin, &case_spec.path, &case_spec.verification, pair_idx, Some(order_cand));
                base_samples.push(b_sample);
                cand_samples.push(c_sample);
            } else {
                let (c_sample, _, _) =
                    measure::measure_sample(meter, &args.candidate_bin, &case_spec.path, &case_spec.verification, pair_idx, Some(order_cand));
                let (b_sample, _, _) = measure::measure_sample(meter, &args.baseline_bin, &case_spec.path, &case_spec.verification, pair_idx, Some(order_base));
                base_samples.push(b_sample);
                cand_samples.push(c_sample);
            }
        }

        let base_agg = compare::compute_case_aggregate(&case_spec.id, &case_spec.tag, &base_samples);
        let cand_agg = compare::compute_case_aggregate(&case_spec.id, &case_spec.tag, &cand_samples);

        base_case_results.push(CaseResult {
            id: case_spec.id.clone(),
            path: case_spec.path.to_string_lossy().to_string(),
            verification_result: "ok".into(),
            samples: base_samples,
            aggregate: base_agg,
        });

        cand_case_results.push(CaseResult {
            id: case_spec.id.clone(),
            path: case_spec.path.to_string_lossy().to_string(),
            verification_result: "ok".into(),
            samples: cand_samples,
            aggregate: cand_agg,
        });
    }

    if !run_quiet_guard(&ws, "post-run") {
        eprintln!("benchmark aborted: machine became busy; no performance verdict recorded");
        record_contaminated_run(&ws, &args, "post-run");
        std::process::exit(3);
    }

    let base_run = BenchmarkRun {
        schema_version: 1,
        run_id: format!("{run_id}_base"),
        timestamp: now_ts,
        git: git.clone(),
        build: base_build,
        host: host.clone(),
        layouts: layouts.clone(),
        command: RunCommandMetadata {
            subcommand: "ab".into(),
            args_summary: format!("suite={} pairs={pairs_count}", args.suite),
        },
        suite: SuiteMetadata {
            name: args.suite.clone(),
            path: "".into(),
            case_count: base_case_results.len(),
        },
        resource_quality: meter.quality(),
        cases: base_case_results,
        summary: RunSummary {
            total_cases: suite_cases.len(),
            passed: suite_cases.len(),
            failed: 0,
            contaminated: 0,
        },
    };

    let cand_run = BenchmarkRun {
        schema_version: 1,
        run_id: format!("{run_id}_cand"),
        timestamp: now_ts,
        git,
        build: cand_build,
        host,
        layouts,
        command: RunCommandMetadata {
            subcommand: "ab".into(),
            args_summary: format!("suite={} pairs={pairs_count}", args.suite),
        },
        suite: SuiteMetadata {
            name: args.suite,
            path: "".into(),
            case_count: cand_case_results.len(),
        },
        resource_quality: meter.quality(),
        cases: cand_case_results,
        summary: RunSummary {
            total_cases: suite_cases.len(),
            passed: suite_cases.len(),
            failed: 0,
            contaminated: 0,
        },
    };

    let comp = compare::compare_runs(&base_run, &cand_run, false, true);

    if args.record {
        let _ = store::save_local(&ws, &base_run, None);
        let _ = store::save_local(&ws, &cand_run, None);
        let _ = store::save_comparison(&ws, &comp);
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&comp).unwrap());
    } else {
        println!("A/B Comparison Result ({pairs_count} pairs):");
        println!("{:<30} {:>10} {:>10} {:>10} {:>8}", "Case ID", "Base (ms)", "Cand (ms)", "Delta %", "Gate");
        println!("{}", "-".repeat(72));
        for case in &comp.cases {
            let base_ms = case.baseline_agg.as_ref().map(|a| a.wall.median).unwrap_or(0.0);
            let cand_ms = case.candidate_agg.as_ref().map(|a| a.wall.median).unwrap_or(0.0);
            let delta_pct = case.delta_pct.unwrap_or(0.0);
            println!(
                "{:<30} {:>10.2} {:>10.2} {:>+9.2}% {:>8}",
                case.id, base_ms, cand_ms, delta_pct, case.gate_result
            );
        }
        println!("\nVerdict: {:?}", comp.verdict);
    }

    if comp.verdict == Verdict::Regressions {
        std::process::exit(1);
    }
}

fn run_quiet_guard(workspace_root: &Path, where_label: &str) -> bool {
    let script = workspace_root.join("benchmarks/vm/ab-guarded.py");
    if !script.exists() {
        return true;
    }
    let status = Command::new("python3")
        .arg(&script)
        .arg("--check-only")
        .arg("--where")
        .arg(where_label)
        .status();

    match status {
        Ok(st) => st.code() == Some(0),
        Err(_) => true,
    }
}

fn record_contaminated_run(workspace_root: &Path, args: &AbArgs, where_label: &str) {
    let now_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let git = env::capture_git();
    let run_id = args.name.clone().unwrap_or_else(|| format!("ab_contaminated_{now_ts}"));

    let run = BenchmarkRun {
        schema_version: 1,
        run_id: run_id.clone(),
        timestamp: now_ts,
        git,
        build: env::capture_build(&args.candidate_bin),
        host: env::capture_host(),
        layouts: env::capture_layouts(),
        command: RunCommandMetadata {
            subcommand: "ab".into(),
            args_summary: format!("ab aborted at {where_label}"),
        },
        suite: SuiteMetadata {
            name: args.suite.clone(),
            path: "".into(),
            case_count: 0,
        },
        resource_quality: ResourceQuality::WallOnly,
        cases: vec![],
        summary: RunSummary {
            total_cases: 0,
            passed: 0,
            failed: 0,
            contaminated: 1,
        },
    };

    let _ = store::save_local(workspace_root, &run, Some(&format!("Aborted due to machine contention at {where_label}")));
}
