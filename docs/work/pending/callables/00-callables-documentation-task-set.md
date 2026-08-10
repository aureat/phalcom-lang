# Task Set 1 — Canonical Callables Documentation

**Project:** Phalcom
**Repository:** `aureat/phalcom-lang`
**Status:** Implementation-ready documentation task-set
**Execution:** May run in parallel with Task Sets 2–4
**Primary output:** a new canonical callable-specification folder
**Code changes:** none
**Other documentation migrations:** explicitly out of scope for this task-set

---

## 1. Mission

Create a new canonical documentation folder:

```text
docs/spec/callables/
```

containing exactly these initial documents:

```text
docs/spec/callables/README.md
docs/spec/callables/method.md
docs/spec/callables/function.md
docs/spec/callables/closure.md
docs/spec/callables/bound-method.md
docs/spec/callables/family.md
```

`README.md` is the general callable-model specification and index. The other five files specify one callable-related object each.

Every document must link to every directly related document using relative Markdown links. `README.md` must link to all five object-specific documents, and every object-specific document must link back to `README.md`.

This task-set intentionally does **not** update the repository's older Block, Function, F.3, callable-return, or ADR documentation. Those documents may contradict this new model; the user will sanction broader documentation migration after reviewing this new folder.

---

## 2. Normative terminology

The new documentation must use these terms consistently:

```text
block
    a brace-delimited syntactic/lexical region

Closure
    a first-class executable value carrying compiled code and lexical captures

Method
    reified holder-owned behavior requiring a compatible receiver

BoundMethod
    an exact Method paired with a compatible receiver

Function
    a sealed abstract VM-backed callable whose remaining runtime inputs
    are only explicitly supplied call arguments

Family
    a bound `::` method-family reference that performs lookup when called
```

Never use `Block` as the public class name for closures.

Never describe `Method` as a subtype of `Function`.

Never describe `BoundMethod` as surfacing as `Block`.

---

## 3. Canonical hierarchy

`README.md` must make this hierarchy prominent:

```text
Object
├── Method                         sealed/final core class
└── Function                       abstract, sealed core class
    ├── Closure                    sealed/final
    ├── BoundMethod                sealed/final
    └── Family                     sealed/final
```

Also explain the deliberate distinction between:

```text
Function
```

and:

```text
an arbitrary user object that defines `call`
```

Application syntax remains ordinary message syntax:

```phalcom
f(a, b)
```

is semantically:

```phalcom
f.call(a, b)
```

A user-defined object may therefore be callable without being a `Function`.

---

## 4. Universal rest/spread notation rule

Every file that discusses argument expansion or rest parameters must state:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
foo(*values)
foo(**labels)
foo(***arguments)

method(*rest) { ... }
method(**rest) { ... }
method(***rest) { ... }

|head, *tail| { ... }
```

The syntax:

```text
args...
```

is **never** rest/spread syntax in Phalcom.

Do not call `...` a spread operator.

---

## 5. `README.md` required contents

`README.md` must be both the index and the general semantic specification.

It must contain at least:

### 5.1 Hierarchy and definitions

Document the hierarchy from section 3 and define each object in one concise paragraph.

### 5.2 Execution-context distinction

Explain:

```text
Method
    still needs a receiver

Function
    needs only explicit arguments
```

Explain why `BoundMethod` and `Family` are Functions but `Method` is not.

### 5.3 `self`

Specify:

- a Method receives dynamic `self` from activation;
- a BoundMethod supplies the stored receiver as that `self`;
- a Closure created in a Method/Closure lexical environment captures the current `self` value when one exists;
- ordinary sends inside a Method dispatch dynamically on `self`;
- `super` remains lexically anchored to the defining Method holder;
- Closure capture of `self` does not change `super` semantics.

### 5.4 Call transport versus parameter acceptance

Explain that the common Function gateway can transport a complete argument shape while concrete Functions may reject parts of that shape.

For example:

```phalcom
const f = |head, *tail| { ... }
```

accepts positional arguments and positional spread, but rejects a non-empty labeled lane.

### 5.5 Return model

Specify:

- ordinary Methods return their final expression;
- Closures return their final expression;
- empty bodies return `()`;
- `return` means `return ()`;
- `return value` returns from the current Method/Closure activation only;
- there is no implicit non-local return;
- `None` is absence, not no-result.

### 5.6 Constructors

State the conceptual compiler-generated model:

```text
@constructor source declaration
    → generated class-side factory
    → allocate instance
    → call generated/hidden instance initializer as an ordinary Method
    → ignore initializer return value
    → return allocated instance
```

The initializer itself obeys ordinary Method runtime return semantics.

However, because the source declaration is marked `@constructor`, the compiler rejects:

```phalcom
return value
```

inside the constructor initializer body. Bare `return` is allowed and returns `()` from the initializer; the generated factory still returns the allocated instance.

Emphasize that this is a compiler restriction attached to generated-constructor semantics, not a special Method return opcode.

### 5.7 Links

Link to:

- [`Method`](method.md)
- [`Function`](function.md)
- [`Closure`](closure.md)
- [`BoundMethod`](bound-method.md)
- [`Family`](family.md)

---

## 6. `method.md` required contents

Document `Method` as reified exact behavior.

Required semantic fields/concepts:

```text
holder
selector
parameter shape
implementation
lexical super anchor
access authority
```

A Method is not a Function because it still lacks a receiver.

Document:

```phalcom
method.bind(receiver)
method.invokeOn(receiver, ***arguments)
```

with these rules:

- both validate receiver compatibility before execution;
- holder/subclass compatibility is required;
- class-side Methods use analogous metaclass ancestry;
- holderless public Method binding/invocation is rejected unless a later specification explicitly creates a safe holderless category;
- `invokeOn` executes the exact reified Method and does not redispatch its selector;
- ordinary sends inside the Method still dynamically dispatch on the supplied receiver;
- `super` remains anchored to the Method's defining holder.

Document constructor-generation semantics from section 5.6.

Link to `README.md`, `bound-method.md`, `function.md`, and `closure.md`.

---

## 7. `function.md` required contents

Document `Function` as:

```text
abstract
sealed
VM-backed
complete callable
```

Required canonical call gateway:

```phalcom
call(***arguments)
```

Application:

```phalcom
f(args)
```

is ordinary message syntax and remains observationally equivalent to:

```phalcom
f.call(args)
```

The `call` base-selector family is final for Function descendants.

Document:

```phalcom
callWith(arguments)
```

as exactly:

```phalcom
self(***arguments)
```

It must not define a second binder.

Do **not** document legacy finite `call`, `call(_)`, `call(_,_)`, ... overload families as the canonical design.

Do **not** put scalar `arity` or generic `name` on the base Function protocol in this specification.

Explain that native implementation may use allocation-free argument views even though the public gateway is spelled `***arguments`.

Link to `README.md`, `closure.md`, `bound-method.md`, and `family.md`.

---

## 8. `closure.md` required contents

Document canonical Closure literals:

```phalcom
|| { ... }
|x| { ... }
|x, y| { ... }
|x| expression
|head, *tail| { ... }
```

For now Closure parameters support:

```text
fixed positional parameters
optional terminal positional rest parameter
```

and reject:

```text
labeled parameters
**rest
***rest
multiple positional-rest parameters
fixed parameters after *rest
```

Rest capture:

```text
zero residual positional arguments  → ()
one or more residual arguments      → Tuple
```

Outgoing calls may still use `*`, `**`, or `***`; the Closure validates the resulting shape. A complete pack containing labels is rejected by a positional-only Closure.

Document lexical capture and local `return`.

Document bare-brace trailing zero-argument Closure sugar:

```phalcom
resource.withLock {
    work()
}
```

as contextual postfix-send sugar for:

```phalcom
resource.withLock || {
    work()
}
```

Braces are not general Closure expressions.

Link to `README.md`, `function.md`, and `method.md`.

---

## 9. `bound-method.md` required contents

Define:

```text
BoundMethod = exact Method + validated receiver
```

Document:

- `BoundMethod < Function`;
- it contains no synthetic cloned Method;
- it contains no nested/rebound BoundMethod wrapper requirement;
- execution uses the underlying exact Method;
- receiver lookup happened when the Method was obtained/bound, not again when the BoundMethod is called;
- ordinary sends within the exact Method remain dynamically dispatched on the stored receiver;
- lexical access and `super` remain those of the original Method;
- no direct BoundMethod rebinding API initially.

If underlying-Method reflection is shown, do not invent unresolved public APIs beyond what has already been ratified.

Link to `README.md`, `method.md`, and `function.md`.

---

## 10. `family.md` required contents

Document both current reference forms conceptually:

```phalcom
obj::name
obj::#selector(...)
```

A Family is a Function because it contains the bound receiver/reference context and only needs explicit call arguments to proceed.

Document:

- open Family: target selector is derived at call time from the family base name plus actual call shape;
- pinned Family: the pinned selector remains authoritative according to the existing method-reference semantics;
- Family calling uses the common Function activation gateway;
- Family callability must not be specified through intentional `doesNotUnderstand` misses;
- target dispatch after routing is an ordinary Phalcom message send.

Link to `README.md`, `function.md`, and `method.md`.

---

## 11. Cross-link requirements

At minimum:

```text
README.md
  → method.md
  → function.md
  → closure.md
  → bound-method.md
  → family.md

method.md
  → README.md
  → function.md
  → closure.md
  → bound-method.md

function.md
  → README.md
  → closure.md
  → bound-method.md
  → family.md

closure.md
  → README.md
  → function.md
  → method.md

bound-method.md
  → README.md
  → function.md
  → method.md

family.md
  → README.md
  → function.md
  → method.md
```

Use relative links only.

---

## 12. Explicit non-goals

Do not, in this task-set:

- modify Rust code;
- modify `.ph` source;
- rewrite existing ADR history;
- update every stale Block document;
- resolve additional reflection APIs;
- add type-system syntax;
- document Closure labeled parameters;
- reintroduce `args...`;
- add new callable subclasses.

---

## 13. Acceptance checklist

The task is complete when:

- `docs/spec/callables/` exists;
- all six required files exist;
- all files use the new hierarchy consistently;
- all direct relationships are cross-linked;
- no file describes `Method < Function`;
- no file describes BoundMethod as Block;
- no file uses `args...` as spread/rest syntax;
- Closure rest is positional-only;
- constructor generation/initializer semantics are documented correctly;
- Family uses common Function call routing, not dNU;
- `None` and Unit are distinguished;
- no unrelated documentation was changed.
