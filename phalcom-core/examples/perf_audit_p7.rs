//! Comprehensive audit and performance benchmarking harness for PERF003.C2.P7 (`universe.telosc`).
//!
//! Provides CLI subcommands to:
//! - Generate persistent canonical `universe.telosc` and record metrics in `ARTIFACT.json`
//! - Measure cold compiler initialization in fresh processes (source vs artifact) across 4 workloads
//! - Measure warm compilation (20 samples unchanged vs modified)
//! - Measure end-to-end package compilation and VM execution (source vs artifact)
//! - Test package compilation parity on `packages/slotmap` and `packages/slotmap/tests/runtime`
//! - Test exact allocation budgets and source-denied compiler execution

use clap::{Parser, Subcommand};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_core::modules::canonical_artifact::{
    build_universe_telosc, load_universe_telosc, load_universe_telosc_from_bytes, write_universe_telosc,
};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::BufferedOutput;
use phalcom_core::vm::VM;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

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
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        out: PathBuf,
        #[arg(long, default_value = "target/perf003-p7-audit/ARTIFACT.json")]
        json: PathBuf,
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
    /// Orchestrate fresh-process cold compilation suite (10 samples per configuration)
    RunColdSuite {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 10)]
        samples: usize,
        #[arg(long, default_value = "target/perf003-p7-audit/RAW-COLD.csv")]
        csv: PathBuf,
    },
    /// Run warm compilation benchmark (20 samples per configuration)
    RunWarmSuite {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
        #[arg(long, default_value_t = 20)]
        repetitions: usize,
        #[arg(long, default_value = "target/perf003-p7-audit/RAW-WARM.csv")]
        csv: PathBuf,
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
        #[arg(long, default_value = "target/perf003-p7-audit/RAW-RUNTIME.csv")]
        csv: PathBuf,
    },
    /// Test compilation of packages/slotmap and packages/slotmap/tests/runtime
    SlotmapEval {
        #[arg(long, default_value = "target/telos/universe/universe.telosc")]
        artifact: PathBuf,
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
    artifact_digest: String,
    source_digest: String,
    interface_digest: String,
    semantic_digest: String,
    query_digest: String,
    metadata_digest: String,
    deterministic: bool,
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
        Commands::Generate { out, json } => cmd_generate(&out, &json),
        Commands::ColdSingle {
            mode,
            workload,
            artifact,
            sample_id,
        } => cmd_cold_single(&mode, &workload, &artifact, sample_id),
        Commands::RunColdSuite {
            artifact,
            samples,
            csv,
        } => cmd_run_cold_suite(&artifact, samples, &csv),
        Commands::RunWarmSuite {
            artifact,
            repetitions,
            csv,
        } => cmd_run_warm_suite(&artifact, repetitions, &csv),
        Commands::RuntimeSingle {
            mode,
            artifact,
            sample_id,
        } => cmd_runtime_single(&mode, &artifact, sample_id),
        Commands::RunRuntimeSuite {
            artifact,
            samples,
            csv,
        } => cmd_run_runtime_suite(&artifact, samples, &csv),
        Commands::SlotmapEval { artifact } => cmd_slotmap_eval(&artifact),
        Commands::AllocationCheck { artifact } => cmd_allocation_check(&artifact),
        Commands::SourceDenialCheck { artifact } => cmd_source_denial_check(&artifact),
    }
}

fn cmd_generate(out: &Path, json_path: &Path) {
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

    println!("[generate] Building second container to verify byte-for-byte determinism...");
    let built2 = build_universe_telosc(&view, &limits).expect("build_universe_telosc 2");
    let deterministic = built1.bytes == built2.bytes;
    assert!(deterministic, "Artifact builds must be byte-for-byte deterministic!");

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

    let is_source_free_complete = loaded.formal_image.sources().is_empty()
        && loaded.formal_image.snapshot().is_source_free_artifact_complete();

    let report = ArtifactReport {
        artifact_path: out.display().to_string(),
        total_bytes: built1.bytes.len(),
        module_world_bytes: built1.summary.module_world_bytes,
        semantic_world_bytes: built1.summary.semantic_world_bytes,
        query_image_bytes: built1.summary.query_image_bytes,
        metadata_bytes: built1.summary.metadata_bytes,
        container_bytes: built1.bytes.len(),
        artifact_digest: to_hex(header.artifact_digest.as_bytes()),
        source_digest: to_hex(header.source_digest.as_bytes()),
        interface_digest: to_hex(header.interface_digest.as_bytes()),
        semantic_digest: to_hex(header.semantic_digest.as_bytes()),
        query_digest: to_hex(header.query_digest.as_bytes()),
        metadata_digest: to_hex(header.metadata_digest.as_bytes()),
        deterministic,
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
                "print42" => ProgramCompiler::compile_entry_selection_with_image(
                    EntrySelection::Inline("print(42);".into()),
                    image.clone(),
                ),
                "option_string" => ProgramCompiler::compile_entry_selection_with_image(
                    EntrySelection::Inline("let s = \"hello\"; let opt = Option.some(s); print(opt);".into()),
                    image.clone(),
                ),
                "synthetic_pkg" => ProgramCompiler::compile_entry_selection_with_image(
                    EntrySelection::Package(synthetic_dir),
                    image.clone(),
                ),
                "slotmap" => ProgramCompiler::compile_entry_selection_with_image(
                    EntrySelection::Package(PathBuf::from("packages/slotmap")),
                    image.clone(),
                ),
                _ => panic!("Unknown workload {workload}"),
            };
            let compile_dur = t_comp.elapsed();
            let status = if res.is_ok() { "SUCCESS" } else { "COMPILE_ERROR" };
            (init_dur, (0u128, 0u128, 0u128, 0u128, 0u128), compile_dur, status)
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
                loaded.timings.file_read.as_micros(),
                loaded.timings.container_validation.as_micros(),
                loaded.timings.section_validation.as_micros(),
                loaded.timings.relocation.as_micros(),
                loaded.timings.publication.as_micros(),
            );

            let t_comp = Instant::now();
            let res = match workload {
                "print42" => loaded.compile_user_entry(EntrySelection::Inline("print(42);".into())),
                "option_string" => loaded.compile_user_entry(EntrySelection::Inline(
                    "let s = \"hello\"; let opt = Option.some(s); print(opt);".into(),
                )),
                "synthetic_pkg" => loaded.compile_user_entry(EntrySelection::Package(synthetic_dir)),
                "slotmap" => loaded.compile_user_entry(EntrySelection::Package(PathBuf::from("packages/slotmap"))),
                _ => panic!("Unknown workload {workload}"),
            };
            let compile_dur = t_comp.elapsed();
            let status = if res.is_ok() { "SUCCESS" } else { "COMPILE_ERROR" };
            (init_dur, timings, compile_dur, status)
        }
        _ => panic!("Unknown mode {mode}"),
    };

    let total_dur = t_proc_start.elapsed();
    let peak_rss = get_peak_rss_bytes();

    // CSV format: mode,workload,sample_id,pid,total_ms,init_ms,user_compile_ms,file_read_us,container_val_us,section_val_us,reloc_us,pub_us,peak_rss_bytes,status
    println!(
        "RAW_COLD_CSV:{mode},{workload},{sample_id},{pid},{:.3},{:.3},{:.3},{},{},{},{},{},{},{status}",
        total_dur.as_secs_f64() * 1000.0,
        init_dur.as_secs_f64() * 1000.0,
        compile_dur.as_secs_f64() * 1000.0,
        timings_tuple.0,
        timings_tuple.1,
        timings_tuple.2,
        timings_tuple.3,
        timings_tuple.4,
        peak_rss,
    );
}

fn cmd_run_cold_suite(artifact: &Path, samples: usize, csv_path: &Path) {
    let current_exe = std::env::current_exe().expect("current executable");
    let workloads = ["print42", "option_string", "synthetic_pkg", "slotmap"];
    let modes = ["source", "artifact"];

    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).expect("create csv dir");
    }

    let mut csv_lines = vec![
        "scenario,workload,sample_number,process_id,duration_ms,init_ms,user_compile_ms,file_read_us,container_val_us,section_val_us,reloc_us,pub_us,peak_rss_bytes,exit_status".to_string(),
    ];

    for workload in workloads {
        for mode in modes {
            println!("=== Running Cold Suite: mode={mode}, workload={workload} ({samples} fresh processes) ===");
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
                    eprintln!(
                        "Cold sample #{sample_id} ({mode}/{workload}) failed:\nSTDOUT: {}\nSTDERR: {}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Some(csv_data) = line.strip_prefix("RAW_COLD_CSV:") {
                        println!("Sample #{sample_id}: {csv_data}");
                        csv_lines.push(csv_data.to_string());
                    }
                }
            }
        }
    }

    fs::write(csv_path, csv_lines.join("\n") + "\n").expect("write RAW-COLD.csv");
    println!("Successfully wrote {}", csv_path.display());
}

fn cmd_run_warm_suite(artifact: &Path, repetitions: usize, csv_path: &Path) {
    let limits = ValidationLimits::default();
    let modes = ["source", "artifact"];

    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).expect("create csv dir");
    }

    let mut csv_lines = vec![
        "scenario,sample_number,process_id,input_variant,compile_ms,peak_rss_bytes,exit_status".to_string(),
    ];

    let pid = std::process::id();

    for mode in modes {
        println!("=== Running Warm Suite: mode={mode} ({repetitions} repetitions) ===");
        let (source_image, loaded_artifact) = if mode == "source" {
            let img = phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("source image");
            (Some(img), None)
        } else {
            let loaded = load_universe_telosc(artifact, &limits).expect("loaded artifact");
            (None, Some(loaded))
        };

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

            let t0 = Instant::now();
            let res = if let Some(image) = &source_image {
                ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Inline(code.into()), (*image).clone())
            } else if let Some(loaded) = &loaded_artifact {
                loaded.compile_user_entry(EntrySelection::Inline(code.into()))
            } else {
                unreachable!()
            };
            let dur = t0.elapsed();
            let status = if res.is_ok() { "SUCCESS" } else { "ERROR" };
            let peak_rss = get_peak_rss_bytes();

            let row = format!(
                "{mode},{sample_id},{pid},{variant},{:.3},{},{status}",
                dur.as_secs_f64() * 1000.0,
                peak_rss
            );
            println!("{row}");
            csv_lines.push(row);
        }
    }

    fs::write(csv_path, csv_lines.join("\n") + "\n").expect("write RAW-WARM.csv");
    println!("Successfully wrote {}", csv_path.display());
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
        ProgramCompiler::compile_entry_selection_with_image(EntrySelection::Package(minimal_dir), image.clone())
            .expect("compile package with source image")
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
    let peak_rss = get_peak_rss_bytes();

    // Format: mode,sample_id,pid,total_ms,comp_init_ms,pkg_comp_ms,vm_init_ms,vm_run_ms,peak_rss_bytes,status,stdout
    println!(
        "RAW_RUNTIME_CSV:{mode},{sample_id},{pid},{:.3},{:.3},{:.3},{:.3},{:.3},{},{status},{stdout}",
        total_dur.as_secs_f64() * 1000.0,
        comp_init_dur.as_secs_f64() * 1000.0,
        pkg_comp_dur.as_secs_f64() * 1000.0,
        vm_init_dur.as_secs_f64() * 1000.0,
        vm_run_dur.as_secs_f64() * 1000.0,
        peak_rss,
    );
}

fn cmd_run_runtime_suite(artifact: &Path, samples: usize, csv_path: &Path) {
    let current_exe = std::env::current_exe().expect("current executable");
    let modes = ["source", "artifact"];

    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).expect("create csv dir");
    }

    let mut csv_lines = vec![
        "scenario,sample_number,process_id,duration_ms,compiler_init_ms,user_compile_ms,vm_bootstrap_ms,vm_exec_ms,peak_rss_bytes,exit_status,output".to_string(),
    ];

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
                eprintln!(
                    "Child sample #{sample_id} (mode={mode}) failed:\nSTDOUT: {}\nSTDERR: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(csv_data) = line.strip_prefix("RAW_RUNTIME_CSV:") {
                    println!("Sample #{sample_id}: {csv_data}");
                    csv_lines.push(csv_data.to_string());
                }
            }
        }
    }

    fs::write(csv_path, csv_lines.join("\n") + "\n").expect("write RAW-RUNTIME.csv");
    println!("Successfully wrote {}", csv_path.display());
}

fn cmd_slotmap_eval(artifact: &Path) {
    println!("=== Evaluating packages/slotmap and packages/slotmap/tests/runtime ===");
    let limits = ValidationLimits::default();

    println!("1. Loading source formal image...");
    let source_image = (*phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("source image")).clone();

    println!("2. Loading artifact formal image...");
    let loaded = load_universe_telosc(artifact, &limits).expect("loaded artifact");

    // Case 1: packages/slotmap
    println!("Testing packages/slotmap under Source-backed...");
    let src_res1 = ProgramCompiler::compile_entry_selection_with_image(
        EntrySelection::Package(PathBuf::from("packages/slotmap")),
        source_image.clone(),
    );
    println!("Testing packages/slotmap under Artifact-backed...");
    let art_res1 = loaded.compile_user_entry(EntrySelection::Package(PathBuf::from("packages/slotmap")));

    let src_err1 = src_res1.err().map(|e| e.to_string()).unwrap_or_else(|| "OK".into());
    let art_err1 = art_res1.err().map(|e| e.to_string()).unwrap_or_else(|| "OK".into());
    println!("packages/slotmap Source error: {src_err1}");
    println!("packages/slotmap Artifact error: {art_err1}");
    assert_eq!(src_err1, art_err1, "Parity check failed for packages/slotmap!");

    // Case 2: packages/slotmap/tests/runtime
    println!("Testing packages/slotmap/tests/runtime under Source-backed...");
    let src_res2 = ProgramCompiler::compile_entry_selection_with_image(
        EntrySelection::Project(PathBuf::from("packages/slotmap/tests/runtime")),
        source_image.clone(),
    );
    println!("Testing packages/slotmap/tests/runtime under Artifact-backed...");
    let art_res2 = loaded.compile_user_entry(EntrySelection::Project(PathBuf::from("packages/slotmap/tests/runtime")));

    let src_err2 = src_res2.err().map(|e| e.to_string()).unwrap_or_else(|| "OK".into());
    let art_err2 = art_res2.err().map(|e| e.to_string()).unwrap_or_else(|| "OK".into());
    println!("packages/slotmap/tests/runtime Source error: {src_err2}");
    println!("packages/slotmap/tests/runtime Artifact error: {art_err2}");
    assert_eq!(src_err2, art_err2, "Parity check failed for packages/slotmap/tests/runtime!");

    println!("Slotmap evaluation complete: 100% parity verified between source-backed and artifact-backed!");
}

fn cmd_allocation_check(artifact: &Path) {
    println!("=== Checking cumulative allocation budgets and limits ===");
    let bytes = fs::read(artifact).expect("read artifact");

    // Binary search for exact minimum transaction budget
    let mut low = 1usize;
    let mut high = 512 * 1024 * 1024;
    let mut exact_min = high;

    while low <= high {
        let mid = (low + high) / 2;
        let limits = ValidationLimits {
            max_total_allocation_bytes: mid as u64,
            ..ValidationLimits::default()
        };
        match load_universe_telosc_from_bytes(&bytes, &limits) {
            Ok(_) => {
                exact_min = mid;
                high = mid.saturating_sub(1);
            }
            Err(_) => {
                low = mid + 1;
            }
        }
    }

    println!("Exact minimum allocation budget for universe.telosc: {exact_min} bytes ({:.2} MiB)", exact_min as f64 / 1048576.0);

    // Verify N-1 fails
    let limits_minus_one = ValidationLimits {
        max_total_allocation_bytes: (exact_min - 1) as u64,
        ..ValidationLimits::default()
    };
    assert!(
        load_universe_telosc_from_bytes(&bytes, &limits_minus_one).is_err(),
        "N-1 budget must fail"
    );

    // Verify N succeeds
    let limits_exact = ValidationLimits {
        max_total_allocation_bytes: exact_min as u64,
        ..ValidationLimits::default()
    };
    assert!(
        load_universe_telosc_from_bytes(&bytes, &limits_exact).is_ok(),
        "Exact budget N must succeed"
    );

    println!("Strict bounded allocation accounting confirmed: fail-closed at N-1 bytes, success at N bytes.");
}

fn cmd_source_denial_check(artifact: &Path) {
    println!("=== Checking Source Denial in Loaded Compiler Image ===");
    let limits = ValidationLimits::default();
    let loaded = load_universe_telosc(artifact, &limits).expect("load artifact");

    assert!(loaded.formal_image.sources().is_empty(), "Loaded formal image must have 0 sources");
    assert!(loaded.formal_image.snapshot().is_source_free_artifact_complete(), "Loaded snapshot must be source-free complete");

    let (pkg_dir, _) = setup_benchmark_packages();
    let compiled = loaded.compile_user_entry(EntrySelection::Package(pkg_dir)).expect("compile user package");
    assert!(!compiled.modules.is_empty(), "Compiled modules must be non-empty");

    println!("Source-free compiler authority verified: 0 sources, complete snapshot, successful user package compilation.");
}
