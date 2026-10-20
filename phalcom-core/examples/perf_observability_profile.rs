use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::{BufferedOutput, VM};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use telos_observability::{ObservabilityLevel, PerfLifecycle, PerfRecorder, PerfRunMetadata};

#[path = "perf_workloads.rs"]
mod perf_workloads;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name} to identify this profile's source state"))
}

fn profile_semantic_update() -> Result<(), String> {
    let source: Arc<str> = Arc::from("class PerfSubject {}\n");
    let program = Arc::new(phalcom_ast::parse_source(&source, 0).map_err(|error| error.to_string())?);
    let module = phalcom_modules::ModuleId::resolved(phalcom_modules::ResolvedProjectId::from_raw(1), phalcom_modules::ModulePath::root());
    let interface =
        phalcom_modules::InterfaceBuilder::build(module.clone(), phalcom_modules::source::ModuleKind::Module, &program).map_err(|error| error.to_string())?;
    let interfaces = BTreeMap::from([(module.clone(), interface)]);
    let universe = Arc::new(phalcom_modules::ProjectUniverse::new());
    let linked = Arc::new(
        phalcom_modules::ModuleLinker::new(universe, interfaces)
            .link(module.clone(), &BTreeMap::new())
            .map_err(|error| error.to_string())?,
    );
    let parsed = Arc::new(phalcom_modules::ParsedModuleUnit::new(
        module.clone(),
        phalcom_modules::source::ModuleKind::Module,
        None,
        source,
        program,
    ));
    let input = phalcom_semantic::workspace::SemanticWorkspaceInput::new(linked, BTreeMap::from([(module, parsed)]), 1);
    phalcom_semantic::session::SemanticWorkspaceSession::new()
        .update(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn profile_canonical_semantic_update() -> Result<(), String> {
    let provider = phalcom_modules::UniverseSourceProvider::new();
    let universe = Arc::new(phalcom_modules::ProjectUniverse::new());
    let filesystem = phalcom_modules::FilesystemSourceProvider::new();
    let mut resolver = phalcom_modules::ModuleResolver::with_universe_source_provider(&universe, &filesystem, provider.clone());
    let mut sources = BTreeMap::new();
    let mut interfaces = BTreeMap::new();
    for node in provider.nodes() {
        let path = phalcom_modules::ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                .collect::<Vec<_>>(),
        );
        let module = phalcom_modules::ModuleId::universe(path);
        sources.insert(module.clone(), provider.load_parsed(&module).map_err(|error| error.to_string())?);
        interfaces.insert(module.clone(), provider.load_interface(&module).map_err(|error| error.to_string())?);
    }
    let mut resolved = BTreeMap::new();
    for (module, interface) in &interfaces {
        for import in &interface.imports {
            let path = match import {
                phalcom_modules::ImportSurface::Module(declaration) => &declaration.path,
                phalcom_modules::ImportSurface::Selective(declaration) => &declaration.path,
                phalcom_modules::ImportSurface::ReExport(declaration) => &declaration.path,
            };
            let target = resolver.resolve_import(module, path).map_err(|error| error.to_string())?;
            resolved.insert((module.clone(), path.to_string()), target.id);
        }
    }
    let linked = Arc::new(
        phalcom_modules::ModuleLinker::new(universe, interfaces)
            .link_all(phalcom_modules::ModuleId::universe_root(), &resolved)
            .map_err(|error| error.to_string())?,
    );
    let input = phalcom_semantic::workspace::SemanticWorkspaceInput::new(linked, sources, 1);
    phalcom_semantic::session::SemanticWorkspaceSession::new()
        .update(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn profile_compiler_projection() -> Result<(), String> {
    ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from("let answer = 42\n")))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn profile_standalone_compile() -> Result<(), String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/universe_v1/standalone_module/solo.ph");
    ProgramCompiler::compile_entry_selection(EntrySelection::Module(path))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn profile_multi_module_project() -> Result<(), String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/universe_v1/project_hierarchy");
    ProgramCompiler::compile_entry_selection(EntrySelection::Project(path))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let first = arguments.next();
    let second = arguments.next();
    let (mode, level) = match first.as_deref() {
        None => ("vm-cold".to_owned(), ObservabilityLevel::Summary),
        Some("off" | "summary" | "detailed" | "trace") => {
            let level = match first.as_deref().unwrap() {
                "off" => ObservabilityLevel::Off,
                "summary" => ObservabilityLevel::Summary,
                "detailed" => ObservabilityLevel::Detailed,
                "trace" => ObservabilityLevel::Trace,
                _ => unreachable!(),
            };
            (second.unwrap_or_else(|| "vm-cold".to_owned()), level)
        }
        Some(mode) => {
            let level = match second.as_deref() {
                None => ObservabilityLevel::Detailed,
                Some("off") => ObservabilityLevel::Off,
                Some("summary") => ObservabilityLevel::Summary,
                Some("detailed") => ObservabilityLevel::Detailed,
                Some("trace") => ObservabilityLevel::Trace,
                Some(other) => panic!("unknown observability level {other}; use off, summary, detailed, or trace"),
            };
            (mode.to_owned(), level)
        }
    };
    assert!(arguments.next().is_none(), "expected at most a workload and an observability level");

    let semantic_baseline = mode == "semantic-baseline";
    let semantic_update = mode == "semantic-update";
    let canonical_update = mode == "canonical-update";
    let compiler_projection = mode == "compiler-projection";
    let session_init = mode == "session-init";
    let standalone_compile = mode == "standalone-compile";
    let multi_module_project = mode == "multi-module-project";
    let feature_dense = mode == "feature-dense";
    let incremental_updates = mode == "incremental-updates";
    if !matches!(
        mode.as_str(),
        "vm-cold"
            | "semantic-baseline"
            | "semantic-update"
            | "canonical-update"
            | "compiler-projection"
            | "session-init"
            | "standalone-compile"
            | "multi-module-project"
            | "feature-dense"
            | "incremental-updates"
    ) {
        panic!(
            "unknown workload {mode}; use vm-cold, semantic-baseline, semantic-update, canonical-update, compiler-projection, session-init, standalone-compile, multi-module-project, feature-dense, or incremental-updates"
        );
    }
    if session_init {
        // This setup is intentionally outside the recorder: the sample measures
        // construction of a new session with the canonical semantic baseline warm.
        phalcom_semantic::canonical_baseline::canonical_universe_semantic_baseline();
    }
    let workload = match mode.as_str() {
        "semantic-baseline" => "canonical_semantic_baseline",
        "semantic-update" => "ordinary_semantic_update",
        "canonical-update" => "canonical_semantic_update",
        "compiler-projection" => "ordinary_compiler_projection",
        "session-init" => "semantic_session_instantiation",
        "standalone-compile" => "standalone_module_compile",
        "multi-module-project" => "multi_module_project_compile",
        "feature-dense" => "feature_dense_semantic_package",
        "incremental-updates" => "incremental_update_variants",
        _ => "process_cold_vm_bootstrap",
    };
    let lifecycle = match mode.as_str() {
        "semantic-update" | "canonical-update" => PerfLifecycle::SessionCold,
        "session-init" | "incremental-updates" => PerfLifecycle::SessionWarm,
        _ => PerfLifecycle::ProcessCold,
    };
    let mut incremental_workspace = if incremental_updates {
        Some(perf_workloads::IncrementalWorkspace::prepare())
    } else {
        None
    };
    let run_id = format!(
        "perf-profile-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let metadata = PerfRunMetadata::new(run_id, level)
        .with_lifecycle(lifecycle)
        .with_baseline_head(required_env("PHALCOM_PERF_BASELINE_HEAD"))
        .with_execution_tree_head(required_env("PHALCOM_PERF_EXECUTION_TREE_HEAD"))
        .with_source_revision(required_env("PHALCOM_PERF_SOURCE_REVISION"))
        .with_branch(std::env::var("PHALCOM_PERF_BRANCH").unwrap_or_else(|_| "branch-not-recorded".to_owned()))
        .with_dirty(std::env::var("PHALCOM_PERF_DIRTY").is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true")))
        .with_toolchain(std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "toolchain-not-recorded".to_owned()))
        .with_workload(workload);
    let recorder = PerfRecorder::new(metadata);
    let installed = recorder.context().install_current_thread();
    let result = if semantic_baseline {
        std::panic::catch_unwind(|| {
            phalcom_semantic::canonical_baseline::canonical_universe_semantic_baseline();
        })
        .map_err(|_| "canonical semantic baseline panicked".to_owned())
    } else if semantic_update {
        std::panic::catch_unwind(profile_semantic_update).unwrap_or_else(|_| Err("semantic update workload panicked".to_owned()))
    } else if canonical_update {
        std::panic::catch_unwind(profile_canonical_semantic_update).unwrap_or_else(|_| Err("canonical semantic update workload panicked".to_owned()))
    } else if compiler_projection {
        std::panic::catch_unwind(profile_compiler_projection).unwrap_or_else(|_| Err("compiler projection workload panicked".to_owned()))
    } else if standalone_compile {
        std::panic::catch_unwind(profile_standalone_compile).unwrap_or_else(|_| Err("standalone compile workload panicked".to_owned()))
    } else if multi_module_project {
        std::panic::catch_unwind(profile_multi_module_project).unwrap_or_else(|_| Err("multi-module project workload panicked".to_owned()))
    } else if feature_dense {
        std::panic::catch_unwind(perf_workloads::run_feature_dense).unwrap_or_else(|_| Err("feature-dense workload panicked".to_owned()))
    } else if incremental_updates {
        match incremental_workspace.as_mut().expect("incremental workload is prepared") {
            Ok(workspace) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| workspace.run_variants()))
                .unwrap_or_else(|_| Err("incremental update workload panicked".to_owned())),
            Err(error) => Err(error.clone()),
        }
    } else if session_init {
        std::panic::catch_unwind(|| {
            drop(phalcom_semantic::SemanticWorkspaceSession::new());
            Ok(())
        })
        .unwrap_or_else(|_| Err("semantic session initialization panicked".to_owned()))
    } else {
        VM::try_new_with_output(Box::new(BufferedOutput::new()))
            .map(|_| ())
            .map_err(|error| error.to_string())
    };
    drop(installed);

    let report = recorder.report();
    println!("{}", report.to_json_string().expect("serialize performance report"));
    if let Err(error) = result {
        eprintln!("Profile workload failed after emitting a partial profile: {error}");
        std::process::exit(1);
    }
}
