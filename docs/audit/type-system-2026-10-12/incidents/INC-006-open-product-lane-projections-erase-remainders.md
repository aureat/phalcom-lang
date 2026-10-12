# INC-006 — Closed after normative review: fixed-entry projections are not whole-shape projections

**Finding ID:** INC-006  
**Status:** **CLEARED / withdrawn — not a confirmed incident**  
**Severity:** None (no defect established)  
**Confidence:** High that the originally alleged contradiction is unsupported  
**Audit slice:** 03  
**Repository:** `aureat/phalcom-lang`, `main` at `608c8c6cd3346556f3bbc98f0713a6df6e0a72ac`  
**Historical note:** An initial, incomplete source review mistakenly characterized these helpers as whole-lane shape projections. This report supersedes that assessment and must not be counted in the defect register.

## Original observation (source-correct, interpretation incorrect)

`phalcom-semantic/src/types/store.rs` implements `TypeStore::project_positionals` (~1233–1274), `project_labeled` (~1277–1319), and `project_complete` (~1322–1340) for `TypeData::Tuple` and `TypeData::Pack` by returning the known fixed positional and labeled entries as a `TypePack`/`LabeledTypePack`/`CompleteTypePack`. The anonymous remainder fields are not included in the projected fixed-entry pack. `types/projection.rs` exposes these results through intrinsic `TuplePositionals`, `TupleLabeled`, and `TupleEntries` normalization.

That behavior alone was initially suspected to erase canonical shape information, especially because reassembling `tuple_from_packs(project_positionals(X), project_labeled(X))` creates a closed Tuple even when `X` is open.

## Normative evidence that resolves the suspicion

The **accepted** `docs/specs/structural-products.md` §4 states explicitly:

```telos
intrinsic data OrderedShape(
    _ entries: CompletePack<Type>,
    _ positionalTail: PositionalTail,
    _ labeledTail: LabeledTail,
)
```

It then explains that **anonymous open tails describe possible structure and are not finite iterable collections**. Thus `entries` is a finite fixed-entry pack and tail openness is a **separate** semantic component. `docs/specs/type-level-domains.md` §4.1 independently defines `TypePack` and `LabeledTypePack` as ordered sequences of proper type forms, not as generalized open-row structures. `structural-products.md` §3.2 likewise models Tuple as `Tuple(P, L, Rem_P, Rem_L)` with fixed packs **plus** independent remainder states.

The original assumption that these finite-entry projections should carry the open tail was unjustified. A projection that intentionally returns a fixed pack is not an isomorphism of the entire `Tuple`/`Pack` shape. In particular, the purported reconstruction identity `TupleFromPacks(Positionals(X), Labeled(X)) == X` **does not follow** when X is open. The round-trip experiment proposed in the initial draft cannot establish a defect.

## Disposition

- **Withdraw the original claimed violation** of invariants T2/T4/T6/T9; the normative shape model expressly separates entries from tails.
- **Do not modify** `project_positionals`, `project_labeled`, or `project_complete` to encode anonymous tails in finite `TypePack` forms on the strength of this incident.
- **Do not merge with INC-004**. INC-004 remains a distinct source-confirmed defect because *generic materialization of the whole canonical Tuple type* uses `store.tuple` and silently erases its tail. That path must preserve the complete shape, unlike a fixed-entry projection.
- **Possible future audit question, not an incident:** verify that all consumers intending to reason about the **whole** `OrderedShape` consult its tail fields separately, and that no consumer mistakenly treats fixed-entry projections as proof of closedness. A concrete violating consumer is required before a new incident is authored.

## Verification

This closure is based on live repository source and the explicit accepted specification cited above. No executable test was needed to resolve the initial mistaken contract premise. No implementation or repository files were modified.
