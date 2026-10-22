# ARTF001.C1.P2.R4 Universe semantic diagnostic inventory

## Scope and result

This census contains every error diagnostic stored for a Universe module in the `CanonicalUniverseFormalImage` source snapshot produced by a temporary focused census harness on 2026-10-21. The harness was removed after collection. It found **211 diagnostics across 37 modules**, covering 12 diagnostic codes. All 211 records have severity `Error`.

The test passed after building the canonical source image. This is a source-semantic snapshot census; it is not a count of runtime failures or diagnostics emitted while compiling the SlotMap entry. It also does not claim to enumerate every diagnostic possible in every Universe package revision or user workspace.

Universe diagnostics are **not gone**. `CanonicalProductFamily::Diagnostics` has persistence class `NeverPersist`, so these source-analysis diagnostics are retained by the formal image snapshot but are not serialized into the compiler artifact. Their omission from an artifact is persistence policy, not evidence that the source or semantic issues were fixed.

No diagnostic was changed, suppressed, or fixed by this census. Each record still needs classification against the accepted language specification, existing semantic tests, and the predecessor revision before choosing a source, checker, or semantic-product correction.

## Current-head closure census — 2026-10-22

The 211-row table below is the initial discovery census and remains historical
evidence. After implementing the bounded Universe-source correction slice,
whose starting revision was `5a68a1b6edfbc22d94ec27b9a3ae0f3a4eef0539` and
substantive implementation commit was
`a21aeb9d33b5f613eafd48e21457ac438b9502c3`, a fresh canonical-image census
reported **48 diagnostics**. The prior delivered branch tip was
`0206f2f6785f814252fc95a39f471f2caf35f02f`; its focused workflow run
`39693098845` passed before the independent projection-equality proof cases
were added. Proof commit `bd4736743277da2dfd155e50120a48ab374b8e87` then
passed focused workflow run `39713887081`, including HKT and scalar projection
positive/negative cases and the standalone module reflection regression:

| Diagnostic code | Current count | Disposition |
|---|---:|---|
| `AssociatedTypeInherentUnsupported` | 43 | Intentional LANG007 residual; associated-type bindings remain invalid in inherent implementations. |
| `EnumCaseStaticBehaviorUnsupported` | 2 | Intentional enum-semantics residual; class-side trait requirements on enum targets remain unsupported. |
| `MatchArmRedundant` | 0 | Five unreachable inner `Result` fallback arms removed from `concurrency.fiber`; outer `Option` fallbacks remain. |
| `ImplConformanceIncomplete` | 0 | `Iterable.fold` now publishes its method-local accumulator generic, matching `Chain.fold`. |
| `GenericConstraintUnsatisfied` | 0 | Associated-projection equality now defers to generic evidence; `Effects.sequence` forwarding is accepted while distinct constructors remain rejected. |
| `AnnotationUnsaturatedConstructor` | 2 | Deferred reflection `ModuleMetadata` gap. |
| `AnnotationUnresolved` | 1 | Deferred intrinsic `UnorderedShape::Lookup` gap. |

The seven rows above account for all 48 current diagnostics. There are no
unexpected or unclassified current-head rows. This closure census does not
retroactively rewrite the historical 211-row discovery list; it records the
post-correction state used by the focused acceptance receipt.

## Counts by diagnostic code

| Diagnostic code | Count |
|---|---:|
| `AnnotationUnresolved` | 66 |
| `AnnotationUnsaturatedConstructor` | 28 |
| `ArgumentMismatch` | 4 |
| `AssociatedTypeInherentUnsupported` | 43 |
| `CallShapeMismatch` | 22 |
| `ConstructorSuperRequired` | 12 |
| `EnumCaseStaticBehaviorUnsupported` | 2 |
| `EnumRequirementMissing` | 2 |
| `GenericInferenceConflict` | 11 |
| `GenericInferenceUnderconstrained` | 10 |
| `ImplConformanceIncomplete` | 1 |
| `ReturnMismatch` | 10 |

## First-pass trace map

These paths identify the semantic layer that emits or presents each code. They are trace starting points, not root-cause verdicts; a shared code can represent different causes in different source locations.

| Diagnostic code | Owning semantic area to inspect |
|---|---|
| `AnnotationUnresolved` | `phalcom-semantic/src/types/annotation.rs`, declaration/type-member checking, and name/type-member resolution. Inspect the message and referenced declaration for each row; this code includes both missing names and unsupported intrinsic type-member operations. |
| `AnnotationUnsaturatedConstructor` | `phalcom-semantic/src/types/annotation.rs`; check whether the source omitted type arguments or the intended annotation form is supported. |
| `ArgumentMismatch`, `CallShapeMismatch` | `phalcom-semantic/src/checker/call.rs`; trace the selected callable signature, argument labels/order, and type terms at the listed source range. |
| `AssociatedTypeInherentUnsupported` | `phalcom-semantic/src/impls.rs`; check the accepted associated-type rules and the implementation’s inherent type-member handling. |
| `ConstructorSuperRequired` | `phalcom-semantic/src/checker/body.rs`; compare the constructor’s direct parent and initializer body with the constructor-super rule and existing tests. |
| `EnumCaseStaticBehaviorUnsupported`, `EnumRequirementMissing` | `phalcom-semantic/src/checker/enum_behavior.rs`, `phalcom-semantic/src/enum_requirements.rs`, and enum conformance handling in `phalcom-semantic/src/impls.rs`. |
| `GenericInferenceConflict`, `GenericInferenceUnderconstrained` | Call, sequence-literal, static-body, statement, and annotation-elaboration inference paths under `phalcom-semantic/src/checker/`; inspect the constraints at each listed construct. |
| `ImplConformanceIncomplete` | `phalcom-semantic/src/session.rs` and implementation/conformance publication in `phalcom-semantic/src/impls.rs`; determine whether required evidence is absent or merely not published. |
| `ReturnMismatch` | `phalcom-semantic/src/checker/body.rs`, `declaration.rs`, or `statement.rs`, according to the listed construct; compare inferred result with the declared contract. |

## Complete source-located list

Records are ordered by module and byte range. Source ranges are UTF-8 byte offsets; line numbers are one-based. Diagnostic IDs (`U001` onward) are stable within this generated inventory and can be used in follow-up classifications.

### `universe:callable.family` — 2 diagnostic(s)

Source: [`phalcom-core/core/universe/src/callable/family.ph`](../../../../phalcom-core/core/universe/src/callable/family.ph)

- **U001 — `AnnotationUnsaturatedConstructor`**, line 19, bytes `787..791` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U002 — `AnnotationUnresolved`**, line 21, bytes `857..863` on `Method`: unresolved type \`Method\`

### `universe:concurrency.fiber` — 11 diagnostic(s)

Source: [`phalcom-core/core/universe/src/concurrency/fiber.ph`](../../../../phalcom-core/core/universe/src/concurrency/fiber.ph)

- **U003 — `GenericInferenceUnderconstrained`**, line 265, bytes `7451..7461` on `List.new()`: generic inference has insufficient value-producing evidence
- **U004 — `GenericInferenceUnderconstrained`**, line 276, bytes `7683..7693` on `List.new()`: generic inference has insufficient value-producing evidence
- **U005 — `GenericInferenceUnderconstrained`**, line 288, bytes `7936..7946` on `List.new()`: generic inference has insufficient value-producing evidence
- **U006 — `GenericInferenceUnderconstrained`**, line 304, bytes `8351..8361` on `List.new()`: generic inference has insufficient value-producing evidence
- **U007 — `GenericInferenceConflict`**, line 342, bytes `9298..9299` on `v`: generic argument constraints conflict
- **U008 — `GenericInferenceUnderconstrained`**, line 357, bytes `9635..9645` on `List.new()`: generic inference has insufficient value-producing evidence
- **U009 — `GenericInferenceUnderconstrained`**, line 643, bytes `17367..17377` on `List.new()`: generic inference has insufficient value-producing evidence
- **U010 — `GenericInferenceUnderconstrained`**, line 650, bytes `17514..17524` on `List.new()`: generic inference has insufficient value-producing evidence
- **U011 — `GenericInferenceUnderconstrained`**, line 720, bytes `19321..19331` on `List.new()`: generic inference has insufficient value-producing evidence
- **U012 — `GenericInferenceUnderconstrained`**, line 726, bytes `19442..19452` on `List.new()`: generic inference has insufficient value-producing evidence
- **U013 — `GenericInferenceUnderconstrained`**, line 766, bytes `20466..20476` on `List.new()`: generic inference has insufficient value-producing evidence

### `universe:errors.result` — 22 diagnostic(s)

Source: [`phalcom-core/core/universe/src/errors/result.ph`](../../../../phalcom-core/core/universe/src/errors/result.ph)

- **U014 — `CallShapeMismatch`**, line 66, bytes `2683..2697` on `ok.call(value)`: unexpected positional argument at position 1
- **U015 — `CallShapeMismatch`**, line 66, bytes `2683..2697` on `ok.call(value)`: missing required parameter at position 1
- **U016 — `CallShapeMismatch`**, line 67, bytes `2726..2741` on `err.call(error)`: unexpected positional argument at position 1
- **U017 — `CallShapeMismatch`**, line 67, bytes `2726..2741` on `err.call(error)`: missing required parameter at position 1
- **U018 — `CallShapeMismatch`**, line 103, bytes `3973..3982` on `f.call(v)`: unexpected positional argument at position 1
- **U019 — `CallShapeMismatch`**, line 103, bytes `3973..3982` on `f.call(v)`: missing required parameter at position 1
- **U020 — `CallShapeMismatch`**, line 122, bytes `4787..4796` on `f.call(e)`: unexpected positional argument at position 1
- **U021 — `CallShapeMismatch`**, line 122, bytes `4787..4796` on `f.call(e)`: missing required parameter at position 1
- **U022 — `CallShapeMismatch`**, line 169, bytes `6634..6643` on `f.call(v)`: unexpected positional argument at position 1
- **U023 — `CallShapeMismatch`**, line 169, bytes `6634..6643` on `f.call(v)`: missing required parameter at position 1
- **U024 — `CallShapeMismatch`**, line 192, bytes `7577..7590` on `f.call(error)`: unexpected positional argument at position 1
- **U025 — `CallShapeMismatch`**, line 192, bytes `7577..7590` on `f.call(error)`: missing required parameter at position 1
- **U026 — `CallShapeMismatch`**, line 207, bytes `8126..8147` on `predicate.call(value)`: unexpected positional argument at position 1
- **U027 — `CallShapeMismatch`**, line 207, bytes `8126..8147` on `predicate.call(value)`: missing required parameter at position 1
- **U028 — `CallShapeMismatch`**, line 223, bytes `8742..8763` on `predicate.call(error)`: unexpected positional argument at position 1
- **U029 — `CallShapeMismatch`**, line 223, bytes `8742..8763` on `predicate.call(error)`: missing required parameter at position 1
- **U030 — `CallShapeMismatch`**, line 244, bytes `9474..9487` on `f.call(value)`: unexpected positional argument at position 1
- **U031 — `CallShapeMismatch`**, line 244, bytes `9474..9487` on `f.call(value)`: missing required parameter at position 1
- **U032 — `CallShapeMismatch`**, line 268, bytes `10333..10346` on `f.call(error)`: unexpected positional argument at position 1
- **U033 — `CallShapeMismatch`**, line 268, bytes `10333..10346` on `f.call(error)`: missing required parameter at position 1
- **U034 — `CallShapeMismatch`**, line 340, bytes `12834..12847` on `f.call(error)`: unexpected positional argument at position 1
- **U035 — `CallShapeMismatch`**, line 340, bytes `12834..12847` on `f.call(error)`: missing required parameter at position 1

### `universe:errors.unsupported` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/errors/unsupported.ph`](../../../../phalcom-core/core/universe/src/errors/unsupported.ph)

- **U036 — `ConstructorSuperRequired`**, line 15, bytes `194..441` on `private create() { // _message = "unsupported" // _kind = #unsupported // _cause = None // _displaced = None super.initialize( with: "unsupported", of: #unsupported, from: None, displaced: None ) }`: subclass constructor must begin with an explicit \`super.new(...)\` call

### `universe:functional.algebra` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/algebra.ph`](../../../../phalcom-core/core/universe/src/functional/algebra.ph)

- **U037 — `EnumCaseStaticBehaviorUnsupported`**, line 16, bytes `700..778` on `impl<T> Monoid for Chain<T> { @class empty -> Chain<T> { Chain<T>::Empty } }`: class-side trait requirements are not supported on enum targets

### `universe:functional.arrow` — 8 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/arrow.ph`](../../../../phalcom-core/core/universe/src/functional/arrow.ph)

- **U038 — `AssociatedTypeInherentUnsupported`**, line 13, bytes `545..559` on `type Input = A`: associated type bindings are valid only in conformance implementations
- **U039 — `AssociatedTypeInherentUnsupported`**, line 13, bytes `545..559` on `type Input = A`: associated type bindings are valid only in conformance implementations
- **U040 — `AssociatedTypeInherentUnsupported`**, line 14, bytes `562..577` on `type Output = B`: associated type bindings are valid only in conformance implementations
- **U041 — `AssociatedTypeInherentUnsupported`**, line 14, bytes `562..577` on `type Output = B`: associated type bindings are valid only in conformance implementations
- **U042 — `GenericInferenceConflict`**, line 16, bytes `722..739` on `|value| { value }`: generic argument constraints conflict
- **U043 — `AnnotationUnresolved`**, line 61, bytes `2557..2574` on `Profunctor<Arrow>`: trait \`Profunctor\` expects 1 type arguments, got 0
- **U044 — `AnnotationUnresolved`**, line 61, bytes `2568..2573` on `Arrow`: conformance trait argument could not be resolved
- **U045 — `AnnotationUnsaturatedConstructor`**, line 61, bytes `2568..2573` on `Arrow`: type constructor requires type arguments and cannot be used directly as a value type

### `universe:functional.chain` — 12 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/chain.ph`](../../../../phalcom-core/core/universe/src/functional/chain.ph)

- **U046 — `AssociatedTypeInherentUnsupported`**, line 12, bytes `382..398` on `type Element = T`: associated type bindings are valid only in conformance implementations
- **U047 — `AssociatedTypeInherentUnsupported`**, line 12, bytes `382..398` on `type Element = T`: associated type bindings are valid only in conformance implementations
- **U048 — `AssociatedTypeInherentUnsupported`**, line 12, bytes `382..398` on `type Element = T`: associated type bindings are valid only in conformance implementations
- **U049 — `ReturnMismatch`**, line 58, bytes `1400..1563` on `while true { match remaining { Chain::Empty => { return count } Chain::Link(_, rest) => { count = count + 1; remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U050 — `ReturnMismatch`**, line 69, bytes `1683..1862` on `while true { match remaining { Chain::Empty => { return result } Chain::Link(head, rest) => { result = result.prepend(head); remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U051 — `ReturnMismatch`**, line 81, bytes `2068..2247` on `while true { match remaining { Chain::Empty => { return result } Chain::Link(head, rest) => { result = result.prepend(head); remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U052 — `ReturnMismatch`**, line 92, bytes `2381..2576` on `while true { match remaining { Chain::Empty => { return result.reverse } Chain::Link(head, rest) => { result = result.prepend(f.call(head)); remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U053 — `ReturnMismatch`**, line 104, bytes `2800..3235` on `while true { match remaining { Chain::Empty => { return result.reverse } Chain::Link(head, rest) => { let mut inner: Chain<U> = f.call(head) while not inner.isEmpty { match inner { Chain::Link(value, tail) => { result = result.prepend(value); inner = tail } Chain::Empty => {} } } remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U054 — `ReturnMismatch`**, line 124, bytes `3380..3622` on `while true { match remaining { Chain::Empty => { return result.reverse } Chain::Link(head, rest) => { if predicate.call(head) { result = result.prepend(head) } remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U055 — `ReturnMismatch`**, line 150, bytes `4152..4465` on `while true { match remaining { Chain::Empty => { return result.reverse } Chain::Link(head, rest) => { match f(head) { Option::Some(value) => { result = result.prepend(value) } Option::None => {} } remaining = rest } } }`: tail expression result is not assignable to method's declared return type
- **U056 — `ReturnMismatch`**, line 258, bytes `7362..7716` on `while true { match left { Chain::Empty => { return right.isEmpty } Chain::Link(a, rest) => { match right { Chain::Link(b, tail) => { if a != b { return false } left = rest right = tail } _ => { return false } } } } }`: tail expression result is not assignable to method's declared return type
- **U057 — `ImplConformanceIncomplete`**, line 296, bytes `8281..8427` on `impl<T> Iterable for Chain<T> { type Item = T type Iterator = ChainIterator<T> iterator -> ChainIterator<T> { ChainIterator<T>.new(self) } }`: conformance for \`Iterable\` does not satisfy all requirements

### `universe:functional.effects` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/effects.ph`](../../../../phalcom-core/core/universe/src/functional/effects.ph)

- **U058 — `AnnotationUnresolved`**, line 59, bytes `2309..2325` on `Effects.traverse`: unresolved type \`Effects\`

### `universe:functional.fold` — 6 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/fold.ph`](../../../../phalcom-core/core/universe/src/functional/fold.ph)

- **U059 — `AssociatedTypeInherentUnsupported`**, line 13, bytes `634..652` on `type StateType = S`: associated type bindings are valid only in conformance implementations
- **U060 — `AssociatedTypeInherentUnsupported`**, line 13, bytes `634..652` on `type StateType = S`: associated type bindings are valid only in conformance implementations
- **U061 — `AssociatedTypeInherentUnsupported`**, line 14, bytes `655..669` on `type Input = A`: associated type bindings are valid only in conformance implementations
- **U062 — `AssociatedTypeInherentUnsupported`**, line 14, bytes `655..669` on `type Input = A`: associated type bindings are valid only in conformance implementations
- **U063 — `AssociatedTypeInherentUnsupported`**, line 15, bytes `672..687` on `type Output = B`: associated type bindings are valid only in conformance implementations
- **U064 — `AssociatedTypeInherentUnsupported`**, line 15, bytes `672..687` on `type Output = B`: associated type bindings are valid only in conformance implementations

### `universe:functional.folds` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/folds.ph`](../../../../phalcom-core/core/universe/src/functional/folds.ph)

- **U065 — `AnnotationUnresolved`**, line 23, bytes `704..717` on `Folds.foldMap`: unresolved type \`Folds\`

### `universe:functional.higher_kinded` — 38 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/higher_kinded.ph`](../../../../phalcom-core/core/universe/src/functional/higher_kinded.ph)

- **U066 — `AssociatedTypeInherentUnsupported`**, line 22, bytes `823..837` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U067 — `AssociatedTypeInherentUnsupported`**, line 22, bytes `823..837` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U068 — `AnnotationUnresolved`**, line 28, bytes `1022..1039` on `Functor<Identity>`: trait \`Functor\` expects 1 type arguments, got 0
- **U069 — `AnnotationUnresolved`**, line 28, bytes `1030..1038` on `Identity`: conformance trait argument could not be resolved
- **U070 — `AnnotationUnsaturatedConstructor`**, line 28, bytes `1030..1038` on `Identity`: type constructor requires type arguments and cannot be used directly as a value type
- **U071 — `AnnotationUnresolved`**, line 31, bytes `1161..1182` on `Applicative<Identity>`: trait \`Applicative\` expects 1 type arguments, got 0
- **U072 — `AnnotationUnresolved`**, line 31, bytes `1173..1181` on `Identity`: conformance trait argument could not be resolved
- **U073 — `AnnotationUnsaturatedConstructor`**, line 31, bytes `1173..1181` on `Identity`: type constructor requires type arguments and cannot be used directly as a value type
- **U074 — `AnnotationUnresolved`**, line 37, bytes `1437..1452` on `Monad<Identity>`: trait \`Monad\` expects 1 type arguments, got 0
- **U075 — `AnnotationUnresolved`**, line 37, bytes `1443..1451` on `Identity`: conformance trait argument could not be resolved
- **U076 — `AnnotationUnsaturatedConstructor`**, line 37, bytes `1443..1451` on `Identity`: type constructor requires type arguments and cannot be used directly as a value type
- **U077 — `AnnotationUnresolved`**, line 42, bytes `1616..1631` on `Functor<Option>`: trait \`Functor\` expects 1 type arguments, got 0
- **U078 — `AnnotationUnresolved`**, line 42, bytes `1624..1630` on `Option`: conformance trait argument could not be resolved
- **U079 — `AnnotationUnsaturatedConstructor`**, line 42, bytes `1624..1630` on `Option`: type constructor requires type arguments and cannot be used directly as a value type
- **U080 — `AnnotationUnresolved`**, line 45, bytes `1747..1766` on `Applicative<Option>`: trait \`Applicative\` expects 1 type arguments, got 0
- **U081 — `AnnotationUnresolved`**, line 45, bytes `1759..1765` on `Option`: conformance trait argument could not be resolved
- **U082 — `AnnotationUnsaturatedConstructor`**, line 45, bytes `1759..1765` on `Option`: type constructor requires type arguments and cannot be used directly as a value type
- **U083 — `AnnotationUnresolved`**, line 51, bytes `2002..2015` on `Monad<Option>`: trait \`Monad\` expects 1 type arguments, got 0
- **U084 — `AnnotationUnresolved`**, line 51, bytes `2008..2014` on `Option`: conformance trait argument could not be resolved
- **U085 — `AnnotationUnsaturatedConstructor`**, line 51, bytes `2008..2014` on `Option`: type constructor requires type arguments and cannot be used directly as a value type
- **U086 — `AnnotationUnresolved`**, line 57, bytes `2241..2255` on `Functor<Chain>`: trait \`Functor\` expects 1 type arguments, got 0
- **U087 — `AnnotationUnresolved`**, line 57, bytes `2249..2254` on `Chain`: conformance trait argument could not be resolved
- **U088 — `AnnotationUnsaturatedConstructor`**, line 57, bytes `2249..2254` on `Chain`: type constructor requires type arguments and cannot be used directly as a value type
- **U089 — `AnnotationUnresolved`**, line 60, bytes `2368..2386` on `Applicative<Chain>`: trait \`Applicative\` expects 1 type arguments, got 0
- **U090 — `AnnotationUnresolved`**, line 60, bytes `2380..2385` on `Chain`: conformance trait argument could not be resolved
- **U091 — `AnnotationUnsaturatedConstructor`**, line 60, bytes `2380..2385` on `Chain`: type constructor requires type arguments and cannot be used directly as a value type
- **U092 — `AnnotationUnresolved`**, line 66, bytes `2647..2659` on `Monad<Chain>`: trait \`Monad\` expects 1 type arguments, got 0
- **U093 — `AnnotationUnresolved`**, line 66, bytes `2653..2658` on `Chain`: conformance trait argument could not be resolved
- **U094 — `AnnotationUnsaturatedConstructor`**, line 66, bytes `2653..2658` on `Chain`: type constructor requires type arguments and cannot be used directly as a value type
- **U095 — `AnnotationUnresolved`**, line 71, bytes `2813..2830` on `Functor<NonEmpty>`: trait \`Functor\` expects 1 type arguments, got 0
- **U096 — `AnnotationUnresolved`**, line 71, bytes `2821..2829` on `NonEmpty`: conformance trait argument could not be resolved
- **U097 — `AnnotationUnsaturatedConstructor`**, line 71, bytes `2821..2829` on `NonEmpty`: type constructor requires type arguments and cannot be used directly as a value type
- **U098 — `AnnotationUnresolved`**, line 74, bytes `2952..2973` on `Applicative<NonEmpty>`: trait \`Applicative\` expects 1 type arguments, got 0
- **U099 — `AnnotationUnresolved`**, line 74, bytes `2964..2972` on `NonEmpty`: conformance trait argument could not be resolved
- **U100 — `AnnotationUnsaturatedConstructor`**, line 74, bytes `2964..2972` on `NonEmpty`: type constructor requires type arguments and cannot be used directly as a value type
- **U101 — `AnnotationUnresolved`**, line 80, bytes `3241..3256` on `Monad<NonEmpty>`: trait \`Monad\` expects 1 type arguments, got 0
- **U102 — `AnnotationUnresolved`**, line 80, bytes `3247..3255` on `NonEmpty`: conformance trait argument could not be resolved
- **U103 — `AnnotationUnsaturatedConstructor`**, line 80, bytes `3247..3255` on `NonEmpty`: type constructor requires type arguments and cannot be used directly as a value type

### `universe:functional.journal` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/journal.ph`](../../../../phalcom-core/core/universe/src/functional/journal.ph)

- **U104 — `EnumCaseStaticBehaviorUnsupported`**, line 110, bytes `3439..3521` on `impl<T> Monoid for Journal<T> { @class empty -> Journal<T> { Journal<T>::Empty } }`: class-side trait requirements are not supported on enum targets

### `universe:functional.optics` — 12 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/optics.ph`](../../../../phalcom-core/core/universe/src/functional/optics.ph)

- **U105 — `AssociatedTypeInherentUnsupported`**, line 8, bytes `306..321` on `type Source = S`: associated type bindings are valid only in conformance implementations
- **U106 — `AssociatedTypeInherentUnsupported`**, line 8, bytes `306..321` on `type Source = S`: associated type bindings are valid only in conformance implementations
- **U107 — `AssociatedTypeInherentUnsupported`**, line 9, bytes `324..338` on `type Focus = T`: associated type bindings are valid only in conformance implementations
- **U108 — `AssociatedTypeInherentUnsupported`**, line 9, bytes `324..338` on `type Focus = T`: associated type bindings are valid only in conformance implementations
- **U109 — `AssociatedTypeInherentUnsupported`**, line 36, bytes `1623..1638` on `type Source = S`: associated type bindings are valid only in conformance implementations
- **U110 — `AssociatedTypeInherentUnsupported`**, line 36, bytes `1623..1638` on `type Source = S`: associated type bindings are valid only in conformance implementations
- **U111 — `AssociatedTypeInherentUnsupported`**, line 37, bytes `1641..1655` on `type Focus = T`: associated type bindings are valid only in conformance implementations
- **U112 — `AssociatedTypeInherentUnsupported`**, line 37, bytes `1641..1655` on `type Focus = T`: associated type bindings are valid only in conformance implementations
- **U113 — `AssociatedTypeInherentUnsupported`**, line 64, bytes `2684..2699` on `type Source = S`: associated type bindings are valid only in conformance implementations
- **U114 — `AssociatedTypeInherentUnsupported`**, line 64, bytes `2684..2699` on `type Source = S`: associated type bindings are valid only in conformance implementations
- **U115 — `AssociatedTypeInherentUnsupported`**, line 65, bytes `2702..2716` on `type Focus = T`: associated type bindings are valid only in conformance implementations
- **U116 — `AssociatedTypeInherentUnsupported`**, line 65, bytes `2702..2716` on `type Focus = T`: associated type bindings are valid only in conformance implementations

### `universe:functional.reader` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/reader.ph`](../../../../phalcom-core/core/universe/src/functional/reader.ph)

- **U117 — `GenericInferenceConflict`**, line 18, bytes `704..733` on `|environment| { environment }`: generic argument constraints conflict

### `universe:functional.reader_result` — 8 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/reader_result.ph`](../../../../phalcom-core/core/universe/src/functional/reader_result.ph)

- **U118 — `AssociatedTypeInherentUnsupported`**, line 6, bytes `208..228` on `type Environment = R`: associated type bindings are valid only in conformance implementations
- **U119 — `AssociatedTypeInherentUnsupported`**, line 6, bytes `208..228` on `type Environment = R`: associated type bindings are valid only in conformance implementations
- **U120 — `AssociatedTypeInherentUnsupported`**, line 7, bytes `231..245` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U121 — `AssociatedTypeInherentUnsupported`**, line 7, bytes `231..245` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U122 — `AssociatedTypeInherentUnsupported`**, line 8, bytes `248..266` on `type ErrorType = E`: associated type bindings are valid only in conformance implementations
- **U123 — `AssociatedTypeInherentUnsupported`**, line 8, bytes `248..266` on `type ErrorType = E`: associated type bindings are valid only in conformance implementations
- **U124 — `GenericInferenceConflict`**, line 23, bytes `1036..1064` on `self.run(environment).map(f)`: generic argument constraints conflict
- **U125 — `GenericInferenceConflict`**, line 26, bytes `1172..1203` on `self.run(environment).mapErr(f)`: generic argument constraints conflict

### `universe:functional.state` — 3 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/state.ph`](../../../../phalcom-core/core/universe/src/functional/state.ph)

- **U126 — `GenericInferenceConflict`**, line 29, bytes `1077..1082` on `state`: generic argument constraints conflict
- **U127 — `GenericInferenceConflict`**, line 35, bytes `1291..1293` on `()`: generic argument constraints conflict
- **U128 — `GenericInferenceConflict`**, line 41, bytes `1507..1509` on `()`: generic argument constraints conflict

### `universe:functional.state_result` — 8 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/state_result.ph`](../../../../phalcom-core/core/universe/src/functional/state_result.ph)

- **U129 — `AssociatedTypeInherentUnsupported`**, line 11, bytes `472..490` on `type StateType = S`: associated type bindings are valid only in conformance implementations
- **U130 — `AssociatedTypeInherentUnsupported`**, line 11, bytes `472..490` on `type StateType = S`: associated type bindings are valid only in conformance implementations
- **U131 — `AssociatedTypeInherentUnsupported`**, line 12, bytes `493..507` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U132 — `AssociatedTypeInherentUnsupported`**, line 12, bytes `493..507` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U133 — `AssociatedTypeInherentUnsupported`**, line 13, bytes `510..528` on `type ErrorType = E`: associated type bindings are valid only in conformance implementations
- **U134 — `AssociatedTypeInherentUnsupported`**, line 13, bytes `510..528` on `type ErrorType = E`: associated type bindings are valid only in conformance implementations
- **U135 — `GenericInferenceConflict`**, line 44, bytes `1871..1888` on `step.value.map(f)`: generic argument constraints conflict
- **U136 — `GenericInferenceConflict`**, line 50, bytes `2089..2109` on `step.value.mapErr(f)`: generic argument constraints conflict

### `universe:functional.writer` — 5 diagnostic(s)

Source: [`phalcom-core/core/universe/src/functional/writer.ph`](../../../../phalcom-core/core/universe/src/functional/writer.ph)

- **U137 — `AssociatedTypeInherentUnsupported`**, line 8, bytes `250..264` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U138 — `AssociatedTypeInherentUnsupported`**, line 8, bytes `250..264` on `type Value = T`: associated type bindings are valid only in conformance implementations
- **U139 — `AssociatedTypeInherentUnsupported`**, line 9, bytes `267..279` on `type Log = W`: associated type bindings are valid only in conformance implementations
- **U140 — `AssociatedTypeInherentUnsupported`**, line 9, bytes `267..279` on `type Log = W`: associated type bindings are valid only in conformance implementations
- **U141 — `GenericInferenceConflict`**, line 43, bytes `1513..1515` on `()`: generic argument constraints conflict

### `universe:option.option` — 4 diagnostic(s)

Source: [`phalcom-core/core/universe/src/option/option.ph`](../../../../phalcom-core/core/universe/src/option/option.ph)

- **U142 — `EnumRequirementMissing`**, line 23, bytes `883..899` on `Some(_ value: T)`: enum variant \`Some(_)\` does not implement required method \`match(some,none)\`
- **U143 — `EnumRequirementMissing`**, line 26, bytes `942..946` on `None`: enum variant \`None\` does not implement required method \`match(some,none)\`
- **U144 — `ReturnMismatch`**, line 189, bytes `6091..6185` on `match self { Option::Some(v) => Option::Some(f.call(v)) Option::None => self }`: tail expression result is not assignable to method's declared return type
- **U145 — `ReturnMismatch`**, line 210, bytes `6914..6994` on `match self { Option::Some(v) => f.call(v) Option::None => self }`: tail expression result is not assignable to method's declared return type

### `universe:reflection.child_module_table` — 2 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/child-module-table.ph`](../../../../phalcom-core/core/universe/src/reflection/child-module-table.ph)

- **U146 — `AnnotationUnsaturatedConstructor`**, line 3, bytes `122..126` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U147 — `AnnotationUnresolved`**, line 6, bytes `229..235` on `Module`: unresolved type \`Module\`

### `universe:reflection.export` — 2 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/export.ph`](../../../../phalcom-core/core/universe/src/reflection/export.ph)

- **U148 — `AnnotationUnresolved`**, line 4, bytes `139..149` on `ExportKind`: unresolved type \`ExportKind\`
- **U149 — `AnnotationUnresolved`**, line 5, bytes `169..175` on `Module`: unresolved type \`Module\`

### `universe:reflection.export_table` — 3 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/export-table.ph`](../../../../phalcom-core/core/universe/src/reflection/export-table.ph)

- **U150 — `AnnotationUnsaturatedConstructor`**, line 3, bytes `114..118` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U151 — `AnnotationUnsaturatedConstructor`**, line 4, bytes `136..140` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U152 — `AnnotationUnresolved`**, line 7, bytes `243..249` on `Export`: unresolved type \`Export\`

### `universe:reflection.message` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/message.ph`](../../../../phalcom-core/core/universe/src/reflection/message.ph)

- **U153 — `AnnotationUnsaturatedConstructor`**, line 5, bytes `158..162` on `List`: type constructor requires type arguments and cannot be used directly as a value type

### `universe:reflection.module` — 15 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/module.ph`](../../../../phalcom-core/core/universe/src/reflection/module.ph)

- **U154 — `AnnotationUnresolved`**, line 5, bytes `182..189` on `Message`: unresolved type \`Message\`
- **U155 — `AnnotationUnresolved`**, line 8, bytes `275..282` on `Package`: unresolved type \`Package\`
- **U156 — `AnnotationUnresolved`**, line 9, bytes `307..314` on `Package`: unresolved type \`Package\`
- **U157 — `AnnotationUnresolved`**, line 10, bytes `339..350` on `PackageInfo`: unresolved type \`PackageInfo\`
- **U158 — `AnnotationUnresolved`**, line 11, bytes `371..382` on `ExportTable`: unresolved type \`ExportTable\`
- **U159 — `AnnotationUnsaturatedConstructor`**, line 12, bytes `404..407` on `Map`: type constructor requires type arguments and cannot be used directly as a value type
- **U160 — `AnnotationUnsaturatedConstructor`**, line 13, bytes `433..437` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U161 — `AnnotationUnresolved`**, line 14, bytes `454..457` on `Uri`: unresolved type \`Uri\`
- **U162 — `AnnotationUnresolved`**, line 15, bytes `479..493` on `ModuleIdentity`: unresolved type \`ModuleIdentity\`
- **U163 — `AnnotationUnresolved`**, line 16, bytes `518..529` on `ExportTable`: unresolved type \`ExportTable\`
- **U164 — `AnnotationUnresolved`**, line 17, bytes `569..575` on `Export`: unresolved type \`Export\`
- **U165 — `AnnotationUnsaturatedConstructor`**, line 19, bytes `656..659` on `Map`: type constructor requires type arguments and cannot be used directly as a value type
- **U166 — `AnnotationUnsaturatedConstructor`**, line 20, bytes `689..693` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U167 — `AnnotationUnresolved`**, line 21, bytes `714..717` on `Uri`: unresolved type \`Uri\`
- **U168 — `AnnotationUnresolved`**, line 23, bytes `765..779` on `ModuleIdentity`: unresolved type \`ModuleIdentity\`

### `universe:reflection.module_dependency` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/module-dependency.ph`](../../../../phalcom-core/core/universe/src/reflection/module-dependency.ph)

- **U169 — `AnnotationUnresolved`**, line 3, bytes `117..123` on `Module`: unresolved type \`Module\`

### `universe:reflection.module_identity` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/module-identity.ph`](../../../../phalcom-core/core/universe/src/reflection/module-identity.ph)

- **U170 — `AnnotationUnresolved`**, line 3, bytes `107..110` on `Uri`: unresolved type \`Uri\`

### `universe:reflection.package_author` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/package-author.ph`](../../../../phalcom-core/core/universe/src/reflection/package-author.ph)

- **U171 — `AnnotationUnresolved`**, line 5, bytes `163..166` on `Uri`: unresolved type \`Uri\`

### `universe:reflection.package_info` — 5 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/package-info.ph`](../../../../phalcom-core/core/universe/src/reflection/package-info.ph)

- **U172 — `AnnotationUnsaturatedConstructor`**, line 6, bytes `191..195` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U173 — `AnnotationUnresolved`**, line 9, bytes `275..278` on `Uri`: unresolved type \`Uri\`
- **U174 — `AnnotationUnresolved`**, line 10, bytes `302..305` on `Uri`: unresolved type \`Uri\`
- **U175 — `AnnotationUnsaturatedConstructor`**, line 11, bytes `331..335` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U176 — `AnnotationUnresolved`**, line 13, bytes `389..404` on `PackageIdentity`: unresolved type \`PackageIdentity\`

### `universe:reflection.package_object` — 3 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/package-object.ph`](../../../../phalcom-core/core/universe/src/reflection/package-object.ph)

- **U177 — `AnnotationUnresolved`**, line 8, bytes `258..269` on `PackageInfo`: unresolved type \`PackageInfo\`
- **U178 — `AnnotationUnresolved`**, line 9, bytes `291..307` on `ChildModuleTable`: unresolved type \`ChildModuleTable\`
- **U179 — `AnnotationUnresolved`**, line 12, bytes `388..404` on `ChildModuleTable`: unresolved type \`ChildModuleTable\`

### `universe:reflection.package_requirement` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/package-requirement.ph`](../../../../phalcom-core/core/universe/src/reflection/package-requirement.ph)

- **U180 — `AnnotationUnresolved`**, line 4, bytes `159..170` on `PackageInfo`: unresolved type \`PackageInfo\`

### `universe:reflection.project` — 5 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/project.ph`](../../../../phalcom-core/core/universe/src/reflection/project.ph)

- **U181 — `AnnotationUnresolved`**, line 5, bytes `172..187` on `ProjectManifest`: unresolved type \`ProjectManifest\`
- **U182 — `AnnotationUnresolved`**, line 6, bytes `212..219` on `Package`: unresolved type \`Package\`
- **U183 — `AnnotationUnsaturatedConstructor`**, line 7, bytes `245..249` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U184 — `AnnotationUnresolved`**, line 8, bytes `279..285` on `Module`: unresolved type \`Module\`
- **U185 — `AnnotationUnresolved`**, line 9, bytes `307..322` on `ProjectIdentity`: unresolved type \`ProjectIdentity\`

### `universe:reflection.project_manifest` — 6 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/project-manifest.ph`](../../../../phalcom-core/core/universe/src/reflection/project-manifest.ph)

- **U186 — `AnnotationUnsaturatedConstructor`**, line 6, bytes `215..219` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U187 — `AnnotationUnresolved`**, line 9, bytes `299..302` on `Uri`: unresolved type \`Uri\`
- **U188 — `AnnotationUnresolved`**, line 10, bytes `326..329` on `Uri`: unresolved type \`Uri\`
- **U189 — `AnnotationUnresolved`**, line 11, bytes `349..352` on `Uri`: unresolved type \`Uri\`
- **U190 — `AnnotationUnsaturatedConstructor`**, line 14, bytes `445..449` on `List`: type constructor requires type arguments and cannot be used directly as a value type
- **U191 — `AnnotationUnsaturatedConstructor`**, line 15, bytes `475..479` on `List`: type constructor requires type arguments and cannot be used directly as a value type

### `universe:reflection.resolved_project_dependency` — 4 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/resolved-project-dependency.ph`](../../../../phalcom-core/core/universe/src/reflection/resolved-project-dependency.ph)

- **U192 — `AnnotationUnresolved`**, line 4, bytes `177..195` on `PackageRequirement`: unresolved type \`PackageRequirement\`
- **U193 — `AnnotationUnresolved`**, line 5, bytes `220..231` on `PackageInfo`: unresolved type \`PackageInfo\`
- **U194 — `AnnotationUnresolved`**, line 6, bytes `256..263` on `Package`: unresolved type \`Package\`
- **U195 — `AnnotationUnresolved`**, line 7, bytes `283..286` on `Uri`: unresolved type \`Uri\`

### `universe:reflection.structural` — 1 diagnostic(s)

Source: [`phalcom-core/core/universe/src/reflection/structural.ph`](../../../../phalcom-core/core/universe/src/reflection/structural.ph)

- **U196 — `AnnotationUnresolved`**, line 92, bytes `1952..1958` on `Lookup`: no intrinsic type-member operation is defined for \`UnorderedShape\`::\`Lookup\`

### `universe:text.errors` — 11 diagnostic(s)

Source: [`phalcom-core/core/universe/src/text/errors.ph`](../../../../phalcom-core/core/universe/src/text/errors.ph)

- **U197 — `ConstructorSuperRequired`**, line 7, bytes `159..268` on `new(validUpTo: Int, errorLength: Option<Int>) { _validUpTo = validUpTo _errorLength = errorLength }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U198 — `ConstructorSuperRequired`**, line 13, bytes `287..400` on `new(_ validUpTo: Int, _ errorLength: Option<Int>) { _validUpTo = validUpTo _errorLength = errorLength }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U199 — `ConstructorSuperRequired`**, line 39, bytes `926..1014` on `new(message: String, index: Option<Int>) { _message = message _index = index }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U200 — `ConstructorSuperRequired`**, line 45, bytes `1033..1100` on `new(message: String) { _message = message _index = None }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U201 — `ConstructorSuperRequired`**, line 51, bytes `1119..1188` on `new(_ message: String) { _message = message _index = None }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U202 — `ConstructorSuperRequired`**, line 81, bytes `1700..1788` on `new(message: String, index: Option<Int>) { _message = message _index = index }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U203 — `ConstructorSuperRequired`**, line 87, bytes `1807..1874` on `new(message: String) { _message = message _index = None }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U204 — `ConstructorSuperRequired`**, line 93, bytes `1893..1962` on `new(_ message: String) { _message = message _index = None }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U205 — `ConstructorSuperRequired`**, line 123, bytes `2487..2575` on `new(message: String, index: Option<Int>) { _message = message _index = index }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U206 — `ConstructorSuperRequired`**, line 129, bytes `2594..2661` on `new(message: String) { _message = message _index = None }`: subclass constructor must begin with an explicit \`super.new(...)\` call
- **U207 — `ConstructorSuperRequired`**, line 135, bytes `2680..2749` on `new(_ message: String) { _message = message _index = None }`: subclass constructor must begin with an explicit \`super.new(...)\` call

### `universe:time.calendar.datetime` — 2 diagnostic(s)

Source: [`phalcom-core/core/universe/src/time/calendar/datetime.ph`](../../../../phalcom-core/core/universe/src/time/calendar/datetime.ph)

- **U208 — `ArgumentMismatch`**, line 22, bytes `739..763` on `text.slice(0, separator)`: argument at position 1 does not match expected parameter type \`text\`
- **U209 — `ArgumentMismatch`**, line 22, bytes `777..813` on `text.slice(separator + 1, text.size)`: argument at position 1 does not match expected parameter type \`text\`

### `universe:time.calendar.period` — 2 diagnostic(s)

Source: [`phalcom-core/core/universe/src/time/calendar/period.ph`](../../../../phalcom-core/core/universe/src/time/calendar/period.ph)

- **U210 — `ArgumentMismatch`**, line 35, bytes `879..888` on `monthText`: argument at position 1 does not match expected parameter type \`text\`
- **U211 — `ArgumentMismatch`**, line 35, bytes `911..918` on `dayText`: argument at position 1 does not match expected parameter type \`text\`

## Remediation boundary

The census shows the diagnostics are present in the canonical source snapshot. It does not show that every one is a language defect. Some may be intended unsupported forms, incorrect Universe declarations, unresolved type-surface dependencies, or checker/inference defects. The source-image compiler path rejects snapshots containing errors, while the ordinary source and artifact paths can still compile the tested SlotMap entry; therefore this list does not by itself explain the current positive or negative compiler-parity failures.

For P2.R4, trace the diagnostics that affect the explicit source-image parity route and keep entry-local parity analysis separate. Do not hide the source diagnostics, change artifact persistence policy, or weaken parity to make either route pass. A source/checker fix should follow only after the listed construct is checked against the normative rule and a focused regression can distinguish the expected behavior.
