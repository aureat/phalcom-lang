# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4
- Follow-on: contextual algebraic literals
- Task/Gate: source-owned Quaternion generic projection
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/complex-math`
- Starting revision: `43a4f466`
- Current revision: the delivery commit containing this incident; starting
  anchor `43a4f466`, subject
  `feat: checkpoint algebraic literal and Quaternion foundations`

## 2. Escalation trigger

Trigger 15 and trigger 16 from the shared consultation protocol:

- generic reification and runtime type identity are uncertain across the
  semantic/compiler/VM boundary;
- the accepted generic data identity contract conflicts with the current
  generic method construction result.

## 3. Required invariant

For the same canonical nominal owner and exact applied scalar type,
`Complex<Int>` constructed directly and through a checked generic method must
share exact runtime data identity after GC. `Complex<Dynamic>` is not an
acceptable substitute for `Complex<Int>`.

## 4. Expected architecture

The proposal requires source-owned generic `Quaternion<T>` methods to preserve
the selected `T` through ordinary checked call-entry recipes and generic data
construction. Runtime lowering must consume canonical semantic type evidence;
it must not add a type-name lookup or special-case Int/Float methods.

## 5. Observed behavior

Input:

```phalcom
from universe.numbers import Quaternion, Complex
let projected = Quaternion(1, 2, 3, 4).complexPart
let expectedProjection = Complex(1, 2)
let projection = projected === expectedProjection
```

The projected object has the expected components `(1, 2)`, but its runtime
descriptor is `Complex<Dynamic>`. The direct construction has `Complex<Int>`.
The exact descriptor IDs differ and `projection` is false. The method is:

```phalcom
impl<T> Quaternion<T> {
  complexPart -> Complex<T> { Complex<T>(self.real, self.i) }
}
```

## 6. Reproduction

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core quaternion_complex_part_preserves_exact_applied_identity -- --nocapture
```

The earlier diagnostic instrumentation showed the projected application with
an overlay `Special("Dynamic")` argument and the direct application with the
canonical `universe.scalar.number.Int` argument. The retained regression prints
the mismatching descriptors and checks both components before descriptor
identity:

```text
assertion `left == right` failed: projection: Some(RuntimeDataDescriptor { ...
  exact_type: Some(Overlay(...)) ... }) versus Some(RuntimeDataDescriptor { ...
  exact_type: Some(Base { ... }) ... })
left: RuntimeDataDescriptorId(...)
right: RuntimeDataDescriptorId(...)
```

## 7. Relevant code path

```text
semantic generic call analysis
  -> RuntimeCallEnvironmentRecipe { Quaternion::T = Int }
  -> compiler method-body invocation
  -> generic Complex<T> data construction
  -> runtime type recipe/materialization
  -> Complex<Dynamic> descriptor
```

Relevant owners:

- `phalcom-semantic/src/impls.rs` — covering-impl/body-coordinate substitution;
- `phalcom-semantic/src/checker/body.rs` — generic body resolver scope;
- `phalcom-semantic/src/checker/declaration_signature.rs` — published method
  generic signature;
- `phalcom-core/src/modules/semantic_lowering.rs` — data-construction recipe;
- `phalcom-core/src/modules/compile.rs` — call-entry recipe projection;
- `phalcom-core/src/typing/environment.rs` — runtime type recipe instantiation;
- `phalcom-core/src/vm/data.rs` — exact data descriptor materialization.

## 8. Scoped implementation diff

The follow-on adds AST suffix literals, source-owned basis/Quaternion modules,
Universe module registration, exhaustive AST-consumer arms, and focused runtime
probes. No production generic reification correction has been retained.
Temporary contextual-realization prototypes were removed when they failed to
establish the planned capability/identity contract.

## 9. Evidence

- The projection's two components are Int values 1 and 2.
- The projected and directly constructed values have different data descriptors.
- Temporary type-graph output showed `Complex<Dynamic>` versus `Complex<Int>`.
- The compiled entry lowering map carries a recipe for the canonical
  `Quaternion.complexPart` callable with declaration-owned T bound to Int.
- `CoveringImplSubstitution` in `impls.rs` maps impl parameters to declaration
  parameters and exposes an inverse source-body instantiation.
- `typing/environment.rs::instantiate_type_ref` substitutes `Dynamic` when a
  metadata parameter has no entry in the active runtime environment.

The original execution point is now localized and repaired: the source-body
template uses the impl-owned parameter while the call recipe previously carried
only the declaration-owned parameter.

## 10. Implementer's causal hypothesis

The covering-impl source body and call-entry recipe may use distinct generic
parameter coordinates. A nested construction template may retain an impl-owned
parameter while the recipe binds the declaration-owned parameter, causing the
runtime erasure fallback. This is a hypothesis requiring an exact template/
active-frame comparison.

## 11. Alternative hypotheses

- The expected recipe is present in lowering but is not applied to the executing
  getter frame.
- The canonical Universe body compilation or metadata relocation loses the
  recipe or parameter binding before construction.

Distinguish these with stable parameter identities and emitted invocation
bytecode, rather than changing descriptor equality.

## 12. Attempts already made

1. Added the generic `complexPart -> Complex<T>` source method and tested the
   direct projection. Components were correct, but exact identity failed.
2. Inspected the emitted call-entry recipe. It correctly contains `T = Int`.
   This rules out a missing top-level call specialization as the sole cause.
3. Considered Int/Float-specific overloads. Rejected: that would be a local
   workaround with different architectural ownership and would hide the generic
   reification defect.

4. Added a general compiler-side call-environment closure. It consumes the
   semantic covering substitution, binds both coordinates to the same actual
   `TypeId`, rejects inconsistent dual bindings, and applies to ordinary,
   conditional, and bound-family selected calls.
5. Added neutral generic product, nested generic call, outer-generic forwarding,
   independent-binder, and semantic declaration-canonical regressions. The
   Quaternion exact descriptor regression remains unchanged and passes.

The generic reification correction is retained and focused-tested. The
remaining red algebraic-literals probe is the separate unfinished typed basis
realization boundary.

## 13. Why local authority is insufficient

An arbitrary parameter remap, changed erasure policy, or source API workaround
would cross stable identity and compiler/runtime generic typing contracts.
The prior architectural ruling authorized the compiler call-entry bridge. That
decision is implemented without changing ownership, identity, VM fallback, or
descriptor equality. The remaining algebraic-literal work is the separate T3/T4
realization scope.

## 14. Decision requested

Where should the canonical substitution from a generic source-body parameter
(`T`) to the runtime data-construction type recipe be applied so nested generic
data construction receives the caller’s reified type environment rather than
falling back to `Dynamic`? The decision must preserve exact descriptor identity,
covering-impl coordinate purity, and existing call-entry recipe ownership.

**Decision applied:** at semantic-to-runtime call-environment projection in
`ProgramCompiler`, using `CoveringImplSubstitution`; preserve distinct stable
owner identities and duplicate the concrete binding in the executable recipe.
