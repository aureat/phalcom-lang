# Phalcom LSP Implementation Spec 2
## Unified Expression Analysis and Canonical Dispatch

**Repository baseline:** `e2ec9e5fb6dc362786c9dd9593470feb47c91d94`  
**Depends on:** Spec 1 source identities/scopes  
**Primary crates:** `phalcom-lsp`, `phalcom-native-surface`; small VM-free selector helpers may move to `phalcom-common`  
**Goal:** every current expression shape uses one recursive inference path and one inheritance/side-aware dispatch resolver.

---

# 1. Scope

This specification replaces the current layered expression inference:

```text
infer_expr
infer_expr_with_returns
infer_expr_with_fields
```

with one exhaustive analyzer.

It also eliminates active side-blind member resolution from semantic inference.

This spec MUST implement:

- ordinary method calls;
- unqualified calls with compiler-compatible lexical precedence;
- getters;
- setters;
- `super`;
- inheritance;
- class-side vs instance-side dispatch;
- constructors;
- unary/binary operators;
- subscripts;
- collections/ranges;
- fields;
- method references where reasonably resolvable;
- string interpolation as a natural result of getter/operator analysis;
- stable native return-shape contracts.

Flow across statements/callable bodies is Spec 3.

---

# 2. Targeted baseline reads

| Purpose | Target |
|---|---|
| current split inferencers | `phalcom-lsp/src/semantic/infer.rs:20-245` |
| field facts / solver context | `phalcom-lsp/src/semantic/infer.rs:220-430` |
| summary expression special cases | `phalcom-lsp/src/semantic/infer.rs:1030-1270` |
| side-aware vs side-blind resolver | `phalcom-lsp/src/semantic/mod.rs:500-920` |
| surface dual maps | `phalcom-lsp/src/semantic/surface.rs:1-300` |
| ValueShape | `phalcom-lsp/src/semantic/facts.rs` |
| completion dispatch behavior | `phalcom-lsp/src/completion.rs:1-420` |
| compiler explicit/super/property/index sends | `phalcom-core/src/compiler/lib/expr.rs:360-760` |
| compiler operator/unary/bare-super semantics | `phalcom-core/src/compiler/lib/expr.rs:1160-1320` |
| constructor lowering | `phalcom-core/src/compiler/attributes.rs:2300-2445` |
| interpolation lowering | `phalcom-ast/src/parser.rs:2100-2225` |
| native surface types | `phalcom-native-surface/src/lib.rs:1-260` |
| native member list | `phalcom-native-surface/src/lib.rs:240-430` |
| core semantic assembly | `phalcom-lsp/src/semantic/core_source.rs:1-80` |

---

# 3. Add canonical `DispatchResolver`

Create:

```text
phalcom-lsp/src/semantic/dispatch.rs
```

Recommended types:

```rust
#[derive(Clone, Debug)]
pub enum DispatchReceiver {
    Instance(ClassId),
    ClassObject(ClassId),
    Super {
        lexical_class: ClassId,
        side: DispatchSide,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum LookupMode {
    Ordinary,
    Super,
}

#[derive(Clone, Debug)]
pub struct ResolvedDispatch {
    pub member: MemberSurface,
    pub receiver_class: ClassId,
    pub side: DispatchSide,
}
```

API:

```rust
pub fn resolve(
    classes: &BTreeMap<ClassId, ClassSurface>,
    receiver: &DispatchReceiver,
    selector: &str,
) -> Option<ResolvedDispatch>;
```

---

# 4. Ordinary dispatch algorithm

For `Instance(C)`:

1. side = `Instance`;
2. start at `C`;
3. inspect `members_by_side[(selector, Instance)]`;
4. walk superclass chain;
5. use core `Object` fallback exactly as current completion resolution does;
6. return the actual declaration owner.

For `ClassObject(C)`:

1. side = `Class`;
2. start at `C`;
3. inspect `members_by_side[(selector, Class)]`;
4. walk class-side inheritance according to the existing Phalcom surface model.

Do not create a `CallableId` using the receiver class unless the callable is actually declared there.

The returned `member.callable.owner` is authoritative.

---

# 5. `super` dispatch algorithm

`super` is not a value shape.

For:

```phalcom
super.foo(...)
super.foo
```

build:

```rust
DispatchReceiver::Super {
    lexical_class,
    side: current_callable.side,
}
```

Then:

1. find lexical class's superclass;
2. begin lookup there;
3. preserve side;
4. walk further ancestors.

Do not infer bare `super`.

A standalone `Expr::SuperVar` returns `Unknown`/invalid evidence but should normally be consumed by the parent send analyzer before generic expression evaluation.

---

# 6. Remove active side-blind semantic resolution

Current:

```rust
resolve_member_surface(...)
```

uses `ClassSurface.members`.

After migration:

- all dispatch-sensitive paths use `DispatchResolver`;
- constructor lookup uses it;
- summary lookup uses it;
- parameter fact targeting uses it;
- dependency extraction uses it.

`ClassSurface.members` may remain temporarily for non-dispatch compatibility, but mark it deprecated/internal and remove once no semantic path depends on it.

Long-term preferred surface:

```rust
members_by_side
```

plus explicit helper queries.

---

# 7. Unified analyzer API

Replace the stacked inferencers with a single recursive function.

Recommended:

```rust
pub struct ExprAnalysis {
    pub value: InferredValue,
}

pub struct AnalysisContext<'a> {
    pub module: &'a ModuleId,
    pub lexical_class: Option<&'a ClassId>,
    pub lexical_callable: Option<&'a CallableId>,
    pub dispatch_side: Option<DispatchSide>,
    pub scopes: &'a ScopeGraph,
    pub resolver: DispatchResolver<'a>,
    pub facts: FactProvider<'a>,
}

pub trait AnalysisSink {
    fn record_resolved_call(&mut self, call: ResolvedCall);
}
```

For now, a no-op sink can be used by direct hover/completion expression queries. Spec 3 will use the events.

Every child expression MUST recurse through the same `analyze_expr`.

Do not add another `*_with_*` wrapper.

---

# 8. Name resolution in `Expr::Var`

Use Spec 1's scope/name result.

Order must mirror compiler intent:

1. lexical binding;
2. import/module/global/class binding;
3. implicit-self message/getter candidate where language resolution requires;
4. unresolved.

A binding with `Unknown` value remains a resolved binding.

Do not fall through to a class merely because the binding's value is unknown.

---

# 9. `self`

`self` depends on callable side.

For an instance-side member:

```text
self -> Instance(current class)
```

For a class-side member:

```text
self -> ClassObject(current class)
```

The current implementation always produces an instance; correct this.

This is required for class-side method chaining and constructor internals.

---

# 10. Constructors

A class-side call whose resolved member is a source constructor/factory has exact result:

```text
Instance(owner/receiver construction class as required by inheritance semantics)
```

Be precise about inherited constructor behavior.

For:

```phalcom
Child.new(...)
```

when `new` is inherited from a parent constructor/factory, runtime allocation semantics should produce `Child` if the factory allocates through `self._$new`.

Therefore the result shape should generally be the *receiver class object*, not the declaration owner.

Test this against current constructor runtime behavior.

Do not infer constructor return from the constructor source body's tail expression.

---

# 11. Getter access

For `Expr::GetProperty`:

1. first determine whether the syntax is a qualified module/class path;
2. if so, resolve `Module.Class`/qualified class without treating it as a getter;
3. otherwise analyze receiver;
4. dispatch bare selector `property`;
5. use target callable/source/native return knowledge;
6. record resolved call event.

For `super.property`, use `DispatchReceiver::Super`.

This fixes:

```phalcom
Savings.rate
instance.rate
super.toString
```

when semantically appropriate.

---

# 12. Setter access

For `Expr::SetProperty`:

1. analyze receiver;
2. analyze RHS;
3. resolve canonical setter selector using existing selector helper;
4. record resolved call + argument evidence;
5. infer expression result according to actual setter-send semantics.

Do not blindly use the RHS unless Phalcom language semantics guarantee assignment expressions return RHS for this form.

If the compiler leaves the setter's return value, use setter summary/native contract.

Lock behavior with compiler fixture/tests.

---

# 13. Explicit method calls

For `Expr::MethodCall`:

1. analyze receiver recursively;
2. construct selector from static argument labels;
3. for each receiver alternative:
   - determine side;
   - resolve inherited declaration;
   - obtain return knowledge;
4. join bounded alternatives;
5. analyze every argument recursively;
6. record one resolved-call event per possible target.

Constructor calls override generic return summary as described above.

Dynamic/computed labels that prevent exact selector resolution should:

- still analyze receiver/arguments;
- produce `Unknown` dispatch result;
- set `dynamic_send` effect in Spec 3.

---

# 14. Unqualified calls

Do not assume implicit self.

Use Spec 1 bare-name resolution.

Cases:

### lexical callable binding

If the name resolves to a binding whose value knowledge represents a callable/block/family, analyze as callable invocation.

### class/global callable

Use the appropriate callable protocol if resolvable.

### implicit self

Only then dispatch:

```text
current `self` receiver
selector from call name + labels
```

with correct instance/class side.

This must mirror compiler precedence.

---

# 15. Operators

Create a canonical VM-free selector mapping.

Prefer sharing code rather than keeping another drift-prone map.

Required mappings include all current `BinaryOp` and `UnaryOp`.

For non-lazy binary op:

```text
left OP right
```

analyze as:

```text
dispatch left.selector(right)
```

For unary:

```text
-op value
```

dispatch the compiler-equivalent zero-arg selector.

For `and`/`or`:

- analyze left as Bool-like receiver;
- analyze RHS block/body lazily;
- result is joined according to actual Bool method/control-flow semantics if known;
- at minimum do not eagerly apply RHS flow state on paths where it may not execute.

Spec 3 refines flow.

---

# 16. Native semantic return contracts

Change `phalcom-native-surface`.

Replace/extend `NativeReturnKnowledge` with an actual VM-free descriptor.

Recommended first version:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeReturnShape {
    Unknown,
    Instance(&'static str),
    Receiver,
    ClassObject(&'static str),
    Argument(usize),
}
```

`NativeMember` gets:

```rust
pub return_shape: NativeReturnShape
```

Do not use formal `Type`.

Populate only stable contracts.

High-value initial contracts:

### exact Bool

- numeric `< <= > >=`;
- equality where runtime guarantees Bool;
- Bool `not()`;
- Bool predicates such as `isInteger`, `isNaN`, etc.

### exact String

- `String +(_)`;
- `String.toString` source getter already gives String via source summary;
- stable string-returning native primitives where guaranteed.

### exact Int

- hash/size/count primitives known to return Int;
- integer bit-count/length operations if guaranteed.

### receiver

Use only where primitive guarantees returning same receiver kind.

### constructors

Constructor result should mostly be handled by constructor semantics rather than hardcoding every `new` native entry.

Everything else remains `Unknown`.

Add table validation tests so runtime/native registrations and semantic surface remain aligned.

---

# 17. Native return integration in core surface

`core_source::build_core_surface` currently discards return contract detail from `NativeMember`.

Extend `MemberSurface` or a parallel callable-contract map with:

```rust
pub native_return: Option<NativeReturnShape>
```

Preferred design:

```rust
enum CallableReturnSource {
    SourceBody,
    Native(NativeReturnShape),
}
```

But do not make `MemberSurface` own inferred summaries. Keep declaration contract separate from solved facts.

`SemanticDb::return_for_callable` should query:

1. explicit/future declared type bridge (later);
2. exact constructor rule;
3. source callable summary;
4. native semantic return contract;
5. Unknown.

Centralize this precedence.

---

# 18. Subscript reads/writes

Use existing selector encoding in `selectors.rs`.

For `Expr::Index`:

1. analyze receiver;
2. derive bracket get selector from static labels/arity;
3. resolve dispatch;
4. analyze arguments;
5. return target summary/native contract.

For `SetIndex`:

1. resolve bracket setter selector;
2. analyze index args + RHS;
3. record call;
4. follow compiler expression-result semantics.

The compiler explicitly preserves RHS for `SetIndex`; match that behavior.

---

# 19. Collections and ranges

The current structured `ValueShape` is useful.

Keep it, but recursively call unified `analyze_expr`, not syntax-only inference.

### Tuple

- analyze positional and labeled values;
- expansions:
  - preserve known tuple pieces if safely projectable;
  - otherwise widen only affected structural lane rather than necessarily entire expression.

### Record

- explicit static labels retain field shapes;
- dynamic expansions conservatively widen unknown labels.

### List/Set

join element shapes using bounded union.

### Map

join key and value shapes.

### Range

join bound shapes.

The first implementation may be conservative for dynamic expansions, but must not skip child expression analysis/call events.

---

# 20. Fields

`Expr::Field` should query field identity/evidence from Spec 3.

Until Spec 3 lands, retain current field-value callback behind `FactProvider`.

Do not treat field names as lexical `BindingId`.

Fields should have class-qualified identity.

---

# 21. Method references / families

Minimum reasonable support:

### pinned exact selector

If receiver shape is known and pinned selector resolves to exactly one callable, return:

```rust
ValueShape::Callable(callable_id)
```

or introduce a distinct `BoundCallable`/`Family` shape if required to preserve runtime semantics.

### open family

An open base name can match multiple arities/selectors at call time.

Do not collapse it to one `CallableId`.

Recommended extension:

```rust
ValueShape::Family {
    receiver: Box<ValueShape>,
    base: String,
}
```

If adding this shape would make the current solver too broad, return `Unknown` for the value while still:

- analyzing receiver;
- recording semantic selector target for pinned refs;
- preserving future extension seam.

Do not guess.

---

# 22. String interpolation acceptance case

No interpolation special-case should be necessary after:

- `GetProperty(toString)` works;
- `Binary(Add)` works;
- `String +(_) -> String` native contract exists.

Add regression:

```phalcom
let name = "Ada"
let s = "Hello \(name)"
```

`s` must infer `String`.

Also:

```phalcom
class A {
  toString { "A" }
}
let a = A.new()
let s = "\(a)"
```

must infer `String`.

---

# 23. Eliminate `collect_expression_environment`

Once scope-aware variable resolution is active, remove the narrow pre-pass in:

- `phalcom-lsp/src/semantic/mod.rs:500-920`

Direct expression queries should provide:

- source snapshot/scopes;
- offset;
- fact provider.

Variable occurrences resolve themselves.

---

# 24. Tests

Create table-driven inference tests for every `Expr` variant.

Must include:

- literal shapes;
- self instance/class side;
- getter;
- inherited getter;
- class-side getter;
- same selector on instance/class side;
- explicit method;
- inherited method;
- chained method result;
- `super` method;
- `super` getter;
- constructor result;
- inherited constructor allocation result;
- unqualified implicit-self call;
- unqualified local callable precedence;
- unary operators;
- every binary operator family;
- string interpolation;
- index get/set;
- tuple/list/set/map/record/range child propagation;
- pinned method ref;
- dynamic selector/label conservative Unknown.

Add a compile-time/exhaustiveness policy:

Avoid `_ => Unknown` for `Expr`.

Every variant should have an explicit match arm. When a future AST variant is added, compilation should force a semantic decision.

---

# 25. Acceptance gate

Spec 2 is complete when:

1. only one recursive expression analyzer is used for editor inference;
2. no inheritance-sensitive path fabricates `CallableId` from receiver class;
3. side-blind resolver is absent from active dispatch inference;
4. getters and `super` getters work;
5. class-side/instance-side collisions are resolved correctly;
6. operators resolve through dispatch;
7. interpolated strings infer String through ordinary semantics;
8. subscript sends participate;
9. constructor result is exact;
10. native return contracts are conservative and canonical.
