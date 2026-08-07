# Spec C — Indexed Access, Range, and Slicing

This implementation unit realizes Phalcom's ratified strict/safe lookup semantics for finite indexed sequences, fixes subscript-assignment expression semantics, replaces the obsolete Range model with the new bound/inclusion model, and makes Range values usable as copy-slice descriptors for List, Tuple, and Bytes.

The semantic authority is the supplied `collections-next` specification set, especially:

- `collections-core-semantics-spec.md` §§5–30;
- `ranges-iteration-and-eagerness-spec.md` §§3–17 and §§45–50;
- `tuple-record-and-symbols-spec.md` §§12–16.

Repository-era `U-INDEX`, `U-COLLTYPES`, `tuple-and-range.md`, `collection-protocol.md`, and old Range comments are implementation history only where they disagree with the new specification.

Spec C assumes Spec A.3 has landed. C.1 also assumes B.1 has landed so `KeyError` and Map's strict/safe association boundary already exist. B.2/B.3 are not prerequisites.

The repository baseline inspected for this plan is `aureat/phalcom-lang` main at commit `5c73279157891ca8e2fc045db5e7dff683c0be5b`. Every implementer must re-inspect actual HEAD before editing; file names and line positions below are architectural anchors, not a substitute for that check.

## Repository diagnosis

HEAD already has the correct selector architecture for indexing:

- `Expr::Index` and `Expr::SetIndex` are explicit AST nodes;
- bracket contents reuse call-shaped arguments, including labels;
- compiler lowering derives real subscript selectors rather than rewriting to `at`;
- setter lowering appends an ordinary `put:` label.

That foundation stays. Spec C changes semantics behind it.

The current runtime behavior is materially obsolete:

1. `List::at_`, `Tuple::at_`, and `Bytes::at_` reject every negative Number before lookup. The new semantics require end-relative negative indices.
2. Their public `[]` behavior is inherited from the old total-lookup contract and does not raise `IndexError` on an out-of-range read.
3. `List::set_` / `Bytes::set_` report out-of-range writes as generic runtime type errors rather than the collection error contract.
4. `Tuple` now has two product lanes after Spec A; integer indexing and slicing must operate over its total linearized product order while preserving labels in slices.
5. `Expr::SetIndex` currently compiles receiver → arguments → RHS → ordinary `Invoke`. The value left by the expression is therefore the setter method's return, while the ratified language rule requires the original RHS object/value.
6. `Token::DotDot` and `Token::DotDotDot` already exist lexically, but Range syntax is not active in the parser and no `..=` token exists.
7. `RangeObject` stores exactly `(start, end, inclusive)` and documents the obsolete convention `a..b` inclusive / `a...b` exclusive. The new language uses `a..b` half-open, `a..=b` closed, supports missing endpoints, and reserves `...` for spread/rest.
8. `Bytes::slice_(start,end)` already supplies a native copy primitive, but it requires pre-normalized nonnegative bounds and rejects rather than clamps. List has no equivalent raw slice primitive; Tuple cannot implement a dynamic labeled-preserving slice purely in `.ph` without a native finalization seam.

## Numeric-tower blocker: exact `Int` versus `Float` index admissibility

The ratified collection specification distinguishes:

```phalcom
list[1]    // valid Int index
list[1.0]  // invalid Float index
```

HEAD cannot observe that distinction. `Token::Number` stores an `f64`, AST `Expr::Number` stores the same numeric value, and runtime `Value::Number` likewise carries `f64`. The source spellings `1` and `1.0` therefore converge before collection indexing.

Spec C MUST NOT pull the entire numeric-tower migration into collections merely to solve this one rule.

Until the numeric tower lands, C uses one isolated compatibility seam:

```text
legacy index representation
    -> finite integral Number accepted
    -> fractional/non-finite Number rejected
```

All index normalization must route through a shared helper so the later numeric migration replaces one representation check with `Int`-only acceptance rather than auditing every collection independently. Add a pending/ignored regression documenting that `1.0` must become invalid once Int/Float are distinct.

This is the only intentional semantic approximation in Spec C.

## Target architecture

```text
subscript syntax
  Expr::Index / Expr::SetIndex
        |
        +-- existing selector derivation retained
        |
        +-- SetIndex compiler preserves original RHS independently
            of setter return

finite indexed sequences
  shared Rust index normalization
        |
        +-- List raw at_/set_
        +-- Tuple raw at_
        +-- Bytes raw at_/set_

  public core.ph
        get(index)                 -> Option
        [index]                    -> value or IndexError
        [index, default: value]    -> value/fallback (eager fallback)
        get(index, orElse: block)  -> value, lazy fallback on miss

Tuple
  numeric [] over total product order
  Symbol [] / get over labeled lane -> KeyError on strict miss

Range literal
  explicit Range AST
        -> BuildRange
        -> RangeObject(lower?, upper?, upperInclusive)

slice consumers
  Range internal slice-bound normalization
        |
        +-- List copy slice in .ph
        +-- Tuple native slice_ preserving labels + Unit normalization
        +-- Bytes existing native slice_
        +-- List native replaceSlice_ for in-place splice
```

## Phase order

### C.1 — Strict/Safe Indexed Access and Assignment Semantics

Migrate List/Tuple/Bytes element coordinates to consistent negative-index normalization; add strict `IndexError` subscript reads while preserving safe `get`; install eager `default:` and lazy `orElse:` lookup forms; complete Tuple Symbol lookup; add Map lazy fallback/orPut integration on top of B.1; reject setter `put:` label collisions; and fix subscript assignment so the expression evaluates to the original RHS, not the setter return.

Expected primitive-floor delta: **0**. Existing raw sequence primitives are retained and semantically corrected; the ambiguity of a stored `None` is resolved by public bounds knowledge rather than changing raw representation.

Artifact: `C.1-indexed-access-and-assignment.md`.

### C.2 — Range Syntax and Bound Representation

Activate `a..b`, `a..=b`, `a..`, `..b`, `..=b`, and `..`; keep `...` non-Range; add explicit Range AST and direct bytecode construction; migrate `RangeObject` to optional lower/upper bounds without increasing arena layout; retire the obsolete public three-argument constructor and old inclusion convention; and leave Progression/descending/eager-exhaustion semantics deferred.

Expected primitive-floor delta: **at most -1/+0 net**, depending on actual post-A/B HEAD: retire the old public `Range.class::new(_,_,_)` binding; retain/rename the three native bound observers as the minimal underivable representation surface. Compute and record the exact delta against implementation HEAD.

Artifact: `C.2-range-syntax-and-representation.md`.

### C.3 — Range Slicing and List Slice Replacement

Interpret Range bounds as clamped slice boundaries; preserve family on read copy slices; preserve Tuple labels and normalize empty Tuple slices to Unit; integrate existing Bytes bulk slicing; add List strict slice assignment and recoverable `replace(range, with:)`; and keep reversed/descending Range semantics and generic replacement-source exhaustion outside C.

Expected primitive-floor delta: **+2 maximum** on the planned architecture: one Tuple raw slice primitive (dynamic product reconstruction is otherwise underivable) and one List raw splice primitive (in-place variable-length replacement is otherwise underivable with the current List floor). List read slicing remains `.ph`; Bytes reuses its existing slice primitive. Recompute against HEAD.

Artifact: `C.3-range-slicing-and-list-replacement.md`.

## Cross-phase invariants

After C.1:

- `seq[-1]` addresses the final element for List/Tuple/Bytes;
- strict sequence `[]` never returns Option/absence merely for an invalid index;
- `seq.get(i)` distinguishes a present stored `None` as `Some(None)` from an invalid coordinate as `None`;
- callbacks in `get(index, orElse:)` receive the original supplied index, not its normalized form;
- Map negative integer keys remain ordinary keys and are never normalized;
- Tuple Symbol lookup is label-domain lookup and missing labels raise `KeyError` under strict `[]`;
- `obj[args] = rhs` evaluates receiver, subscript arguments, then RHS exactly once and returns the original RHS after successful setter dispatch regardless of the setter's return value.

After C.2:

- `..` and `..=` are the only Range punctuation families;
- `...` is never interpreted as a Range;
- Range construction does not dispatch through an overridable `Range.new`;
- missing Range bounds are represented distinctly from an explicitly supplied surface `None` endpoint;
- old `a..b`-inclusive / `a...b`-exclusive behavior is gone.

After C.3:

- slicing clamps boundaries instead of applying strict element-index failure;
- List slice returns List, Bytes slice returns Bytes, Tuple slice returns Tuple except that an empty Tuple slice canonically becomes Unit;
- Tuple slice labels survive exactly for selected labeled components;
- List slice replacement may change List length and preserves relative order outside the replaced interval;
- source-level slice assignment returns the original RHS through the C.1 assignment-expression machinery.

## Deliberate exclusions

Spec C does **not** implement:

- the Int/Float runtime split itself;
- full List mutation vocabulary (`insert`, removal, move, swap, sorting) — Spec D;
- concrete transformation vocabulary — Spec D;
- full Range/Progression runtime semantics;
- `Range.by(step)` / Progression construction until its validation/direction contract is fully specified;
- descending/reversed Range iteration;
- complete Range equality/hash semantics;
- Range eager-exhaustion boundedness analysis — Spec E;
- lazy iterator protocol or pipeline execution — Spec E;
- `*`/`**`/`***` expansion — Spec F;
- generic Iterable replacement sources for List slice assignment unless separately ratified before implementation;
- stepped slicing;
- zero-copy slice views;
- Record field lookup syntax;
- mutation-during-iteration behavior;
- final collection/Range printing policy.

## Verification gate

Every phase must inspect actual HEAD immediately before implementation. During development run focused AST/core tests, but completion requires:

```sh
./scripts/verify.sh --full
```

Range lexer/parser work must review snapshot changes intentionally. Do not bulk-accept snapshots unrelated to Range punctuation or the phase's explicit AST changes.
