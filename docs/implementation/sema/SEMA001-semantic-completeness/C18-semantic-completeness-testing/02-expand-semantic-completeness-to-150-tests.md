# Plan 2 — Expand `phalcom-semantic` Toward ~150 Focused Semantic-Completeness Tests

> **Fixture syntax invariant:** Every program in this plan uses canonical pipe-block syntax: `|x| { ... }` for statement bodies and `|x| value` for expression bodies; use `|| { ... }` / `|| value` for zero parameters. New fixtures must follow the same rule.

**Project:** Phalcom  
**Crate:** `phalcom-semantic`  
**Repository snapshot:** `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`  
**Purpose:** expand source-level semantic integration coverage after Plan 1 supplies deep oracles.  
**Target:** approximately 150 focused semantic scenarios, plus the separate ten golden fixtures in Plan 3.

## 1. Counting policy

“150” is a coverage target, not a reason to manufacture duplicates.

The historical capability baseline was 40. Current `main` already has 44 tests in `semantic_capabilities` because four generic epistemic tests were added. This plan defines **106 coverage slots**.

Implementation rule:

1. migrate existing tests under Plan 4;
2. map every existing source-level test onto this ledger;
3. let an existing test satisfy a slot only if it proves the same law at the required depth;
4. add new tests for uncovered slots;
5. aim for roughly 150 focused source/integration semantic tests;
6. retain lower-level algebra/DB tests, but do not inflate the source-semantic count with trivial mechanics.

The ten golden fixtures are additional composition gates.

## 2. Required status for each scenario

Every scenario must be classified:

```text
READY
  syntax and semantic product exist; execute now

RED-CAPABILITY
  syntax is current/parseable but semantic behavior is intentionally incomplete

STAGED
  source surface exists but a named prerequisite product is still landing

GATED
  feature/API is not available enough for a truthful source integration test;
  keep lower-level invariants and promote later
```

No retired parser syntax may be counted as a checker regression.

# 3. Coverage ledger — 106 slots

## Evidence / epistemic knowledge (8)

- [ ] E01 Literal synthesis is Known/Established/Syntax and never Assumed.
- [ ] E02 Constructor result specializes `Self` and is established from ConstructorSemantics.
- [ ] E03 Callable parameter knowledge uses the correct assumed/established policy and never pretends source declaration is a proof.
- [ ] E04 `NoTypeEvidence` plus an eligible source contract may become Assumed/DeveloperAnnotation.
- [ ] E05 `UnresolvedName` plus a source contract must remain Unknown; the annotation cannot launder a checker failure.
- [ ] E06 Reachable Unknown is absorbing at a branch join.
- [ ] E07 Dynamic boundary remains Dynamic with its exact `DynamicReason`.
- [ ] E08 Mixed established/assumed flow joins use weakest support and Flow origin.

## Bindings / contracts / recovery (7)

- [ ] B01 Broad declaration validates a narrow established initializer without widening current.
- [ ] B02 Incompatible declaration keeps actual current and becomes Refuted.
- [ ] B03 Mutable `let` assignment updates current without rewriting declared contract.
- [ ] B04 Invalid later assignment remains actual evidence in recovery flow.
- [ ] B05 Shadowed binding has distinct identity and does not alter the outer binding.
- [ ] B06 Pattern leaves derive from scrutinee current knowledge, not merely declared contract.
- [ ] B07 Causal invalidity suppresses only dependent facts; independent siblings remain analyzable.

## Dispatch, constructors and Self (8)

- [ ] D01 Instance-side exact dispatch.
- [ ] D02 Class-side exact dispatch.
- [ ] D03 Wrong dispatch side is not laundered to Dynamic/Unknown.
- [ ] D04 Inherited dispatch resolves defining callable while preserving dynamic receiver specialization.
- [ ] D05 Constructor `Self` on subclass receiver specializes to subclass.
- [ ] D06 Nested `Box<Self>` specializes transitively.
- [ ] D07 Labeled and positional selector shapes remain distinct.
- [ ] D08 Bad argument does not erase independently fixed call return.

## Method families / method references (9)

- [ ] F01 Exact getter Family capture preserves receiver and selector identity.
- [ ] F02 Exact method Family preserves slot/label shape.
- [ ] F03 Pattern Family remains a pattern at capture time and does not preselect a target.
- [ ] F04 Exact Family invocation resolves current callable and specializes result.
- [ ] F05 Family through a local binding retains receiver/selector semantics.
- [ ] F06 Instance-side and class-side Family capture remain distinct.
- [ ] F07 Inherited Family invocation records the correct callable/hierarchy dependencies.
- [ ] F08 Wrong Family call shape fails without fabricating a result.
- [ ] F09 Generic/Self-returning Family member specializes from bound receiver and arguments.

## Generic inference (9)

- [ ] G01 Method-owned type parameter identity is distinct from class-owned parameter with same spelling.
- [ ] G02 Single argument solves identity return.
- [ ] G03 Two independent variables solve independently.
- [ ] G04 Receiver application supplies class generic substitution to member return.
- [ ] G05 Receiver and argument constraints combine across nested applications.
- [ ] G06 Expected result may constrain but may not overwrite stronger evidence.
- [ ] G07 Conflicting constraints report inference conflict/refutation and preserve independent facts.
- [ ] G08 Underconstrained result remains Unknown rather than using expected annotation as fact.
- [ ] G09 Generic callable publication propagates specialized results through A→B→C.

## Generic constraints (7)

- [ ] C01 Class `where T <: Number` accepts Int and rejects String.
- [ ] C02 Method-owned bound is checked at invocation.
- [ ] C03 Multiple subtype constraints are conjunctive.
- [ ] C04 Equivalence `T == U` solves compatible facts and rejects incompatible facts.
- [ ] C05 Class and method generic constraints coexist without owner confusion.
- [ ] C06 Generic superclass template propagates substitution/constraints.
- [ ] C07 Bound violation diagnostic is distinct from ordinary cannot-infer.

## Kinds and type lambdas (9)

- [ ] K01 Int has kind Type.
- [ ] K02 Box has kind Type -> Type.
- [ ] K03 Multi-parameter constructor has expected arrow kind and full application returns Type.
- [ ] K04 Wrong-kinded value annotation is a kind/application error, not generic Unknown.
- [ ] K05 `<T> =>> Box<T>` lowers to Lambda of kind Type -> Type.
- [ ] K06 Alpha-equivalent lambdas normalize equivalently.
- [ ] K07 Type-lambda application beta-reduces to canonical applied type.
- [ ] K08 Nested lambda scoping avoids accidental capture.
- [ ] K09 Wrong arity/kind application has precise diagnostic.

## Variance and generic inheritance (6)

- [ ] V01 Covariant Producer<Cat> <: Producer<Animal>.
- [ ] V02 Contravariant Consumer relation reverses.
- [ ] V03 Invariant generic refuses both directions absent equality.
- [ ] V04 Broad covariant binding contract preserves narrower current fact.
- [ ] V05 Generic superclass template substitutes through inheritance.
- [ ] V06 Nested variance/callable occurrence agrees with declaration variance checking.

## Structural types and patterns (8)

- [ ] S01 Nested tuple retains every constituent.
- [ ] S02 Tuple labels participate in canonical structure.
- [ ] S03 Record row exposes exact fields/types.
- [ ] S04 Heterogeneous list forms union element without arbitrary top widening.
- [ ] S05 Nested tuple pattern recursively decomposes.
- [ ] S06 List/rest pattern produces independent leaves where supported.
- [ ] S07 Pattern refutation owns a diagnostic and does not rewrite scrutinee.
- [ ] S08 Structural result survives generic call + destructuring.

## Flow and loops (8)

- [ ] L01 Same-type if arms collapse to one type with Flow provenance.
- [ ] L02 Heterogeneous if arms join to union.
- [ ] L03 Return arm is excluded from continuing value join.
- [ ] L04 Throw arm is excluded from continuing value join.
- [ ] L05 Nested branch joins compose transitively.
- [ ] L06 Loop post-state contains zero-iteration preheader and body/backedge.
- [ ] L07 break contributes exit; continue contributes backedge/retest.
- [ ] L08 Captured block write does not affect outer flow merely because block was created.

## Iteration semantics (5)

- [ ] I01 Custom iterator element comes from iteratorValue result, not generic position.
- [ ] I02 Receiver generic substitution specializes iterator element.
- [ ] I03 Structured for pattern decomposes protocol element.
- [ ] I04 Multi-lane/index iteration keeps lane identities independent where implemented.
- [ ] I05 Unknown/dynamic protocol result stays incomplete; no collection-shape heuristic fabricates element.

## Callables, blocks and publication (7)

- [ ] P01 Context supplies closure parameter type at the correct epistemic strength.
- [ ] P02 Closure result synthesis builds callable return type.
- [ ] P03 Callable subtyping validates parameter contravariance/result covariance.
- [ ] P04 Captured binding identity survives nested closure analysis.
- [ ] P05 Unannotated method tail publishes caller-visible return.
- [ ] P06 Multi-hop publication reanalyzes dependent callers.
- [ ] P07 Recursive publication terminates honestly with recursive/fixpoint Unknown.

## Fields and aliases (5)

- [ ] A01 Instance field follows declaration/current authority rules.
- [ ] A02 Generic field read specializes from receiver application.
- [ ] A03 Inherited/Self-typed field read preserves receiver specialization where supported.
- [ ] A04 Transparent alias normalizes for relations while retaining provenance/navigation identity.
- [ ] A05 Generic/nested alias normalizes without duplicate identity or infinite cycle.

## Modules and workspace identity (4)

- [ ] M01 Same leaf name in two modules has distinct semantic identity.
- [ ] M02 Imported superclass/call target resolves to exporting module.
- [ ] M03 Re-export/alias preserves target identity while local source resolves correctly.
- [ ] M04 Cross-module callable/generic dependencies invalidate consumer and not unrelated modules.

## Diagnostics / explanations / dependencies / advisory / native / reflection (6)

- [ ] X01 Explanation rule/status/origin agrees with expression knowledge.
- [ ] X02 One contradiction owns one primary diagnostic; dependent suppression does not kill independent analysis.
- [ ] X03 Dependency set contains consumed callable/hierarchy/surface products and excludes unrelated declarations.
- [ ] X04 Advisory evidence can enrich but cannot convict or establish formal knowledge.
- [ ] X05 Native signature result uses NativeSignature origin and agrees with canonical source/native type identity.
- [ ] X06 Reflection metadata, once source API is implemented, agrees with canonical types and does not alter dispatch/layout; otherwise keep as a gated metadata test.


# 4. What to cover inside each area

## 4.1 Evidence and epistemic knowledge

Cover:

- `TypeKnowledge::{Known, Unknown, Dynamic}`;
- `EvidenceStatus::{Established, Assumed}`;
- every source-reachable `EvidenceOrigin`;
- exact `UnknownReason`;
- exact `DynamicReason`;
- contract assumption eligibility;
- weakest-support join;
- expected-type influence without fabrication;
- body fact versus public callable contract;
- explanation/knowledge consistency;
- bounded provenance where stable.

Negative laws:

- Assumed never silently strengthens to Established;
- Dynamic never silently becomes formal Known;
- non-eligible Unknown never becomes a declaration assumption;
- advisory evidence never convicts.

## 4.2 Bindings / contracts

Cover:

- declared versus current;
- source-contract origin;
- no-contract inferred initializer;
- callable parameter contract;
- contextual closure parameter;
- pattern binding;
- every `BindingConsistency` state that has a production path;
- causal invalidity;
- shadowing identity;
- mutable assignment;
- downstream reads after refutation.

## 4.3 Dispatch and `Self`

Test the full path:

```text
receiver knowledge
  -> dispatch side
  -> selector identity
  -> inheritance traversal
  -> CallableId
  -> formal signature
  -> receiver/class generic substitution
  -> Self specialization
  -> call-result knowledge
  -> explanation/dependencies
```

`Self` cases:

- constructor;
- instance return;
- inherited return;
- override;
- nested `Box<Self>`;
- tuple/callable Self;
- class vs instance role;
- generic subclass.

## 4.4 Method families

Current parser supports hashless `receiver::name`, `receiver::name(_)`, and structural `receiver::name(...)` forms. Do not copy retired `receiver::#name` examples from older docs.

Cover:

- exact getter/nullary/method;
- labels/slots;
- pattern family;
- receiver capture;
- **no target resolution at capture**;
- exact invocation;
- pattern invocation;
- inherited member;
- class/instance receiver;
- Family through local/parameter/return;
- generic member;
- Self return;
- wrong shape/missing target;
- dependencies;
- runtime `Family` versus reflection `MethodFamily`.

## 4.5 Generics

Cover:

- owner/index identity;
- class vs callable owner;
- lexical shadowing;
- argument constraints;
- receiver constraints;
- nested applied types;
- expected-result constraints;
- multiple variables;
- underconstraint;
- conflict;
- lower-level occurs check;
- return specialization;
- publication;
- generic-signature dependencies;
- assumption-strength propagation.

## 4.6 Generic constraints

Use the ratified `where` model:

- `T <: Upper`;
- lower-bound form when current source implementation supports it;
- `T == U`;
- multiple constraints;
- class and callable owners;
- generic superclass templates;
- expected-result interaction;
- precise diagnostics.

Do not invent the deferred finite-domain constraint syntax.

## 4.7 Kinds and type constructors

Cover:

- `Type`;
- arrow kinds;
- multi-parameter constructor kinds;
- proper-type requirement in value annotation positions;
- partial application only where current source/product path supports it;
- non-constructor application;
- too many arguments;
- argument-kind mismatch;
- type-parameter kind annotation such as `F: Type -> Type`;
- kind of type lambda.

## 4.8 Type lambdas

Cover:

- `<T> =>> ...`;
- binder kinds;
- body;
- alpha equivalence;
- beta reduction;
- residual application where supported;
- nested binders;
- enclosing generic capture;
- accidental-capture prevention;
- wrong arity/kind;
- alias to lambda.

## 4.9 Variance / generic inheritance

Cover:

- `+T`, `-T`, invariant;
- relation checks;
- broad binding contracts;
- nested applications;
- callable-position occurrence validation;
- generic supertype template;
- inherited specialized dispatch;
- source provenance.

## 4.10 Structural types / patterns

Cover:

- tuple elements/labels;
- nested tuples;
- unions;
- exact record rows/fields/tails;
- list element union;
- callable structural type;
- recursive tuple/list/rest patterns;
- decomposition origin;
- pattern mismatch;
- structural values through generics and flow;
- normalization/deduplication.

## 4.11 Flow / loops

Cover:

- arm facts before join;
- join result/status/origin;
- reachability;
- return/throw exclusion;
- nested joins;
- binding merge;
- zero-iteration loop path;
- backedge/widening;
- break exit;
- continue edge;
- captured writes;
- Unknown/Dynamic joins;
- flow explanations;
- body exits.

Out of scope: VM bytecode shape, sacred inliner speed, closure allocation and runtime loop efficiency.

## 4.12 Iteration

Cover:

```text
iterable receiver
 -> iteration protocol
 -> iteratorValue(_)
 -> receiver generic substitution
 -> element TypeKnowledge
 -> pattern decomposition
 -> loop binding
 -> dependency products
```

Explicitly reject a generic-position shortcut unless the formal protocol says so.

## 4.13 Callables / blocks / publication

Cover:

- explicit closure syntax (`|x| { ... }`, `|| { ... }`) for standalone closure values;
- contextual parameters;
- return synthesis;
- callable labels/rest;
- callable subtyping;
- capture identity;
- nested/returned blocks where supported;
- unannotated callable publication;
- fixed-point caller reanalysis;
- recursive cycles;
- explicit broad return preserving narrower body facts.

## 4.14 Fields / aliases

Fields:

- instance/class side;
- declared/current;
- generic field;
- inherited field;
- constructor write;
- invalid write;
- Self;
- field signature identity.

Aliases:

- simple;
- generic;
- applied;
- tuple/union/record/callable/lambda;
- chains;
- cycles;
- semantic equivalence versus navigation/provenance identity.

## 4.15 Modules / incrementality

Use real `analyze_workspace` and a `WorkspaceFixture`.

Cover:

- ModuleId identity;
- same leaf in different modules;
- import/alias/re-export;
- inheritance;
- cross-module call;
- generic signature across modules;
- linked-interface dependencies;
- revision edits;
- stable product fingerprints stopping propagation;
- changed products invalidating only consumers.

## 4.16 Diagnostics / explanations / advisory / native / reflection

Cover:

- exact diagnostic code/site/count;
- cascade discipline;
- causal suppression;
- explanation rule/status/origin/evidence/parents;
- dependency set;
- advisory/formal separation;
- native-signature origin and canonical agreement.

Reflection source tests are **GATED** until the corresponding source/runtime API can expose truthful semantic products. When available, cover:

```text
nominal identity
applied origin/arguments
generic parameter owner/index/name/kind/variance/constraints
callable parameters/result
tuple elements
record fields
union members
Self representation
type-lambda binder/body
reflected canonical type == checker canonical type
reflection metadata never changes selector identity/layout/allocation
```

# 5. Ready complex Phalcom program catalog

These programs are intended to be copied into fixtures after parser validation at the implementation commit. They are deliberately multi-mechanism. Use one program for several focused laws rather than copying it blindly for every assertion.

### P01 — Broad contract preserves precision

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }

class Probe {
  @class
  run() {
    let actual = Cat.new()
    let animal: Animal = actual
    let again = animal
  }
}
```

**Required observations**

- `Cat.new()` is `Cat/Established/ConstructorSemantics`.
- `animal.declared == Animal`, `animal.current == Cat`, consistency `Validated`.
- `again` follows current knowledge rather than blindly using the declaration.
- No mismatch diagnostic.

### P02 — Refutation and recovery

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }

class Probe {
  @class
  run() {
    let x: String = Cat.new()
    let y = x
    let independent = 42
  }
}
```

**Required observations**

- `x.current == Cat`, `x.declared == String`, consistency `Refuted`.
- Exactly one owning initializer/type diagnostic.
- `y` must not become String merely because of the refuted declaration.
- `independent` remains `Int/Established/Syntax`.

### P03 — Receiver-derived generic substitution

```phalcom
class Box<T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Probe {
  @class
  run() {
    let ints = Box<Int>.new(42)
    let strings = Box<String>.new("x")
    let a = ints.value()
    let b = strings.value()
  }
}
```

**Required observations**

- Inspect Box's generic parameter owner/index/kind.
- `ints : Box<Int>`, `strings : Box<String>`.
- Both calls resolve the same `value()` callable but specialize independently.
- `a == Int`, `b == String`; no substitution leakage.

### P04 — Class generic versus method generic shadowing

```phalcom
class Envelope<T> {
  _stored: T
  @constructor new(_ value: T) { _stored = value }
  stored() -> T { _stored }

  echo<T>(_ value: T) -> T {
    value
  }
}

class Probe {
  @class
  run() {
    let e = Envelope<String>.new("inside")
    let a = e.stored()
    let b = e.echo(42)
    let c = e.echo("outside")
  }
}
```

**Required observations**

- Class-owned and callable-owned `T` must have distinct owners/IDs.
- `stored()` uses class T=String.
- `echo(42)` uses method T=Int; `echo("outside")` uses method T=String.
- Solving method T never mutates class T.

### P05 — Generic bound success and failure

```phalcom
class NumericBox<T> where T <: Number {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Probe {
  @class
  run() {
    let good = NumericBox<Int>.new(42)
    let goodValue = good.value()
    let bad = NumericBox<String>.new("no")
    let independent = 1
  }
}
```

**Required observations**

- `NumericBox<Int>` satisfies the declared bound and `goodValue == Int`.
- `NumericBox<String>` owns a precise bound/application diagnostic.
- Do not collapse the bound violation to a generic Unknown.
- `independent` remains analyzable.

### P06 — Covariance under a broad contract

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }

class Producer<+T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Probe {
  @class
  run() {
    let cats = Producer<Cat>.new(Cat.new())
    let animals: Producer<Animal> = cats
    let catAgain = cats.value()
    let animalView = animals.value()
  }
}
```

**Required observations**

- Prove `Producer<Cat> <: Producer<Animal>`.
- Inspect declared/current on `animals`.
- `catAgain == Cat`.
- Ensure no unrelated heuristic fabricates precision for `animalView`.

### P07 — Type lambda and canonical application

```phalcom
class Box<T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

type Boxer = <T> =>> Box<T>

class Probe {
  @class
  run() {
    let a: Boxer<Int> = Box<Int>.new(42)
    let b: Boxer<String> = Box<String>.new("x")
    let ai = a.value()
    let bs = b.value()
  }
}
```

**Required observations**

- `Boxer` is a lambda of kind `Type -> Type`.
- `Boxer<Int>` beta-reduces/canonicalizes to `Box<Int>`.
- Both contracts validate.
- `ai == Int`, `bs == String`.

### P08 — Nested generic structural result

```phalcom
class Pairer {
  @class
  pair<A, B>(_ a: A, _ b: B) -> (A, B) {
    (a, b)
  }
}

class Probe {
  @class
  run() {
    let nested = Pairer.pair(1, Pairer.pair("x", 2.5))
    let (first, rest) = nested
    let (second, third) = rest
  }
}
```

**Required observations**

- Inner solve is `(String,Float)`; outer is `(Int,(String,Float))`.
- Nested tuple structure is exact.
- Pattern leaves are Int/String/Float with independent BindingIds.
- Inspect GenericInference and PatternDecomposition origins.

### P09 — Branch union under broad declaration

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }

class Probe {
  @class
  run(_ flag: Bool) {
    let chosen: Animal = if flag {
      Cat.new()
    } else {
      Dog.new()
    }
    let useAgain = chosen
  }
}
```

**Required observations**

- Both constructor leaves are established.
- Branch joins to `Cat | Dog` with Flow origin.
- Declared `Animal` validates the union.
- Current fact is not rewritten to Animal.

### P10 — Abrupt branch exits

```phalcom
class Probe {
  @class
  choose(_ returnEarly: Bool, _ throwEarly: Bool) {
    let x = if returnEarly {
      return 1
    } else {
      if throwEarly {
        throw "stop"
      } else {
        2
      }
    }

    let after = x
    after
  }
}
```

**Required observations**

- Return and throw go to `BodyExitFacts`, not continuing value join.
- `x`/`after` derive only from the continuing `2` path.
- Inspect nested flow explanation/graph.
- No false union with return/throw payloads.

### P11 — Loop fixed-point semantics

```phalcom
class Probe {
  @class
  run(_ keepGoing: Bool, _ chooseText: Bool) {
    let value: Number | String = 1

    while keepGoing {
      if chooseText {
        value = "changed"
      } else {
        value = 2.5
      }
    }

    let after = value
  }
}
```

**Required observations**

- Preheader Int participates because the loop may execute zero times.
- Body contributes String/Float.
- Post-loop current uses reachable fixed-point knowledge.
- This is a semantic flow test only; no bytecode/performance assertion.

### P12 — break / continue flow

```phalcom
class Probe {
  @class
  run(_ skip: Bool) {
    let x = 1

    for n in [1, 2, 3] {
      if skip {
        x = "continued"
        continue
      }
      x = 2.5
      break
    }

    let after = x
  }
}
```

**Required observations**

- Preheader Int is an exit possibility.
- Continue String contributes to backedge/retest knowledge.
- Break Float contributes to exit.
- Post-loop `after` joins reachable exits.

### P13 — Custom iterable generic protocol

```phalcom
class Weird<A, B> {
  iteratorValue(_ cursor: Int) -> B {
    mystery()
  }
}

class Probe {
  @class
  run(_ weird: Weird<String, Int>) {
    for value in weird {
      let observed = value
    }
  }
}
```

**Required observations**

- Resolve iterator element through `iteratorValue(_)` contract.
- Specialize B to Int from `Weird<String,Int>`.
- `value` and `observed` must be Int and must not be String.
- Track dependency on the relevant callable signature.

### P14 — Exact Family through a binding

```phalcom
class Formatter {
  render(_ value: Int) -> String { "rendered" }
}

class Probe {
  @class
  run(_ formatter: Formatter) {
    let family = formatter::render(_)
    let result = family(42)
  }
}
```

**Required observations**

- Family capture retains receiver/selector and does not invoke the target.
- `family(42)` resolves `Formatter#render(_)`.
- `result == String`.
- Inspect Family/call dependencies.

### P15 — Pattern Family with overloaded shapes

```phalcom
class Formatter {
  render() -> String { "zero" }
  render(_ value: Int) -> String { "one" }
  render(value: String) -> String { "named" }
}

class Probe {
  @class
  run(_ formatter: Formatter) {
    let family = formatter::render(...)
    let a = family()
    let b = family(1)
    let c = family(value: "x")
  }
}
```

**Required observations**

- Capture retains a selector pattern rather than one Method.
- Each invocation resolves a different callable shape.
- All return String but call-resolution identities differ.
- No target should be preselected at capture.

### P16 — Exact Family wrong-shape recovery

```phalcom
class Formatter {
  render(value: Int) -> String { "ok" }
}

class Probe {
  @class
  run(_ formatter: Formatter) {
    let exact = formatter::render(value)
    let good = exact(value: 1)
    let bad = exact(1)
    let independent = 42
  }
}
```

**Required observations**

- Exact Family stores labeled selector identity.
- `good` resolves and returns String.
- `bad` is a shape failure, not a fabricated alternate-selector type mismatch.
- `independent` remains Int.

### P17 — Contextual closure typing

```phalcom
class Apply {
  @class
  twice(_ value: Int, with transform: (Int) -> Int) -> Int {
    transform(transform(value))
  }
}

class Probe {
  @class
  run() {
    let addOne = |x| { x + 1 }
    let result = Apply.twice(40, with: addOne)
  }
}
```

**Required observations**

- Contextual callable expectation supplies `x` according to formal assumption policy.
- Closure body/result becomes Int.
- `addOne` has callable shape `(Int)->Int`.
- `result == Int`.

### P18 — Capture without speculative execution

```phalcom
class Probe {
  @class
  run() {
    let x = 1

    let action = || {
      x = "changed"
    }

    let before = x
  }
}
```

**Required observations**

- Closure captures outer BindingId.
- Closure-body analysis may record a write in closure context.
- Creating the closure alone must not change outer flow.
- `before == Int`.

### P19 — Callable publication chain

```phalcom
class C {
  @class build() { (1, "x") }
}
class B {
  @class forward() { C.build() }
}
class A {
  @class start() { B.forward() }
}
class Probe {
  @class
  run() {
    let result = A.start()
    let (number, text) = result
  }
}
```

**Required observations**

- Publish C `(Int,String)` then B then A.
- Dependency chain A→B→C.
- `number == Int`, `text == String`.
- Companion incremental test edits C and checks reanalysis.

### P20 — Recursive publication honesty

```phalcom
class Probe {
  @class first() { second() }
  @class second() { third() }
  @class third() { first() }

  @class
  run() {
    let x = first()
    let independent = "still-known"
  }
}
```

**Required observations**

- Recursive cycle terminates.
- `x` remains Unknown with recursive/fixpoint reason.
- No Unit/nominal return is fabricated.
- `independent == String`.

### P21 — Generic alias + tuple

```phalcom
type NamedPair<T> = (String, T)

class Factory {
  @class
  pair<T>(_ value: T) -> NamedPair<T> {
    ("value", value)
  }
}

class Probe {
  @class
  run() {
    let pair = Factory.pair(42)
    let (name, value) = pair
  }
}
```

**Required observations**

- Generic alias application normalizes to `(String,Int)`.
- Generic return specializes correctly.
- Pattern decomposition gives String/Int.
- Retain alias source identity for navigation/provenance where product supports it.

### P22 — Generic inheritance substitution

```phalcom
class Sequence<T> {
  first() -> T { mystery() }
}

class Names<T> is Sequence<T> {}

class Probe {
  @class
  run(_ names: Names<String>) {
    let first = names.first()
  }
}
```

**Required observations**

- Store superclass template `Sequence<T>`.
- Substitute T=String through inheritance.
- Inherited target is Sequence#first; result is String.
- Record hierarchy and callable dependencies.

### P23 — Nested Self specialization

```phalcom
class Box<T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Node {
  @constructor new() {}
  boxed() -> Box<Self> {
    Box<Self>.new(self)
  }
}

class Leaf is Node {}

class Probe {
  @class
  run() {
    let box = Leaf.new().boxed()
    let leaf = box.value()
  }
}
```

**Required observations**

- `Self` is owner-relative to Node.
- Dispatch on Leaf specializes `Box<Self>` to `Box<Leaf>`.
- `box.value()` returns Leaf.
- Inspect Self owner/side/role and substitution order.

### P24 — Record + generic + branch composition

```phalcom
class Envelope<T> {
  @class
  make(_ value: T) -> #{value: T, ok: Bool} {
    #{value: value, ok: true}
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    let record = if flag {
      Envelope<Int>.make(42)
    } else {
      Envelope<Int>.make(7)
    }

    let observed = record
  }
}
```

**Required observations**

- Generic result is a structural row `value:Int, ok:Bool`.
- Both arms have equivalent structural type.
- Join preserves one record type.
- Inspect exact row fields rather than only `TypeData::Record`.


# 6. Turning complex programs into focused tests

A complex source may back multiple tests, but each test has one primary law.

For P04, for example:

```text
test A: class T and method T have distinct owner identity
test B: stored() specializes from class T
test C: echo(42) specializes callable-owned T
test D: solving method T does not mutate class T
test E: both echo invocations resolve the same callable identity
```

Keep source visible in the module, perhaps as one `const SOURCE: &str`.

# 7. Implementation order

1. Land Plan 1 helper infrastructure.
2. Land Plan 4 organization before adding dozens of files.
3. Put the 106-slot ledger in developer test documentation.
4. Map migrated tests to slots.
5. Tag every slot READY / RED-CAPABILITY / STAGED / GATED.
6. Fill authority, generics, flow, patterns, publication and families first.
7. Then constraints/kinds/type lambdas/variance/Self.
8. Then workspace/incrementality.
9. Promote reflection tests only when the product API exists.
10. Keep coverage status current.

# 8. Commands

After Plan 4:

```bash
cargo fmt --check
cargo test -p phalcom-semantic --test semantic --no-run
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-semantic --test semantic capabilities::generics
cargo test -p phalcom-semantic --test semantic capabilities::method_families
cargo test -p phalcom-semantic --test semantic capabilities::flow_branches
cargo test -p phalcom-semantic --test semantic integration::workspace
cargo clippy -p phalcom-semantic --tests -- -D warnings
```

# 9. Acceptance criteria

Plan 2 is complete when:

- roughly 150 focused source/integration semantic scenarios exist after deduplication;
- all 106 slots are mapped to a test or explicitly GATED/STAGED with a concrete prerequisite;
- every major area has a granular sub-feature list;
- method families, kinds, lambdas, constraints, variance, Self, aliases, fields, modules and epistemic states have meaningful breadth;
- complex Phalcom programs exercise composition;
- positive, negative, causal and recovery laws are asserted;
- unsupported syntax is never mislabeled as a checker regression;
- lower-level algebra and source integration remain distinct;
- no helper reimplements the checker;
- runtime loop/codegen efficiency remains out of scope.
