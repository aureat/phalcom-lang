# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4.P4 (adjacent stdlib-math corrective work)
- Plan: `stdlib-math-luna-patch-grade-implementation-plan.md`
- Task/Gate: T9.5 — negative numeric type cases
- Repository: aureat/phalcom-lang
- Branch: `stdlib-random-math`
- Starting revision: `9742d085580a98fee8822e1c66026d179ddd772e`
- Current revision: `9742d085580a98fee8822e1c66026d179ddd772e` plus the uncommitted corrective math worktree

## 2. Escalation trigger

Mandatory RED trigger 9 / 16: the plan's expected semantic oracle is
contradicted by live compilation. The required Float-only public API cannot be
accepted while its exact Families compile calls with `Int` arguments.

## 3. Required invariant

`MATH-P-INV-11` and T9.5 require Float elementary operations to accept
`Float` only, with no implicit `Int -> Float` conversion. In particular:

```telos
import universe.math as math
math.sqrt(4)
```

must be rejected by semantic compilation before runtime.

## 4. Expected architecture

The facade exports exact `Family` captures from typed source declarations:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
```

The native kernel remains a Float-only backend. The plan does not authorize a
runtime coercion, a second checker, or a math-specific dispatch path.

## 5. Observed behavior

The new focused T9 test confirms the exported values are exact Families with
the expected selectors, then fails because `ProgramCompiler` successfully
compiles `math.sqrt(4)`.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  universe_math_exports_exact_families_and_rejects_non_matching_numeric_arguments \
  -- --nocapture
```

The test fails at the first negative source case because compilation returns a
`CompiledProgram`, rather than `ProgramCompileError::Semantic`.

## 7. Relevant code path

```text
universe.math/package.ph::sqrt
  -> &FloatMathKernel.sqrt(_)
  -> semantic callable/call checking
  -> ProgramCompiler::compile_entry_selection
  -> native FloatMathKernel backend at runtime
```

The exact Family construction is verified by the same test. The failure occurs
before VM execution, so changing `primitive/math.rs` would not correct it.

## 8. Scoped implementation diff

The current worktree contains the general exported-static-constant identity
repair, hidden math-child topology, `log(_,base)`, source-owned angle helpers,
math corpus fixtures, and the T9 acceptance test. The static-constant focused
tests and math corpus pass.

## 9. Evidence

- `RUSTFLAGS='' cargo check -p phalcom-core`: PASS.
- Focused exported-constant getter, re-export/GC, and hidden-child tests: PASS.
- `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture`: PASS (1 selected).
- The T9 test observes `sqrt`, `atan2`, `log`, and `gcd` as exact
  `Family` values with selectors `sqrt(_)`, `atan2(_,_)`,
  `log(_,base)`, and `gcd(_,_)`.
- The same test observes `math.sqrt(4)` compiling successfully.

## 10. Implementer's causal hypothesis

The current checker permits the call through the public Family without
enforcing the native/source callable's declared Float argument type at this
cross-module Family application boundary.

## 11. Alternative hypotheses

1. The checker has the type information but deliberately treats imported
   Families as dynamic.
2. The kernel declaration's type metadata is not propagated to the public
   exact capture.
3. The plan's Float-only source declaration is insufficient under current
   callable semantics, requiring an accepted language-level typing decision.

## 12. Attempts already made

One focused acceptance test was added. Its initial inline form was corrected
mechanically to use a temporary package, because inline/REPL compilation has no
module-import context. The package form then reproduced the semantic gap.
No semantic or runtime workaround was attempted.

## 13. Why local authority is insufficient

Correcting this requires choosing the owner and contract for typed
cross-module Family application. A math-specific check, implicit conversion,
or runtime rejection would violate the plan's semantic-authority and
no-workaround rules.

## 14. Decision requested

Should typed exact Family calls across module exports enforce their captured
source callable signature in the canonical semantic call checker, and if so,
which existing Family/call-lowering owner should carry that fact without
introducing a second source of callable type authority?
