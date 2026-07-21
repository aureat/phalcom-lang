# Compiler-directive decorators — what the compiler may be told

- Status: **Evaluation file; verdicts, mostly rejections.** The question it
  answers: which decorators exist to *communicate with the compiler* — hints,
  directives, suppressions, optimizations — and which of those Phalcom
  admits.
- The governing positions: ADR-0051 (measure-first; no optimization without a
  benchmark, a profile, and a recorded number; behavior-invariant), ADR-0019
  (floor admission: inexpressibility only, *speed is explicitly never
  sufficient*), ADR-0018 (the inliner's soundness comes from guards, not
  promises).

## The dividing line: facts, not hints

A decorator may tell the compiler a **semantic fact** it can verify or that
changes what programs are legal. It may not deliver an **unverifiable
performance promise**. Every admitted "compiler-communication" decorator on
HEAD is already on the right side of this line:

| Decorator | What it tells the compiler | Fact or hint? |
|---|---|---|
| `@sealed` | the subclass set is closed | fact (enforced at subclass site) |
| `@native` | a Rust binding backs this member; drop the body | fact (machine-checked by the invariant test) |
| `@ignore` | this member is not code | fact (definitionally) |
| `@class` / `@constructor` (ADR-0063) | placement / kind | fact |
| `@override` (proposed) | an inherited selector exists | fact (checked at class-definition) |

## Performance hints — `@inline`, `@noinline`, `@hot`, `@cold`, `@unroll` — rejected

Four independent grounds, any one sufficient:

1. **Unverifiable claim.** The compiler cannot check "this is hot"; a wrong
   hint is a lie it must either trust (mis-optimizing) or ignore (dead
   surface). Precedent with consequence: HotSpot ignores source-level
   inlining wishes entirely — profile data beat programmer belief so
   thoroughly that Java never grew the annotation; C's `inline` decayed into
   a linkage keyword; Rust's `#[inline]` is meaningful *only because* the
   compiler is AOT and static — neither condition holds here.
2. **Wrong machine.** Phalcom is a bytecode interpreter with late-bound
   sends. Inlining a *user* method is unsound without per-site guards and a
   deopt path — the entire apparatus ADR-0018 builds for six sacred
   selectors, each justified by measurement. A decorator cannot conjure the
   guard; it can only request unsoundness politely.
3. **ADR-0051 discipline.** No optimization without a reproducible number.
   A hint decorator is an optimization with *someone else's* number —
   the author's guess, frozen into source, unfalsifiable by the benchmark
   suite.
4. **ADR-0019's counter-move.** The named answer to hot-path cost is "fund
   an inline cache or JIT above the floor" — infrastructure that measures,
   not annotations that assert. When an IC lands, it needs zero source
   cooperation.

Rejected as a family. A future JIT with profile feedback still needs no
hints; if one ever genuinely does (whole-method `@compile` pragmas à la
Graal), that is a superseding PDR with numbers attached.

## Diagnostic control — `@allow`/`@deny`/`@suppress(lint)` — deferred with a trigger

The legitimate directive family every language grows. Phalcom has **no
warning tier** — every diagnostic is an error, so there is nothing to
suppress (I-1 recorded the same for a hypothetical `@ignore` warning).
Deferred until a warn tier exists; when it does, the design constraints are
known in advance and recorded here so the future design starts constrained:
scope must be member-granular (matching attribute grammar), suppressions must
name the specific lint code (`@allow(unused_binding)`, never blanket), and a
suppression of a nonexistent code is itself an error (Rust's
`#[allow(nonexistent)]` warns for exactly this reason — silent typo'd
suppressions suppress nothing and rot).

## Conditional compilation — `@cfg`-style — rejected for v0.2

`@ignore` is unconditional by ruling (its own preclusion list). A
`@when(debug)` conditional member would introduce a build-mode axis into the
member list — the one precedent Phalcom has (`CompileMode` stripping contract
*guards*) is deliberately narrow and semantics-preserving. Whole-member
conditional presence is neither. No current need; rejected until one exists.

## `@intrinsic` / binding directives — rejected

"Install *this* Rust fn for *this* selector" from source. native.md already
precludes it (the anchor-vs-directive readings are incompatible; the drop is
committed), and bootstrap ownership of the binding table is what keeps the
floor auditable (the census counts installs in one place). A source-side
binding directive would scatter the floor across the corpus.

## What this precludes

- Any lowercase builtin whose semantics are "trust me, it's faster."
- Growing `@ignore` into conditional compilation or lint suppression — three
  adjacent temptations, three separate rejections, one shared rationale:
  each is a *different* directive with its own design space, and overloading
  the drop mechanism forecloses designing them properly.
