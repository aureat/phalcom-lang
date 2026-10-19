//! Comprehensive audit and performance benchmarking harness for PERF003.C2.P7 (`universe.telosc`).
//!
//! Provides CLI subcommands to:
//! - Generate persistent canonical `universe.telosc` and record metrics in `ARTIFACT.json`
//! - Measure cold compiler initialization in fresh processes (source vs artifact) across 4 workloads
//! - Measure warm compilation (unchanged vs modified)
//! - Measure end-to-end package compilation and VM execution (source vs artifact)
//! - Test package compilation parity on `packages/slotmap` and `packages/slotmap/tests/runtime`
//! - Test exact allocation budgets (R.G4 baseline: 366,789,583 bytes) and source-denied compiler execution

use clap::{Parser, Subcommand};
use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_core::modules::canonical_artifact::{
    TeloscArtifactError, build_universe_telosc, load_universe_telosc, load_universe_telosc_from_bytes, load_universe_telosc_from_bytes_with_allocation_summary,
    write_universe_telosc,
};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::BufferedOutput;
use phalcom_core::vm::VM;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// Escapes a field according to standard RFC 4180 CSV rules.
pub fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

#[derive(Parser)]
#[command(name = "perf_audit_p7", about = "PERF003.C2.P7 Audit and Benchmark Suite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate persistent universe.telosc and emit ARTIFACT.json
    Generate {
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Single cold compilation sample in current process
    ColdSingle {
        #[arg(long)]
        mode: String, // "source" or "artifact"
        #[arg(long)]
        workload: String, // "print42", "option_string", "synthetic_pkg", "slotmap"
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 1)]
        sample_id: usize,
    },
    /// Orchestrate fresh-process cold compilation suite (20 samples per configuration)
    RunColdSuite {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 20)]
        samples: usize,
        #[arg(long, alias = "csv", default_value = "target/perf003-p7-audit/RAW-COLD.csv")]
        output: PathBuf,
        #[arg(long)]
        workload: Option<String>,
        #[arg(long, default_value_t = false)]
        append: bool,
    },
    /// Run warm compilation benchmark in isolated source/artifact child processes
    RunWarmSuite {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, alias = "repetitions", default_value_t = 10)]
        samples: usize,
        #[arg(long, alias = "csv", default_value = "target/perf003-p7-audit/RAW-WARM.csv")]
        output: PathBuf,
    },
    /// Execute one warm benchmark scenario. Parent harness only.
    #[command(name = "warm-child", hide = true)]
    WarmChild {
        #[arg(long)]
        mode: String,
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 10)]
        repetitions: usize,
    },
    /// Single end-to-end runtime sample
    RuntimeSingle {
        #[arg(long)]
        mode: String,
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 1)]
        sample_id: usize,
    },
    /// Orchestrate end-to-end runtime benchmark suite (10 samples per configuration)
    RunRuntimeSuite {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 10)]
        samples: usize,
        #[arg(long, alias = "csv", default_value = "target/perf003-p7-audit/RAW-RUNTIME.csv")]
        output: PathBuf,
    },
    /// Test compilation of packages/slotmap and packages/slotmap/tests/runtime
    SlotmapEval {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, alias = "csv")]
        output: Option<PathBuf>,
    },
    /// Verify exact allocation budget accounting and limits
    AllocationCheck {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
    },
    /// Verify source denial in loaded compiler image
    SourceDenialCheck {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
    },
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Serialize)]
struct ArtifactReport {
    artifact_path: String,
    total_bytes: usize,
    module_world_bytes: usize,
    semantic_world_bytes: usize,
    query_image_bytes: usize,
    metadata_bytes: usize,
    container_bytes: usize,
    digest_algorithm: &'static str,
    artifact_digest: String,
    source_digest: String,
    interface_digest: String,
    semantic_digest: String,
    query_digest: String,
    metadata_digest: String,
    same_image_serialization_deterministic: bool,
    source_free_complete: bool,
    generation_duration_ms: f64,
    serialization_duration_ms: f64,
    validation_duration_ms: f64,
    publication_duration_ms: f64,
    peak_rss_bytes: u64,
    schema_versions: SchemaVersions,
}

#[derive(Serialize)]
struct SchemaVersions {
    container_schema: u32,
    minimum_reader_schema: u32,
    semantic_model_version: u32,
    stable_identity_schema: u32,
    module_semantic_abi: u32,
    semantic_artifact_abi: u32,
    query_fingerprint_abi: u32,
    native_surface_schema_version: u32,
    type_metadata_schema_version: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryStats {
    pub count: usize,
    pub min: f64,
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub max: f64,
    pub stddev: f64,
}

pub fn compute_stats(samples: &[f64]) -> SummaryStats {
    assert!(!samples.is_empty(), "cannot compute stats on empty samples");
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let count = sorted.len();
    let min = sorted[0];
    let max = sorted[count - 1];
    let sum: f64 = sorted.iter().sum();
    let mean = sum / count as f64;
    let median = if count % 2 == 1 {
        sorted[count / 2]
    } else {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    };
    let p95_idx = ((count as f64 * 0.95).ceil() as usize).saturating_sub(1).min(count - 1);
    let p95 = sorted[p95_idx];
    let variance = sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;
    let stddev = variance.sqrt();
    SummaryStats {
        count,
        min,
        mean,
        median,
        p95,
        max,
        stddev,
    }
}

fn get_peak_rss_bytes() -> u64 {
    #[cfg(target_os = "macos")]
    unsafe {
        #[repr(C)]
        struct RUsage {
            ru_utime: [u64; 2],
            ru_stime: [u64; 2],
            ru_maxrss: i64,
            _rest: [i64; 14],
        }
        unsafe extern "C" {
            fn getrusage(who: i32, usage: *mut RUsage) -> i32;
        }
        let mut usage: RUsage = std::mem::zeroed();
        if getrusage(0, &mut usage) == 0 {
            return usage.ru_maxrss.max(0) as u64;
        }
    }
    #[cfg(target_os = "linux")]
    unsafe {
        #[repr(C)]
        struct RUsage {
            ru_utime: [u64; 2],
            ru_stime: [u64; 2],
            ru_maxrss: i64,
            _rest: [i64; 14],
        }
        unsafe extern "C" {
            fn getrusage(who: i32, usage: *mut RUsage) -> i32;
        }
        let mut usage: RUsage = std::mem::zeroed();
        if getrusage(0, &mut usage) == 0 {
            return (usage.ru_maxrss.max(0) as u64) * 1024;
        }
    }
    0
}

fn setup_benchmark_packages() -> (PathBuf, PathBuf) {
    let base = PathBuf::from("target/telos/benchmark-packages");
    let synthetic_dir = base.join("synthetic_pkg");
    let minimal_dir = base.join("minimal_pkg");

    fs::create_dir_all(&synthetic_dir).expect("create synthetic_pkg dir");
    fs::create_dir_all(&minimal_dir).expect("create minimal_pkg dir");

    // Synthetic multi-file package
    fs::write(synthetic_dir.join("package.ph"), "expose .main\n").expect("write synthetic package.ph");
    fs::write(
        synthetic_dir.join("main.ph"),
        "let message = \"synthetic package test\";\nlet opt = Option.some(message);\nprint(opt);\n",
    )
    .expect("write synthetic main.ph");

    // Minimal package for VM execution
    fs::write(minimal_dir.join("package.ph"), "expose .main\n").expect("write minimal package.ph");
    fs::write(
        minimal_dir.join("main.ph"),
        r#"let greeting = "Hello, Telos Persisted Universe!"
let opt = Option.some(greeting)
System.print(greeting)
System.print(opt)
"#,
    )
    .expect("write minimal main.ph");

    (synthetic_dir, minimal_dir)
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { out, json } => cmd_generate(out.as_deref(), json.as_deref()),
        Commands::ColdSingle {
            mode,
            workload,
            artifact,
            sample_id,
        } => cmd_cold_single(&mode, &workload, &artifact, sample_id),
        Commands::RunColdSuite {
            artifact,
            samples,
            output,
            workload,
            append,
        } => cmd_run_cold_suite(&artifact, samples, &output, workload.as_deref(), append),
        Commands::RunWarmSuite { artifact, samples, output } => cmd_run_warm_suite(&artifact, samples, &output),
        Commands::WarmChild { mode, artifact, repetitions } => cmd_warm_child(&mode, &artifact, repetitions),
        Commands::RuntimeSingle { mode, artifact, sample_id } => cmd_runtime_single(&mode, &artifact, sample_id),
        Commands::RunRuntimeSuite { artifact, samples, output } => cmd_run_runtime_suite(&artifact, samples, &output),
        Commands::SlotmapEval { artifact, output } => cmd_slotmap_eval(&artifact, output.as_deref()),
        Commands::AllocationCheck { artifact } => cmd_allocation_check(&artifact),
        Commands::SourceDenialCheck { artifact } => cmd_source_denial_check(&artifact),
    }
}

fn cmd_generate(out_opt: Option<&Path>, json_opt: Option<&Path>) {
    let out_buf = if let Some(p) = out_opt {
        p.to_path_buf()
    } else if let Some(env_out) = std::env::var_os("PHALCOM_BUILD_UNIVERSE_TELOSC") {
        PathBuf::from(env_out)
    } else {
        PathBuf::from("target/telos/universe/universe.telosc")
    };
    let out = out_buf.as_path();

    let json_buf = if let Some(p) = json_opt {
        p.to_path_buf()
    } else {
        out.parent().unwrap_or_else(|| Path::new(".")).join("ARTIFACT.json")
    };
    let json_path = json_buf.as_path();

    println!("[generate] Compiling canonical Universe from source...");
    let t0 = Instant::now();
    let program = canonical_universe_program().expect("canonical universe program");
    let gen_dur = t0.elapsed();

    println!("[generate] Deriving compiler image view...");
    let t_view = Instant::now();
    let view = program.compiler_image_view().expect("compiler image view");
    let view_dur = t_view.elapsed();

    println!("[generate] Building universe.telosc container bytes...");
    let limits = ValidationLimits::default();
    let t_ser = Instant::now();
    let built1 = build_universe_telosc(&view, &limits).expect("build_universe_telosc");
    let ser_dur = t_ser.elapsed();

    println!("[generate] Building second container to verify same-image serialization determinism...");
    let built2 = build_universe_telosc(&view, &limits).expect("build_universe_telosc 2");
    let deterministic = built1.bytes == built2.bytes;
    assert!(
        deterministic,
        "Same-image serialization determinism: identical in-memory view must yield identical BLAKE3 digest and bytes"
    );

    println!("[generate] Writing artifact to {}...", out.display());
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("create artifact parent dir");
    }
    let t_write = Instant::now();
    write_universe_telosc(out, &built1).expect("write_universe_telosc");
    let write_dur = t_write.elapsed();

    let disk_bytes = fs::read(out).expect("read artifact from disk");
    assert_eq!(disk_bytes, built1.bytes);

    println!("[generate] Loading artifact from disk...");
    let t_val = Instant::now();
    let loaded = load_universe_telosc(out, &limits).expect("load_universe_telosc");
    let val_dur = t_val.elapsed();

    let container_view = TeloscContainerView::parse(&built1.bytes, &limits).expect("parse container");
    let header = container_view.header();

    let is_source_free_complete = loaded.formal_image.sources().is_empty() && loaded.formal_image.snapshot().is_source_free_artifact_complete();

    let report = ArtifactReport {
        artifact_path: out.display().to_string(),
        total_bytes: built1.bytes.len(),
        module_world_bytes: built1.summary.module_world_bytes,
        semantic_world_bytes: built1.summary.semantic_world_bytes,
        query_image_bytes: built1.summary.query_image_bytes,
        metadata_bytes: built1.summary.metadata_bytes,
        container_bytes: built1.bytes.len(),
        digest_algorithm: "BLAKE3-256",
        artifact_digest: to_hex(header.artifact_digest.as_bytes()),
        source_digest: to_hex(header.source_digest.as_bytes()),
        interface_digest: to_hex(header.interface_digest.as_bytes()),
        semantic_digest: to_hex(header.semantic_digest.as_bytes()),
        query_digest: to_hex(header.query_digest.as_bytes()),
        metadata_digest: to_hex(header.metadata_digest.as_bytes()),
        same_image_serialization_deterministic: deterministic,
        source_free_complete: is_source_free_complete,
        generation_duration_ms: gen_dur.as_secs_f64() * 1000.0,
        serialization_duration_ms: (view_dur + ser_dur).as_secs_f64() * 1000.0,
        validation_duration_ms: val_dur.as_secs_f64() * 1000.0,
        publication_duration_ms: write_dur.as_secs_f64() * 1000.0,
        peak_rss_bytes: get_peak_rss_bytes(),
        schema_versions: SchemaVersions {
            container_schema: header.container_schema as u32,
            minimum_reader_schema: header.minimum_reader_schema as u32,
            semantic_model_version: header.semantic_model_version,
            stable_identity_schema: header.stable_identity_schema,
            module_semantic_abi: header.module_semantic_abi,
            semantic_artifact_abi: header.semantic_artifact_abi,
            query_fingerprint_abi: header.query_fingerprint_abi,
            native_surface_schema_version: header.native_surface_schema_version,
            type_metadata_schema_version: header.type_metadata_schema_version,
        },
    };

    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent).expect("create json parent dir");
    }
    let json_text = serde_json::to_string_pretty(&report).expect("serialize json");
    fs::write(json_path, &json_text).expect("write json file");

    println!("ARTIFACT_REPORT_JSON:\n{}", json_text);
}

fn cmd_cold_single(mode: &str, workload: &str, artifact: &Path, sample_id: usize) {
    let t_proc_start = Instant::now();
    let pid = std::process::id();
    let (synthetic_dir, _) = setup_benchmark_packages();

    let limits = ValidationLimits::default();
    let (init_dur, timings_tuple, compile_dur, status) = match mode {
        "source" => {
            let t0 = Instant::now();
            let image = match phalcom_semantic::canonical_image::canonical_universe_formal_image() {
                Ok(img) => (*img).clone(),
                Err(e) => {
                    eprintln!("Source image init failed: {e}");
                    std::process::exit(1);
                }
            };
            let init_dur = t0.elapsed();

            let t_comp = Instant::now();
            let res = match workload {
                "print42" => ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Inline("print(42);".into()), image.clone()),
                "option_string" => ProgramCompiler::compile_entry_selection_with_image(
                    EntrySelection::Inline("let s = \"hello\"; let opt = Option.some(s); print(opt);".into()),
                    image.clone(),
                ),
                "synthetic_pkg" => ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Package(synthetic_dir), image.clone()),
                "slotmap" => ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Project(PathBuf::from("packages/slotmap")), image.clone()),
                _ => panic!("Unknown workload {workload}"),
            };
            let compile_dur = t_comp.elapsed();
            let status = if res.is_ok() { "SUCCESS" } else { "COMPILE_ERROR" };
            (init_dur, (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64), compile_dur, status)
        }
        "artifact" => {
            let t0 = Instant::now();
            let loaded = match load_universe_telosc(artifact, &limits) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Artifact load failed: {e}");
                    std::process::exit(1);
                }
            };
            let init_dur = t0.elapsed();
            let timings = (
                loaded.timings.file_read.as_secs_f64() * 1000.0,
                loaded.timings.container_validation.as_secs_f64() * 1000.0,
                loaded.timings.section_validation.as_secs_f64() * 1000.0,
                loaded.timings.relocation.as_secs_f64() * 1000.0,
                loaded.timings.publication.as_secs_f64() * 1000.0,
            );

            let t_comp = Instant::now();
            let res = match workload {
                "print42" => loaded.compile_user_entry(EntrySelection::Inline("print(42);".into())),
                "option_string" => loaded.compile_user_entry(EntrySelection::Inline("let s = \"hello\"; let opt = Option.some(s); print(opt);".into())),
                "synthetic_pkg" => loaded.compile_user_entry(EntrySelection::Package(synthetic_dir)),
                "slotmap" => loaded.compile_user_entry(EntrySelection::Project(PathBuf::from("packages/slotmap"))),
                _ => panic!("Unknown workload {workload}"),
            };
            let compile_dur = t_comp.elapsed();
            let status = if res.is_ok() { "SUCCESS" } else { "COMPILE_ERROR" };
            (init_dur, timings, compile_dur, status)
        }
        _ => panic!("Unknown mode {mode}"),
    };

    let total_dur = t_proc_start.elapsed();
    let peak_rss_bytes = get_peak_rss_bytes();
    let peak_rss_kb = peak_rss_bytes / 1024;

    // Fields: mode,workload,sample_id,pid,total_ms,init_ms,user_compile_ms,file_read_ms,container_val_ms,section_decode_ms,reloc_ms,pub_ms,peak_rss_kb,status
    println!(
        "RAW_COLD_CSV:{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{}",
        csv_escape(mode),
        csv_escape(workload),
        sample_id,
        pid,
        total_dur.as_secs_f64() * 1000.0,
        init_dur.as_secs_f64() * 1000.0,
        compile_dur.as_secs_f64() * 1000.0,
        timings_tuple.0,
        timings_tuple.1,
        timings_tuple.2,
        timings_tuple.3,
        timings_tuple.4,
        peak_rss_kb,
        csv_escape(status),
    );
}

fn cmd_run_cold_suite(artifact: &Path, samples: usize, csv_path: &Path, workload_filter: Option<&str>, append: bool) {
    use std::io::Write;
    let current_exe = std::env::current_exe().expect("current executable");
    let all_workloads = ["print42", "option_string", "synthetic_pkg", "slotmap"];
    let workloads: Vec<&str> = if let Some(w) = workload_filter {
        assert!(all_workloads.contains(&w), "Unknown workload filter: {w}");
        vec![w]
    } else {
        all_workloads.to_vec()
    };
    let modes = ["source", "artifact"];

    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).expect("create csv dir");
    }

    let header = [
        "scenario",
        "workload",
        "sample_number",
        "process_id",
        "duration_ms",
        "init_ms",
        "user_compile_ms",
        "artifact_file_read_ms",
        "container_validation_ms",
        "section_decode_ms",
        "semantic_relocation_ms",
        "final_publication_ms",
        "peak_rss_kb",
        "exit_status",
    ]
    .join(",");

    if !append || !csv_path.exists() {
        fs::write(csv_path, format!("{header}\n")).expect("initialize cold CSV");
    }

    let mut total_samples_recorded = 0;

    for workload in &workloads {
        for mode in modes {
            println!("=== Running Cold Suite: mode={mode}, workload={workload} ({samples} fresh processes) ===");
            let mut durations = Vec::with_capacity(samples);
            let mut init_times = Vec::with_capacity(samples);
            let mut compile_times = Vec::with_capacity(samples);
            let mut peak_rss_vals = Vec::with_capacity(samples);

            for sample_id in 1..=samples {
                let output = Command::new(&current_exe)
                    .arg("cold-single")
                    .arg("--mode")
                    .arg(mode)
                    .arg("--workload")
                    .arg(workload)
                    .arg("--artifact")
                    .arg(artifact)
                    .arg("--sample-id")
                    .arg(sample_id.to_string())
                    .output()
                    .expect("execute cold-single process");

                if !output.status.success() {
                    panic!(
                        "Cold sample #{sample_id} ({mode}/{workload}) exited with non-zero status {:?}:\nSTDOUT: {}\nSTDERR: {}",
                        output.status.code(),
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut found_row = false;
                for line in stdout.lines() {
                    if let Some(csv_data) = line.strip_prefix("RAW_COLD_CSV:") {
                        found_row = true;
                        total_samples_recorded += 1;

                        // Append row immediately to disk
                        let mut file = fs::OpenOptions::new().append(true).open(csv_path).expect("open CSV for appending row");
                        writeln!(file, "{csv_data}").expect("append row to CSV");

                        let cols: Vec<&str> = csv_data.split(',').collect();
                        if cols.len() >= 14 {
                            if cols[13] != "SUCCESS" {
                                panic!("Cold sample #{sample_id} ({mode}/{workload}) reported compilation error: {}", csv_data);
                            }
                            if let Ok(dur) = cols[4].parse::<f64>() {
                                durations.push(dur);
                            }
                            if let Ok(init) = cols[5].parse::<f64>() {
                                init_times.push(init);
                            }
                            if let Ok(comp) = cols[6].parse::<f64>() {
                                compile_times.push(comp);
                            }
                            if let Ok(rss) = cols[12].parse::<f64>() {
                                peak_rss_vals.push(rss);
                            }
                        }
                    }
                }
                if !found_row {
                    panic!(
                        "Cold sample #{sample_id} ({mode}/{workload}) produced no CSV row output:\nSTDOUT: {}\nSTDERR: {}",
                        stdout,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }

            if !durations.is_empty() {
                let dur_stats = compute_stats(&durations);
                let init_stats = compute_stats(&init_times);
                let comp_stats = compute_stats(&compile_times);
                let rss_stats = compute_stats(&peak_rss_vals);
                println!(
                    "  Summary [{mode}/{workload}] (N={}): total_median={:.2}ms, init_median={:.2}ms, comp_median={:.2}ms, rss_median={:.0}KB",
                    dur_stats.count, dur_stats.median, init_stats.median, comp_stats.median, rss_stats.median
                );
            }
        }
    }

    assert_eq!(
        total_samples_recorded,
        workloads.len() * modes.len() * samples,
        "Total recorded raw CSV rows must strictly match expected count"
    );

    println!(
        "Successfully completed cold suite run ({} total recorded samples in {})",
        total_samples_recorded,
        csv_path.display()
    );
}

#[derive(Debug)]
struct WarmSampleRow {
    csv: String,
    compile_ms: f64,
    peak_rss_bytes: u64,
}

fn validate_warm_child_output(stdout: &str, mode: &str, repetitions: usize, pid: u32) -> Result<Vec<WarmSampleRow>, String> {
    let rows: Vec<&str> = stdout.lines().filter_map(|line| line.strip_prefix("RAW_WARM_CSV:")).collect();
    if rows.len() != repetitions {
        return Err(format!("expected {repetitions} warm rows, received {}", rows.len()));
    }

    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            let columns: Vec<&str> = row.split(',').collect();
            if columns.len() != 8 {
                return Err(format!("row {} has {} columns, expected 8", index + 1, columns.len()));
            }
            if columns[0] != mode {
                return Err(format!("row {} has scenario {:?}, expected {mode:?}", index + 1, columns[0]));
            }
            if columns[1].parse::<usize>().ok() != Some(index + 1) {
                return Err(format!("row {} has invalid sample number {:?}", index + 1, columns[1]));
            }
            if columns[2].parse::<u32>().ok() != Some(pid) {
                return Err(format!("row {} does not report child PID {pid}", index + 1));
            }
            if columns[3] != "1" {
                return Err(format!("row {} must report exactly one untimed warmup", index + 1));
            }
            let expected_variant = if index == 0 {
                "first_compilation"
            } else if (index + 1) % 2 == 0 {
                "modified_input"
            } else {
                "unchanged_input"
            };
            if columns[4] != expected_variant {
                return Err(format!("row {} has input variant {:?}, expected {expected_variant:?}", index + 1, columns[4]));
            }
            let compile_ms = columns[5]
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value >= 0.0)
                .ok_or_else(|| format!("row {} has invalid compile duration {:?}", index + 1, columns[5]))?;
            let peak_rss_bytes = columns[6]
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| format!("row {} has invalid peak RSS {:?}", index + 1, columns[6]))?;
            if columns[7] != "SUCCESS" {
                return Err(format!("row {} reports non-success status {:?}", index + 1, columns[7]));
            }
            Ok(WarmSampleRow {
                csv: row.to_owned(),
                compile_ms,
                peak_rss_bytes,
            })
        })
        .collect()
}

fn cmd_run_warm_suite(artifact: &Path, repetitions: usize, csv_path: &Path) {
    assert!(repetitions > 0, "warm suite requires at least one timed repetition");
    let modes = ["source", "artifact"];
    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).expect("create csv dir");
    }

    let header = [
        "scenario",
        "sample_number",
        "process_id",
        "warmup_count",
        "input_variant",
        "compile_ms",
        "peak_rss_bytes",
        "exit_status",
    ]
    .join(",");
    let mut csv_lines = vec![header];
    let current_exe = std::env::current_exe().expect("current executable");
    let mut process_ids = Vec::with_capacity(modes.len());

    for mode in modes {
        println!("=== Running Warm Suite: mode={mode} ({repetitions} repetitions) ===");
        let child = Command::new(&current_exe)
            .arg("warm-child")
            .arg("--mode")
            .arg(mode)
            .arg("--artifact")
            .arg(artifact)
            .arg("--repetitions")
            .arg(repetitions.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn isolated warm scenario");
        let pid = child.id();
        let output = child.wait_with_output().expect("wait for isolated warm scenario");
        if !output.status.success() {
            panic!(
                "Warm scenario {mode} exited with status {:?}:\nstdout:\n{}\nstderr:\n{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let stdout = String::from_utf8(output.stdout).expect("warm child stdout is UTF-8");
        let rows = validate_warm_child_output(&stdout, mode, repetitions, pid).unwrap_or_else(|error| {
            panic!(
                "Invalid warm child output for {mode}: {error}\nstdout:\n{stdout}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        let compile_times: Vec<f64> = rows.iter().map(|row| row.compile_ms).collect();
        let peak_rss_values: Vec<f64> = rows.iter().map(|row| row.peak_rss_bytes as f64).collect();
        csv_lines.extend(rows.into_iter().map(|row| row.csv));
        process_ids.push(pid);

        let stats = compute_stats(&compile_times);
        let rss_stats = compute_stats(&peak_rss_values);
        println!(
            "  Warm Summary [{mode}] (N={}): min={:.2}ms, median={:.2}ms, mean={:.2}ms, p95={:.2}ms, max={:.2}ms, stddev={:.2}ms, peak_rss={:.0} bytes",
            stats.count, stats.min, stats.median, stats.mean, stats.p95, stats.max, stats.stddev, rss_stats.max
        );
    }

    assert_ne!(
        process_ids[0], process_ids[1],
        "source and artifact warm scenarios must run in different processes"
    );
    fs::write(csv_path, csv_lines.join("\n") + "\n").expect("write warm CSV");
    println!("Successfully wrote {}", csv_path.display());
}

fn cmd_warm_child(mode: &str, artifact: &Path, repetitions: usize) {
    assert!(matches!(mode, "source" | "artifact"), "unknown warm mode {mode}");
    assert!(repetitions > 0, "warm child requires at least one timed repetition");
    let limits = ValidationLimits::default();
    let source_image = if mode == "source" {
        Some(phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("source image"))
    } else {
        None
    };
    let loaded_artifact = if mode == "artifact" {
        Some(load_universe_telosc(artifact, &limits).expect("loaded artifact"))
    } else {
        None
    };
    let compile = |code: &str| {
        if let Some(image) = &source_image {
            ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Inline(code.into()), (**image).clone())
        } else if let Some(loaded) = &loaded_artifact {
            loaded.compile_user_entry(EntrySelection::Inline(code.into()))
        } else {
            unreachable!()
        }
    };

    let warmup = "let greeting = \"warmup baseline\"; let opt = Option.some(greeting); print(opt);";
    compile(warmup).unwrap_or_else(|error| panic!("Warmup compile failed for {mode}: {error:?}"));
    let pid = std::process::id();
    for sample_id in 1..=repetitions {
        let is_modified = sample_id % 2 == 0;
        let code = if is_modified {
            "let greeting = \"modified warm string\"; let opt = Option.some(greeting); print(opt);"
        } else {
            "let greeting = \"warm string unchanged\"; let opt = Option.some(greeting); print(opt);"
        };
        let variant = if sample_id == 1 {
            "first_compilation"
        } else if is_modified {
            "modified_input"
        } else {
            "unchanged_input"
        };
        let started = Instant::now();
        compile(code).unwrap_or_else(|error| panic!("Warm compile failed for {mode} on sample #{sample_id}: {error:?}"));
        let compile_ms = started.elapsed().as_secs_f64() * 1000.0;
        let peak_rss_bytes = get_peak_rss_bytes();
        println!(
            "RAW_WARM_CSV:{},{},{},1,{},{:.3},{},SUCCESS",
            csv_escape(mode),
            sample_id,
            pid,
            csv_escape(variant),
            compile_ms,
            peak_rss_bytes
        );
    }
}

fn cmd_runtime_single(mode: &str, artifact: &Path, sample_id: usize) {
    let t_start = Instant::now();
    let pid = std::process::id();
    let (_, minimal_dir) = setup_benchmark_packages();
    let limits = ValidationLimits::default();

    // 1. Compiler init
    let t_comp_init = Instant::now();
    let (source_image, loaded_artifact) = if mode == "source" {
        let img = phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("source image");
        (Some((*img).clone()), None)
    } else {
        let loaded = load_universe_telosc(artifact, &limits).expect("loaded artifact");
        (None, Some(loaded))
    };
    let comp_init_dur = t_comp_init.elapsed();

    // 2. User package compilation
    let t_pkg_comp = Instant::now();
    let program = if let Some(image) = &source_image {
        ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Package(minimal_dir), image.clone()).expect("compile package with source image")
    } else if let Some(loaded) = &loaded_artifact {
        loaded
            .compile_user_entry(EntrySelection::Package(minimal_dir))
            .expect("compile package with loaded artifact")
    } else {
        unreachable!()
    };
    let pkg_comp_dur = t_pkg_comp.elapsed();

    // 3. VM construction
    let t_vm_init = Instant::now();
    let output_sink = BufferedOutput::new();
    let output_handle = output_sink.handle();
    let vm_res = VM::try_new_with_output(Box::new(output_sink));
    let vm_init_dur = t_vm_init.elapsed();

    // 4. Run compiled program on VM
    let (status, stdout, vm_run_dur) = match vm_res {
        Ok(mut vm) => {
            let t_vm_run = Instant::now();
            let run_res = vm.run_compiled(&program);
            let vm_run_dur = t_vm_run.elapsed();
            let status = if run_res.is_ok() { "SUCCESS" } else { "RUNTIME_ERROR" };
            let stdout = String::from_utf8_lossy(&output_handle.bytes()).trim().to_string();
            (status, stdout, vm_run_dur)
        }
        Err(err) => {
            let status = "BASELINE_BOOTSTRAP_ERROR";
            let stdout = format!("VM bootstrap failed: {err}");
            (status, stdout, std::time::Duration::from_millis(0))
        }
    };

    let total_dur = t_start.elapsed();
    let peak_rss_bytes = get_peak_rss_bytes();
    let peak_rss_kb = peak_rss_bytes / 1024;

    println!(
        "RAW_RUNTIME_CSV:{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{}",
        csv_escape(mode),
        sample_id,
        pid,
        total_dur.as_secs_f64() * 1000.0,
        comp_init_dur.as_secs_f64() * 1000.0,
        pkg_comp_dur.as_secs_f64() * 1000.0,
        vm_init_dur.as_secs_f64() * 1000.0,
        vm_run_dur.as_secs_f64() * 1000.0,
        peak_rss_kb,
        csv_escape(status),
        csv_escape(&stdout),
    );
}

fn cmd_run_runtime_suite(artifact: &Path, samples: usize, csv_path: &Path) {
    let current_exe = std::env::current_exe().expect("current executable");
    let modes = ["source", "artifact"];

    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).expect("create csv dir");
    }

    let header = [
        "scenario",
        "sample_number",
        "process_id",
        "duration_ms",
        "compiler_init_ms",
        "user_compile_ms",
        "vm_bootstrap_ms",
        "vm_exec_ms",
        "peak_rss_kb",
        "exit_status",
        "output",
    ]
    .join(",");

    let mut csv_lines = vec![header];

    for mode in modes {
        println!("=== Running Runtime Suite: mode={mode} ({samples} fresh processes) ===");
        for sample_id in 1..=samples {
            let output = Command::new(&current_exe)
                .arg("runtime-single")
                .arg("--mode")
                .arg(mode)
                .arg("--artifact")
                .arg(artifact)
                .arg("--sample-id")
                .arg(sample_id.to_string())
                .output()
                .expect("execute runtime-single process");

            if !output.status.success() {
                panic!(
                    "Child runtime sample #{sample_id} (mode={mode}) exited with error:\nSTDOUT: {}\nSTDERR: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut found_row = false;
            for line in stdout.lines() {
                if let Some(csv_data) = line.strip_prefix("RAW_RUNTIME_CSV:") {
                    found_row = true;
                    csv_lines.push(csv_data.to_string());
                }
            }
            if !found_row {
                panic!("Runtime sample #{sample_id} ({mode}) produced no CSV row output:\nSTDOUT: {}", stdout);
            }
        }
    }

    fs::write(csv_path, csv_lines.join("\n") + "\n").expect("write RAW-RUNTIME.csv");
    println!("Successfully wrote {}", csv_path.display());
}

fn cmd_slotmap_eval(artifact: &Path, output: Option<&Path>) {
    println!("=== Evaluating packages/slotmap and packages/slotmap/tests/runtime ===");
    let limits = ValidationLimits::default();

    println!("1. Loading artifact formal image...");
    let loaded = load_universe_telosc(artifact, &limits).expect("loaded artifact");

    println!("2. Loading canonical source formal image...");
    let source_image = phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("source image");

    let mut csv_rows = vec!["package,mode,modules_count,compile_ms,parity_status".to_string()];

    // Case 1: packages/slotmap
    println!("Testing packages/slotmap under Artifact-backed...");
    let t_art1 = Instant::now();
    let art_prog1 = loaded
        .compile_user_entry(EntrySelection::Project(PathBuf::from("packages/slotmap")))
        .expect("artifact compilation of packages/slotmap");
    let dur_art1 = t_art1.elapsed().as_secs_f64() * 1000.0;
    println!(
        "packages/slotmap Artifact compilation: SUCCESS ({} modules in {:.2}ms)",
        art_prog1.modules.len(),
        dur_art1
    );

    println!("Testing packages/slotmap under Source-backed...");
    let t_src1 = Instant::now();
    let src_prog1 = ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Project(PathBuf::from("packages/slotmap")), (*source_image).clone())
        .expect("source compilation of packages/slotmap");
    let dur_src1 = t_src1.elapsed().as_secs_f64() * 1000.0;
    println!(
        "packages/slotmap Source compilation: SUCCESS ({} modules in {:.2}ms)",
        src_prog1.modules.len(),
        dur_src1
    );
    println!("Verifying packages/slotmap structural product parity...");
    phalcom_core::modules::parity::assert_compiled_program_parity(&src_prog1, &art_prog1);
    println!("packages/slotmap structural product parity: VERIFIED (topology, interfaces, exports, lowering, bindings match)");

    csv_rows.push(format!("packages/slotmap,artifact,{},{:.3},PASS", art_prog1.modules.len(), dur_art1));
    csv_rows.push(format!("packages/slotmap,source,{},{:.3},PASS", src_prog1.modules.len(), dur_src1));

    // Case 2: packages/slotmap/tests/runtime
    println!("Testing packages/slotmap/tests/runtime under Artifact-backed...");
    let t_art2 = Instant::now();
    let art_prog2 = loaded
        .compile_user_entry(EntrySelection::Project(PathBuf::from("packages/slotmap/tests/runtime")))
        .expect("artifact compilation of packages/slotmap/tests/runtime");
    let dur_art2 = t_art2.elapsed().as_secs_f64() * 1000.0;
    println!(
        "packages/slotmap/tests/runtime Artifact compilation: SUCCESS ({} modules in {:.2}ms)",
        art_prog2.modules.len(),
        dur_art2
    );

    println!("Testing packages/slotmap/tests/runtime under Source-backed...");
    let t_src2 = Instant::now();
    let src_prog2 = ProgramCompiler::compile_entry_selection_with_image(
        EntrySelection::Project(PathBuf::from("packages/slotmap/tests/runtime")),
        (*source_image).clone(),
    )
    .expect("source compilation of packages/slotmap/tests/runtime");
    let dur_src2 = t_src2.elapsed().as_secs_f64() * 1000.0;
    println!(
        "packages/slotmap/tests/runtime Source compilation: SUCCESS ({} modules in {:.2}ms)",
        src_prog2.modules.len(),
        dur_src2
    );
    println!("Verifying packages/slotmap/tests/runtime structural product parity...");
    phalcom_core::modules::parity::assert_compiled_program_parity(&src_prog2, &art_prog2);
    println!("packages/slotmap/tests/runtime structural product parity: VERIFIED (topology, interfaces, exports, lowering, bindings match)");

    csv_rows.push(format!(
        "packages/slotmap/tests/runtime,artifact,{},{:.3},PASS",
        art_prog2.modules.len(),
        dur_art2
    ));
    csv_rows.push(format!(
        "packages/slotmap/tests/runtime,source,{},{:.3},PASS",
        src_prog2.modules.len(),
        dur_src2
    ));

    if let Some(out_path) = output {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).expect("create output parent dir");
        }
        fs::write(out_path, csv_rows.join("\n") + "\n").expect("write slotmap results CSV");
        println!("Successfully wrote {}", out_path.display());
    }

    println!("Slotmap evaluation complete: 100% normalized compilation product parity verified between source-backed and artifact-backed!");
}

fn cmd_allocation_check(artifact: &Path) {
    println!("=== Checking cumulative allocation budgets and limits ===");
    let bytes = fs::read(artifact).expect("read artifact");
    let limits = ValidationLimits::default();

    let (summary_res, ledger_requested) = load_universe_telosc_from_bytes_with_allocation_summary(&bytes, &limits);
    let initial_artifact = summary_res.expect("initial load without memory measurement must succeed");
    drop(initial_artifact);

    println!("Ledger requested bytes (N): {ledger_requested}");
    assert_eq!(ledger_requested, 366_789_583, "Ledger requested bytes must exactly equal baseline 366,789,583");

    // Verify N-1 fails with typed BudgetExceeded
    let limits_minus_one = ValidationLimits {
        max_total_allocation_bytes: (ledger_requested - 1) as u64,
        ..ValidationLimits::default()
    };
    let (failed_cumulative, _) = load_universe_telosc_from_bytes_with_allocation_summary(&bytes, &limits_minus_one);
    match failed_cumulative {
        Err(TeloscArtifactError::BudgetExceeded { limit_bytes }) => {
            assert_eq!(limit_bytes, ledger_requested - 1);
        }
        Err(other) => panic!("expected TeloscArtifactError::BudgetExceeded, got {other:?}"),
        Ok(_) => panic!("load with N - 1 budget must fail closed"),
    }

    // Verify exact N succeeds
    let limits_exact = ValidationLimits {
        max_total_allocation_bytes: ledger_requested as u64,
        ..ValidationLimits::default()
    };
    let exact_res = load_universe_telosc_from_bytes(&bytes, &limits_exact);
    assert!(exact_res.is_ok(), "Exact budget N must succeed");

    println!("R.G4 allocation check verified: exact 366,789,583 bytes, N-1 fail-closed, clean retry PASS.");
}

fn cmd_source_denial_check(artifact: &Path) {
    println!("=== Checking Source Denial in Loaded Compiler Image ===");
    let limits = ValidationLimits::default();
    let loaded = load_universe_telosc(artifact, &limits).expect("load artifact");

    assert!(loaded.formal_image.sources().is_empty(), "Loaded formal image must have 0 sources");
    assert!(
        loaded.formal_image.snapshot().is_source_free_artifact_complete(),
        "Loaded snapshot must be source-free complete"
    );

    let (pkg_dir, _) = setup_benchmark_packages();
    let compiled = loaded.compile_user_entry(EntrySelection::Package(pkg_dir)).expect("compile user package");
    assert!(!compiled.modules.is_empty(), "Compiled modules must be non-empty");

    println!("Source-free compiler authority verified: 0 sources, complete snapshot, successful user package compilation.");
}

#[cfg(test)]
mod warm_child_output_tests {
    use super::validate_warm_child_output;

    #[test]
    fn accepts_complete_rows_from_the_expected_child_process() {
        let output = concat!(
            "RAW_WARM_CSV:source,1,42,1,first_compilation,1.250,4096,SUCCESS\n",
            "RAW_WARM_CSV:source,2,42,1,modified_input,1.500,4096,SUCCESS\n",
        );
        let rows = validate_warm_child_output(output, "source", 2, 42).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].compile_ms, 1.5);
        assert_eq!(rows[0].peak_rss_bytes, 4096);
    }

    #[test]
    fn rejects_missing_rows_and_wrong_process_ids() {
        let missing = "RAW_WARM_CSV:source,1,42,1,first_compilation,1.250,4096,SUCCESS\n";
        assert!(
            validate_warm_child_output(missing, "source", 2, 42)
                .unwrap_err()
                .contains("expected 2 warm rows")
        );

        let wrong_pid = "RAW_WARM_CSV:source,1,43,1,first_compilation,1.250,4096,SUCCESS\n";
        assert!(validate_warm_child_output(wrong_pid, "source", 1, 42).unwrap_err().contains("child PID 42"));
    }

    #[test]
    fn rejects_failed_or_malformed_rows() {
        let failed = "RAW_WARM_CSV:artifact,1,42,1,first_compilation,1.250,4096,COMPILE_ERROR\n";
        assert!(
            validate_warm_child_output(failed, "artifact", 1, 42)
                .unwrap_err()
                .contains("non-success status")
        );

        let malformed = "RAW_WARM_CSV:artifact,1,42,1,first_compilation,NaN,4096,SUCCESS\n";
        assert!(
            validate_warm_child_output(malformed, "artifact", 1, 42)
                .unwrap_err()
                .contains("invalid compile duration")
        );
    }
}
