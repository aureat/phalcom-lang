//! Compile-only cold/warm measurement for the in-memory canonical Universe image.
//!
//! Run with `--features test-timings`. The warm mode preloads the process-shared
//! image before timing a new compilation session, then measures repeated fixture
//! replacement without VM bootstrap noise.

#[cfg(feature = "test-timings")]
mod harness {
    use phalcom_core::modules::ProgramCompilationSession;
    use std::path::Path;
    use std::time::Instant;
    use telos_observability::{ObservabilityLevel, PerfLifecycle, PerfRecorder, PerfRunMetadata};

    const SOURCE: &str = "let result = 1 + 2\n";

    fn emit(scope: &str, sample: usize, field: &str, nanos: u128) {
        println!("SAMPLE,{scope},{sample},{field},{nanos}");
    }

    pub fn run() {
        let mut args = std::env::args().skip(1);
        let mode = args.next().unwrap_or_else(|| "cold".to_owned());
        let repetitions = args.next().and_then(|value| value.parse().ok()).unwrap_or(20);
        assert!(matches!(mode.as_str(), "cold" | "warm"), "use cold or warm [N]");

        if mode == "warm" {
            let started = Instant::now();
            phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("canonical image warmup must succeed");
            emit("warmup", 0, "canonical_image_build_excluded", started.elapsed().as_nanos());
        }

        let lifecycle = if mode == "cold" {
            PerfLifecycle::ProcessCold
        } else {
            PerfLifecycle::ProcessWarm
        };
        let metadata = PerfRunMetadata::new(format!("compile-cache-{}", std::process::id()), ObservabilityLevel::Detailed)
            .with_lifecycle(lifecycle)
            .with_branch("perf/universe-characterization")
            .with_workload("compile_only_tiny_fixture");
        let recorder = PerfRecorder::new(metadata);
        let installed = recorder.context().install_current_thread();

        let session_started = Instant::now();
        let mut session = ProgramCompilationSession::standalone().expect("compilation session must initialize");
        emit(&mode, 0, "session_init_total", session_started.elapsed().as_nanos());

        let total_started = Instant::now();
        let samples = if mode == "cold" { 1 } else { repetitions };
        for sample in 1..=samples {
            let source = if sample % 2 == 0 { "let result = 2 + 3\n" } else { SOURCE };
            let compile_started = Instant::now();
            session
                .compile_fixture(Path::new("perf-characterization/tiny.ph"), source)
                .expect("compile-only fixture must succeed");
            let timing = session.last_fixture_timing();
            emit(&mode, sample, "compile_fixture_total", compile_started.elapsed().as_nanos());
            emit(&mode, sample, "compile_io", timing.compile_io.as_nanos());
            emit(&mode, sample, "compile_parse", timing.compile_parse.as_nanos());
            emit(&mode, sample, "compile_link", timing.compile_link.as_nanos());
            emit(&mode, sample, "semantic_session_init", timing.semantic_session_init.as_nanos());
            emit(&mode, sample, "semantic_update", timing.semantic_update.as_nanos());
            emit(&mode, sample, "compile_projection", timing.compile_projection.as_nanos());
        }
        emit(&mode, samples, "compile_loop_total", total_started.elapsed().as_nanos());

        drop(installed);
        println!("REPORT_JSON,{}", recorder.report().to_json_string().expect("serialize report"));
    }
}

#[cfg(feature = "test-timings")]
fn main() {
    harness::run();
}

#[cfg(not(feature = "test-timings"))]
fn main() {
    eprintln!("rebuild this example with `--features test-timings`");
}
