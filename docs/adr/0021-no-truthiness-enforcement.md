# 21. No-truthiness enforcement: typed branch floor + literal-only compile check

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/v0.2/values-and-absence.md` §3.5;
  [ADR-0007](0007-option-as-abstract-with-some-none.md);
  [ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md);
  [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md);
  `phalcom-core/src/compiler/lib.rs` (`is_option_literal`, `branch_condition_of`),
  `phalcom-core/src/compiler/inliner.rs`

## Context

[ADR-0007](0007-option-as-abstract-with-some-none.md) settled that absence is
modelled as an abstract `Option` with `Some`/`None` subclasses, and that `if
(opt)` must be rejected — no implicit truthiness (Invariant 6). The spec states
this categorically: `values-and-absence.md` §3.5 says `if (opt)` is a **compile
error** and "a condition must be a `Bool`."

Taken literally, that is not implementable. Phalcom has **no static type system
and no flow analysis**: the type of an arbitrary condition expression is only
known at runtime, so general compile-time detection of a non-`Bool` condition is
impossible. Only *syntactically obvious* Option conditions — a bare `None`, or a
`Some.new(…)` construction written directly in the condition — can be recognized
by the compiler. This was open decision **DEC-C** (U6): "How is `if(opt)` a
compile error, when no static analysis exists?"

[ADR-0018](0018-sacred-selector-inliner-and-override-guard.md) (U5) is the other
half of the picture: surface `if`/`while`/`and`/`or` lower to inlined branch
opcodes guarded by a `GuardBool` that deopts when the branch value is not a
`Bool`. That guard is a ready-made runtime floor.

## Decision

Ship **Option A** (pre-authorized by the user as the DEC-C recommendation).
No-truthiness is enforced in **two composing layers**, not one:

1. **Runtime no-coercion floor (general).** `Option`, `Some`, and `None` **never
   implement the boolean-branch protocol**. Every branch tests its condition
   through the `GuardBool` opcode
   ([ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)), which
   requires a `Bool`; any non-`Bool` condition — including any `Option` — is a
   **hard runtime type error** with no coercion, ever. This covers *every*
   dynamically-typed condition, whatever its surface form.

2. **Compile-time literal rejection (statically detectable subset).** The
   compiler rejects syntactically-detectable literal `Option` conditions before
   the program runs, via `is_option_literal` / `branch_condition_of` in
   `compiler/lib.rs`. `branch_condition_of` extracts the branch-tested receiver
   of a recognized sacred conditional send
   (`ifTrue`/`ifFalse`/`ifTrue:ifFalse:`/`and`/`or`); `is_option_literal` matches
   the two truthless `Option` surface literals — the `None` singleton (lexes to
   `Var { value: "None" }`) and a `Some.new(…)` construction (a `MethodCall` of
   `new` on `Some`; Phalcom has no bare `Some(x)` call syntax, so construction is
   always the explicit static send). A branch whose condition is such a literal
   fails to compile.

Together these **refine spec §3.5** from a blanket "compile error" to: **compile
error where statically detectable, hard runtime type error otherwise.** The
mechanism **composes with U5's branch-opcode typing** — the same `GuardBool` that
deopts a non-`Bool` sacred send is the runtime floor here; no new analysis pass
is introduced.

## Consequences

- Every `if (non-Bool)` is rejected: a literal `Option` (`if (None)`, `if
  (Some.new(x))`) at compile time; every other non-`Bool` condition at runtime.
  No path silently coerces a value to a truth value — the whole point of removing
  `nil` (Invariant 6) is preserved.
- The early check is **syntactic only**. Indirection defeats it: `var x = None;
  if (x)` is caught at runtime, not compile time. That gap is accepted and
  tracked (DEFERRED #13 — captured-`let` reassignment not rejected; DEFERRED #14
  — `if(opt)` literal-only, plus the `OptionTruthiness` diagnostic currently
  carries no source span).
- Enforcement is **cheap**: it reuses the inliner's condition extraction and the
  existing `GuardBool`; there is no type checker and no dataflow pass to maintain.
- The compile check is **coupled to surface spellings** (`None`, `Some.new`): if
  those forms are ever renamed, `is_option_literal` must be updated in lock-step.
- Spec §3.5's wording ("compile error") is now precise rather than aspirational;
  the corpus `compile-errors/compile_error_if_option_truthiness` and the runtime
  branch-guard cases pin both layers.

## Alternatives considered

- **Full static / flow typing** to detect all non-`Bool` conditions at compile
  time (the literal reading of §3.5). Phalcom has no type system; this would be a
  large new subsystem, out of scope for U6, and premature for a dynamically-typed
  language.
- **Runtime floor only** (drop the compile check). Simpler, but forfeits the
  spec's promise of catching the obvious `if (None)` before the program runs; the
  literal check is nearly free, so keeping it is worth the small coupling cost.
- **Implicit truthiness** (coerce `Option`/`Some`/`None` to `Bool`). Familiar
  from JavaScript, but explicitly rejected by
  [ADR-0007](0007-option-as-abstract-with-some-none.md): it reintroduces exactly
  the nil-like coercion the absence model exists to remove.
