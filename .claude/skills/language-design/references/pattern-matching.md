# Pattern Matching & Destructuring

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing match expressions, destructuring, exhaustiveness, guards, or binding forms.

## Contents
- What can be matched
- Bindings in patterns
- Guards
- Exhaustiveness & usefulness
- Match order & semantics
- Destructuring outside match
- Compilation
- Extensibility (open sums, dynamic/message langs)

## What can be matched
| Option | Langs | Consequence |
|---|---|---|
| Literals + wildcards | all | Base case; wildcard `_` binds nothing, always matches. |
| Constructors / ADT variants | Haskell, OCaml, Rust, Scala, Swift | Tag-test + payload destructure; the reason ADTs exist. |
| Tuples / records / structs | Rust, OCaml, Scala, Swift | Positional or field patterns; nested arbitrarily deep. |
| List cons / `[x, *rest]` | Haskell, Erlang, Elixir, Python 3.10 | Head/tail or split patterns; drives recursive processing. |
| Ranges | Rust (`1..=9`), Swift | Interval test as a pattern; needs ordered scrutinee type. |
| Type patterns (`is T`, type-case) | Scala, Swift, C#, TypeScript | Runtime type test + narrowing bind; leans on RTTI/downcast. |
| View / active patterns | F# active, Haskell view, Scala `unapply` | Match calls a user function → abstraction over representation. |

**Syntax.** Rust `Some(Point{x, y})` · OCaml `x :: rest` · Erlang `[H|T]` · Elixir `%{key: v}` · Python `case Point(x=0, y=y):` · Swift `case .some(let x)` · F# `(| Even | Odd |)`.
**Impl.** Constructor patterns compile to tag-load + branch + field-project; view patterns compile to a *call* then match on its result (opaque to exhaustiveness).
**Hazard — view/active pattern ⊗ exhaustiveness.** A match arm gated by an arbitrary function is opaque to the checker; it can't prove the set of `unapply`/active results is covered, so exhaustiveness silently degrades to "assume incomplete." → overlay

## Bindings in patterns
| Option | Langs | Consequence |
|---|---|---|
| Variable binding | all | Sub-term captured into scope; the payload of matching. |
| `@` / as-patterns | Rust (`n @ 1..=9`), Haskell (`all@(x:_)`) | Bind whole *and* destructure it; no re-reconstruct. |
| Or-patterns, shared binds | OCaml/Rust (`A(x)\|B(x)`), Python (`\|`) | One arm, many shapes — but every branch must bind the same vars. |
| Linearity (no double-bind) | Rust, Haskell, OCaml | Same var twice in one pattern = error, not equality test. |
| Non-linear = equality | Erlang, Prolog | Repeated var means "must be equal" — unification, not binding. |

**Syntax.** Rust `x @ Some(_)` · Haskell `s@(x:xs)` · OCaml `(A x \| B x)` · Erlang `{X, X}` (equality) · Python `case [x] \| [x, _]:`.
**Impl.** Linear binder = one fresh slot per name; Erlang non-linear = bind-then-guard-`=:=`; or-pattern = compile each alt to the *same* result binding set.
**Hazard — or-patterns ⊗ binding consistency (CROWN JEWEL).** Every branch of `A(x) | B(x)` must bind identical variable names at identical types, or a later reference reads an unbound/ill-typed slot. Unsound unless the checker rejects mismatched branch bindings at compile time. → overlay
**Hazard — non-linear binding ⊗ language mode.** `{X, X}` means *equality* in Erlang but *double-bind error* in Rust; picking one silently reinterprets every repeated-variable pattern. → overlay

## Guards
| Option | Langs | Consequence |
|---|---|---|
| Boolean guard (`if`/`when`) | Rust, Swift, Scala, Haskell, Erlang | Extra predicate on an arm; runs after structural match. |
| Guard vocabulary restricted | Erlang (guard BIFs only) | Guards can't call arbitrary/side-effecting code → stay analyzable. |
| Pattern guard (bind + match in guard) | Haskell (`\| Just y <- f x`) | Guard itself destructures; chains dependent matches. |
| No guards | Python match (structural + `if`? yes) / early ML | Fewer arms; conditions pushed into arm bodies. |

**Syntax.** Rust `Some(n) if n > 0 =>` · Swift `case let x where x > 0:` · Haskell `\| x > 0 =` · Erlang `when X > 0 ->` · Haskell pattern-guard `\| Just y <- lookup k m =`.
**Impl.** Guard = branch appended *after* the arm's structural test; on guard-false, fall through to the next arm (must re-test, defeats simple decision-tree sharing).
**Hazard — guards ⊗ exhaustiveness (CROWN JEWEL).** A guard makes an arm's coverage undecidable — the checker cannot prove `if n > 0` plus `if n <= 0` is total, so it conservatively treats any guarded arm as *not* covering its pattern. Guards convert a checkable match into a "possibly non-exhaustive" one. → overlay
**Hazard — guard ⊗ side effects.** If guards may call effectful code, arm order + fall-through means an effect can fire on a *non-selected* arm; Erlang forbids this by restricting guard BIFs. → overlay

## Exhaustiveness & usefulness
| Option | Langs | Consequence |
|---|---|---|
| Compiler-enforced total | OCaml, Rust, Haskell `-Wincomplete` | Missing case = warning/error; refactors caught at compile. |
| Usefulness / dead-arm warning | OCaml, Rust | Unreachable arm flagged; redundancy is a bug signal. |
| Unchecked / runtime error | Python `match`, Ruby `case/in` | No case matched → fall-through or `NoMatchingPatternError`. |
| No check at all | JS (switch), Lua | Silent fall-through; forgotten case = wrong result, no signal. |

**Syntax.** Rust: non-exhaustive `match` = hard error · Haskell `{-# OPTIONS -Wincomplete-patterns #-}` · Python raises at runtime if no `case` and no `case _`.
**Impl.** Exhaustiveness = Maranget's usefulness algorithm: a match is total iff the wildcard row is *not* useful against the pattern matrix; same engine flags redundant (non-useful) arms ([recipes.md#match-compile](recipes.md#match-compile)).
**Hazard — open sums ⊗ exhaustiveness (CROWN JEWEL).** Exhaustiveness is only meaningful over a *closed* set of variants. On an open/extensible sum, adding a variant elsewhere silently makes every existing match non-exhaustive with no local edit — *sealedness* is the property that makes the check sound. → overlay
**Hazard — unchecked match ⊗ evolution.** Without an exhaustiveness check (Python/JS), adding a variant compiles clean and fails only at runtime on the input that hits the missing arm. → overlay

## Match order & semantics
| Option | Langs | Consequence |
|---|---|---|
| First-match, top-to-bottom | Rust, OCaml, Haskell, Swift, Python | Order is semantics; a broad arm shadows later specific ones. |
| Best/most-specific match | CLOS multimethods, some type-case | Order-independent, but needs a specificity lattice + tie rules. |
| Refutable vs irrefutable split | Rust (`let` vs `if let`), Haskell | Irrefutable (single-shape) binds always; refutable may fail. |
| Sequential + fall-through | C/JS `switch` | No auto-break → accidental fall-through is the classic footgun. |

**Syntax.** Rust `match` (first-match) · irrefutable `let (a, b) = pair;` · refutable `if let Some(x) = o` · Haskell lazy irrefutable `~(a, b)`.
**Impl.** First-match = ordered linear/decision-tree test; a wildcard or bare-var arm above a specific arm makes the lower arm *unreachable* → usefulness pass warns.
**Hazard — first-match ⊗ overlapping patterns.** With top-to-bottom order, a more-general arm placed first silently swallows every later specialized arm; only a usefulness/redundancy pass surfaces the dead code. → overlay
**Hazard — match ⊗ side-effecting/lazy scrutinee.** Matching *evaluates* the scrutinee (and forces lazy thunks / runs getters) to inspect its shape; a match on `f()` or a lazy value has observable timing/effect the reader may not expect. → overlay

## Destructuring outside match
| Option | Langs | Consequence |
|---|---|---|
| Irrefutable `let` binding | Rust, Swift, OCaml, JS | `let (a, b) = …` must not fail → only single-shape patterns allowed. |
| Function-parameter patterns | Haskell, OCaml, Rust, JS | Destructure in the arg list; clean but each param must be irrefutable. |
| Loop destructuring | Python, Rust, JS (`for [k, v] of`) | `for (k, v) in m` binds per iteration; refutable element = runtime risk. |
| List/rest `[first, *rest]` | JS, Python, Rust slice-pat | Split head/tail outside match; rest collects remainder. |
| Object / record destructuring | JS, Elixir, Rust struct-pat | Bind by field name; JS defaults + rename (`{a: x = 0}`) add call-shape. |

**Syntax.** JS `const {a, b = 1, ...rest} = o` · Python `first, *rest = xs` · Rust `let Point { x, .. } = p` · Haskell `f (x:xs) =` · Swift `for (k, v) in dict`.
**Impl.** Irrefutable destructure = direct field/index projection, no branch; a refutable pattern in `let` is either rejected at compile time or lowered to a fail-on-mismatch trap.
**Hazard — destructuring ⊗ refutability (CROWN JEWEL).** An irrefutable `let`/param binding a *refutable* pattern (`let Some(x) = opt`) can fail at runtime. The language must either reject it at compile time (Rust) or panic — ties directly to the absence/Option story: how you destructure a nilable value is how you force its presence. → overlay

## Compilation
| Option | Langs | Consequence |
|---|---|---|
| Naive backtracking automaton | early ML, many teaching impls | Simple; re-tests columns → exponential redundant tests, code blowup. |
| Decision tree (Maranget) | OCaml, Rust, GHC | Each constructor tested once per path; may duplicate arm bodies. |
| Column-selection heuristics | OCaml, Rust | Which column to switch on first controls tree size (NP-hard optimal). |
| Backtracking + sharing (DAG) | GHC (join points) | Bounds code size vs tree; join points share arm continuations. |

**Syntax.** — (compilation, not surface).
**Impl.** Cluster arms into a pattern matrix; recursively pick a column, `switch` on its head constructor, specialize/default the matrix; leaves = arm bodies. Heuristics (first-row, small-branching, needed-column) tune tree size ([recipes.md#match-compile](recipes.md#match-compile)).
**Hazard — decision tree ⊗ code-size blowup.** Sharing constructor tests can duplicate arm *bodies* across leaves; a wide nested match compiles to exponential code unless you emit join points / share continuations. → overlay
**Hazard — guards ⊗ decision-tree sharing.** A failing guard must fall through to the *next textual arm*, which the decision tree may have already specialized away — guards force backtracking edges that break the "test each constructor once" invariant. → overlay

## Extensibility (open sums, dynamic/message langs)
| Option | Langs | Consequence |
|---|---|---|
| Sealed / closed sum | Rust `enum`, Kotlin `sealed`, Scala `sealed` | Variant set fixed → exhaustiveness sound, but adding a case edits the type. |
| Open sum / extensible variant | OCaml poly-variants, Haskell open unions | New variants anywhere; matches must carry a catch-all, no totality. |
| Visitor / double dispatch | Java, Smalltalk, C++ | "Match" = method per receiver type; easy to add types, hard to add ops. |
| `is`-test + downcast | Swift, C#, TS, dynamic langs | Type-case chain; no compiler totality, order-sensitive like match arms. |
| Library `caseOf:` / `matches:` | Smalltalk, Ruby `===`, Scala extractors | Matching as ordinary message-sends; flexible, uncheckable, uncached. |

**Syntax.** Smalltalk double-dispatch `node acceptVisitor: self` · Ruby `case x; when Integer` (uses `===`) · Scala `case class` + `unapply` · Swift `if case .foo = x`.
**Impl.** Sealed sum → dense integer tag + jump table (cacheable). Message-oriented "match" = a chain of `isKindOf:`/`respond_to?` sends or a visitor's virtual dispatch — no tag table, no exhaustiveness, no inline-cache win over ordinary sends.
**Hazard — visitor ⊗ expression problem.** Double dispatch makes adding a *type* cheap and adding an *operation* expensive; real pattern matching inverts that trade. Choosing one locks which axis of growth stays open. → overlay
**Hazard — adding `match` to a message language ⊗ open classes.** A dynamic, open-class Smalltalk-style language has no closed variant set, so a real `match` gets *no* exhaustiveness guarantee and its tag-dispatch is unsound the moment a class is added at runtime; it degrades to sugar over `isKindOf:` chains — buying ergonomics, not the safety that motivates `match`. → overlay
