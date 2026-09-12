# LANG004.C1.P2.R1 — Setter Value Lane Invocation ABI Remediation Plan

> **Status:** PLAN ONLY — no implementation changes are performed by this document.
>
> **Repository:** `aureat/phalcom-lang`
>
> **Prepared against verified remote `main`:** `9639f27d320e361bb8c56793974a4c8aaaf18dce`
>
> **Parent implementation commit:** `420cf292b8048e6e10b451eb229bf583c114e6f3` — `feat: complete LANG004 selector and family activation`
>
> **Scope:** Correct the runtime invocation-shape model for setters, especially labeled subscript setters and `Family.set(Tuple, value)`, without changing selector identity, setter syntax, physical stack order, or the current rule that assignment/set expressions are typed/evaluated as `Unit`.
>
> **Non-goal:** This remediation does **not** revisit assignment-expression result semantics. Property assignment, subscript assignment, `Family.value =`, direct Family subscript assignment, and related set expressions remain `Unit` for now.

---

# 1. Executive objective

The implementation already has the correct **language model** and mostly the correct **physical stack order** for setters:

```text
receiver
structural positional values...
structural labeled values...
setter RHS
```

The defect is in runtime metadata. `ArgumentView` currently models only:

```text
receiver
positionals...
labeled values...
```

and therefore several setter paths pretend the setter RHS is an extra positional value. This is harmless for positional-only shapes but corrupts labeled setter shapes such as:

```phalcom
setter.set((key, debug: true), rhs)
```

because the stack is physically:

```text
receiver | key | true | rhs
```

while the current `ArgumentView` metadata can describe it as:

```text
positionals = [key, true]
labeled(debug) = rhs
```

instead of:

```text
structural positionals = [key]
labeled(debug) = true
setter value = rhs
```

The architectural fix is to introduce a single runtime authority for invocation lanes, conceptually:

```rust
InvocationLayout {
    structural_positionals: usize,
    labels: Box<[Symbol]>,
    setter_value: bool,
}
```

and make `ArgumentView` expose:

```rust
positional_count()
positional(...)
labels()
labeled_value(...)
has_setter_value()
setter_value(...)
physical_arity()
```

Selector identity remains unchanged. `SignatureKind::Setter` and `SignatureKind::SubscriptSet(n)` continue to encode selector kind and structural index arity. The setter RHS remains outside `Selector.slots`.

The second half of the remediation is to centralize selector-shaped dispatch so static sends, dynamic packs, Family forwarding, AssociatedFamily forwarding, BoundMethodFamily activation, and shape-aware native primitives cannot independently reconstruct setter lanes.

---

# 2. Ratified invariants

These invariants are implementation constraints, not suggestions.

## I-R1 — Assignment/set expressions remain `Unit`

Do not change semantic or runtime result policy in this remediation.

```phalcom
object.property = value
object[index] = value
family.value = value
family[index] = value
```

remain `Unit` expressions.

Any existing tests asserting `Unit` should stay authoritative.

## I-R2 — Selector identity remains unchanged

Setter syntax and selector identity remain:

```text
property=(_)
[_, debug]=(_)
```

The assigned value is not an ordinary selector slot.

```rust
Selector {
    base: SelectorBase::Subscript,
    kind: SelectorKind::SubscriptSet,
    slots: [
        SelectorSlot::Positional,
        SelectorSlot::Label("debug"),
    ],
}
```

No `_` representing the RHS may be appended to `slots`.

## I-R3 — Physical stack order remains unchanged

For subscript setters the runtime layout is:

```text
receiver
structural positionals...
structural labeled values...
setter value
```

The RHS remains last.

No stack-reordering workaround may move the setter value before labels merely to satisfy the old `ArgumentView` model.

## I-R4 — Setter value is its own lane

The setter RHS is neither positional nor labeled.

It is a distinguished trailing lane in the runtime invocation layout.

The runtime must be able to distinguish:

```text
structural positional count
structural labeled count / labels
setter-value presence
```

without arithmetic such as “subtract one positional for setters.”

## I-R5 — Getter/method/subscript-get cannot carry setter lane

The following are invalid internal states:

```text
Getter + setter-value
Method + setter-value
SubscriptGet + setter-value
```

The following must require a setter lane:

```text
Setter
SubscriptSet
```

Malformed internal combinations must fail with an invariant/internal error, not be silently interpreted.

## I-R6 — `SignatureKind` remains the selector-kind authority

Do not redesign:

```rust
SignatureKind::Getter
SignatureKind::Setter
SignatureKind::Method(n)
SignatureKind::SubscriptGet(n)
SignatureKind::SubscriptSet(n)
```

`SubscriptSet(n)` continues to mean:

```text
n structural index selector slots
+
one dedicated setter value
```

Its physical parameter arity may continue to be derived as `n + 1`.

## I-R7 — One invocation-layout authority

All runtime paths that need argument shape must use the same representation.

The following must not maintain independent setter-specific positional arithmetic:

- static `Bytecode::Invoke`
- dynamic `InvokePack`
- dynamic `InvokeSubscriptSetPack`
- `Family.set(_)`
- `Family.value=(_)`
- `Family.set(_,_)`
- direct `family[...] = rhs`
- bound `Family`
- `AssociatedFamily`
- `BoundMethodFamily`
- shape-aware native primitives
- selector-shaped forwarding helpers

---

# 3. Current defect map

The remediation must explicitly remove these current compensations.

## D1 — `Family.set(Tuple,value)` lies about positional count

Current gateway rebuilds:

```text
receiver | tuple-positionals | tuple-labeled-values | rhs
```

but constructs an `ArgumentView` as though:

```text
positionals = tuple_positionals + rhs
```

This must be replaced by an explicit setter lane.

Primary file:

```text
phalcom-core/src/primitive/family.rs
```

Primary symbol:

```rust
family_set_shape
```

## D2 — Family exact/pattern dispatch uses `+1/-1` setter arithmetic

Current logic includes forms conceptually equivalent to:

```rust
SubscriptSet => expected_positional + 1
```

and:

```rust
view.positional_count().saturating_sub(1)
```

Primary file:

```text
phalcom-core/src/vm/send.rs
```

These arithmetic repairs must disappear.

## D3 — BoundMethodFamily setter matching subtracts setter values

Current subscript-family candidate derivation has logic equivalent to:

```rust
let setter_values = if is_setter { 1 } else { 0 };
let slot_positionals =
    view.positional_count().checked_sub(setter_values)?;
```

This must become direct structural use of:

```rust
view.positional_count()
view.labels()
view.has_setter_value()
```

## D4 — `ArgumentView` cannot represent setter lane

Current `ArgumentView` stores:

```rust
receiver_index
positional_count
labeled_count
selector
labels
caller authority
```

and computes labeled offset as:

```text
receiver + 1 + positional_count
```

There is no trailing value-lane metadata.

Primary file:

```text
phalcom-core/src/method/object.rs
```

## D5 — shape parameter is under-specified

Current method activation accepts:

```rust
shape: Option<(usize, usize)>
```

meaning roughly:

```text
(positionals, labeled_count)
```

This cannot encode setter-lane presence.

Primary files:

```text
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
```

## D6 — Family subscript interception is not centralized

Static subscript sends have Family/AssociatedFamily interception inside dispatch, while dynamic selector invocation follows another path.

The remediation must ensure static and dynamic sends share one selector-shaped dispatch authority.

---

# 4. Target runtime abstraction

The exact names may follow local style, but the semantic split is mandatory.

Recommended shape:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationLayout {
    structural_positionals: usize,
    labels: Box<[Symbol]>,
    setter_value: bool,
}
```

Recommended constructors:

```rust
impl InvocationLayout {
    pub(crate) fn ordinary(
        structural_positionals: usize,
        labels: Box<[Symbol]>,
    ) -> Self;

    pub(crate) fn setter(
        structural_positionals: usize,
        labels: Box<[Symbol]>,
    ) -> Self;

    pub(crate) fn physical_arity(&self) -> usize;

    pub(crate) fn has_setter_value(&self) -> bool;
}
```

`ArgumentView` should contain or otherwise delegate to this layout.

Recommended value-access semantics:

```rust
impl ArgumentView {
    pub fn positional_count(&self) -> usize;
    pub fn labeled_count(&self) -> usize;
    pub fn positional(&self, vm: &VM, index: usize) -> Option<Value>;
    pub fn labeled_value(&self, vm: &VM, index: usize) -> Option<Value>;

    pub fn has_setter_value(&self) -> bool;
    pub fn setter_value(&self, vm: &VM) -> Option<Value>;

    pub fn labels(&self) -> &[Symbol];
    pub fn physical_arity(&self) -> usize;
}
```

Offsets:

```text
positional(i)
    receiver + 1 + i

labeled(i)
    receiver + 1
    + structural_positionals
    + i

setterValue
    receiver + 1
    + structural_positionals
    + labeled_count
```

The setter value remains physically last.

---

# 5. Checkpoint map

| Checkpoint | Goal | Primary evidence |
|---|---|---|
| **C0 — Baseline and call-site inventory** | Pin repository state and enumerate every shape-construction/consumption site. | search inventory + focused baseline tests |
| **C1 — InvocationLayout / ArgumentView authority** | Add setter-value lane without changing selector or stack semantics. | method/runtime unit tests + crate compile |
| **C2 — Canonical selector-shaped dispatch** | Replace tuple shape metadata and centralize static/dynamic selector dispatch. | direct/static/dynamic send regressions |
| **C3 — Family and BoundMethodFamily remediation** | Remove setter positional arithmetic and repair tuple-shaped Family setters. | exact/pattern/labeled Family tests |
| **C4 — AssociatedFamily and shape-aware native correctness** | Ensure associated/frozen families and native ABI consume the same layout. | associated + native-like shape tests |
| **C5 — Negative cleanup and delivery gates** | Prove no setter-lane arithmetic authority remains and run affected suites. | negative searches + core/semantic checks |

---

# 6. C0 — Baseline and call-site inventory

## Goal

Establish the exact remote/local implementation state and identify all constructors/consumers of `ArgumentView`, selector-shaped call metadata, and setter-specific arity arithmetic before editing.

## Required inspections

Search:

```bash
rg -n   'ArgumentView::|shaped_with_labels|positional_window|with_selector|call_method_with_selector|dispatch_shape_at_as|FamilyInvocationKind::SubscriptSet|saturating_sub\(1\)|checked_sub\(setter|expected_positional \+ 1|SubscriptSet'   phalcom-core/src
```

Also inspect:

```text
phalcom-core/src/method/object.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/family.rs
phalcom-core/src/heap/selector_pattern.rs
phalcom-core/src/bytecode.rs
```

## Baseline tests

Run before modification:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_family_runtime -- --nocapture
```

Run the nearest indexed-assignment/dynamic-pack tests that cover:

```text
SubscriptSet
InvokeSubscriptSetPack
argument expansion
```

Record failures as baseline rather than repairing unrelated issues.

## C0 completion criteria

- [ ] current SHA recorded;
- [ ] every `ArgumentView` constructor call recorded;
- [ ] every `shape: Option<(usize,usize)>` call recorded;
- [ ] every setter `+1/-1/subtract-one` compensation recorded;
- [ ] static and dynamic Family subscript interception paths identified;
- [ ] baseline relevant tests recorded.

---

# 7. C1 — Introduce canonical invocation layout

## Goal

Make the runtime able to describe:

```text
receiver | structural positionals | labeled values | optional setter value
```

without changing physical stack order.

## Primary files

```text
phalcom-core/src/method/object.rs
phalcom-core/src/method/mod.rs
```

## Task 1 — Add `InvocationLayout`

Preferred placement:

```text
phalcom-core/src/method/object.rs
```

unless local module organization strongly favors a dedicated file.

The type should be internal runtime metadata, not a language-level selector object.

### Required fields

```rust
structural_positionals: usize
labels: Box<[Symbol]>
setter_value: bool
```

`labeled_count` should normally be derivable from `labels.len()` rather than stored redundantly.

### Required methods

At minimum:

```rust
ordinary(...)
setter(...)
structural_positionals()
labels()
labeled_count()
has_setter_value()
physical_arity()
```

### Required invariant checks

Add a helper that validates layout against `SignatureKind`.

Conceptually:

```rust
fn validate_for_kind(
    &self,
    kind: SignatureKind,
) -> Result<(), RuntimeError>
```

Rules:

```text
Getter:
    0 structural positionals
    0 labels
    no setter value

Setter:
    0 structural positionals
    0 labels
    setter value present

Method:
    no setter value

SubscriptGet:
    no setter value

SubscriptSet:
    setter value present
```

Do not enforce exact method/subscript structural arity in a way that conflicts with rest-family actual-call shapes; exact selector matching may validate separately.

## Task 2 — Rebase `ArgumentView` on the layout

Replace the independent positional/labeled counters with the new authority or make them strict delegates.

Target semantics:

```rust
ArgumentView {
    receiver_index,
    layout,
    selector,
    caller_access,
    caller_internal,
}
```

### Required accessors

```rust
positional_count()
labeled_count()
labels()
positional(vm, i)
labeled_value(vm, i)
has_setter_value()
setter_value(vm)
physical_arity()
```

### Required offset test

Construct a stack:

```text
receiver | 10 | 20 | true | 99
```

with layout:

```text
positionals = 2
labels = [debug]
setter = true
```

Assert:

```text
positional(0) == 10
positional(1) == 20
labeled_value(0) == true
setter_value() == 99
physical_arity() == 4
```

## Task 3 — Add explicit constructors

Replace ambiguous constructors with names that describe the lane model.

Recommended:

```rust
ArgumentView::ordinary_window(...)
ArgumentView::setter_window(...)
ArgumentView::from_layout(...)
```

Avoid a generic constructor whose boolean arguments are easy to invert.

Keep compatibility wrappers temporarily only if needed to make the patch bisectable; remove them by C5.

## C1 hostile cases

- setter layout with no setter value accessor result;
- ordinary Method layout accidentally exposing `setter_value`;
- labeled setter shape with zero structural positionals;
- subscript setter with several positional indices plus labels;
- zero-index subscript setter if grammar permits it.

## C1 evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core method::
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Use exact local module filters if `method::` is not a valid test selector.

## C1 completion criteria

- [ ] setter value has a first-class layout representation;
- [ ] labeled offsets no longer depend on counting RHS as positional;
- [ ] physical arity is derived from layout;
- [ ] no selector type changed;
- [ ] no physical stack order changed.

---

# 8. C2 — Canonical selector-shaped dispatch

## Goal

Remove `Option<(usize,usize)>` as the shape authority and ensure static/dynamic sends converge on one dispatch operation.

## Primary files

```text
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
```

## Task 4 — Replace tuple shape metadata

Current activation API conceptually accepts:

```rust
arity: usize,
selector: Symbol,
shape: Option<(usize, usize)>,
```

Replace `shape` with:

```rust
layout: Option<InvocationLayout>
```

or equivalent.

During migration it is acceptable to retain `arity` as physical arity, but add:

```rust
debug_assert_eq!(
    arity,
    layout.physical_arity()
);
```

whenever a layout is supplied.

Do not compute structural positionals from `arity - labels` for setters.

## Task 5 — Derive exact invocation layout from selector kind

For an exact selector:

```text
Getter
    ordinary(0, [])

Setter
    setter(0, [])

Method(slots)
    ordinary(method positional count, method labels)

SubscriptGet(slots)
    ordinary(index positional count, index labels)

SubscriptSet(slots)
    setter(index positional count, index labels)
```

Create one helper, e.g.:

```rust
fn exact_layout_from_selector(
    &mut self,
    selector: Symbol,
) -> PhResult<InvocationLayout>
```

or a pure helper over decoded selector data.

Do not duplicate this logic in interpreter branches.

## Task 6 — Introduce one selector-shaped dispatch gateway

Create one internal operation responsible for:

```text
receiver
selector
invocation layout
source range
authority
```

Suggested conceptual API:

```rust
fn dispatch_selector_window_as(
    &mut self,
    receiver_idx: usize,
    selector: Symbol,
    layout: InvocationLayout,
    source_range: SourceRange,
    caller_authority: (Option<ClassId>, bool),
) -> PhResult<CallOutcome>
```

Responsibilities:

1. Validate physical window length against layout.
2. Detect `Family` / `AssociatedFamily` subscript selectors.
3. Route them to:
   ```text
   SubscriptGet
   SubscriptSet
   ```
4. Otherwise perform ordinary exact lookup.
5. Fall through to rest lookup where valid.
6. Fall through to DNU.
7. Pass the same layout into shape-aware native activation.

The gateway must not reinterpret setter RHS as positional.

## Task 7 — Route static `Bytecode::Invoke` through canonical layout

For ordinary exact invocation:

- decode/obtain selector;
- derive `InvocationLayout`;
- use physical arity only as a consistency check;
- route through the shared selector-shaped dispatch function.

Special Family subscript interception should disappear from the bytecode arm once the common gateway owns it.

## Task 8 — Route dynamic packs through canonical layout

### `InvokePack`

For Method/SubscriptGet dynamic packs:

```rust
InvocationLayout::ordinary(
    positionals.len(),
    labels,
)
```

### `InvokeSubscriptSetPack`

The dynamic setter path already has the ideal separated data:

```rust
positionals
labels
labeled values
rhs
```

Construct:

```rust
InvocationLayout::setter(
    positionals.len(),
    labels,
)
```

The stack remains:

```text
receiver | positionals | labeled-values | rhs
```

Then route through the same selector-shaped dispatch gateway.

Remove any later derivation that treats the RHS as an extra positional.

## C2 hostile cases

- static subscript setter on ordinary object;
- dynamic/expanded subscript setter on ordinary object;
- static subscript setter on Family;
- dynamic/expanded subscript setter on Family;
- exact shape-aware native subscript setter;
- method call with identical total arity but different positional/labeled split.

## C2 evidence

Add runtime tests proving static and dynamic sends deliver identical lane metadata.

Minimum pair:

```phalcom
table[42, debug: true] = 99
table[***shape] = 99
```

where `shape` reconstructs the same index shape.

## C2 completion criteria

- [ ] static and dynamic sends share one layout model;
- [ ] Family interception is centralized;
- [ ] no setter shape derives structural positionals from total physical arity;
- [ ] `InvokeSubscriptSetPack` passes a setter layout directly.

---

# 9. C3 — Family and BoundMethodFamily remediation

## Goal

Remove all Family-specific setter positional arithmetic and repair `Family.set(Tuple,value)` for labeled shapes.

## Primary files

```text
phalcom-core/src/primitive/family.rs
phalcom-core/src/vm/send.rs
```

## Task 9 — Repair `Family.set(Tuple,value)`

Current physical reconstruction may remain:

```rust
vm.stack.truncate(receiver_idx + 1);
vm.stack.extend(positionals);
vm.stack.extend(labeled_values);
vm.stack.push(value);
```

Replace the old view construction:

```rust
positionals.len() + 1
```

with a setter layout:

```rust
InvocationLayout::setter(
    positionals.len(),
    labels,
)
```

The RHS is now accessed through:

```rust
view.setter_value(vm)
```

not `view.positional(...)`.

## Task 10 — Normalize named setter gateways

Different Family API entrypoints need explicit conversion semantics.

### `family.set(rhs)`

Incoming selector is Method `set(_)`.

Incoming view:

```text
one ordinary positional
no setter lane
```

Before forwarding to captured setter, reclassify the existing physical value:

```text
structural positionals = 0
setter lane = present
```

No stack movement is required.

Provide a dedicated helper, for example:

```rust
ArgumentView::reclassify_unary_method_as_setter()
```

or reconstruct from the same receiver index with `InvocationLayout::setter(0, [])`.

### `family.value = rhs`

Incoming selector is itself `Setter`.

Once the canonical selector-shaped call gateway is fixed, its incoming view should already have a setter lane.

Do not manually reinterpret it as positional.

### `family.get()` / `family.value`

No setter lane.

## Task 11 — Remove Family exact/pattern `+1/-1` logic

In:

```text
activate_family_with_kind
```

replace logic such as:

```rust
SubscriptSet => expected_positional + 1
SubscriptSet => view.positional_count().saturating_sub(1)
```

with:

```text
view.positional_count() = structural index positionals
view.labels()           = structural index labels
view.has_setter_value() = true
```

For exact SubscriptSet:

```text
selector structural positionals
==
view.positional_count()
```

and labels must match exactly.

For pattern SubscriptSet:

```rust
pattern.matches_call(
    SelectorKind::SubscriptSet,
    view.positional_count(),
    view.labels(),
)
```

## Task 12 — Remediate BoundMethodFamily

Remove:

```rust
checked_sub(setter_values)
```

and any equivalent compensation.

Subscript setter candidate selection must require:

```text
view.has_setter_value() == true
```

and match pattern shape using unmodified structural positionals/labels.

## Task 13 — Preserve captured target physical activation

When Family forwarding replaces the Family receiver with the captured receiver, dispatch using:

```rust
layout.physical_arity()
```

not:

```rust
view.positional_count() + view.labels().len()
```

otherwise the selected setter receives one value too few.

This requirement applies to:

```text
dispatch_shape_at_as
activate_captured_method_as
associated target forwarding
```

or their refactored replacements.

## C3 decisive regression fixture

Add a labeled Tuple case:

```phalcom
class Table {
  [_ key, debug flag]=(_ value) {
    // record all three values independently
  }
}

const table = Table.new()
const setter = &table[_, debug]=(_)

setter.set(
  (42, debug: true),
  99
)
```

The target must observe:

```text
key   = 42
flag  = true
value = 99
```

A positional-only test is insufficient.

## C3 additional matrix

```phalcom
const exact = &table[_, debug]=(_)
exact[42, debug: true] = 99
exact.set((42, debug: true), 99)

const pattern = &table[...]=(_)
pattern[42, debug: true] = 99
pattern.set((42, debug: true), 99)

const accessors = &table[...]=
accessors[42, debug: true] = 99
```

All set expressions remain `Unit`.

## C3 completion criteria

- [ ] labeled `set(Tuple,value)` is correct;
- [ ] exact/pattern Family setter paths use no positional subtraction;
- [ ] BoundMethodFamily setter paths use no positional subtraction;
- [ ] direct brackets and explicit Tuple API agree;
- [ ] assignment result policy remains unchanged.

---

# 10. C4 — AssociatedFamily and shape-aware native correctness

## Goal

Prove that the layout abstraction works not only for bytecode closures but also frozen associated family descriptors and shape-aware native consumers.

## Primary files

```text
phalcom-core/src/vm/send.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/primitive/*
phalcom-core/tests/core/execution/family_runtime.rs
```

Only modify semantic lowering if runtime descriptor shape actually requires it. Prefer no semantic changes.

## Task 14 — AssociatedFamily activation

Ensure:

```text
AssociatedFamily + Setter
AssociatedFamily + SubscriptSet
```

consume the same `InvocationLayout`.

Candidate operation matching must use:

```text
structural positionals
labels
setter lane presence
```

The RHS must not affect selector structural shape.

Do not route associated families through live bound-family method lookup.

## Task 15 — Shape-aware native setter regression

The bug is easiest to hide with bytecode closures because closures consume physical stack order directly.

Add or create the smallest possible shape-aware primitive test target that explicitly reads:

```rust
view.positional(...)
view.labeled_value(...)
view.setter_value(...)
```

For:

```text
positionals = [42]
labels = [debug]
setter value = 99
```

assert independently:

```text
positional(0)   = 42
labeled_value(0)= true
setter_value()  = 99
```

This test is mandatory.

It proves the metadata itself, not merely downstream closure behavior.

## Task 16 — Dynamic pack + shape-aware native setter

Force:

```text
InvokeSubscriptSetPack
```

into the same shape-aware native target.

The test must verify:

```text
static send layout == dynamic pack send layout
```

for the same selector.

## Task 17 — Setter invariant tests

At Rust/runtime level, test malformed internal layouts where practical:

```text
SubscriptSet without setter lane
Getter with setter lane
Method with setter lane
Setter with labels/structural positionals
```

These should fail deterministically as internal invariant errors.

Do not expose these as user-facing syntax errors; parser/semantic layers already prevent ordinary source construction.

## C4 completion criteria

- [ ] AssociatedFamily setter paths use canonical layout;
- [ ] shape-aware native consumer sees correct lanes;
- [ ] dynamic pack produces the same layout as static send;
- [ ] malformed lane/kind combinations are rejected;
- [ ] no selector identity changes.

---

# 11. C5 — Cleanup, negative gates, and delivery

## Goal

Remove temporary compatibility APIs and prove that the old setter-as-positional model is gone.

## Task 18 — Delete transitional constructors

Remove obsolete forms such as:

```rust
ArgumentView::shaped_with_labels(...)
```

if they cannot represent setter lanes safely.

If retained for ordinary-call convenience, ensure they can only construct non-setter layouts and are named accordingly.

## Task 19 — Remove arithmetic setter compensations

Required searches:

```bash
rg -n   'saturating_sub\(1\)|checked_sub\(setter|setter_values|expected_positional \+ 1|positionals\.len\(\) \+ 1'   phalcom-core/src/vm   phalcom-core/src/primitive/family.rs
```

Every remaining hit must be unrelated to setter-lane reconstruction.

## Task 20 — Remove tuple shape metadata

Search:

```bash
rg -n   'shape: Option<\(usize, usize\)>|Some\(\(positional_count, labels\.len\(\)\)'   phalcom-core/src
```

Expected result after migration: zero relevant call-shape ABI hits.

## Task 21 — Ensure no selector pollution

Search for any code that appends an RHS/value pseudo-slot to subscript setter selectors.

The only structural slots must be index slots.

---

# 12. Required test suite

## Focused runtime tests

At minimum add/extend tests for:

1. `ArgumentView` lane offsets.
2. Exact Family labeled subscript setter via direct brackets.
3. Exact Family labeled setter via `set(Tuple,value)`.
4. Pattern Family labeled subscript setter.
5. Getter+setter subscript family.
6. BoundMethodFamily labeled setter shape.
7. AssociatedFamily setter where applicable.
8. Shape-aware native setter reading all lanes.
9. Dynamic pack setter to same native-like target.
10. Setter invariant rejection cases.

## Regression syntax

Use a shape with at least:

```text
one positional index
one labeled index
one setter RHS
```

Example:

```text
[_, debug]=(_)
```

This is the minimum shape that exposes the bug.

Also include a wider case:

```text
[_, _, debug]=(_)
```

to prevent accidental off-by-one repair.

---

# 13. Verification commands

Run smallest-first.

## C1

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

plus focused method/ArgumentView unit tests.

## C2/C3

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_family_runtime -- --nocapture
```

Run the exact indexed/dynamic pack module filters containing `InvokeSubscriptSetPack`.

## C4

Run the new shape-aware-native regression explicitly before broad suites.

Then:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core --no-run
```

## C5

```bash
cargo fmt --all -- --check

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
```

Run broader workspace tests only after focused evidence is green.

Any previously known unrelated REPL/workspace baseline failures must be classified separately rather than hidden.

---

# 14. Hostile-case table

| Case | Failure defeated |
|---|---|
| `setter.set((42,),99)` | basic positional setter lane |
| `setter.set((42,debug:true),99)` | original labeled-lane corruption |
| `setter.set((1,2,debug:true),99)` | wider positional offset errors |
| `family[42,debug:true]=99` | direct Family setter path |
| dynamic/expanded equivalent | `InvokeSubscriptSetPack` divergence |
| pattern `&table[...]=(_)` | pattern structural subtraction |
| accessor family `&table[...]=` | kind-set forwarding |
| shape-aware native target | proves metadata, not just stack order |
| AssociatedFamily target | frozen descriptor correctness |
| malformed Getter+setter lane | invariant enforcement |
| malformed SubscriptSet without setter lane | invariant enforcement |

---

# 15. Files expected to change

Primary:

```text
phalcom-core/src/method/object.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/family.rs
phalcom-core/tests/core/execution/family_runtime.rs
```

Probable depending on implementation details:

```text
phalcom-core/src/bytecode.rs
phalcom-core/src/heap/selector_pattern.rs
phalcom-core/tests/core/...
```

Only if compile errors or explicit evidence require them:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-semantic/*
phalcom-ast/*
```

The semantic selector model should not need redesign for this fix.

---

# 16. Explicitly forbidden fixes

Do **not**:

- move RHS before labeled values;
- count RHS as positional;
- append RHS to selector slots;
- reintroduce a synthetic `put` label;
- relax global positional-before-labeled ordering;
- change `SignatureKind::SubscriptSet(n)` meaning;
- change setter syntax;
- change `Family.set(Tuple,value)` public API;
- make `ArgumentView` guess setter layout by subtracting one from physical arity at consumption sites;
- patch only `family_set_shape`;
- create separate incompatible ABIs for Family setters and ordinary setters;
- change assignment/set result semantics away from `Unit`;
- route `AssociatedFamily` through live receiver dispatch;
- rely only on bytecode closure tests.

---

# 17. Recommended implementation sequence

```text
C0 inventory
    ↓
InvocationLayout
    ↓
ArgumentView lane access
    ↓
method activation layout parameter
    ↓
central selector-shaped dispatch
    ↓
static Invoke migration
    ↓
dynamic pack migration
    ↓
Family explicit gateways
    ↓
Family exact/pattern routing
    ↓
BoundMethodFamily routing
    ↓
AssociatedFamily routing
    ↓
shape-aware native regression
    ↓
negative arithmetic cleanup
    ↓
broad delivery gates
```

Do not start with `primitive/family.rs`. That produces another local workaround instead of fixing the runtime model.

---

# 18. Checkpoint completion report template

For each checkpoint:

```text
Checkpoint C<N> COMPLETE

Established:
    <dominant invariant>

Changed:
    <file> — <symbol/responsibility>

Evidence:
    <command> — PASS

Hostile cases:
    <case> — PASS

Negative searches:
    <search> — <result>

Deferred:
    <gate> → C<N+1>/Final

Unexpected findings:
    none | <fact>

Next:
    C<N+1>
```

---

# 19. Final acceptance criteria

The remediation is complete only when all are true:

- [ ] assignment/set expression result semantics remain `Unit`;
- [ ] `InvocationLayout` or equivalent is the runtime argument-lane authority;
- [ ] `ArgumentView` exposes a distinct setter value;
- [ ] setter RHS remains physically last;
- [ ] labeled values are indexed independently of setter RHS;
- [ ] `SignatureKind` and selector identity are unchanged;
- [ ] static `Invoke` and dynamic pack sends converge on one layout model;
- [ ] direct Family subscript setters use canonical layout;
- [ ] `Family.set(Tuple,value)` handles labeled Tuple shapes correctly;
- [ ] exact Family matching contains no setter positional `+1`;
- [ ] Family pattern matching contains no setter positional `-1`;
- [ ] BoundMethodFamily matching contains no setter subtraction;
- [ ] AssociatedFamily uses the same lane model;
- [ ] shape-aware native setter regression proves positional/labeled/RHS lanes independently;
- [ ] `InvokeSubscriptSetPack` produces the same layout as static send;
- [ ] malformed lane/kind combinations are rejected centrally;
- [ ] no synthetic `put` mechanism returns;
- [ ] negative searches find no unexplained setter-lane arithmetic authority;
- [ ] focused core tests are green;
- [ ] affected broad tests/checks are green or known unrelated baseline failures are explicitly recorded.

---

# 20. Final architectural rule

> **Selector shape describes behavior identity. Invocation layout describes runtime argument lanes. A setter value belongs to invocation layout, never to selector slots and never to the ordinary positional lane.**

For:

```phalcom
table[row, debug: true] = value
```

the final model is:

```text
Selector
    base: Subscript
    kind: SubscriptSet
    slots:
        Positional
        Label(debug)

InvocationLayout
    structural_positionals: 1
    labels: [debug]
    setter_value: true

Physical stack
    receiver
    row
    true
    value
```

That same structure must be observed by ordinary dispatch, dynamic packs, Family forwarding, AssociatedFamily forwarding, BoundMethodFamily activation, and shape-aware native primitives.
