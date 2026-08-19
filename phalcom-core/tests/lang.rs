//! Language acceptance corpus.
//!
//! One `#[test]` per feature label so the suite can be filtered by label with
//! `cargo test -p phalcom-core --test lang <label>`.

mod support;

#[test]
fn lexical() {
    support::check_pass("lexical");
}

#[test]
fn arithmetic() {
    support::check_pass("arithmetic");
}

#[test]
fn booleans() {
    // U5: `and`/`or` are lazy sends over a block argument (control-flow.md
    // §2) — graduated from PENDING.
    support::check_pass("booleans");
}

#[test]
fn bindings() {
    support::check_pass("bindings");
}

#[test]
fn messages() {
    support::check_pass("messages");
}

#[test]
fn dispatch() {
    support::check_pass("dispatch");
}

#[test]
fn classes() {
    support::check_pass("classes");
}

#[test]
fn classes_negative() {
    // U-REOPEN-FIX (ADR-0018 "attaches methods, not shadows"): the two
    // reopen shapes ruled out of scope for the reopen-appends-methods fix —
    // adding fields (the reused `ClassId` is never relayouted) and changing
    // the superclass (forbidden by U13 sealed inheritance) — are rejected at
    // compile time with a clear diagnostic instead of silently mishandled.
    support::check_negative("classes/negative");
}

#[test]
fn control_flow() {
    support::check_pass("control-flow");
}

#[test]
fn syntax_errors() {
    support::check_negative("syntax-errors");
}

#[test]
fn runtime_errors() {
    support::check_negative("runtime-errors");
}

#[test]
fn absence() {
    // Immediate `Option` (`Some`/`None`) — canonical `Some(_)`, compatibility
    // `Some(_)`, and the `match(some:none:)` eliminator.
    support::check_pass("absence");
}

#[test]
fn absence_negative() {
    // Ported from Wren `test/core/null/{no_constructor,not}.wren`: `None`
    // has no `new()` (it is an immediate variant, ADR-0007/PDR-0033) and no
    // `not` (Phalcom's `!` is `Bool`-only, ADR-0021 — no truthiness
    // coercion makes `None` an exception) — both plain does-not-understand.
    support::check_negative("absence/negative");
}

#[test]
fn compile_errors() {
    // U6: compile-time diagnostics — surface `nil` is undefined, `let` requires
    // an initializer and rejects reassignment (ADR-0014), and a literal
    // `Option` condition has no truth value (BD-U6-1 Option A).
    support::check_negative("compile-errors");
}

#[test]
fn metaclass() {
    support::check_pass("metaclass");
}

#[test]
fn reflection() {
    // U-CORE-1: kernel reflection — `hash` (stable/distinct/content-based) and
    // `Behavior#name` (a class's own name), all over the ADR-0023 floor.
    support::check_pass("reflection");
}

#[test]
fn blocks() {
    support::check_pass("blocks");
}

#[test]
fn functions() {
    // U-CORE-3: `Object#methodFor(_)` / `Method#invokeOn(_,***)` / `Method#bind(_)`
    // / `Method#selector` / `Method#holder` — the Method reflection surface,
    // exercised over already-supported syntax (`Symbol.new(_)`,
    // `List.new().add(_)`). The remaining pending fixtures below are gated on
    // U-LEX's `#...`/`[...]`/`::` literals.
    support::check_pass("functions");
}

#[test]
fn functions_negative() {
    // Ported from Wren `test/core/function/{call_extra_arguments,
    // call_missing_arguments,call_runtime_error}.wren`: unlike Wren's
    // pad/truncate call-arity leniency, `Block#call` (`primitive/block.rs`'s
    // `block_call`) is strict — any arity mismatch raises
    // `RuntimeError::Arity`; a body error still propagates through `call`
    // unchanged.
    support::check_negative("functions/negative");
}

#[test]
fn errors() {
    // U-ERR (ADR-0008/0031/0037/0038): `throw`/`try`/`on`/`catch`/`ensure` +
    // `Result`/`Ok`/`Err` + `Block#attempt()` — graduated from PENDING.
    support::check_pass("errors");
}

#[test]
fn system() {
    support::check_pass("system");
}

#[test]
#[ignore = "PENDING: System/IO — later"]
fn system_pending() {
    support::check_pending("system");
}

#[test]
fn concurrency() {
    // U-FIBER (ADR-0030): bare cooperative `Fiber` — `call`/`try`/`yield`/
    // `current`/`abort`, the restricted-yield guard, and fiber-floor error
    // capture. `Future`/`async`/`await` stay pending (see below).
    //
    // `each_generator_raises.ph` (U-ITER deferred item 5): `List#each { Fiber.yield }`
    // yields across `each`'s native block-call frame — `CannotYieldAcrossNativeFrame`,
    // same guard as `concurrency_fiber_restricted_yield_guard.ph` but reached via the
    // collection protocol rather than `Function#call` directly.
    support::check_pass("concurrency");
}

#[test]
fn concurrency_negative() {
    // U-FIBER reviewer follow-ons: `Fiber.abort(_)` on the root fiber
    // (no resumer, spec §2 rule 7/§6), `Fiber#call` gated by the
    // restricted-switch guard underneath a native re-entrant frame (a
    // resume-specific diagnostic, distinct from the yield-specific one),
    // and C-FIB-5 — a block escaping to a *different* fiber's stack still
    // raises `DeadFrameError` once its home activation is dead (ADR-0013
    // fencing is fiber-agnostic).
    support::check_negative("concurrency/negative");
}

#[test]
fn option() {
    // U-STD: `Option` transform/extract combinators — `map(_)`, `flatMap(_)`,
    // `filter(_)`, `ifSome(_)`, `unwrapOr(_)`, all pure `.ph` over the native
    // `match` eliminator (values-and-absence.md §3.3; catalog-delta.md §2.2).
    support::check_pass("option");
}

#[test]
fn list() {
    // U-LIST: kernel `List` — native array storage, `.ph`-defined
    // at(_:)/size/add(_:)/each(_:) protocol over the floor primitives.
    support::check_pass("list");
}

#[test]
fn bytes() {
    // U-BYTES (PDR-0011 + PDR-0013 ruling 4): the kernel octet buffer —
    // laws 1-8 of bytes.md §5, positive lane. The yield-mid-iteration row
    // (law 8) lives in the concurrency lane
    // (`concurrency_fiber_yield_through_block_call.ph`).
    support::check_pass("bytes");
}

#[test]
fn bytes_negative() {
    // bytes.md law 1's raise half: every precondition violation raises with
    // the named diagnostic — never `None`, never a silent clamp.
    support::check_negative("bytes/negative");
}

#[test]
fn collections() {
    // U-CORE-5: the shared collection-protocol contract, certified against
    // `List` as its reference implementation — sequence laws (size/at/add/
    // each) plus the structural `==`/`!=` this unit adds. Does not duplicate
    // `list()`'s own-unit corpus (see `list_map_and_filter.ph` etc.); this
    // corpus guards the shared contract any future collection must satisfy.
    support::check_pass("collections");
}

// NB: U-COLL's list/tuple/grouping/brace-disambiguation PASS fixtures live in
// `tests/lang/collections/` and are exercised by `collections()` above; the
// deferred-runtime cases are split out below.

#[test]
fn collections_literals_negative() {
    // Collection-literal lane mismatches plus E.3/F.2 provably-unbounded
    // outgoing spread diagnostics. These are real negative cases, not pending
    // success fixtures.
    support::check_negative("collections/negative");
}

#[test]
#[ignore = "deferred collection spread and boundedness"]
fn collections_pending() {
    support::check_pending("collections");
}

#[test]
fn collections_d1() {
    // D.1 is complete independently of the still-deferred spread fixtures in
    // the collections pending directory; keep its comprehensive gate active.
    support::check_pending_case("collections", "eager_operations");
}

#[test]
fn rest_dispatch() {
    // F.3: lane-aware rest declarations, structural selectors, tuple capture,
    // and exact-selector miss fallback.
    support::check_pass("rest");
}

#[test]
fn inheritance() {
    // U-INH: single inheritance — `class B is A`, inherited instance
    // methods with subclass overrides, the parallel-metaclass rule making
    // `static` members inherit (ADR-0002 rule 4), `super.sel(…)` sends via the
    // `SuperSend` opcode (method-lookup.md §1.14), and constructor initializer chaining.
    support::check_pass("inheritance");
}

#[test]
fn iteration() {
    // U-ITER (ADR-0035, iteration.md): the two-selector cursor protocol
    // (`iterate(_)`/`iteratorValue(_)`) on `List`, the `for (x in coll)`
    // surface lowering to an inlined cursor `while`, and `break`/`continue`
    // as jump-based loop control — all pure `.ph` + compiler lowering over the
    // existing floor (zero new primitives).
    //
    // `for_generator_suspends.ph` (C-ITER-8, U-ITER deferred item 5): a `for`
    // loop body running inside a `Fiber` suspends at each `Fiber.yield` and
    // resumes at the next cursor position on the next `call` — the direct-jump
    // `for` lowering (C-ITER-4) composes with fiber suspension.
    support::check_pass("iteration");
}

#[test]
fn iteration_disasm() {
    // C-ITER-4 (the §7.1 preclusion guard, D-ITER-2): a `for` body lowers to a
    // direct jump loop — `JumpIfFalse`/`Loop` + `iterate`/`iteratorValue`
    // sends — and emits **no** `Closure`/`block_call` on the taken path, so a
    // `for` inside a fiber can `yield` freely.
    support::check_for_no_block_call("iteration/for_disasm_no_block_call.ph");
    support::check_for_zero_alloc_loop("iteration/for_disasm_no_block_call.ph");
    support::check_for_no_wrapsome("iteration/zero_alloc_disasm_probe.ph");
}

#[test]
fn iteration_negative() {
    // U-REOPEN-FIX (graduated from `iteration/pending`; ADR-0035 §3,
    // iteration.md §3): `break`/`continue` reached through a block the
    // inliner materializes as a real closure — the deopt fallback of a
    // non-Bool `if` condition, or an ordinary block-arg closure like
    // `each { break }` — cannot statically jump into the enclosing loop's
    // chunk (`same_function` false). Rather than the silent no-op U-ITER
    // originally shipped with, `Compiler::emit_deopt_block_control_trap`
    // (compiler/lib.rs) now emits `Error.new(message).raise()` with a
    // descriptive message, so the rare cross-block case fails **loudly**
    // instead of quietly no-oping. The common `if (Bool) { break }` path is
    // unaffected (inliner fast path never takes this deopt twin for a real
    // Bool). See `docs/forge/DEFERRED.md` for the full non-local-break
    // follow-on this does not attempt.
    support::check_negative("iteration/negative");
}

#[test]
fn sequence() {
    // D.1/E.1: labeled predicate queries, explicit eager/lazy receiver semantics,
    // and the remaining cursor-based sequence tests.
    support::check_pass("sequence");
}

#[test]
fn iterator() {
    support::check_pass("iterator");
}

#[test]
fn iterator_negative() {
    support::check_negative("iterator/negative");
}

#[test]
fn sequence_negative() {
    // U-SEQ: guard clauses raise `Error` for negative/non-number counts in
    // `SkipView`/`TakeView` construction.
    support::check_negative("sequence/negative");
}

#[test]
fn values() {
    // U-CORE-4: per-type `toString` — `Number`/`String`/`Bool`/`Symbol`/
    // `None`/`Some(_)` message rendering, kept in agreement with the native
    // print path (`Value::to_string`), plus the `Object` default
    // (`"<ClassName>"`, ADR-0015, DEFERRED F4).
    support::check_pass("values");
}

#[test]
#[ignore = "Modules v1 retires physical imports; runtime module loading is out of scope for Part I"]
fn imports() {
    // Retained as historical U15 runtime coverage. Modules v1 is compile-time
    // only and intentionally rejects this physical-import corpus.
    support::check_pass("imports");
}

#[test]
fn family() {
    // Current Family semantics (`docs/spec/callables/family.md` §§1–5):
    // bound exact getter/nullary/method/setter references retain selector
    // identity, structural patterns route live overload shapes, class-side
    // and inherited receivers remain bound, and a receiver-side
    // `doesNotUnderstand` can handle a missing exact route.
    support::check_pass("family");
}

#[test]
fn family_negative() {
    // Current Family negatives: exact and pattern construction defers missing
    // route errors until call time, exact Families reject incompatible call
    // shapes, and missing pattern routes reach ordinary dNU.
    support::check_negative("family/negative");
}

#[test]
fn string() {
    // Wren-suite port (test/core/string/*.wren, string_byte_sequence/*.wren,
    // string_code_point_sequence/*.wren): the behavior that already carries
    // over onto today's thin `String` floor (`+(_)`/`hash`/`toString`/static
    // `new`, primitive/string.rs) — content-equality `==`/`!=`, `+`
    // concatenation, `toString` identity, `isA`/`class` type tests, and the
    // `String.new(_)` any-value coercion (a deliberate divergence from
    // Wren's constructor-less `String` metaclass).
    support::check_pass("string");
}

#[test]
fn strings() {
    // U-STRING corpus (plan §3/§6): split/trim/multiply smoke case, promoted
    // from tests/lang/string/pending/ once split() crash, trim*() dispatch,
    // and codePointAt() stub were fixed; plus a bytes/codePoints sequence-view
    // golden. See docs/forge/units/U-STRING/plan.md.
    support::check_pass("strings");
}

#[test]
fn strings_negative() {
    // U-STRING ArgumentError guards (plan §6): split/replace empty-delimiter,
    // non-String charset/delimiter, indexOf empty needle, and *(count)
    // range/type guards — the corpus gap the U-STRING review flagged as
    // unreachable by the gate.
    support::check_negative("strings/negative");
}

// NB: no `string_negative` test — the two Wren-suite NEGATIVE ports
// (`string_concatenation_wrong_arg_type.ph`, `string_not_operator_unsupported.ph`)
// live in `tests/lang/runtime-errors/`, already fully exercised by
// `runtime_errors()` above (a second `check_negative("runtime-errors")` call
// would just re-run the same directory).

#[test]
#[ignore = "Modules v1 retires physical imports; runtime module loading is out of scope for Part I"]
fn imports_negative() {
    // Retained as historical U15 runtime coverage. Modules v1 is compile-time
    // only and intentionally rejects this physical-import corpus.
    support::check_negative("imports/negative");
}

#[test]
fn indexing() {
    // U-INDEX: postfix `[]` and `[]=` syntax sugar over `at(_)` / `at(_,put:)`.
    support::check_pass("indexing");
}

#[test]
fn indexing_negative() {
    // U-INDEX: negative indexing scenarios (OOB write raises, non-indexable doesNotUnderstand).
    support::check_negative("indexing/negative");
}

#[test]
fn decorators() {
    // Standalone core-library classes the ratified decorator specs need
    // (Tracer/OffBehavior/Backoff, decorators-behavioral.md/
    // decorators-dispatch-observability.md) — shipped ahead of the
    // Install/Dispatch/Runtime decorator mechanism itself (PLAN-DECORATORS.md).
    support::check_pass("decorators");
}

#[test]
fn ic() {
    // Dispatch-cache coherence: five tests verify that a version counter
    // correctly invalidates cached resolutions when the world is modified at
    // runtime. (1)-(4) cover U-IC's monomorphic method cache and its global
    // `world_version`; (5) covers F12's per-callsite *global-name* cache and its
    // per-module `globals_version`. Tests cover:
    // (1) override-after-caching — a missing version bump produces stale results;
    // (2) add-method-invalidates — adding a subclass override invalidates the
    //     parent method's cached entry;
    // (3) megamorphic-still-correct — one call site hit by 4+ receiver types
    //     thrashes the monomorphic slot but returns the right method each time;
    // (4) class-side-init-fallback — repeated resolution via the `init`
    //     selector fallback (value/mod.rs:171) maintains behavior parity;
    // (5) global-cache-shadow-invalidates — a site resolved through the core
    //     fallback stops seeing core's binding once this module declares the
    //     name, while assignment and re-declaration (which reuse the slot) do
    //     not disturb the cache. This is the case F12's unguarded prototype got
    //     wrong.
    support::check_pass("ic");
}

#[test]
fn streams() {
    support::check_pass("streams");
}

#[test]
fn streams_negative() {
    support::check_negative("streams/negative");
}

#[test]
fn path() {
    support::check_pass("path");
}

#[test]
fn path_negative() {
    support::check_negative("path/negative");
}
