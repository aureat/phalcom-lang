# Checkpoint Record — LANG005.C2 Inherent Impl and Inherent Behavior Subsystem

## Lifecycle

```yaml
checkpoint: LANG005.C2
status: IN_PROGRESS
active_plan: LANG005.C2.P3
completion: PARTIAL
verification: FOCUSED_TESTED
```

## Plan Ledger

| Plan | Title | Status | Completion | Verification |
|---|---|---|---|---|
| `LANG005.C2.P1` | First-Class Inherent `impl` and Effective Declaration Surfaces | COMPLETE | IMPLEMENTED | FOCUSED_TESTED |
| `LANG005.C2.P2` | Variants-Only Enums and Impl Behavior Migration | COMPLETE | IMPLEMENTED | FOCUSED_TESTED |
| `LANG005.C2.P3` | Constrained & Specialized Inherent Impl Applicability | PLANNED | NOT_STARTED | UNVERIFIED |

## Established Takeover Interfaces & Architecture from C1
- `ProductStorage`, `ProductShapeRegistry`, `RuntimeAnonymousProductDescriptorRegistry` fully unified.
- `VirtualProductKind` generalized for scalar replacement and allocation sinking.
- `RuntimeTypeRecipe` (`Closed`, `Template`) and `RuntimeTypeEnvironmentId` wired to `CallFrame` and `BlockObject`.

## Established C2.P1 & C2.P2 Invariants & Implementation
1. **AST & Syntax:** `EnumDef` holds `pub variants: Vec<VariantDecl>`. `EnumMember`, `VariantBody`, and `VariantDecl.body` removed. Behavior syntax in enums rejected with `SyntaxErrorKind::EnumBehaviorUnsupported`. `Statement::Impl(ImplDef)` supports nominal roots (`Class`, `Data`, `Enum`) and exact enum cases (`InherentImplTarget::ExactEnumCase(VariantId)`).
2. **Target Resolution & Validation:** `resolve_inherent_impl_target` validates same-module targets, enforces full generic covering bijection, verifies exact enum cases against declared variants, and canonicalizes generic signatures.
3. **Effective Surface & Enum Behavior:** `build_effective_surface` and `build_enum_behavior` construct effective surfaces and `EnumBehaviorProduct` exclusively from normalized `InherentImplContribution`s. All secondary fallback AST-walking passes removed.
4. **Query & Session Wiring:** Callable lookup and body analysis for enums route through `QueryKey::CallableDefinition` and `InherentImplContribution`.
5. **Compiler Lowering & VM Dispatch:** Exact case methods emit `Bytecode::VariantMethod` and attach to hidden case behavior classes. Root methods attach to root behavior classes and dispatch across cases without modifying `ProductStorage` or runtime case memory layout.
6. **Shipping Sources:** `Option`, `Result`, `Ordering`, and `Either` separated into structural enum declarations and root `impl` blocks.

## Verification Evidence
- `RUSTFLAGS='' cargo test -p phalcom-ast` — 226 passed, 0 failed, 1 ignored.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations` — 14 passed, 0 failed.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls` — 40 passed, 0 failed.
- `RUSTFLAGS='' cargo test -p phalcom-core --test core language::inherent_impl` — 12 passed, 0 failed.
- `RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data` — 44 passed, 0 failed, 19 ignored.
- `rg 'EnumMember|VariantBody|parse_enum_behavior_member' phalcom-*/src` — zero production hits.
- Diagnostic baseline in `semantic-diagnostics-baseline.txt` updated to reflect exact source offset shifts from `Result` separation and confirmed `Option`/`Fiber` typing behavior.

## Active Plan & Next Action
- **Latest Completed Plan:** `LANG005.C2.P2` (*Variants-Only Enums and Impl Behavior Migration*).
- **Active Plan:** `LANG005.C2.P3` (*Constrained & Specialized Inherent Impl Applicability*).
- **Next Action:** Review `LANG005.C2.P3` requirements and plan for constrained/specialized inherent `impl` applicability.
