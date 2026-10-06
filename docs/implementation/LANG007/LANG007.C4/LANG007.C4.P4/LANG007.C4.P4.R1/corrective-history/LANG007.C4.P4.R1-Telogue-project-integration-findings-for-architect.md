# LANG007.C4.1 Telogue Integration Findings for Architect Planning

This note records the issues found while completing the C4.1 audit hardening
and Telogue integration slice. It supplements
[`LANG007.C4.P4.R1-revised-covering-impl-and-Telogue-integration-audit-ee7e8ee7.md`](LANG007.C4.P4.R1-revised-covering-impl-and-Telogue-integration-audit-ee7e8ee7.md) and the
[`LANG007.C4.1 walkthrough`](LANG007.C4.P4.R1-historical-covering-impl-closure-walkthrough.md); it does not replace
the audit report or change its scope.

## Context

- Program/checkpoint: `LANG007.C4.1`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `fix/LANG007-a1-match-equality-authority`
- `HEAD` when these findings were recorded: `2d306c13da02fb3b85ee362498cf4c0280de3825`
- Reproducer: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core telogue_project_compiles_from_declared_entry`
- Reproducer selection: one integration test, loading `packages/telogue` as a project and compiling its manifest-declared `prolog.main` entry.

## Issues encountered

### 1. Telogue used rejected applied `Self` return annotations

Four covering-impl methods used `Self<...>` return annotations. The current
C4.1 behavior rejects applied `Self`; the source-relative spelling is bare
`Self`. The returns in `base.ph`, `identity-token.ph`, `predicate.ph`, and
`variable.ph` were migrated to bare `Self`.

`query.ph` also had a variadic parameter annotated as `*Goals`. It was changed
to `Pack<*Goals>`, matching the package's Pack-typed variadic form. The explicit
record-constructor spelling `Query<*Goals> { ... }` is rejected by the parser,
so the current source uses inferred `Query { ... }` construction.

### 2. The real project compile reaches semantic analysis but fails

The integration regression proves project loading and declared-entry
selection work, then reports these Telogue diagnostics:

- `main.ph` lines 15–16: two generic-pack-contribution errors at
  `Predicate<Person, Person>`. The diagnostic says generic pack contributions
  require an explicit `*`, `**`, or `***` marker.
- `predicate.ph` lines 21 and 32: invalid positional-lane projection errors
  for `Ts::Map<Term>` in the `call` parameter and `Relationship.args` field.
  The diagnostic says a positional lane cannot be projected from the current
  type-kind argument.
- `query.ph` line 11: `GenericInferenceConflict` at the `goals` record-field
  value in `Query.of`.

These remain package-level compile blockers. Full Telogue project compilation
has not been established.

## Investigation and classification

- The `query.ph` inference conflict also appeared when the return annotation
  was changed temporarily from bare `Self` to explicit `Query<*Goals>`. A
  temporary local rename from `goals` to `goal_pack` did not change it. These
  probes do not identify the root cause, but the conflict is not specific to
  the bare `Self` return spelling.
- The `main.ph` errors concern generic pack contribution syntax, and the
  `predicate.ph` errors concern pack mapping/projection. The current evidence
  does not show that either is caused by C4.1 covering-impl `Self`
  specialization.
- No semantic authority or cross-layer contract was changed to work around
  these diagnostics. Their owning layer and whether the Telogue source is
  invalid under the accepted syntax remain to be determined.
- The C4.1 semantic body gate passes with the added six-permutation and
  nested-shape coverage. The `phalcom-semantic` crate check also passes after
  the override-field rename. Those results do not imply the Telogue project
  compile passes.

## Architect planning questions

1. For `Predicate<*Ts>`, what is the accepted source spelling for supplying
   concrete generic-pack elements in `main.ph`, and should this be corrected
   in the Telogue fixture or does it expose a parser/semantic gap?
2. Is `Ts::Map<Term>` intended to map a type constructor over every element of
   a generic type pack in both callable parameter and record-field positions?
   Which semantic layer owns the reported positional-lane projection failure?
3. What is the supported construction path for `Query<*Goals>` from
   `Query.of` when explicit generic arguments on the record constructor are
   rejected by the parser? Should the field-based inference be supported, or
   should Telogue express the construction differently?
4. Should these package fixes be planned as a separate Telogue integration
   work unit, while retaining the C4.1 focused completion classification, or
   does any finding change the C4.1 acceptance boundary?

Please ground any implementation plan in the accepted syntax/specification
and the owning semantic layer. Keep the project-entry regression as the
end-to-end discriminator once the package blockers are resolved.

## Current focused evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic` — passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls::bodies` — 21 passed, 1926 filtered out.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core telogue_project_compiles_from_declared_entry` — selected one test; failed with the package diagnostics above.
- `git diff --check` over the scoped implementation files passed, and this note has no trailing whitespace. The supplied `LANG007.C4.P4.R1-revised-covering-impl-and-Telogue-integration-audit-ee7e8ee7.md` has pre-existing trailing whitespace and was excluded from the scoped check.

## Relevant files

- Regression: [`telogue_project.rs`](../../../../../../../phalcom-core/tests/core/modules/telogue_project.rs)
- Telogue constructor: [`query.ph`](../../../../../../../packages/telogue/src/query.ph)
- Pack projection: [`predicate.ph`](../../../../../../../packages/telogue/src/predicate.ph)
- Generic pack uses: [`main.ph`](../../../../../../../packages/telogue/src/main.ph)
- Evidence record: [`LANG007.C4.1 walkthrough`](LANG007.C4.P4.R1-historical-covering-impl-closure-walkthrough.md)
