# INC-002 — Tuple subtype checking ignores labeled-entry order

Finding ID: `INC-002`  
Severity: High / P1  
Confidence: High  
Status: Confirmed by source/specification contradiction; direct regression fixture is deferred because the audit is read-only  

Subsystem: Canonical type relations / structural tuple typing  
Owning file: `phalcom-semantic/src/types/relation.rs`  
Owning function: `check_subtype_impl`  
Affected consumers: Assignability, callable/argument compatibility, generic constraints, declaration checking, reflection-facing type relations, and any compiler product that relies on tuple subtype results  

## Specification

The authoritative tuple rules are in:

- `docs/specs/tuple-and-record-typing.md`, §10.1 and §11;
- `docs/specs/structural-products.md`, §§2, 3, and 8.

They define labeled Tuple entries as ordered entries. A labeled entry occupies
an ordered position, and a source satisfies a fixed labeled target entry only
when the matching label is at the corresponding position. Structural equality
also includes labeled-entry order.

## Expected behavior

For two closed labeled Tuple types with the same labels in different positions,
subtyping must reject the relation unless the ordered positions and labels agree
according to the normative Tuple rules.

For example, with suitable proper types:

```text
(first: Int, second: String) <: (second: String, first: Int)
```

must not be established merely because the same label/type pairs occur.

## Actual behavior

`check_subtype_impl` splits Tuple elements into positional and labeled vectors,
then checks labeled entries by label lookup rather than by corresponding
position. The relevant implementation is:

```rust
let sub_lab: Vec<_> = sub_tuple.elements.iter().filter(|e| e.label.is_some()).collect();
let sup_lab: Vec<_> = sup_tuple.elements.iter().filter(|e| e.label.is_some()).collect();

for target_field in sup_lab.iter() {
    let target_name = target_field.label.as_ref().unwrap();
    if let Some(source_field) = sub_lab.iter().find(|e| e.label.as_ref().unwrap() == target_name) {
        // only the label-selected value types are compared
    }
}
```

The same label-based matching occurs in both the Tuple and Pack relation
branches. The implementation checks presence, surplus labels, and value-type
relations, but it does not compare the labeled vectors positionally.

`TypeStore::open_tuple` preserves element order in `TypeData::Tuple`; it only
enforces that labeled entries form a suffix and that labels are unique. Thus
the relation implementation can receive two distinct ordered Tuple types with
the same label set.

## Minimal reproducer

Construct two closed Tuple types in a `TypeStore` with the same two labeled
entries in opposite order, then call `is_subtype` in either direction. The
source path predicts that both relations are accepted when corresponding value
types are compatible, even though the ordered labels differ.

Equivalent construction shape:

```rust
let left = store.tuple(Box::new([
    TupleTypeElement::labeled("first".into(), int_ty),
    TupleTypeElement::labeled("second".into(), string_ty),
]));
let right = store.tuple(Box::new([
    TupleTypeElement::labeled("second".into(), string_ty),
    TupleTypeElement::labeled("first".into(), int_ty),
]));
assert!(!is_subtype(&mut store, &hierarchy, left, right));
```

The assertion is the normative expectation. The current source's `.find`
matching predicts that it will instead return `true`.

## Reproduction command

No repository test was added or modified because the audit is read-only. The
owning focused test target is `phalcom-semantic/tests/semantic` and a direct
unit fixture in `types::relation`; execution is required for final behavioral
confirmation after an authorized regression fixture exists.

## Observed output

Source inspection establishes the incorrect relation path. Existing relation
tests cover positional Tuple order and lane independence, but no test found in
the audited test inventory asserts that opposite labeled Tuple order is
rejected.

## Expected output

The relation must be `Refuted` for opposite ordered labeled entries, with no
successful assignability or conformance proof.

## Root cause

Tuple and Record structural representations share a filtering shape in the
relation implementation, but their semantics differ. Record fields are
unordered and should be found by key. Tuple labeled entries are ordered and
must be checked by position. The Tuple branch reuses Record-like name lookup
instead of preserving the ordered-lane contract.

## Violated invariant

- **T2 — Type identity:** distinct ordered Tuple types are treated as
  interchangeable by a subtype relation.
- **T4 — Evidence integrity:** a successful relation proof does not correspond
  to the exact ordered Tuple judgment.
- **T9 — Determinism/semantic consistency:** relation behavior is inconsistent
  with the normative identity model, even though construction order remains
  deterministic.

## Failure classification

Type-system soundness failure (`A`). A value with one ordered labeled Tuple
shape can be accepted where a different ordered labeled shape is required. It
is also a completeness/identity contract failure because the implementation
does not preserve a semantic distinction explicitly required by the
specification.

## Affected feature combinations

- Structural products × subtyping.
- Labeled Tuple types × callable argument contracts.
- Labeled Tuple types × generic associated-type specialization.
- Labeled Tuple types × Pack/tuple conversion and rest lanes.
- Type relation proofs × compiler lowering or runtime argument transport.

## Related historical incidents

The implementation history contains repeated labeled-Tuple identity and
transport work, including LANG005.C5 associated-type fixtures. Those records
are context only; this finding is based on the current relation implementation
and current normative specification.

## Recommended correction layer

Correct the canonical relation owner in `types/relation.rs`. Do not patch
callable resolution, compiler lowering, or runtime consumers independently.
The Tuple branch should compare labeled entries according to the ordered Tuple
rule, while Record row comparison should retain key-based unordered behavior.

## Required regression tests

1. Opposite labeled-entry order is rejected.
2. Same labeled order with covariant value subtyping is accepted.
3. Positional/labeled lane boundaries remain distinct.
4. Surplus labels still require an explicit labeled remainder.
5. The equivalent Pack relation observes the same ordered labeled-lane rule.
6. A Record with the same key/value set remains order-independent.
7. Callable and associated-type consumers do not reconstruct a different rule.

## Potential collateral effects

Changing this relation may expose existing fixtures that accidentally rely on
unordered labeled Tuple matching. Callable argument compatibility and generic
associated-item tests should be reviewed together with the relation regression.

## Verification status

Source/specification contradiction confirmed on the audit reference revision.
Behavioral reproduction is pending an authorized test-only probe; no source or
test files were modified for this incident.
