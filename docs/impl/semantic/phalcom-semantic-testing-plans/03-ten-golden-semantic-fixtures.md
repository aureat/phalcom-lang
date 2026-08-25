# Plan 3 — Ten Golden Semantic Composition Fixtures

> **Golden-fixture syntax invariant:** All block values use canonical pipe-block syntax: `|x| { ... }` or `|x| value`; zero-parameter blocks use `|| { ... }` or `|| value`. Golden programs must remain valid examples of the current language surface.

**Project:** Phalcom  
**Crate:** `phalcom-semantic`  
**Repository snapshot:** `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`

## 1. Definition

A golden semantic fixture is **not** a textual snapshot of `Debug` output.

It is a substantial Phalcom program plus a stable, named set of semantic observation points. A failure should identify which semantic boundary diverged.

Typical fixture:

- 4–8 classes/types;
- 5–12 callables;
- at least three call hops;
- multiple bindings;
- generics/flow/structure;
- 10–30 observation points;
- selected explanation and dependency assertions;
- either diagnostic-clean or one deliberately owned contradiction.

Do not serialize an entire `SemanticSnapshot` to `.snap`; that would freeze incidental IDs/order/debug formatting.

## 2. Required assertion layers

Every golden fixture should include:

### Leaf facts

Literals, constructors, parameter assumptions, structural literals.

### Intermediate calls

For important hops:

```text
receiver
selector/side
resolved CallableId
formal signature
specialized result
status/origin
```

### Composition

As relevant:

- generic substitution;
- branch join;
- pattern decomposition;
- alias/lambda normalization;
- variance;
- Self;
- publication.

### Binding contracts

- declared/current;
- consistency;
- causal invalidity.

### Explanations

2–5 strategically chosen structural explanation assertions per fixture.

### Dependencies

At least one meaningful dependency chain.

### Diagnostics

- valid fixture: no errors;
- deliberate-error fixture: exact owning diagnostic + no unrelated cascades.

## 3. Observation-point style

Each file should include a short manifest:

```text
LAW CHAIN
1. constructor -> concrete Established fact
2. call A specializes generic result
3. call B publishes it
4. flow joins compatible alternatives
5. binding validates broad contract without rewriting current
6. downstream call uses actual current knowledge
```

The manifest documents intent; executable assertions remain the oracle.

# 4. The ten golden programs

## GOLDEN-01 — Generic inheritance + variance + nested Self + long call chain

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }

class Box<+T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Maker<T> {
  wrap(_ value: T) -> Box<T> {
    Box<T>.new(value)
  }
}

class AnimalMaker<T> is Maker<T> {
  echo(_ value: T) -> T { value }
}

class SelfNode {
  @constructor new() {}
  boxed() -> Box<Self> {
    Box<Self>.new(self)
  }
}

class CatNode is SelfNode {}

class Service {
  @class
  makeCat(_ maker: AnimalMaker<Cat>) -> Box<Animal> {
    maker.wrap(maker.echo(Cat.new()))
  }

  @class
  makeNode() {
    CatNode.new().boxed()
  }
}

class Probe {
  @class
  run(_ maker: AnimalMaker<Cat>) {
    let animals: Box<Animal> = Service.makeCat(maker)
    let animal = animals.value()

    let nodeBox = Service.makeNode()
    let node = nodeBox.value()

    (animal, node)
  }
}
```

### Required semantic observation points

- generic superclass `AnimalMaker<T> is Maker<T>` and hierarchy dependency
- receiver substitution T=Cat through inherited `wrap`
- nested call `maker.wrap(maker.echo(Cat.new()))`
- covariant `Box<Cat>` relation to `Box<Animal>`
- `SelfNode#boxed -> Box<Self>` specialized on CatNode
- unannotated `Service.makeNode` publication
- final tuple facts and dependencies across the chain

## GOLDEN-02 — Flow + patterns + publication + abrupt exits

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }

class Factory {
  @class
  pair(_ flag: Bool) {
    if flag {
      (Cat.new(), 1)
    } else {
      (Dog.new(), 2)
    }
  }
}

class Service {
  @class
  choose(_ flag: Bool, _ abort: Bool) {
    let pair = Factory.pair(flag)
    let (animal, count) = pair

    let result = if abort {
      return (animal, count)
    } else {
      (animal, count)
    }

    result
  }
}

class Probe {
  @class
  run(_ flag: Bool, _ abort: Bool) {
    let result = Service.choose(flag, abort)
    let (animal, count) = result
    (animal, count)
  }
}
```

### Required semantic observation points

- Factory branch join gives `(Cat|Dog,Int)` without component widening
- tuple PatternDecomposition identities
- Service return edge vs continuing edge
- BodyExitFacts and publication Factory→Service→Probe
- callable dependency chain
- no accidental widening to `(Animal,Number)`

## GOLDEN-03 — Custom iteration + generics + nested calls + loop exits

```phalcom
class Entry<K, V> {
  _key: K
  _value: V

  @constructor
  new(_ key: K, _ value: V) {
    _key = key
    _value = value
  }

  key() -> K { _key }
  value() -> V { _value }
}

class Strange<A, B> {
  iteratorValue(_ cursor: Int) -> Entry<B, A> {
    mystery()
  }
}

class Decoder {
  @class
  decode(_ value: Int) -> Int { value }
}

class Service {
  @class
  collect(_ source: Strange<Int, String>, _ stop: Bool) {
    let last = 0

    for entry in source {
      let key = entry.key()
      let value = Decoder.decode(entry.value())
      last = value

      if stop {
        break
      }
    }

    (last, "done")
  }
}

class Probe {
  @class
  run(_ source: Strange<Int, String>, _ stop: Bool) {
    let result = Service.collect(source, stop)
    let (last, status) = result
  }
}
```

### Required semantic observation points

- iterator element specializes `Entry<B,A>` to `Entry<String,Int>`
- prove element is not inferred from generic position
- `entry.key()->String`, `entry.value()->Int`
- nested Decoder call
- loop preheader/body/break exit for `last`
- tuple publication and protocol/member dependencies

## GOLDEN-04 — Families + first-class callable + overloaded routes

```phalcom
class Formatter {
  render(_ value: Int) -> String { "int" }
  render(value: String) -> String { value }
  render() -> String { "empty" }
}

class Router {
  @class
  use(_ family: (Int) -> String, _ value: Int) -> String {
    family(value)
  }
}

class Service {
  @class
  run(_ formatter: Formatter) {
    let exact = formatter::render(_)
    let pattern = formatter::render(...)

    let a = Router.use(exact, 42)
    let b = pattern(value: "x")
    let c = pattern()

    (a, b, c)
  }
}

class Probe {
  @class
  run(_ formatter: Formatter) {
    Service.run(formatter)
  }
}
```

### Required semantic observation points

- Family capture itself does not resolve/activate a Method
- exact receiver/selector identity
- exact Family compatibility with `(Int)->String` only if formally justified
- pattern calls choose labeled/nullary routes
- distinct CallResolution identities despite common String result
- Service→Probe publication and Family dependencies

## GOLDEN-05 — Type lambda + constraint + variance

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }

class Box<+T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

type BoxOf = <T> =>> Box<T>

class Constrained<T> where T <: Animal {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Factory {
  @class
  cat() -> BoxOf<Cat> {
    Box<Cat>.new(Cat.new())
  }

  @class
  constrained() -> Constrained<Cat> {
    Constrained<Cat>.new(Cat.new())
  }
}

class Probe {
  @class
  run() {
    let box = Factory.cat()
    let broad: Box<Animal> = box
    let cat = box.value()

    let constrained = Factory.constrained()
    let constrainedCat = constrained.value()

    (cat, constrainedCat)
  }
}
```

### Required semantic observation points

- BoxOf lambda kind/binder/body
- `BoxOf<Cat>` beta-reduces to canonical `Box<Cat>`
- covariance validates `Box<Cat>` against `Box<Animal>`
- Constrained<Cat> satisfies `T <: Animal`
- constructor and member substitutions
- broad declaration/current precision

## GOLDEN-06 — Multi-module identity + inheritance + dispatch

```phalcom
// FILE app/model.ph
class Entity {
  id() -> Int { 1 }
}

class User is Entity {
  @constructor new() {}
  name() -> String { "Ada" }
}

type UserId = Int
export Entity, User, UserId

// FILE app/repository.ph
import app.model.User

class UserRepository {
  @class
  load() -> User {
    User.new()
  }
}

export UserRepository

// FILE app/service.ph
import app.model.User
import app.repository.UserRepository

class UserService {
  @class
  current() -> User {
    UserRepository.load()
  }
}

export UserService

// FILE app/controller.ph
import app.service.UserService

class Controller {
  @class
  run() {
    let user = UserService.current()
    let id = user.id()
    let name = user.name()
    (id, name)
  }
}

export Controller
```

### Required semantic observation points

- implement as four real WorkspaceFixture modules; comments delimit files only in this document
- module-qualified identities
- cross-module User→Entity hierarchy
- inherited `id` and own `name` dispatch
- Repository→Service→Controller publication
- linked-interface/callable/hierarchy dependencies
- alias provenance when UserId is observed

## GOLDEN-07 — Unknown/assumption boundary + independent proof

```phalcom
class CellNum {
  @constructor new() {}
  value() -> Int { 1 }
}

class Factory {
  @class
  known() -> CellNum {
    CellNum.new()
  }

  @class
  opaque() {
    mystery()
  }
}

class Service {
  @class
  run(_ chooseKnown: Bool) {
    let certain: CellNum = Factory.known()
    let uncertain: CellNum = Factory.opaque()

    let selected = if chooseKnown {
      certain
    } else {
      uncertain
    }

    let independent = certain.value()
    (selected, independent)
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    Service.run(flag)
  }
}
```

### Required semantic observation points

- certain validates from established constructor/call signature
- Factory.opaque remains epistemically incomplete
- uncertain may become Assumed only for an eligible UnknownReason
- branch must not hide reachable weakness
- `independent == Int` remains provable
- Service→Probe publication must preserve epistemic strength

## GOLDEN-08 — Variance + flow + deliberate refutation + recovery

```phalcom
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }

class Producer<+T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Shelter {
  @class
  choose(_ flag: Bool) {
    if flag {
      Producer<Cat>.new(Cat.new())
    } else {
      Producer<Dog>.new(Dog.new())
    }
  }
}

class Service {
  @class
  run(_ flag: Bool) {
    let producer: Producer<Animal> = Shelter.choose(flag)
    let animal = producer.value()

    let bad: Producer<String> = Shelter.choose(flag)
    let independent = 42

    (animal, independent)
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    Service.run(flag)
  }
}
```

### Required semantic observation points

- branch join of Producer<Cat>/Producer<Dog>
- variance relation to Producer<Animal>
- broad declaration validation without arbitrary widening
- bad Producer<String> refutation retains actual
- one owning diagnostic and independent Int
- downstream publication/dependencies

## GOLDEN-09 — Nested closure + capture + contextual typing + flow

```phalcom
class Apply {
  @class
  apply(_ value: Int, with f: (Int) -> Int) -> Int {
    f(value)
  }
}

class Service {
  @class
  run(_ flag: Bool) {
    let base = 1

    let transform = |x| {
      let local = if flag {
        x
      } else {
        base
      }
      local
    }

    let result = Apply.apply(42, with: transform)
    let stillBase = base
    (result, stillBase)
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    Service.run(flag)
  }
}
```

### Required semantic observation points

- contextual parameter knowledge for x
- capture of base uses outer BindingId and does not mutate it
- closure-internal branch join
- closure return synthesis
- callable compatibility at Apply.apply
- Service→Probe publication
- explanation/dependency chain spanning capture, flow and call

## GOLDEN-10 — Mixed nominal/structural pipeline with one local contradiction

```phalcom
class User {
  _name: String
  _score: Int

  @constructor
  new(_ name: String, _ score: Int) {
    _name = name
    _score = score
  }

  name() -> String { _name }
  score() -> Int { _score }
}

class Repository<T> {
  load() -> T {
    mystery()
  }
}

class UserRepository is Repository<User> {}

class Normalizer {
  @class
  normalize(_ user: User) -> #{name: String, score: Int} {
    #{name: user.name(), score: user.score()}
  }
}

class Pipeline {
  @class
  fetch(_ repo: UserRepository, _ fallback: Bool) {
    let user = if fallback {
      User.new("fallback", 0)
    } else {
      repo.load()
    }

    let record = Normalizer.normalize(user)
    record
  }
}

class Presenter {
  @class
  present(_ record: #{name: String, score: Int}) -> (String, Int) {
    ("user", 1)
  }
}

class Probe {
  @class
  run(_ repo: UserRepository, _ fallback: Bool) {
    let record = Pipeline.fetch(repo, fallback)

    let bad: String = record
    let presented = Presenter.present(record)
    let (label, count) = presented

    let independent = User.new("ok", 5).score()
    (label, count, independent)
  }
}
```

### Required semantic observation points

- generic inheritance Repository<User> and inherited load result
- branch between constructor User and repository result
- Normalizer structural record
- Pipeline publication
- deliberate record→String refutation keeps actual record
- Presenter call still analyzes from real record
- tuple decomposition
- independent constructor/member call remains Int
- diagnostic ownership and full dependency chain


# 5. Staging matrix

| Fixture | Primary composition | Status expectation |
|---|---|---|
| GOLDEN-01 | generic inheritance + variance + nested Self + publication | valid when generic inheritance/Self complete |
| GOLDEN-02 | flow + tuple pattern + exits + publication | valid capability |
| GOLDEN-03 | custom iteration + generics + loop exit | red/valid according to iteration semantics |
| GOLDEN-04 | Family + callable + overloaded shape | red until formal Family semantics complete |
| GOLDEN-05 | type lambda + variance + constraint | red until source lambda/constraint semantic path complete |
| GOLDEN-06 | multi-module identity + inheritance + long chain | `WorkspaceFixture` |
| GOLDEN-07 | Unknown/assumption boundary | intentionally incomplete epistemically |
| GOLDEN-08 | variance + flow + deliberate refutation | one deliberate error |
| GOLDEN-09 | closure/capture + contextual type + flow | red until closure semantic path complete |
| GOLDEN-10 | generic inheritance + record + publication + contradiction | one deliberate error |

Do not weaken a golden fixture to current broken behavior. It may remain a named red capability gate.

# 6. File layout

Under Plan 4:

```text
tests/semantic/golden/
  mod.rs
  generic_self_chain.rs
  flow_pattern_publication.rs
  iterator_chain.rs
  family_callable.rs
  type_lambda_constraints.rs
  workspace_chain.rs
  unknown_authority.rs
  variance_recovery.rs
  closure_flow.rs
  mixed_pipeline.rs
```

Only `tests/semantic.rs` is the Cargo integration target.

# 7. Implementation sequence

1. Land Plan-1 helper DSL.
2. Land Plan-4 tree.
3. Add source + law manifests.
4. Parse-validate every source at the implementation commit.
5. Add named source-site observations.
6. Add leaf/type assertions.
7. Add call/signature assertions.
8. Add flow/generic/pattern assertions.
9. Add explanation/dependency assertions.
10. Add exact diagnostic assertions.
11. Run each golden fixture in isolation.
12. Run the full semantic binary.

Commands:

```bash
cargo test -p phalcom-semantic --test semantic golden::generic_self_chain
cargo test -p phalcom-semantic --test semantic golden::flow_pattern_publication
cargo test -p phalcom-semantic --test semantic golden
cargo test -p phalcom-semantic --test semantic
```

# 8. Acceptance criteria

- all ten programs exist;
- sources parse at the implementation commit or are explicitly staged behind a named parser prerequisite;
- each has at least ten meaningful observations;
- intermediate facts are checked;
- exact call targets are checked where appropriate;
- generic/flow/pattern/Self/Family derivations check status/origin where meaningful;
- explanation checks are structural;
- dependency checks exist;
- valid fixtures enforce diagnostic cleanliness;
- intentional-error fixtures enforce error ownership and recovery;
- no full-snapshot textual golden files;
- no VM bytecode/loop-performance assertions.
