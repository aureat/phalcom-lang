# IMPLEMENTATION INCIDENT

## 1. Identity

- Program/checkpoint: ARTF001.C1.
- Plan/task: ARTF001.C1.P2.R4, T2 build/export discriminator; T4 query-product contract.
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`.
- Branch: `feat/artifact-universe-consolidation`.
- Starting/current HEAD: `4c3ec841272631c42524e00ed97fec417e33c36b`.
- Current corrections are uncommitted. Checkpoint status remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED.

## 2. Escalation trigger

The next correction may change query-product representation and its cross-layer
contract. Preserving a native alias shell requires selecting a different wire
product table from a nominal shell. Several representations are possible; the
implementer cannot silently select one or prune the product. These trigger the
repository rules for representation, cross-layer contracts, and alternatives with
different architectural consequences.

## 3. Required invariant

Retain canonical native product identity and compiler-observable equality. Alias
authority belongs to the canonical alias table. Preserve distinct nominal and
alias products without manufacturing nominal declarations. Required roots must
not silently disappear. Retained products need exact project-qualified owners,
valid product references, dependency closure, and current fingerprints.

## 4. Expected architecture

ModuleWorldV2 + SemanticWorldV4 + QueryImageV3 reconstruct one canonical image.
Current alias authority is carried by `StructuralPrimaryContractsV2.type_aliases`.
`SemanticWorkspaceSession::from_image` consumes that image through the ordinary
query overlay. The query transport must identify the actual product family.

## 5. Observed behavior

The runtime-obligation correction passes its focused gates. A separate V4-to-V3
validation projection was then corrected to retain aliases; its focused tests
pass. The next full run reaches query export and fails:

```text
declaration-shell query owner has no semantic row
```

Test execution took 112.88s (129.52s including compilation). No artifact was saved;
no load, allocation-stage, source-denial, or speedup proof ran.

The complete diagnostic census found 16 failures among 2,564 reusable candidates,
all DeclarationShell roots. Fifteen native Alias products have a V4 alias row and
no nominal/static-domain/intrinsic-owner row:

| Module | Alias owners |
|---|---|
| `collections.interval` | `LowerBound`, `UpperBound` |
| `collections.pack` | `CompletePack`, `LabeledPack`, `LabeledTypePackView`, `PositionalPack`, `PositionalTypePackView` |
| `collections.record` | `EmptyRecord`, `FieldKey` |
| `option.unit` | `Unit` |
| `reflection.structural` | `FixedFields`, `LabeledTail`, `PositionalTail`, `VariantNames`, `Variants` |

The sixteenth is native Nominal `reflection.package_object::Package`, absent from
all four checked tables. Its source exists in `reflection/package-object.ph` as
`native primitive class Package is Module`. Its absence needs a separate
source-query/final-aggregate consistency ruling; the alias mapping correction
alone cannot explain or resolve it.

The diagnostic test failed after 109.50s (129.15s including compilation), returned
the original first error, and saved no artifact. Complete evidence is preserved
in `/tmp/artf001-r4-query-census.log`; temporary instrumentation is removed.

## 6. Minimal reproduction

From the artifact worktree:

```sh
PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
```

The failure is at `tests/artifact_allocation.rs:60` during `build_universe_telosc`.
Expected: complete export, save artifact, then enter load/accounting assertions.
The preserved failing log is `/tmp/artf001-r4-alias-vertical.log`.

## 7. Relevant code path

```text
session.rs: alias shell query publication
  -> CanonicalSemanticQueryImage::freeze
  -> core::build_universe_telosc
  -> export_query_image_v3
  -> portable_query_product_ref
  -> unique_row(structural_contracts.declaration_shells)
```

`TypeDeclarationShell` is `Nominal(DeclarationTypeInfo) | Alias(TypeAliasInfo)`.
The session publishes both under `QueryKey::DeclarationShell`. The portable
exporter, V4 row-count check, and product-owner check currently use only nominal
`declaration_shells`. Native query reconstruction already supports an alias-table
fallback and creates `TypeDeclarationShell::Alias`.

## 8. Scoped implementation diff

The accepted runtime-obligation correction adds precise obligation uniqueness,
full-record sorting, ownership-specific repeat suppression, and wire/native
regressions. The subsequent mechanical alias adapter correction preserves alias
forms and superclass rows, handles unordered input, and preserves duplicate rows
for validation. Query-product representation has not been changed.

## 9. Evidence

- Compiler-artifact suite: 114 tests pass after the alias adapter correction.
- Producer regressions: 4/4 pass.
- Native requirements through exporter/V4/canonical import: 1/1 passes.
- Alias adapter regressions: 3/3 pass, including unordered input and preserved duplicate rows.
- `git diff --check` passes.
- Live source establishes the nominal/alias native sum type and the nominal-only wire lookup described above.

## 10. Implementer's causal hypothesis

The portable product mapping conflates a query kind with one physical row family.
An alias declaration-shell query has canonical alias authority but no nominal
declaration-shell row. Transport cannot currently reference that existing alias
row as the product of a DeclarationShell query.

## 11. Alternative hypotheses

For the 15 aliases, a missing alias export row is ruled out by the census. For
Package, stale retained query authority, a blocked declaration removed from the
final aggregate, or a missing nominal projection remain alternatives. The session
removes blocked declarations near final publication, while canonical query freeze
does not compare DeclarationShell products with the final aggregate. That is a
causal lead, not proof that Package was blocked. No assertion should be weakened
or row manufactured on that hypothesis.

Subsequent bounded diagnosis established a more specific source-resolution lead.
A temporary probe built the actual Package source interface and production
symbolic resolver. Its interface has no imports, Module is absent from the
canonical prelude, and unqualified Module resolves to None in Package's scope
despite being known. Object resolves from the prelude and Module resolves in its
own module as positive controls. Exactly one test passed in 0.07s; log:
`/tmp/artf001-r4-package-scope-probe.log`. The probe was removed, leaving
`resolver.rs` unchanged. This verifies the unresolved superclass lookup; it does
not yet prove the exact later blocked-declaration removal event. No source fix or
query consistency correction was made.

## 12. Attempts already made

No semantic correction to query transport has been attempted. Diagnostic
instrumentation collected mapping failures and returned the original first error;
it neither pruned roots nor published an incomplete artifact. Instrumentation was
removed after capture.

## 13. Why local authority is insufficient

Possible fixes include an explicit typed product-family selector, an
owner-resolved multi-table row contract, or a composite ordinal space. They have
different representation, validation, and compatibility consequences. Treating
aliases as nominal declarations or dropping alias roots would change canonical
semantics. This requires an advisory decision before implementation.

An adjacent importer check also requires a nominal declaration-shell row for
every static-definition alias link, after finding its V2 alias row. That is a
prospective incompatibility with alias-only authority, not yet an observed load
failure. Resolve its intended contract alongside the alias query decision.

## 14. Decision requested

Specify how a DeclarationShell query references nominal versus alias authority:
the exact target selector/index contract, owner and kind validation, portable and
live fingerprint obligations, required-root treatment, and schema/ABI decision
before freeze. Confirm whether a static-definition alias link is validated by its
canonical V2 alias row alone or requires another explicitly carried authority.
Separately specify the source-free publication invariant for a Ready nominal
query whose owner is absent from the final declaration aggregate, and the bounded
diagnosis/correction for Package.
Return a bounded T2/T4 amendment and focused acceptance matrix. Preserve strict
missing/foreign/wrong-family rejection and prohibit synthetic nominal shells or
silent root pruning.
