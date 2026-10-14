# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C4`
- Plan: `LANG007.C4.P7`
- Task/Gate: `T10 / G10` (Pack map/filter/fold source acceptance)
- Repository: `aureat/phalcom-lang`
- Branch: `main`
- Starting revision: `cb8a068c`
- Current revision: `cb8a068c` plus uncommitted P7 work

## 2. Escalation trigger

RED triggers 2, 11, 12, and 18: the plan asks for ordinary source Pack
map/filter/fold, which requires Pack spreads, but ordinary checked Pack spread
calls currently publish neither a ready type nor a canonical constructor
selection. Fixing this requires entering generic constructor inference rather
than only extending the P7 static evaluator's checked-product consumer.

## 3. Required invariant

P7-I6: static evaluation consumes ordinary checked semantic products, never
guesses argument-to-lane identities from source syntax. P7-I10: cold exact
execution and open specialization agree. `structural-products.md` §2.2
requires fused Pack spreads in O(total contributed entries), ordered Symbol
labels, and O(1) rest views; `type-level-domains.md` keeps ordinary Packs
separate from kinded TypePacks.

## 4. Expected architecture

Ordinary checker inference classifies each Pack spread's source shape and
mode, yields a ready proper Pack result, and publishes a checked lane or
expansion selection. Static evaluation fuses exact contributions and defers
unresolved extents via the checked enclosing application without inventing
an evaluator-local checker. The `INC-008` decision already places basic
source Pack construction in P7.

## 5. Observed behavior

P7 now checks, evaluates, and specializes ordinary explicit positional and
labeled Pack entries and iterates exact Packs. A source Pack *expansion*
checks without diagnostics, yet its outer expression is `Blocked(UnknownType
(InferenceBlocked))` and has no `CheckedStaticConstruction`.

## 6. Minimal reproduction

During a focused `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic
source_pack_spread_uses_checked_constructor_lanes -- --nocapture` probe, a
workspace with:

```telos
type WrapPack: Pack<***: Int> { Pack(*Pack(1, 2)) }
```

reported no checked-body diagnostics, but only the inner `Pack(1, 2)` had a
checked constructor. The outer expression's analysis was
`Unknown(InferenceBlocked)` with status
`Blocked(UnknownType(InferenceBlocked))`. Expected: ready exact Pack type and
a checked positional spread product, followed by cold execution producing
`Pack(1, 2)`. The temporary failing probe was removed from the passing test
tree after classification.

## 7. Relevant code path

```text
checker/expression.rs::synthesize_unqualified_call
  -> call.rs::DataConstructorShapeAdapter::bind_arguments
      -> ApplicationArgument::Expansion => DynamicShape
  -> call inference outcome Blocked(InferenceBlocked)
  -> no ready CheckedStaticConstruction publication
static_eval/machine.rs::Expr::UnqualifiedCall
  -> requires the checked construction; cannot execute this spread
```

## 8. Scoped implementation diff

The `INC-008` checked Pack source path is in the current P7 diff. A temporary
`PackExpansion` payload and evaluator splice were tried, then removed when
the ordinary checker did not publish the outer call. No inference solver or
source-language spread semantics was changed for this incident.

## 9. Evidence

- The exact source above parses and the checked static body has no
  diagnostics; inner construction is ready, outer construction is blocked.
- `DataConstructorShapeAdapter::bind_arguments` explicitly classifies
  `ApplicationArgument::Expansion` as `DynamicShape`, even for a statically
  known inner Pack type.
- After removal of the incomplete expansion path,
  `integration::static_recursive_evaluation` passes 23/23. Existing explicit
  Pack lanes, static iteration, and cold/incremental constant parity remain
  verified at their focused gates.

## 10. Implementer's causal hypothesis

The ordinary generic constructor adapter treats all source spreads as
dynamic and never derives exact Pack lane contributions from a checked
operand type, so no ready checked product exists for static evaluation to
consume. Adding an evaluator-only expansion branch cannot repair the
checker terminal.

## 11. Alternative hypotheses

An existing ordinary expansion product outside the data-constructor adapter
may encode this exact source shape but is not routed into Pack constructor
inference. In either case, canonical call selection must precede static
execution; fabricating the missing product from raw AST would violate P7-I6.

## 12. Attempts already made

Added a checked `PackExpansion` payload and evaluator splice behind
`result.status.is_ready()`. Predicted: an accepted source spread would
publish the outer Pack construction. Observed: only the inner constructor
published, with the outer result still `InferenceBlocked`. This speculative
extension and its failing test were removed, preserving the passing basic
Pack slice. No further attempt was made at the inference boundary.

## 13. Why local authority is insufficient

Extending ordinary constructor inference for spreads changes canonical
shape/argument semantics and may affect runtime Pack calls as well as static
computation. Deferring it would require amending P7.G10's map/filter/fold
acceptance despite the `INC-008` source Pack ruling.

## 14. Decision requested

Should P7.T10 extend the ordinary constructor shape/inference authority to
classify exact Pack spreads (and then publish checked expansion products for
static evaluation), or should spread-dependent Pack algorithms be gated to a
follow-up plan while P7.G10 certifies the existing explicit-lane construction
and iterative Pack algorithms? If P7 owns it, which already-published
ordinary expansion product should be the source of exact spread identity?

## 15. Ruling and resolution

- Ruling: Implement in P7 (Recommended). Extend ordinary constructor shape
  inference for exact Pack spreads, then publish checked expansion products for
  static evaluation.
- Implementation:
  1. `DataConstructorArgumentRole::Expansion { parameter, mode }` added to
     ordinary shape binding. When a constructor has no fixed positional
     parameters (such as `Pack<***Es>(***Es)`), compatible spread arguments are
     bound directly to the expansion parameter.
  2. In `apply_generic_callable_in_context`, `extract_pack_expansion_elements`
     decomposes closed exact Pack/Tuple/Record operand types into ordered
     positional and labeled elements, instantiating and appending them to
     `expansion_terms.positional` and `expansion_terms.labeled`.
  3. `CheckedStaticPayloadId::PackExpansion(mode)` added to checked constructor
     products and published by `expression.rs` for static Pack construction.
  4. The static evaluator fuses exact spread elements into the resulting
     `StaticValueData::Pack` or `StaticResidualData::Pack` without intermediate
     collections.
  5. Verified by `source_pack_spread_uses_checked_constructor_lanes` and
     `source_pack_spread_concat_and_prepend`.

