# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4
- Plan: algebraic-literals-follow-on
- Task/Gate: T3 source-owned realization capabilities
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/complex-math`
- Starting revision: `16d79566`
- Current revision: working tree after generic reification repair

## 2. Escalation trigger

Trigger 16 from the shared consultation protocol: the active proposal, current
implementation, and normative trait specification materially conflict.

The proposal requires target-side class-callable capabilities such as:

```phalcom
trait BasisRealizable<B> {
  type Scalar
  @class along(_ coefficient: Self::Scalar, _ basis: B) -> Self
}
```

The normative trait specification explicitly says that metatype/class-side
conformance syntax is outside the current language model and requires a
separate specification if introduced.

## 3. Required invariant

T3 must publish exact capability/conformance evidence for `Zero`, `One`,
`BasisRealizable<B>`, and `RealEmbeddable`. `Complex<J/K>` must remain without
realization evidence. T4 must consume those checked products rather than
reconstructing class-side capability selection in the compiler.

## 4. Expected architecture

The algebraic-literals proposal places realization on the target type and uses
class-side callable requirements. The active implementation's trait surface
currently publishes behavioral trait requirements on `DispatchSide::Instance`.
The parser preserves `@class` as an attribute, but the trait semantic builder
does not use it to form class-side requirements or metatype conformances.

## 5. Observed behavior

Direct source attempts to add the proposed capability vocabulary cannot honestly
establish the required evidence: a trait method declared with `@class` would be
treated by the current trait surface as an instance requirement, while the
target implementations are intended to expose class-side `along`, `fromReal`,
`zero`, and `one` callables.

The normative boundary is explicit in `docs/specs/objects/traits.md`:

```text
... no metatype/class-side conformance syntax. Those facilities require
separate specifications if introduced.
```

## 6. Minimal reproduction / source path

Relevant current owners:

```text
phalcom-ast/src/parser.rs::parse_trait_member
    -> BehaviorMember with @class retained as an attribute
phalcom-semantic/src/traits.rs::build_trait_surface
    -> side = DispatchSide::Instance
phalcom-semantic/src/impls.rs
    -> conformance/witness matching includes requirement side
docs/specs/objects/traits.md
    -> class-side/metatype conformance is a non-feature
```

## 7. Scoped implementation diff

No T3 production capability code was added. The completed neighboring repair is
the compiler call-entry coordinate closure in `phalcom-core/src/modules/compile.rs`.

## 8. Evidence

- `build_trait_surface` sets `side = DispatchSide::Instance` for all ordinary
  behavioral trait members.
- `TraitRequirementId` includes dispatch side, so treating a proposed class
  requirement as instance-side changes the semantic witness identity.
- The normative trait specification excludes metatype/class-side conformance
  syntax.
- The proposal's implementation sketches explicitly require `@class` callable
  requirements and associated `Self::Scalar` results.
- Existing generic runtime reification regressions pass independently; this is a
  separate T3 design boundary.

## 9. Implementer's causal hypothesis

The proposal assumes a class-side trait/conformance feature that the current
language has intentionally not specified. Implementing T3 directly would either
publish incorrect instance-side evidence or create an undocumented second
conformance model in semantic/compiler code.

## 10. Alternative hypotheses

- The proposal may intend realization to be ordinary class-owned static methods
  selected by a new compiler special case; that would violate the requirement
  that T3 publish exact conformance evidence.
- The normative trait specification may need an approved extension for
  class-side/metatype requirements and conformance coherence.
- The capability contract may need a revised instance-side representation,
  which would materially change the proposal's target-side operation and
  compiler projection design.

## 11. Attempts already made

1. Inspected the parser and AST path. `@class` is retained as an attribute, but
   no trait-side static requirement representation exists.
2. Inspected trait surface formation. All behavioral requirements are published
   as instance-side requirements.
3. Compared the proposal against the normative trait specification. The
   specification explicitly defers metatype/class-side conformance syntax.

No speculative T3 patch was retained.

## 12. Why local authority is insufficient

Choosing between extending the trait/metatype language and redesigning the
capability contract changes dispatch-side identity, conformance evidence,
reflection, generic witness selection, and normative source semantics. A local
instance-side workaround would make later T4/T5 evidence unsound.

## 13. Decision and implementation boundary

On 2026-10-10 the user chose **Add class-side traits**.

Interpret the decision as class-side behavioral requirements selected through
the existing nominal `impl Trait for Target` conformance model. Dispatch side
remains part of requirement and callable identity. No conformance to a
metatype/class-object type is introduced. Enum class-side trait requirements
remain unsupported under the enum semantic boundary; class-side index
requirements are also excluded.

The normative contract is recorded in `docs/specs/objects/traits.md`. The live
implementation now publishes `@class` requirements and properties on the
class side, preserves side through default-body and explicit-witness signature
formation, prevents instance data components from witnessing class-side
getters, and rejects enum class-side trait conformances. `universe.numbers`
declares `Zero`, `One`, `RealEmbeddable`, and `BasisRealizable`, with direct
Complex/Quaternion implementations.

Rust `cargo check` passes for `phalcom-semantic` and `phalcom-core`. No
source-language acceptance probes or tests were run in this continuation, so
the source capability declarations and exact runtime evidence remain
unverified. T4 canonical contextual realization and T5 compiler projection/
lowering remain incomplete.
