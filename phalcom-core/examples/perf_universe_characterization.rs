//! Narrow startup and repeated-fixture characterization harness.
//!
//! Run with:
//! `cargo run -p phalcom-core --example perf_universe_characterization --features test-timings -- <mode>`

#[cfg(feature = "test-timings")]
mod harness {
    use phalcom_core::modules::{CompiledProgram, ProgramCompilationSession};
    use phalcom_core::vm::VM;
    use std::hint::black_box;
    use std::path::Path;
    use std::time::{Duration, Instant};

    const FIXTURE_PATH: &str = "perf-characterization/tiny.ph";
    const TINY_SOURCE: &str = "let result = 1 + 2\n";

    fn emit(scope: &str, sample: usize, field: &str, duration: Duration) {
        println!("SAMPLE,{scope},{sample},{field},{}", duration.as_nanos());
    }

    fn emit_fixture(scope: &str, sample: usize, timing: &phalcom_core::FixtureTiming) {
        emit(scope, sample, "compile_io", timing.compile_io);
        emit(scope, sample, "compile_parse", timing.compile_parse);
        emit(scope, sample, "compile_link", timing.compile_link);
        emit(scope, sample, "semantic_session_init", timing.semantic_session_init);
        emit(scope, sample, "semantic_update", timing.semantic_update);
        emit(scope, sample, "compile_projection", timing.compile_projection);
    }

    fn emit_vm(scope: &str, sample: usize, constructor: Duration, vm: &VM) {
        emit(scope, sample, "vm_constructor_total", constructor);
        let timing = vm.bootstrap_timing();
        emit(scope, sample, "canonical_universe_product", timing.canonical_universe_product);
        emit(scope, sample, "vm_kernel_native", timing.kernel_native);
        emit(scope, sample, "vm_universe_compile", timing.universe_module_compilation);
        emit(scope, sample, "vm_universe_initializers", timing.universe_initializer_execution);
    }

    fn emit_counters(scope: &str) {
        println!("COUNTERS,{scope},full_vm_bootstraps,{}", phalcom_core::full_vm_bootstrap_count());
        println!(
            "COUNTERS,{scope},canonical_universe_product_builds,{}",
            phalcom_core::canonical_universe_product_build_count()
        );
        println!(
            "COUNTERS,{scope},bootstrap_image_instantiations,{}",
            phalcom_core::bootstrap_image_instantiation_count()
        );
        println!(
            "COUNTERS,{scope},universe_module_compilations,{}",
            phalcom_core::universe_module_compilation_count()
        );
        println!(
            "COUNTERS,{scope},universe_initializer_executions,{}",
            phalcom_core::universe_initializer_execution_count()
        );
        println!(
            "COUNTERS,{scope},semantic_baseline_builds,{}",
            phalcom_semantic::semantic_baseline_build_count()
        );
        println!(
            "COUNTERS,{scope},semantic_session_instantiations,{}",
            phalcom_semantic::semantic_session_instantiation_count()
        );
    }

    fn reset_counters() {
        phalcom_core::reset_core_counters();
        phalcom_semantic::reset_semantic_counters();
    }

    fn measure_baseline_and_session() -> ProgramCompilationSession {
        reset_counters();

        let started = Instant::now();
        let _ = phalcom_semantic::canonical_universe_semantic_baseline();
        emit("setup", 0, "semantic_baseline_build", started.elapsed());

        let started = Instant::now();
        let session = ProgramCompilationSession::standalone().expect("create canonical-backed compilation session");
        emit("setup", 0, "compilation_session_init_after_baseline", started.elapsed());
        session
    }

    fn compile_fixture(session: &mut ProgramCompilationSession, source: &str, scope: &str, sample: usize) -> CompiledProgram {
        let started = Instant::now();
        let program = session.compile_fixture(Path::new(FIXTURE_PATH), source).expect("compile tiny fixture");
        emit_fixture(scope, sample, session.last_fixture_timing());
        emit(scope, sample, "compile_fixture_total", started.elapsed());
        program
    }

    fn new_full_vm(scope: &str, sample: usize) -> VM {
        let started = Instant::now();
        let vm = VM::new();
        emit_vm(scope, sample, started.elapsed(), &vm);
        vm
    }

    fn run_compiled(vm: &mut VM, program: &CompiledProgram, scope: &str, sample: usize) {
        let started = Instant::now();
        vm.run_compiled(program).expect("run compiled tiny fixture");
        emit(scope, sample, "run_compiled_materialize_bytecode_compile_and_initialize", started.elapsed());
    }

    fn run_cold_once() {
        let all_started = Instant::now();
        let mut session = measure_baseline_and_session();
        let program = compile_fixture(&mut session, TINY_SOURCE, "cold", 0);
        let mut vm = new_full_vm("cold", 0);
        run_compiled(&mut vm, &program, "cold", 0);
        emit("cold", 0, "cold_end_to_end_internal_wall", all_started.elapsed());
        emit_counters("cold");
    }

    fn run_warm_loop(repetitions: usize) {
        let mut session = measure_baseline_and_session();

        // Warm both process-wide products and the retained compilation session.
        let warmup_program = compile_fixture(&mut session, TINY_SOURCE, "warmup", 0);
        let mut warmup_vm = new_full_vm("warmup", 0);
        run_compiled(&mut warmup_vm, &warmup_program, "warmup", 0);
        drop(warmup_vm);

        let loop_started = Instant::now();
        for sample in 1..=repetitions {
            // Keep the program shape fixed while changing its literal so the
            // compiler session follows its ordinary fixture-replacement path.
            let source = if sample % 2 == 0 { "let result = 2 + 3\n" } else { TINY_SOURCE };
            let program = compile_fixture(&mut session, source, "warm", sample);
            let mut vm = new_full_vm("warm", sample);
            run_compiled(&mut vm, &program, "warm", sample);
            drop(vm);
        }
        emit(
            "warm",
            repetitions,
            "loop_total_including_fixture_updates_and_fresh_vms",
            loop_started.elapsed(),
        );
        emit_counters("warm");
    }

    fn run_vm_tiers(warm_samples: usize) {
        reset_counters();

        let started = Instant::now();
        let kernel = VM::new_kernel();
        emit("tier.kernel", 0, "constructor_total", started.elapsed());
        drop(black_box(kernel));

        let started = Instant::now();
        let native = VM::new_native();
        emit("tier.native", 0, "constructor_total", started.elapsed());
        drop(black_box(native));

        let mut retained_vm = new_full_vm("tier.full", 0);
        for sample in 1..=warm_samples {
            let started = Instant::now();
            let vm = VM::new();
            emit_vm("tier.full", sample, started.elapsed(), &vm);
            if sample == warm_samples {
                retained_vm = vm;
            } else {
                drop(vm);
            }
        }
        emit_counters("tier");

        // Reuse the already-created VM to isolate compiled-callable execution from
        // compiler and bootstrap work. This is executor evidence, not startup data.
        let module = retained_vm.create_module("perf-execution", "<perf-execution>");
        let started = Instant::now();
        let closure = retained_vm.compile_closure(module, "1 + 2").expect("compile executor microprobe");
        emit("executor", 0, "compile_closure_once", started.elapsed());
        retained_vm.run_in_module(module, closure).expect("executor warmup");
        let started = Instant::now();
        for _ in 0..1_000 {
            retained_vm.run_in_module(module, closure).expect("execute compiled closure");
        }
        emit("executor", 1_000, "compiled_closure_1000_executions", started.elapsed());
    }

    fn run_compile_bootstrap_execution_decomposition() {
        let mut session = measure_baseline_and_session();
        let program = compile_fixture(&mut session, TINY_SOURCE, "decompose", 0);
        let mut vm = new_full_vm("decompose", 0);

        let started = Instant::now();
        vm.materialize_program(&program).expect("materialize tiny program");
        emit("decompose", 0, "user_runtime_materialization", started.elapsed());

        let compiled_module = program.modules.get(&program.entry).expect("tiny entry module");
        let source_text = compiled_module.source_text.as_deref().expect("tiny module source text");
        let started = Instant::now();
        let _closure = vm
            .compile_program_module_closure(&program.entry, source_text, &program)
            .expect("compile tiny user bytecode");
        emit("decompose", 0, "user_bytecode_compilation", started.elapsed());

        let started = Instant::now();
        vm.initialize_program(&program).expect("execute tiny user initializer");
        emit("decompose", 0, "user_initializer_execution", started.elapsed());

        // `run_compiled` also performs the final metadata-callable binding. At
        // this point its materialization, compilation, and initializer work are
        // already complete, so the residual is recorded separately.
        let started = Instant::now();
        vm.run_compiled(&program).expect("finish ordinary run_compiled path");
        emit("decompose", 0, "run_compiled_post_initialization_finalization", started.elapsed());
        emit_counters("decompose");
    }

    pub fn run() {
        let mut args = std::env::args().skip(1);
        let mode = args.next().unwrap_or_else(|| "cold".to_owned());
        match mode.as_str() {
            "cold" => run_cold_once(),
            "warm" => {
                let repetitions = args.next().and_then(|value| value.parse().ok()).unwrap_or(20);
                run_warm_loop(repetitions);
            }
            "tiers" => {
                let warm_samples = args.next().and_then(|value| value.parse().ok()).unwrap_or(20);
                run_vm_tiers(warm_samples);
            }
            "decompose" => run_compile_bootstrap_execution_decomposition(),
            other => panic!("unknown mode `{other}`; use cold, warm [N], tiers [N], or decompose"),
        }
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
