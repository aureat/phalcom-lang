# T7 brief — dispatch AFTER T2 (cli.rs/main.rs) AND E002 (dispatch.rs) both land

Implement traceback plan unit T7 — observability batch.
READ FIRST: implementation-spec.md §12; plan.md §T7; catalog §3-§5 (target renders).
Deliverables:
1. Fiber-switch trace events (spawn/switch/yield/done/fail) via tracing::debug! from
   switch_to_fiber_and_deliver (dispatch.rs ~:387) + spawn/completion sites. NO cfg gate (cold
   path — perf-log 003's 18.2% figure is per-opcode callsites only, Cargo.toml:20-26 comment).
   Text format per catalog §4; JSON per catalog §5 (one event per line, stderr, never colored).
2. CLI: --trace=<targets> (v1: fibers), --trace-format=text|json on Cli derive.
3. main.rs: replace hardcoded LevelFilter::OFF (main.rs:15) with filter built from --trace
   (OFF when absent). vm-trace cfg feature STAYS; --trace=dispatch without it → one warning
   naming the feature. Delete #![allow(warnings)] (main.rs:1), rustdoc the bin.
4. Recursive disasm (bin/phalcom/disasm.rs): walk constants for closures, indent nesting,
   headers name/slots/upvalues, constants resolved readable, line N via Chunk::span_at (T1 —
   if T1 not landed yet, add a local accessor and note it), Invoke operands in selector shape,
   Closure captures annotation, fused superinstructions render fusion + shadowed slot
   (dispatch.rs:538-543 context).
Write-set: phalcom-core/src/vm/dispatch.rs (trace events only — COORDINATE with any holder),
phalcom-core/bin/phalcom/main.rs, cli.rs, disasm.rs, phalcom-core/tests/**.
Tests: JSON trace fixtures (field-assert, not byte-assert) on a small fiber program; disasm
golden on nested-closure program (structure lane); negative-control.
Gate + GIT rules + rustdoc: same as T5 above.
Return: event vocabulary as implemented, flag semantics, SHAs, test evidence.
