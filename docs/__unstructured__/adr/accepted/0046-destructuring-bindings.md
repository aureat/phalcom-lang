# 46. Destructuring `let`/`var` bindings — irrefutable tuple + list, `at(_)` protocol

- Status: Accepted
- Date: 2026-07-13
- Amends (ruling scope only): `docs/spec/current/open-questions.md` Q7's original
  ruling, which shipped irrefutable **tuple** destructuring now and deferred
  **list**/`*rest` destructuring to a future refutable `match`/`if let` unit. This
  ADR ships **both** now, kept irrefutable (see "Why list destructuring ships
  irrefutable now" below) — the deferred item narrows to *further* pattern forms
  (map patterns, match arms), not list/`*rest` itself.
- Related: `docs/spec/current/open-questions.md` Q7; `docs/adr/accepted/0014-let-and-var-bindings.md`
  (the base `let`/`var` binding this extends); `docs/adr/accepted/0020-kernel-list-native-array-protocol.md`
  (`at(_)` — the accessor this reuses); `docs/adr/accepted/0032-collections-representation-and-literals.md` (the `(a,b)`/
  `[…]` collection-literal grammar this reuses in pattern position);
  `docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md` (native
  `Tuple`/`List`); `../../forge/units/U14/u14.md`; `phalcom-ast/src/ast.rs::Pattern`;
  `phalcom-ast/src/parser.rs::Parser::parse_pattern`;
  `phalcom-core/src/compiler/lib.rs::Compiler::compile_pattern_bind_top_of_stack`

## Context

`let`/`var` (ADR-0014) bound a single bare name. Open-question Q7 ruled (no ADR at
the time) that Phalcom should ship **irrefutable tuple destructuring** —
`let (a, b) = point` — immediately, since a `Tuple`'s arity is part of its type (an
`(a, b)` pattern against a `Tuple` can only ever be a *shape* mismatch, never a
*value*-dependent branch), and explicitly **deferred** list/`*rest` destructuring
(`let [first, *rest] = list`) to a later pattern-matching unit, reasoning that a
list's length is a runtime property, so a length mismatch is *refutable* — it
should fail into a `match`/`if let` arm, not raise.

Dispatching the implementation unit (U14) revisited that split. A `List`'s
`at(_)` is already **total** (ADR-0020: an out-of-range read returns the `None`
singleton, never raises or panics) — so an *irrefutable* list destructuring
bind is no harder to build correctly than the tuple case: the compiler emits the
same inline arity guard either way (`size` compared against the pattern's
expected shape), and the guard raises a clean `Error` on mismatch precisely
because `at(_)` itself would otherwise silently hand back `None` for a
too-short list, masking the mismatch instead of surfacing it. There is no
`match`/`if let` construct in the language yet (that is still future work, see
"Forward-looking" below) to receive a *refutable* failure branch even if we
wanted one, so gating list destructuring behind that future unit would have
meant either (a) blocking a small, mechanically-similar feature on a much larger
one, or (b) shipping a half-irrefutable/half-refutable model with no visible
difference in the interim. This ADR resolves that by shipping **both** tuple and
list-with-`*rest` destructuring now, **both irrefutable**, sharing one lowering.
The deferred item is narrowed to genuinely new pattern *forms* (map patterns,
a real `match` with fallthrough/guard arms) — not list/`*rest` itself.

## Decision

### 1. One accessor protocol: `at(_)`

A destructuring pattern desugars to a **single** evaluation of the initializer
into a synthetic scratch local, followed by positional element reads through
the *same* `at(_)` selector `List` and `Tuple` already expose (ADR-0020) — there
is **no** parallel `_0`/`_1` field-accessor protocol, and no iterator-based
spread. Concretely:

```text
let (a, b) = point
```
lowers to (informally, in surface syntax — the compiler emits the equivalent
bytecode directly, without going through a second parse):
```text
let $t = point
let a  = $t.at(0)
let b  = $t.at(1)
```

and
```text
let [first, *rest] = list
```
lowers to:
```text
let $t     = list
let first  = $t.at(0)
let rest   = <a fresh List holding $t's elements from index 1 onward>
```

The rest tail has no dedicated slice/tail selector on `List` — one was not
worth adding to the frozen floor (ADR-0019) for this alone — so the compiler
realizes it as an inlined `while` copy loop over the existing `List.new()`/
`add(_)`/`size`/`at(_)` sends (mirroring `Compiler::compile_for`'s own
hand-rolled loop skeleton, U-ITER specification §3.1), not a synthesized
Phalcom-source string. This keeps the U9 "`*rest`" spelling load-bearing in
exactly one place: the parser's rest-detection, reused identically from
[`ParameterDef::is_rest`]'s call-parameter grammar (messages-and-selectors.md
§5 spread parity) into `Pattern::List::rest`.

Patterns nest recursively (`let ((a, b), c) = …`): a nested `Pattern::Tuple`/
`Pattern::List` sub-pattern claims its own `.at(i)` read into a fresh scratch
local and recurses through the identical lowering — no separate machinery.

### 2. Irrefutable, with an inline arity guard

Both forms are **irrefutable**: there is no partial-match fallback, no boolean
test, and no failure branch. A shape mismatch — wrong `Tuple` arity, wrong
rest-less `List` length, or a `List` shorter than a `*rest` pattern's fixed
prefix — raises a clean `Error` at runtime (via `Error.new(message).raise()`,
the same idiom `Compiler::emit_deopt_block_control_trap` uses for its own
compiler-synthesized raise) rather than panicking, silently truncating, or
handing back a `None`-padded partial bind. The compiler emits this check
inline, once per pattern level, immediately before reading that level's
elements:

- **`Tuple` pattern** (`(p1, …, pn)`): requires `scrutinee.size == n` exactly.
- **`List` pattern with no `*rest`** (`[p1, …, pn]`): requires
  `scrutinee.size == n` exactly too (not "at least" — a rest-less list pattern
  drops nothing silently, matching the tuple case's exactness).
- **`List` pattern with `*rest`** (`[p1, …, pn, *rest]`): requires
  `scrutinee.size >= n`; the rest sub-pattern binds whatever remains (possibly
  empty).

A scrutinee that isn't a `Tuple`/`List` at all (has no `size`/`at(_)`) is not
specially detected — it falls through to the natural `doesNotUnderstand` miss
on the `size` send, which is already a clean, precise runtime error and needs
no bespoke type check.

A destructuring pattern — `Pattern::Tuple`/`Pattern::List`, at any nesting
depth — **always requires an initializer**, for both `let` and `var`: there is
nothing to unpack from an absent value, unlike a bare-name `var x` (still
legal, still reads the surface `None`, ADR-0007/ADR-0014 unchanged for that
one case).

### 3. `var` vs `let`

Unchanged from ADR-0014: `let` produces immutable leaf bindings (reassignment
is a compile error); `var` produces mutable ones. The distinction is threaded
through every leaf of the pattern, including nested and rest sub-patterns — a
destructuring `var (a, b) = …` allows `a = a + 1` afterward exactly as a plain
`var a = …` would.

### 4. Reserved for a future `match`

The `Pattern` AST node (`phalcom-ast/src/ast.rs`) is written to be reused by a
future refutable `match`/`if let`: the "raise on mismatch" behavior lives
entirely in the compiler's *lowering* (`Compiler::emit_pattern_arity_check`),
not in the node's shape, so a later refutable evaluator can walk the same
`Pattern::Name`/`Pattern::Tuple`/`Pattern::List` tree and produce a
success/failure result instead of an unconditional raise, without reshaping
this node. Map patterns and match-arm syntax remain explicitly future work.

## Consequences

- **No new absence semantics.** A destructured slot that legitimately reads
  `None` (e.g. `let (a, b) = (1, None)`) is just `None` — destructuring
  introduces no new sentinel (ADR-0007 unchanged).
- **No frozen-floor change.** The whole feature is a compiler desugar over
  already-primitive sends (`at(_)`, `size`, `new`, `add(_)`, `+`/`<`/`!=`,
  `raise`) — zero new native primitives, so ADR-0019's floor is untouched
  (floor delta 0).
- **Fiber-local, no shared state.** The scratch temp(s) live on the frame's
  own operand stack as ordinary compiler locals — never a global — so a
  destructuring bind inside a fiber body introduces no cross-fiber state
  (concurrency.md's isolation guarantee is preserved for free).
- **Grammar stays unambiguous.** `(a, b)`/`[…]` in binding-target position
  (parsed by `Parser::parse_pattern`) is a syntactically distinct path from
  the same delimiters in expression position (`Parser::parse_paren_or_tuple`/
  `Parser::parse_list_literal`, U-COLL) — reached only after `let`/`var`, so
  the two never collide.
- **A rest-less `List` pattern is stricter than one might expect** —
  `[a, b]` against a 3-element `List` raises, it does not silently take the
  first two. This mirrors the `Tuple` pattern's exactness and was a deliberate
  choice (§2) over a more permissive "at most" reading, so an accidental
  length mismatch is always visible.

## Alternatives considered

- **Defer list/`*rest` destructuring to the future `match` unit** (the
  original Q7 ruling). Rejected for this implementation pass: `at(_)`'s
  existing totality means the *irrefutable* form costs nothing extra to build
  correctly now, and there is no `match` construct yet to host a genuinely
  refutable branch — deferring would have bought no design cleanliness, only
  a longer wait for a small, useful feature.
- **A dedicated `_0`/`_1` field-accessor protocol**, distinct from `at(_)`.
  Rejected: it would be a second indexed-read path alongside `at(_)` for no
  behavioral gain, splitting the collection-access surface for no reason.
- **An iterator-based spread** (destructure any `Iterable`, not just
  concrete `Tuple`/`List`). Noted as a strictly larger surface (the U14 plan's
  soft flag) and left for a future unit if `for`'s `iterate(_)`/
  `iteratorValue(_)` protocol turns out to need the same treatment.
- **A dedicated `List` tail/slice selector** (`sliceFrom(_)` or similar) to
  build the `*rest` tail directly, instead of an inlined copy loop. Rejected
  for this unit: it would be a new floor primitive (or a new `.ph` method
  outside this unit's write-set) for a single call site; the inlined loop
  costs a little bytecode-emission code but adds nothing to the floor. Left as
  a DEFERRED optimization candidate if `*rest` construction ever needs to be
  fast-pathed.
