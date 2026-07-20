# T5 brief — dispatch AFTER T2 lands (conflicts: cli.rs, diagnostics/, interpret.rs shared with nothing else in flight)

Implement traceback plan unit T5 — entry-path unification + exit codes.
Repo: /Users/altunhasanli/dev/phalcom/phalcom, main directly.
READ FIRST: docs/spec/traceback/implementation-spec.md §3.5 + §7; plan.md §T5. graphify before
source reads (project rule).

Deliverables:
1. cmd_run (phalcom-core/bin/phalcom/cli.rs:137-182) rewritten to call
   vm.interpret_source(module, &source) and map the returned PhError to the sysexits table at
   interpret.rs:22-30 (Compile incl. parse → 65, Runtime → 70, missing file → 66, io → 74,
   else 1). Delete cmd_run's duplicated reporting (the compiler_error/runtime_error calls move
   with interpret_source ownership). Keep compile_mode/strip_contract_metadata setup.
2. compiler_error signature gains context: compiler_error(&mut self, err, module: ObjRef,
   source_id: u32) (dispatch.rs:180; both callers interpret.rs:208 + phalcom-repl repl.rs:116
   have both values). Span-carrying CompilerError variants render a caret block via
   diagnostics::caret; span-less variants render message-only.
3. Syntax errors one register: print_parse + cmd_check text mode migrate to the caret renderer
   (single label until the parser provides opener+closer span pairs). cmd_check --format=json
   unchanged.
4. Delete SOURCE_MAP + register_source (api.rs:103-112, duplicated double-insert body);
   migrate any reader to ModuleObject::source_at.
Write-set: phalcom-core/bin/phalcom/cli.rs, phalcom-core/src/interpret.rs,
phalcom-core/src/vm/dispatch.rs (compiler_error only — COORDINATE: if another agent holds
dispatch.rs, wait), phalcom-core/src/vm/api.rs, phalcom-core/src/diagnostics/mod.rs,
phalcom-repl/src/repl.rs, phalcom-core/tests/**.
Tests: exit-code fixture per class (65/70/66); run-vs-check identical syntax diagnostic
(strip-SGR compare); negative-lane migration for restyled parse errors; negative-control all.
Gate: cargo build && cargo test && cargo clippy --workspace. Rustdoc mandatory.
GIT: pathspec commits only (`git commit -- <paths>`), never add -a / checkout -b, ignore
unrelated dirty files, stop if write-set files dirty. End messages:
Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Return: mapping table implemented, deleted duplication summary, SHAs, test evidence.

