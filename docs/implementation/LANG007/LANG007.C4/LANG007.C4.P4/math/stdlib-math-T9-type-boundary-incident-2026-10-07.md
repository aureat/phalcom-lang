# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4.P4 (adjacent stdlib-math corrective work)
- Plan: `stdlib-math-luna-patch-grade-implementation-plan.md`
- Task/Gate: T9.5 — negative numeric type cases
- Repository: aureat/phalcom-lang
- Branch: `stdlib-random-math`
- Starting revision: `9742d085580a98fee8822e1c66026d179ddd772e`
- Current revision at incident closure: `06e37110` plus the current T9 member-body continuation changes

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

## 14. Architectural decision received

The approved owner is the semantic module-value boundary, not the callable
checker. A `PublicExport(module, name)` resolves to its canonical
`LinkedExportTarget::Binding(SymbolId)`, whose `ModuleBindingFact` supplies the
unchanged `BindingState` (both `TypeKnowledge` and denotation). Existing
`synthesize_family_value_call -> apply_resolved_callable` remains the sole
call-check authority.

## 15. Implemented focused slice

- Top-level semantic analysis now performs a second, canonical-binding pass.
  Selective imports seed fresh local bindings from the source `BindingState`.
- Qualified module value access now resolves a linked public export to the
  canonical binding fact before normal Family/callable application. It does
  not add synthetic module methods or change ordinary dispatch.
- The focused semantic regression proves the source export, a selective alias,
  a qualified alias, and a one-hop re-export all retain `TypeData::Family` and
  reject incompatible arguments.
- The original `math.sqrt(4)` compiler-facing T9 reproducer now passes.

## 16. Closure and remaining boundary

The approved module-value environment is now threaded through
`FormalQueryInputs`, `BodyAnalysisContext`, and callable-body revalidation.
Selective imports seed fresh lexical bindings from canonical `SymbolId`
facts; qualified module exports resolve through linked public names. Body
products record linked-name and `ModuleStaticBindings` dependencies.

Focused semantic coverage now rejects incompatible arguments in top-level and
class member bodies for selective and qualified calls through a facade
re-export. An incremental provider-contract change updates the member-body
diagnostic and matches cold analysis. The compiler-facing T9 Family/type test
and hidden-child import test pass. The original T9 semantic owner issue is
resolved; remaining M1 numeric/property coverage and T12 broad verification
are tracked by the active plan and latest handoff.
