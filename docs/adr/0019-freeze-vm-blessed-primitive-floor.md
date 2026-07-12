# 19. Freeze the VM-blessed primitive floor

- Status: Accepted
- Date: 2026-07-11
- Amended by: [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
  (2026-07-12) — admits `hash`, `Behavior#name`/`methods`, `Method` reflection,
  `Number#toString`, and `Error#message`/`raise` to the floor (73 → 88 ceiling,
  applied per-unit as U-CORE-1/3/4/6 land). This ADR's floor list and rationale
  otherwise stand unamended.
- Related: [ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md); [ADR-0005](0005-number-as-flat-f64.md); [ADR-0006](0006-function-as-abstract-callable-root.md); [ADR-0007](0007-option-as-abstract-with-some-none.md); [ADR-0009](0009-handle-arena-heap.md); [ADR-0010](0010-tagged-value-enum.md); `docs/spec/v0.2/experimental/bootstrapping-and-self-hosting.md` (D1)

## Context

Phalcom is a hybrid native-runtime language: a native Rust VM plus a bootstrap
core library (`phalcom-core/core/core.ph`) authored in Phalcom itself. Today the
dogfooded surface is tiny — `core.ph` is 37 lines (10 reopened class shells and
one method body), while ~80 primitive methods across 11 `primitive/*.rs` modules
carry the actual behavior. The stated direction (overlay §Core-vs-std) is to grow
`core.ph` into a real standard library, and possibly (see the experimental
bootstrapping note, Rung C) to self-host the front-end.

Both directions run straight into the **native-vs-library boundary-creep hazard**
([bootstrapping.md](../../.claude/skills/language-design/references/bootstrapping.md)):
every primitive pulled into the VM "for speed" shrinks the dogfoodable surface and
freezes semantics the language could otherwise redefine. The boundary is a
**one-way ratchet** — it only ever moves toward more native code — unless it is
fixed by decision. Without a written floor, each performance-motivated move of a
method into Rust is locally reasonable and globally corrosive: the "language
implemented in itself" surface erodes one primitive at a time, and no single
commit is ever the one to blame.

There is a second, correctness-shaped reason to draw the line now. The kernel load
order is a topological sort over a dependency DAG (ADR-0002/0003; experimental
bootstrapping note D3). "Which classes are VM-blessed" *is* the boundary of that
DAG: blessed classes are built by native `create_core_classes`/`install_primitives`
before any `core.ph` source loads; everything above the line is ordinary Phalcom
that may only send to things already installed. An unwritten boundary means an
unwritten DAG edge, which is exactly the shape that produces a hard boot failure
with no user frame to blame.

## Decision

Fix a **permanent VM-blessed floor**. Everything at or below the floor stays
native Rust indefinitely; everything above it is authored in `core.ph` (or, under
Rung C, in a self-hosted compiler) and may be redefined, introspected, and
dogfooded like any user class.

**The floor (native forever):**

1. **Allocation & object graph** — instance allocation (`new`), field get/set
   (`GetField`/`SetField`), class/metaclass wiring, the cyclic apex
   (ADR-0002/0003). Cannot be expressed in-language without already having the
   object graph.
2. **Dispatch** — `Invoke`, `perform`, and the `doesNotUnderstand(_)` reification
   path (ADR-0012). Message send is the substrate every `.ph` method compiles to.
3. **Absence & Bool roots** — `Some`/`None` construction and the `match(some:,
   none:)` eliminator (ADR-0007); `True`/`False` and the sacred `ifTrue`/`and`/`or`
   primitives (ADR-0004). Already VM-blessed; the absence bootstrap cycle
   (Invariant 4) and truthiness ban (Invariant 6) forbid moving them up.
4. **`Number` arithmetic** — the native numeric operations: exact `Int`
   (`checked_*` + bignum boxing) and `Float` (`f64`) ops ([ADR-0024](0024-numeric-surface-split-int-float-and-division.md),
   amending ADR-0005). Cannot be defined without recursing into themselves.
5. **`Block` `call`** — the thunk primitive that all control flow (blocks-as-sends)
   is built on (ADR-0006, ADR-0018).
6. **`System` I/O** — `print` and source-file read. The compiler and REPL need a
   native I/O primitive to reach the outside world.

**Above the floor (fair game for `.ph`):** `List`/`Map`/`Set` and the iteration
protocol, higher-order `String` manipulation (split/join/format beyond raw byte
access), Option/Result combinators (`map`/`flatMap`/`orElse`/…), `Message.args`,
rest-parameter collection, reflection helpers, and — under Rung C — the lexer,
parser, and compiler themselves.

A method may only move **downward** across the floor (into the VM) via a new
superseding ADR that amends this list. Moving a method up (native → `.ph`) is
always allowed and needs no ADR — the ratchet is deliberately broken in the
dogfooding direction only.

## Consequences

- The "implemented in itself" surface has a **defined lower bound** that cannot
  silently erode. Reviews can reject a "move it into Rust for speed" change by
  citing this ADR; the counter-move (fund an inline cache or JIT *above* the floor)
  is named as the intended answer to hot-path cost.
- The floor **is** the kernel-load-DAG boundary: `create_core_classes` +
  `install_primitives` build exactly the floor before `core.ph` loads, and
  `verify_invariants()` gates the finished graph once (D3). The boundary is now a
  written invariant a test can assert against, not folklore.
- This is a **performance commitment**, not just an organizational one. Accepting
  that `List`, iteration, and (eventually) the compiler run as ordinary dispatched
  `.ph` means accepting slower hot paths until inline-cache population (deferred,
  ADR-0012) or a JIT lands. That trade is chosen deliberately: a smaller native
  surface is more auditable and keeps the object model uniform.
- It does **not** freeze representation choices that live *below* the floor behind
  their existing `Value`/`Number` APIs: the Int/Float surface split is now decided
  ([ADR-0024](0024-numeric-surface-split-int-float-and-division.md)) and NaN-boxing
  (deferred, ADR-0010) remains open — both internal to the
  blessed `Number` primitive, not boundary moves.
- It leaves `String` deliberately split: raw byte/codepoint access is a floor
  primitive; higher-order manipulation is `.ph`. The exact cut line for `String`
  is the one genuinely fuzzy edge and is called out as the first place a future
  amendment is likely.

## Alternatives considered

- **No written floor (status quo).** Let the boundary float and decide per method.
  Rejected: this is the ratchet itself — every local decision is defensible and the
  aggregate outcome is a native language with a vestigial `core.ph`, the opposite of
  the stated goal.
- **Maximal floor (keep collections/strings native, CPython/Lua style).** Fast cold
  start and fast hot paths, but the stdlib can't be reshaped or introspected as
  objects and the self-hosting surface (Rung C) shrinks to nothing. Rejected for a
  Smalltalk-modeled language whose whole premise is a dogfooded, reflective library.
- **Minimal floor (push arithmetic/Bool into `.ph` too).** Maximally dogfooded, but
  re-hits the absence/arithmetic bootstrap cycles (a `Number` method that adds needs
  addition; a `Bool` method that branches needs branching) and would force a
  metacircular interpreter substrate (Rung D), which this project explicitly does not
  pursue. Rejected.
