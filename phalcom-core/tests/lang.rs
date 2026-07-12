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
#[ignore = "spec target: lexical"]
fn lexical_pending() {
    support::check_pending("lexical");
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
#[ignore = "spec target: bindings"]
fn bindings_pending() {
    support::check_pending("bindings");
}

#[test]
fn messages() {
    support::check_pass("messages");
}

#[test]
#[ignore = "spec target: messages"]
fn messages_pending() {
    support::check_pending("messages");
}

#[test]
fn dispatch() {
    support::check_pass("dispatch");
}

#[test]
#[ignore = "spec target: dispatch"]
fn dispatch_pending() {
    support::check_pending("dispatch");
}

#[test]
fn classes() {
    support::check_pass("classes");
}

#[test]
#[ignore = "spec target: classes"]
fn classes_pending() {
    support::check_pending("classes");
}

#[test]
fn control_flow() {
    support::check_pass("control-flow");
}

#[test]
#[ignore = "spec target: control-flow"]
fn control_flow_pending() {
    support::check_pending("control-flow");
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
    // U6: absence is `Option` (`Some`/`None`) — the shared `None` singleton,
    // `Some.new(_)` construction, and the `match(some:none:)` eliminator.
    support::check_pass("absence");
}

#[test]
#[ignore = "spec target: absence — prettier printString + Some(x) sugar are U-STD"]
fn absence_pending() {
    support::check_pending("absence");
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
#[ignore = "PENDING: metaclass tower — U2"]
fn metaclass_pending() {
    support::check_pending("metaclass");
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
#[ignore = "spec target: blocks"]
fn blocks_pending() {
    support::check_pending("blocks");
}

#[test]
fn functions() {
    // U-CORE-3: `Object#methodFor(_)` / `Method#invokeOn(_,_)` / `Method#bind(_)`
    // / `Method#selector` / `Method#holder` — the Method reflection surface,
    // exercised over already-supported syntax (`Symbol.new(_)`,
    // `List.new().add(_)`). The remaining pending fixtures below are gated on
    // U-LEX's `#...`/`[...]`/`::` literals.
    support::check_pass("functions");
}

#[test]
#[ignore = "PENDING: functions — U-LEX selector/list/family literals"]
fn functions_pending() {
    support::check_pending("functions");
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
#[ignore = "spec target: concurrency — Future/async/await"]
fn concurrency_pending() {
    support::check_pending("concurrency");
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
    // U-COLL: the map literal (`{k: v}`) and a spread element (`[*xs]`) are
    // *recognised* by the parser but their runtime lowering is deferred to the
    // collection-runtime unit (U-COLLTYPES; DEC-COLL-B), so each raises a
    // precise "pending" diagnostic instead of silently mis-parsing.
    support::check_negative("collections/negative");
}

#[test]
#[ignore = "spec target: native Tuple/Map arms land in U-COLLTYPES (DEC-COLL-A/B)"]
fn collections_literals_pending() {
    // U-COLL: the tuple and map literals desugar in the parser here, but their
    // native heap arms (`Tuple`, `Map`) are built by U-COLLTYPES. These
    // fixtures pin the intended runtime (a `Tuple` distinct from `List`;
    // symbol-keyed map lookup) and graduate to PASS when U-COLLTYPES lands.
    support::check_pending("collections");
}

#[test]
fn variadics() {
    // U9: rest parameters (`*name`) — declaration, `<name>(*)` selector
    // encoding, the VM call-prologue rest-arg collapse, and the
    // derived-selector miss-path probe.
    support::check_pass("variadics");
}

#[test]
fn inheritance() {
    // U-INH: single inheritance — `class B extends A`, inherited instance
    // methods with subclass overrides, the parallel-metaclass rule making
    // `static` members inherit (ADR-0002 rule 4), `super.sel(…)` sends via the
    // `SuperSend` opcode (method-lookup.md §1.14), and super-construct chaining.
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
}

#[test]
#[ignore = "spec target: non-local break/continue across a materialized block — U-ITER follow-on"]
fn iteration_pending() {
    // Reviewer-found gap (ADR-0035 §3, iteration.md §3): `break`/`continue`
    // reached through a block the inliner materializes as a real closure — the
    // deopt fallback of a non-Bool `if` condition, or an ordinary block-arg
    // closure like `each { break }` — silently no-ops instead of leaving the
    // loop (the jump lands in the closure's own chunk, `same_function` false).
    // The common `if (Bool) { break }` path is unaffected (inliner fast path).
    // These fixtures pin the intended semantics and fail today; see
    // docs/forge/DEFERRED.md for the adjudication and the real-fix options.
    support::check_pending("iteration");
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
fn imports() {
    // U15 (DEC-U15 A+A, object-model.md §4): `import "path" as Name` —
    // relative file-path resolution + whole-module binding. Basic member
    // access as an ordinary send, one-object-per-canonical-path
    // memoization (module *and* class identity), no global-namespace
    // pollution, kernel visibility without import, and cyclic-import
    // termination (`lib/` holds the imported units — never a standalone
    // test case in its own right).
    support::check_pass("imports");
}

#[test]
fn family() {
    // U16-Open (selectors.md §3, ADR-0047): `::` method references, Open
    // form — `obj::name`/`Type::name` produce a callable `Family`
    // whose call builds the selector from the base name + the call site's
    // labels and performs an ordinary send; inheritance-flattened base-name
    // index; a `doesNotUnderstand` override keeps an otherwise-empty family
    // callable.
    //
    // U16-Pinned (selectors.md §3, ADR-0047): the Pinned arm —
    // `obj::#move(_,to,duration)`/`Type::#square()` — pins the full
    // selector identity at the reference site; a call ignores the call
    // site's own labels (only its argument count is checked) and dispatches
    // the exact pinned selector. Includes the exact-overload proof: a class
    // defining both a labeled and a positional `move` overload, each pinned
    // and dispatched independently.
    support::check_pass("family");
}

#[test]
fn family_negative() {
    // U16-Open: `obj::typo` on a class with no `typo` method and no
    // `doesNotUnderstand` override errors at `::` reference time, naming
    // the class (selectors.md §3 error table).
    //
    // U16-Pinned: the same empty-family check against the exact pinned
    // selector (not just the base name); a Pinned call's argument-count
    // mismatch against the pinned selector's arity; and the LOCKED parser
    // ambiguity rule rejecting a bare name symbol after `::` (`::#name`,
    // no parens) with a clear diagnostic.
    support::check_negative("family/negative");
}

#[test]
fn imports_negative() {
    // U15: a missing import target and the documented cyclic-import
    // partial-init hazard (a name read across the not-yet-complete edge of
    // a mutual import) both fail cleanly — never a hang, never a panic.
    support::check_negative("imports/negative");
}
