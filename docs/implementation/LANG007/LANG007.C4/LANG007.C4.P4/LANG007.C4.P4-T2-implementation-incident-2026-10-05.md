# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C4`
- Plan: `LANG007.C4.P4`
- Task/Gate: T2, canonical row/store/tail migration
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: not recorded when this interrupted, already-dirty T2 continuation began
- Current revision: `419ec6e6` (working tree contains uncommitted implementation changes)

## 2. Escalation trigger

The original T2.6 stop condition fired because the type-form path had no checked
projection for computed static Record keys. That boundary was consulted and
recorded in the checkpoint and plan. A follow-up consultation resolved the
dependency loop: const/general/symbolic keys fail closed for T2 and the shared
type/pattern projection moves to T7.0 after T4/T5/T6. The metadata compatibility
question raised by the compile migration was also consulted. No unresolved
architecture question remains in this incident; the recent repeated compiler
corrections were local representation mismatches.

## 3. Required invariant

- `P4-INV-32`: legacy name-only metadata identifies Symbol keys only.
- `P4-INV-33`: metadata that cannot represent a heterogeneous key or tail domain
  must fail closed.
- Canonical Record rows retain `FieldKey` identity and anonymous tails retain
  both their key and value types.
- Type-form resolution does not become a second evaluator or name resolver.

## 4. Expected architecture

T2 migrates canonical and scoped rows to `FieldKey` and typed anonymous tails.
The existing metadata schema remains name-only during this task: the exporter
may project Symbol keys and exact Symbol tail domains, but must reject other
keys/domains. Lossless tagged metadata is assigned to T8; `phalcom-type-meta`
must not be changed as a T2 compile workaround. T2 may complete with supported
literal keys and tested fail-closed handling for unsupported computed keys;
T7.0 owns the later type/pattern checker-product bridge and CV-10B.

## 5. Observed behavior

The T2 migration exposed compile errors at generic row-equation and legacy
metadata boundaries. After the exporter was changed to preserve the legacy
Symbol-only contract and the row fallback was kept on canonical `TypeId` terms,
`cargo check -p phalcom-semantic` completed successfully with warnings and no
errors. The follow-up check after removing an unused scoped export helper and
unreachable substitution match arms also completed successfully. No tests were
run during this continuation, and the current working tree remains dirty.

## 6. Minimal reproduction

Command:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --message-format=short
```

Observed: the latest run after cleanup ended with
`Finished dev profile [unoptimized] target(s) in 3.38s`; it reported 25 warnings
but no compiler errors. An earlier intermediate run reported three errors in
`checker/row_inference.rs`: a canonical-versus-inference failure type mismatch,
a missing Record field presence, and the TypeId-only solver lacking
`solve_many` for `InferenceTerm`.

Expected: the semantic library compiles while preserving the canonical solver
boundary and rejecting non-representable metadata exports.

## 7. Relevant code path

```text
type-form Record annotation
  -> checked TypeStore RecordRowData<FieldKey>
  -> scoped and inference row/tail representations
  -> canonical row fallback equations (TypeId payloads)
  -> MetadataExporter legacy compatibility projection
```

Relevant files are `phalcom-semantic/src/types/annotation.rs`,
`types/row.rs`, `types/row_solver.rs`, `checker/row_inference.rs`,
`checker/inference.rs`, and `metadata/export.rs`.

## 8. Scoped implementation diff

- The current T2 working set migrates Record field identity and anonymous-tail
  key/value types through TypeStore, scoped types, substitution, inference, and
  downstream consumers.
- `metadata/export.rs` adds explicit errors for non-Symbol keys and non-Symbol
  remainder domains; it leaves `phalcom-type-meta` unchanged.
- The fallback row solver continues to receive canonical `TypeId` rows. Those
  tails are lifted to inference terms only when compared against inference
  records.
- The existing staged and unstaged P10 plan is unrelated and was preserved.

## 9. Evidence

- The latest compiler output in `/tmp/lang007-c4-p4-check-short.log` ends in a
  successful semantic crate check after the final warning cleanup.
- The successful check reported 25 warnings and no compiler errors.
- No tests were run during this continuation.
- `phalcom-type-meta` has no working-tree changes.

## 10. Implementer's causal hypothesis

The confusing compiler loop came from carrying `InferenceTerm` through a
fallback that intentionally solves canonical residual rows with the TypeId
solver. Generalizing that fallback would cross the existing solver boundary and
change payload-equality behavior. Retaining canonical equations and explicitly
lifting only at the inference comparison seam resolved the reported compiler
errors.

## 11. Alternative hypotheses

- A generic `RecordRowSolver<InferenceTerm>` fallback might appear to remove the
  type mismatch, but its `solve_many` API is intentionally implemented for
  canonical `TypeId` rows; changing that boundary would need separate semantic
  justification.
- Updating the metadata schema would represent more key kinds, but it would
  cross serialization, validation, fingerprint, and compatibility surfaces
  owned by T8.

## 12. Attempts already made

1. Added the scoped legacy metadata projection: Symbol keys map to legacy names;
   other keys and non-Symbol remainder domains return explicit export errors.
   Prediction: metadata compile errors disappear without schema changes.
   Observed: the next check reported only row-inference generic mismatches.
2. Temporarily typed fallback row equations as `InferenceTerm`.
   Prediction: align the equation storage with the lifted tail.
   Observed: compilation then showed the TypeId-only solver boundary and a
   canonical failure conversion mismatch. This representation was reverted.
3. Kept fallback equations canonical and lifted their tails only for comparison
   with inference records. Prediction: preserve solver ownership and compile.
   Observed: the following semantic crate check completed without errors.

## 13. Why local authority is insufficient

The dependency ordering among T2, T3, T4, and T7.0 crossed semantic ownership:
T3 consumes canonical rows, T4 evaluates static Record values, while T7 owns
pattern reduction and checked static-key products. A local reorder could have
silently changed accepted T2 semantics or created a second evaluator. The
architectural consultation assigned the deferred seam to T7.0, split CV-10,
and preserved fail-closed canonical row formation. Metadata ownership was
separately resolved as a T2 compatibility bridge with lossless schema staging
deferred to T8.

## 14. Decision requested

None. The consultations resolved the relevant architecture questions and the
plan/checkpoint now record the T7.0 amendment. The next implementation action
is to complete T2's focused verification before claiming T2 focused-tested.
