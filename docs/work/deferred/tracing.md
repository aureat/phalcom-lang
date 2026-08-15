> **STALENESS NOTE (2026-07-20).** Superseded in part. PDR-0008 landed after this was written:
> `cmd_run` now reports through `compiler_error`/`runtime_error`, defects 1 (unwrap) and 2
> (ip underflow) below are fixed, and `compiler_error` is implemented (message-only). The
> locked capture-at-raise decision is superseded by PDR-0010 §3 (Proposed). Authoritative
> current picture: `docs/spec/current/traceback/verification-2026-07-20.md`; the spec and plan live in
> `docs/spec/current/traceback/implementation-spec.md` + `err-plan.md`. Kept for the audit trail.

You are continuing work on: U-TRACE — wire and finish Phalcom's existing-but-dead runtime traceback, plus the observability + error-rendering cleanups batched with it.

First: adopt /forge senior. Start from the entry points below — do NOT re-survey. In particular, do NOT grep for "backtrace|traceback|stack_trace" — that vocabulary returns zero hits and produced a false "no traceback machinery exists" premise earlier in this work. The machinery is spelled print_rt / SourceLoc / runtime_error / print_line_information.

Done so far: audit only, no code written, no commits. Working tree at branch `main`, uncommitted: Cargo.lock, Cargo.toml, docs/forge/UNITS-TRACKER.md modified; docs/forge/perf-log/011-attack-on-010.md, docs/forge/units/U-BINDINGS/implementation-spec.md, testing/ untracked. Three read-only audit lenses completed (unwind/handler surface, panic discipline, error-message quality). No spec document written yet — writing docs/forge/units/U-TRACE/implementation-spec.md is the immediate next step.

CORRECTED CENTRAL FINDING (supersedes any earlier claim that traceback machinery is missing):
Python-style traceback machinery EXISTS and is unwired.
  - phalcom-core/src/diagnostics.rs — print_rt(msg, &[SourceLoc]) emits the literal
    "Traceback (most recent call last):" header. SourceLoc { module_name, method_name, span, source }.
  - phalcom-core/src/vm/dispatch.rs:121 — runtime_error() walks self.frames, resolves
    module + method + span per frame, calls print_rt.
  - phalcom-core/src/interpret.rs:186 — interpret_source() wires both compile and runtime error hooks.
  - Why it never fires: phalcom-core/bin/phalcom/cli.rs:137 cmd_run calls compile_closure +
    run_in_module directly, then `eprintln!("{e}"); std::process::exit(1)`. interpret_source is
    called ONLY from tests/gc.rs, tests/invariants.rs, benches/vm_bench.rs, and interpret.rs:125.
  - Empirically confirmed: `class T { a { self.b } b { self.c } c { 1.bogusSelector } }` + `T.new().a`
    prints exactly `1 does not understand 'bogusSelector'`, exit 1. No file, no line, no frames.

It CANNOT simply be wired up. Four latent defects inside runtime_error (dispatch.rs:121-144):
  1. `module_source.unwrap()` — panics whenever ModuleObject::source is None.
  2. `chunk.spans[frame.ip - 1]` — underflows when ip == 0.
  3. `self.frames.clone()` — clones the whole frame vec on every error.
  4. `.rev()` yields newest-first, but print_rt's own doc contract says caller→callee (oldest first). Inverted.
Also: print_frame (diagnostics.rs:91) is half-commented-out — calls only print_line_information, so the
`File "...", line N` header never prints and SourceLoc's module_name/method_name are computed then discarded.
Also: `pub fn compiler_error(&mut self, err: PhError) {}` at dispatch.rs:147 is an empty stub.

Next step(s), in order:
  1. Write docs/forge/units/U-TRACE/implementation-spec.md against the corrected picture (this unit is
     "finish and wire what is half-built", NOT "build a traceback" — smaller than originally scoped).
  2. Fix the four runtime_error defects; store module_id and resolve source lazily so defect 1's unwrap
     has nowhere to live.
  3. Restore print_frame; implement or delete compiler_error.
  4. Wire cmd_run through the diagnostic path.
  5. Unify run-path vs check-path syntax-error rendering.
  6. Dead/duplicate RuntimeError cleanup (see below).
  7. Observability batch: fiber-switch debug! at dispatch.rs:352, --trace flag, disasm recursion,
     LevelFilter::OFF unhardcode.

IN-SCOPE findings (all verified, file:line):
  - Syntax errors render worse on `phalcom foo.ph` than `phalcom check foo.ph`. cmd_run (cli.rs:137-166)
    propagates via `?` into anyhow Debug → `SyntaxError` bare Display `"{kind} (at bytes {start}..{end})"`
    (phalcom-ast/src/error.rs:46-52). cmd_check (cli.rs:194-218) converts to line/col via
    byte_offset_to_line_col (cli.rs:226-238) and calls diagnostics::print_parse → caret-underlined snippet.
    Two visual registers for one error class.
  - RuntimeError::UndefinedVar (error.rs:115) is DEAD — zero construction sites. Live path uses ad-hoc
    RuntimeError::Message(format!("Undefined variable '{}'.", name)) at dispatch.rs:664 and :716.
    Divergent quoting (backtick vs single-quote, trailing period). Editing the enum text does nothing.
  - Dead variants, zero construction sites: UnsupportedOperation (error.rs:96), BinaryNotSupported
    (error.rs:99), UnaryNotSupported (error.rs:106). ZeroDivision (error.rs:118) also dead but BY DESIGN
    (IEEE-754, number_div at primitive/number.rs:104-114 documents 1/0 → inf) — needs a doc comment
    saying so, not deletion.
  - RuntimeError::Internal leaks Rust Debug of Value on user-reachable paths: dispatch.rs:1009/:1012
    `format!("NewInstance operand is not a class: {:?}", class_val)`. `let Foo = 5; Foo.new()` yields
    "NewInstance operand is not a class: Number(5.0)" — labeled "Internal" as if a VM bug, not user error.
    Same pattern at dispatch.rs:956/:959/:984/:987 for field access.
  - No did-you-mean / near-miss selector suggestion anywhere in object_does_not_understand
    (primitive/object.rs:230-251). Receiver rendering itself is CLEAN (to_string → "<ClassName instance>",
    no Debug leak). Additive, no dependency — defer without cost if the unit gets large.

VERIFIED CLEAN — do not re-audit:
  - `ensure` runs exactly once on all three paths (normal, NLR-through, uncaught raise). block_ensure at
    primitive/block.rs:303-323. Cleanup-supersedes correct (cleanup_err wins; cleanup NLR wins via
    frames.len() < frames_before_cleanup).
  - Non-local return through an in-flight ensure/on fires cleanup correctly (ReturnNonLocal
    dispatch.rs:1110-1160 vs run_until_inner's frames.len() <= base_frames check at dispatch.rs:490-495).
  - An error raised inside a handler/ensure body propagates without corrupting handler state.
  - Fiber floor always reifies to a catchable surface Error; no native Rust error leaks across the
    boundary (capture_error_value dispatch.rs:370-379).
  - Rethrow preserves origin: Result#unwrap/unwrapErr (core.ph:586, :593) re-raise the same instance.
  - The unwind core matches ADR-0008. This is the expensive-to-be-wrong part and it is sound.

OUT OF SCOPE — do not bundle into U-TRACE:
  - BLOCKER, separate fix, higher priority than U-TRACE: Map/Set locate() sends user hash/== reentrantly
    while holding a live slot index. primitive/map.rs:54-66 consumed at :80/:108/:137; primitive/set.rs:47-59
    consumed at :110; heap/map.rs:106-108 set_value_at and :128-154 remove_at (both doc'd "Panics if slot
    out of range", precondition unenforced). User code mutating the same map from `==` leaves the index
    stale → indexing panic OR SILENT CORRUPTION pointing at a swap-removed entry with no error surfaced.
  - Typed-catch reification: every non-Raise RuntimeError wraps to base Error on catch (block.rs:253-263,
    dispatch.rs:370-379), so on(DeadFrameError)/on(RangeError) never fire — those kernel classes do not
    exist (only Error, MessageNotUnderstood, CannotYieldAcrossNativeFrame). Three worked examples in
    docs/spec/current/error-handling.md:143-146 are currently FALSE. Deliberately deferred per
    docs/forge/units/U-ERR/plan.md:277-281, but the spec doc was never marked aspirational. CHEAP FIX
    WORTH DOING NOW: annotate those spec examples as unimplemented so the doc stops lying.
  - block_on rustdoc (primitive/block.rs:226-227) claims non-Raise errors are "re-propagated unchanged";
    code wraps and catches them (block.rs:253-278). Doc contradicts behavior. Cheap doc fix.
  - GC ensure temp-root UAF and F1 fiber-floor upvalue-close crash. SEQUENCING HAZARD: runtime_error
    dereferences heap.closure(frame.closure) while walking; a stale ObjRef from either bug makes the
    traceback ITSELF the crash site (panic at heap/mod.rs:188 "dangling ObjRef"), converting a clean error
    into a panic. Fix those before the traceback runs on every error path.

Decisions locked (do not re-litigate):
  - Traceback surface: Python ordering (most-recent-call-last, error at bottom). Plain frame lines; miette
    caret block on the INNERMOST frame only — not all frames (a 40-frame miette render is unreadable).
  - Core-library frames elided by default with a count, expandable via --trace-core. FrameView::is_core().
  - Cross-fiber: traceback CHAINS across the fiber floor with a "raised in fiber #N, spawned at file:line"
    link (Python __context__/__cause__ precedent). Does not stop at the floor.
  - Primitive shape: a walkable live stack object in Rust; the string formatter is ONE CONSUMER, not the
    primitive. Consumers: traceback render, fiber-switch log, later REPL `where`.
  - Capture timing: capture a COMPACT frame record at raise (module id + closure id + ip per frame — no
    strings, no source resolution); resolve to text lazily at render. Rationale: .attempt()/on(_) handlers
    already exist (core.ph:1427), so walk-time-only capture loses origin for every CAUGHT error. Also
    kills the module_source.unwrap() panic. Hang it next to RuntimeError::Raise's existing `rendered`
    field (error.rs:88), which is already a raise-time snapshot.
  - Frame walk yields LOGICAL frames, not physical — 1:many expansion from day one (today always 1:1,
    costs nothing). Protects against future inlining/TCO making the traceback silently lie. Precedent:
    superinstruction fusion already solved span-fidelity-under-transformation at dispatch.rs:538-543.
  - Trace output stability: golden fixtures assert FIELDS via a --trace-format=json line-per-event stream,
    never byte-exact human layout. Same for the traceback — assert frame SEQUENCE, not box-drawing.
  - Error.stackTrace surface reflection: BACKLOGGED, not in this unit. Rides on the compact-capture record
    whenever wanted.
  - Fiber-switch debug! at dispatch.rs:352 needs NO cfg gate — it is a cold path, so perf-log 003's
    per-opcode argument does not apply. Runtime filter alone suffices.

Open decisions:
  - Does U-TRACE need its own ADR, or does it ride an existing one? If new: flip docs/adr/STATUS.md in the
    SAME pass (two-way sync rule), and note docs/adr/README.md still lists ADR-0014 as Accepted though its
    own file says Superseded.
  - Which code paths leave ModuleObject::source as None (REPL? `-i` inline? core.ph bootstrap?).
    Determines where source-line echo degrades to bare file:line. [verify: enumerate None-yielding paths]
  - Whether the cross-fiber spawn-site link collides with U-FIBER's floor-capture ownership (DEC-FIB-A
    gave U-FIBER that territory).
  - spans is 16 bytes/instruction (SourceRange = CopyRange<usize>, phalcom-common/src/range.rs:122), one
    per instruction parallel to code — very likely outweighs the bytecode it annotates. NOT asking to fix
    now; asking to record as named debt with migration shape (delta-encoded line table, binary search) AND
    to consume spans through an accessor rather than indexing spans[ip] directly at N call sites, so the
    later change stays contained.

Constraints / invariants / gotchas:
  - Rust docs are MANDATORY: //! on every module, /// on every public item incl. fields and enum variants.
    Undocumented public API is an incomplete change. See docs/rust-documentation-guidelines.md.
  - main has LIVE CONCURRENT SESSIONS. Verify against the tree before trusting any landed-state claim.
    Commit narrow paths on `main` itself. NEVER `git add -a`. NEVER `git checkout -b` (it hijacks other
    sessions' commits; narrow staging does not protect against this).
  - Commit per green checkpoint, not one end-of-unit batch. Never commit a non-compiling tree. An
    implementer's in-tree gate hides partial-stage commits — verify each SHA in a throwaway worktree.
  - vm-trace is DOUBLE-blocked: the cfg feature gates the per-opcode callsites (Cargo.toml:26,
    dispatch.rs:551) AND phalcom-core/bin/phalcom/main.rs:15 hardcodes
    `registry().with(stdout_log.with_filter(filter::LevelFilter::OFF)).init()`. Both must fall for output.
    No env var, no CLI flag today. Cli already has three #[arg(long)] flags (cli.rs:12-48) — hang --trace there.
  - disasm (phalcom-core/bin/phalcom/disasm.rs:11) prints ONLY the top-level chunk: takes
    closure.callable.chunk, iterates constants + code flat. Nested closures sit IN constants as Value::Obj
    and print as {:?} handles — method bodies and block bodies are invisible. Needs recursion.
  - Do not harden the ctor-inherit guard — DEC-CTOR-H schedules the whole guard for deletion in U-CTOR-4.
  - Perf numbers come ONLY from docs/forge/perf-log/SCOREBOARD.md. Never quote perf from memory.
  - Phalcom syntax notes confirmed this session: `nil` is NOT a keyword (None is the absence value);
    constructors are `T.new()`; fibers are `Fiber.new { ... }` / `Fiber.yield(v)` / `Fiber.current` / `.call()`.

Verify green with: cargo build && cargo test && cargo clippy --workspace