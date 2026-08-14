# Effects and Observational Equivalence

Two expressions are interchangeable only when no permitted program context can distinguish them according to language observations. This is the semantic foundation of optimization, refactoring, static proving, and effect analysis.

## 1. Define observations first

Potential Phalcom observations include:

- returned surface value;
- normal versus thrown error;
- local/field/global mutation;
- allocation/object identity;
- IO trace;
- module initialization effects;
- reflected class/method/source metadata;
- method-table mutation;
- yield/scheduling behavior;
- termination/divergence;
- external process/filesystem/network effects;
- timing where API exposes it.

If an observation is omitted, optimizer may erase it accidentally.

## 2. Contextual equivalence

Conceptually:

```text
e₁ ≈ e₂
```

when every allowed program context `C[-]` produces indistinguishable observations:

```text
Obs(C[e₁]) = Obs(C[e₂])
```

This is very strong. Implementations usually establish sufficient local conditions rather than prove contextual equivalence directly.

## 3. Effect dimensions

Useful categories:

```text
readsLocal / writesLocal
readsField / writesField
readsGlobal / writesGlobal
allocates
mayThrow
mayNonLocalReturn
performsIO
reflects
mutatesMethodTables
callsUnknown
mayYield
mayBlockThread
mayCancel
callsNative
```

A single `pure: bool` is too lossy for a reflective concurrent object language.

## 4. Read/write footprints

```text
R(e) = resources expression may read
W(e) = resources expression may write
```

A simple sufficient condition for commuting deterministic expressions is roughly:

```text
W(e1) ∩ (R(e2) ∪ W(e2)) = ∅
W(e2) ∩ (R(e1) ∪ W(e1)) = ∅
```

plus absence of observable control/IO/allocation/yield interactions. Alias analysis determines whether object footprints overlap.

## 5. Allocation effect

Allocation may be observable through identity, constructor effects, resource limits, finalization/weak references if ever exposed, reflection, or GC APIs. State actual policy before allocation DCE.

## 6. Exception effect

Reordering `e1; e2` changes behavior if either can throw because first error and preceding side effects can differ. Value equivalence alone is insufficient.

## 7. Reflection effect

Reflection can expose source range, holder, parameter names/annotations, class identity, method enumeration, call stack, and module ownership. Optimization must know which are stable observations.

## 8. Unknown calls

In open dynamic code, unresolved send may conservatively mean:

```text
callsUnknown = true
mayReadAnything = true
mayWriteAnything = true
mayThrow = true
mayYield = maybe, according to runtime/native rules
```

Precision can improve when target is resolved and summary known. Absence of inferred effect is not proof of absence.

## 9. Higher-order effects

If method invokes block parameter `p`, summary can record:

```text
invokesParameter(p)
```

Then passed block's latent effects enter reachable caller effects. This matches existing Phalcom LSP callable-summary direction.

## 10. Yield as interference

Yield may not mutate state itself, but permits other fibers to run. Shared mutable facts can therefore become invalid across it.

Track:

```text
mayYield
```

for optimization/proof even on one OS thread.

## 11. Blocking versus yielding

A blocking native call can stall entire cooperative scheduler. It is different from suspension:

```text
mayYield
mayBlockThread
```

should be distinct effects.

## 12. Callback effect

Native or higher-order code that invokes unknown Phalcom callback can transitively produce arbitrary language effects. Track callback capability explicitly instead of assuming native operation's own body is simple.

## 13. Send versus intrinsic equivalence

Replacing send with intrinsic must preserve:

```text
lookup target
access behavior
argument laziness/eagerness
return value
errors
observable reflection hooks
state effects
scheduling
```

Primitive integer fast path may need guards + fallback.

## 14. Refactoring equivalence

Tooling uses different equivalence relations:

```text
runtime contextual equivalence
binding/identity equivalence
source-map/debug equivalence
formatting normal-form equivalence
```

Rename must preserve target identities; formatter must preserve semantic parse; optimizer preserves runtime observations.

## 15. Algebraic laws are conditional

`x + 0 -> x` is not universally valid if `+` is overridable or removing send changes errors/effects. Mathematical identity must be lifted through language semantics.

Likewise, commutativity of values does not imply expressions can be reordered if effects differ.

## 16. Effect polymorphism intuition

A higher-order method can be pure except for effects of passed block:

```text
map(block): effects = iteration effects ∪ effects(block invocation)
```

Future typed/effect systems may express this parametrically; current semantic summaries can model it operationally.

## 17. Observational testing

Compare values, side-effect order, exceptions, identity comparisons, reflection results, and scheduling events between optimized/unoptimized paths.

## 18. Competency checks

1. Why is same returned value insufficient equivalence?
2. What condition helps commute two stateful expressions?
3. Why can yield invalidate shared-state refinement without direct write?
4. How can reflection make metadata semantic?
5. When may send become primitive opcode safely?
