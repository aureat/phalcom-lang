# F.3 — Rest Capture and Rest-Pattern Dispatch
## Amended implementation specification and implementation plan

Status: **implementation specification and implementation plan — amended**

Supersedes:

- the original F.3 rest-capture implementation specification;
- the U9 positional-only variadic model;
- the temporary F.2 wording that refers to U9-style "variadic fallback" after dynamic exact lookup.

Requires:

- F.1 pack/selector syntax and canonical label escaping;
- F.2 amended outgoing pack assembly and `InvokePack`/`SuperSendPack`;
- A.3 Tuple/Unit product finalization through `finish_tuple`;
- the current method/class installation and reflective method-definition machinery.

Expected public primitive-floor delta: **0**.

---

# 1. Mission

Replace the old U9 positional-only variadic model with lane-aware rest capture and rest-pattern dispatch:

```text
*rest      positional residual lane
**rest     labeled residual lane
***rest    complete residual pack
```

Rest captures are canonical products:

```text
empty capture      -> Unit
non-empty capture  -> Tuple
```

Never capture rest arguments into List, Map, Record, or a new public pack object.

F.3 consumes the canonical call shape established by F.2:

```text
positionals = ordered positional values
labels      = ordered label Symbols
values      = labeled values corresponding 1:1 with labels
```

Dispatch remains selector/pack-shape based. F.3 introduces **no type-based dispatch** and does not consult annotations or runtime argument value types when selecting a method.

Exact selector lookup remains authoritative and always precedes rest-pattern fallback.

---

# 2. Syntax baseline and notation

Current method-parameter syntax does **not** use a trailing colon to mark a labeled parameter.

Use forms such as:

```phalcom
foo(*rest, x)
foo(timeout, **rest)
method(a, b, *rest, timeout, mode, **extra)
```

Do not use obsolete declaration spellings such as:

```text
x:
timeout:
mode:
```

The parser/AST is responsible for classifying parameters into the positional and labeled lanes according to the current grammar. Runtime matching operates only on the normalized lane metadata described below.

Selector-pattern examples in this document use canonical structural slot notation:

```text
sum(*)
format(_,*)
log(_,*,format,**)
```

Whitespace is illustrative only; canonical encoder output should follow the repository's existing formatting convention.

---

# 3. Explicit F.4 boundary: block rest is deferred

F.3 implements **method** rest capture and method-family rest fallback only.

Block/closure rest parameters are deliberately out of scope for F.3.

The current block syntax is based on parameter bars, for example:

```phalcom
|x| { ... }
```

F.3 does not define, parse, lower, or execute a rest extension of that syntax.

Block rest support is reserved for a separate F.4 specification to be designed later. F.3 must not pre-implement or partially expose that feature.

Consequences:

- no `Callable` rest-layout changes in F.3;
- no block argument repacker in F.3;
- no block `*`/`**`/`***` syntax decisions in F.3;
- no change to `Block#arity` in F.3.

---

# 4. Baseline to retire

The U9 baseline conceptually contains:

```text
ParameterDef.is_rest: bool
SignatureKind::Variadic(fixed_positional_arity)
Signature.variadic: bool
selector fallback probe based on name(*)
call prologue that packs trailing positional values into List
```

F.3 supersedes that architecture completely.

Do not preserve the old List capture behavior as a compatibility mode.

Do not keep `SignatureKind::Variadic` as a second source of truth.

Do not retain `name(*)` probing as the runtime rest-dispatch mechanism.

---

# 5. Ratified design decisions

## D1 — F.2 must be fully landed first

Before F.3 implementation begins, confirm that the amended F.2 dynamic-pack path is active and stable:

```text
InvokePack exists
SuperSendPack exists
canonical outgoing lanes exist
dynamic selector derivation exists
F.2 dNU forwarding uses the concrete selector
A.3 finish_tuple is available
```

Do not implement F.3 against a partially migrated F.2/U9 hybrid dispatch path.

## D2 — Block rest is deferred to F.4

No block-rest implementation belongs in F.3.

## D3 — Rest metadata lives on Signature

`Signature` owns the normalized rest layout.

`MethodObject` continues to own/use `Signature`; it must not carry a second independently mutable rest-layout copy.

## D4 — No stored `RestMode::None`

The canonical no-rest state is:

```rust
signature.rest == None
```

A stored `RestLayout` therefore represents an actual rest-capable signature and has only meaningful modes:

```rust
pub enum RestMode {
    Positional,
    Labeled,
    Split,
    Complete,
}
```

If an existing parser-facing enum temporarily needs a `None` value during migration, normalize it away before constructing the runtime `Signature`.

## D5 — One rest-capable method per base family per class

A class may define at most one rest-capable method for a given base selector name.

Exact methods remain unrestricted by this rule.

This restriction is deliberate. F.3 does not invent a specificity or ambiguity ordering for overlapping wildcard patterns.

## D6 — Rest selector text is not a dispatch key

Canonical wildcard selector strings exist for:

- reflection;
- diagnostics;
- disassembly/debugging;
- method-table identity where the repository already stores every declared method by encoded selector.

Rest fallback itself uses structured metadata:

```text
base family
actual positional count
actual ordered labels
RestLayout
```

Do not parse wildcard selector text during lookup.

## D7 — Exact lookup completes across inheritance before rest lookup begins

Dispatch is two-pass:

```text
Pass 1: exact concrete selector across full lookup chain
Pass 2: rest-family fallback across the same applicable lookup chain
```

Never interleave exact and rest lookup per class.

An inherited exact method therefore beats a subclass rest fallback.

## D8 — Rest acceptance is a pure allocation-free predicate

Matching performs no user dispatch and no allocation.

It compares only counts and interned Symbol identity/order.

## D9 — Rest capture is constructed by the VM before frame entry

Compiled method bodies receive ordinary declaration-local parameter slots.

Bodies do not inspect raw call packs.

## D10 — Native/primitive rest methods are unsupported in F.3

Reject a native method declaration carrying a rest layout unless a pre-existing primitive explicitly already has a separately ratified ABI for it.

F.3 does not silently change the native ABI.

## D11 — U9 metadata is removed during F.3, not kept indefinitely

During migration, temporary compatibility accessors may derive from `Signature.rest`, but by F.3 completion:

```text
SignatureKind::Variadic is gone
Signature.variadic storage is gone
U9 variadic selector cache/probe is gone
List rest packing is gone
```

## D12 — Rest dispatch shares one cold fallback seam

The same rest-family lookup semantics must be used by:

- eligible Method-kind `Bytecode::Invoke` exact miss;
- eligible Method-kind `Bytecode::InvokePack` exact miss;
- `SuperSend` exact miss if ordinary static super sends can target rest methods;
- `SuperSendPack` exact miss;
- reflective ordinary send, if that API models language message send.

Do not duplicate acceptance/lookup logic across send kinds.

## D13 — Static hot-path exact dispatch remains materially unchanged

F.3 adds work to the **exact-miss** path.

Do not add rest-shape branches to every successful static `Invoke` merely for code sharing.

## D14 — F.2's temporary "variadic fallback" wording is replaced by F.3 rest fallback

After F.3 lands, F.2 dynamic dispatch semantics become:

```text
exact selector lookup
rest-family fallback
ordinary dNU
```

The old U9 all-positional `name(*)` probe no longer exists.

## D15 — Rest fallback is eligible only for ordinary Method selector families

F.3 rest declarations belong to named method families. Subscript-get, subscript-set, and other non-Method selector kinds do not acquire wildcard fallback merely because they share `Invoke`/`InvokePack` machinery.

The shared dispatch seam must therefore gate rest fallback by selector/send kind:

```text
Method        -> exact, then rest, then dNU
SubscriptGet  -> existing exact/dNU semantics
SubscriptSet  -> existing exact/dNU semantics
other kinds   -> existing semantics unless separately specified
```

A future specification may define rest-capable non-Method selector domains explicitly; F.3 does not.

---

# 6. Parser/AST rest binder representation

The old boolean cannot represent all F.3 forms.

Replace or supersede:

```rust
is_rest: bool
```

with a lane-aware binder representation, conceptually:

```rust
pub enum RestParameterKind {
    Positional, // *rest
    Labeled,    // **rest
    Complete,   // ***rest
}
```

A parameter is either ordinary or carries one `RestParameterKind`.

Exact AST field names should follow the repository's current parameter-node architecture; the semantic requirement is that the compiler can distinguish all three forms without reparsing source text.

---

# 7. Normalized runtime metadata

Use one normalized runtime/compiler layout:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestMode {
    Positional {
        param_index: u16,
    },
    Labeled {
        param_index: u16,
    },
    Split {
        positional_param_index: u16,
        labeled_param_index: u16,
    },
    Complete {
        param_index: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestLayout {
    fixed_positionals: u8,
    fixed_labels: Box<[Symbol]>,
    mode: RestMode,
}
```

On `Signature`:

```rust
pub rest: Option<RestLayout>
```

Fields should be private unless repository conventions strongly require public fields.

Expose narrow accessors and semantic operations instead:

```rust
impl RestLayout {
    pub fn mode(&self) -> RestMode;
    pub fn fixed_positionals(&self) -> u8;
    pub fn fixed_labels(&self) -> &[Symbol];
    pub fn accepts(&self, positional_count: usize, labels: &[Symbol]) -> bool;
}
```

The parameter-index integer type may follow the repository's actual local-slot width if it is not `u16`. Carrying the indices inside `RestMode` deliberately makes invalid combinations of three independent optional index fields unrepresentable.

---

# 8. Rest mode normalization

Normalize declaration binders to one `RestMode`:

```text
*rest only              -> Positional
**rest only             -> Labeled
*rest + **rest           -> Split
***rest only             -> Complete
```

Reject all other rest combinations.

There is no `RestLayout` for an ordinary exact-only method.

---

# 9. Declaration ordering and validity

Compiler validation must reject invalid signatures before method installation.

Required rules:

1. At most one positional rest binder (`*rest`).
2. At most one labeled rest binder (`**rest`).
3. At most one complete rest binder (`***rest`).
4. `***rest` is terminal.
5. `***rest` cannot coexist with `*rest`.
6. `***rest` cannot coexist with `**rest`.
7. No parameter may appear after `**rest`.
8. No parameter may appear after `***rest`.
9. Once positional rest has consumed the residual positional lane, later fixed parameters must belong to the labeled lane according to the current grammar/AST classification.
10. Split mode is exactly one positional rest plus one labeled rest with any fixed labeled-prefix parameters between them as permitted by the grammar.

Examples of valid conceptual declarations:

```phalcom
sum(*numbers)
method(a, *rest)
method(timeout, **rest)
method(a, b, *rest, timeout, mode, **extra)
method(a, fixed, ***remaining)
```

Examples of invalid combinations:

```text
*rest + ***remaining
**rest + ***remaining
*rest + **rest + ***remaining
parameter after **rest
parameter after ***rest
```

---

# 10. Canonical fixed-lane model

A rest declaration consumes prefixes of the two canonical incoming lanes.

Let:

```text
P = actual positional count
L = actual ordered label list
F = fixed positional count
K = fixed ordered label prefix
```

For a conceptual declaration:

```phalcom
method(a, b, *rest, timeout, mode, **extra)
```

normalized requirements are:

```text
F = 2
K = [#timeout, #mode]
mode = Split
```

The labeled prefix is ordered selector identity.

A call whose labeled lane is:

```text
[#debug, #timeout, #mode]
```

must **not** match merely because `#timeout` and `#mode` occur later.

No name search or label reordering is permitted.

---

# 11. Acceptance predicates

`RestLayout::accepts` implements exactly these rules.

## 11.1 Positional rest

Declaration has `*rest` and no `**rest`.

Accept iff:

```text
P >= F
L == K
```

Capture:

```text
positionals[F..P]
```

The labeled lane must match the fixed label sequence exactly; unexpected labels are rejected as a rest-pattern miss.

## 11.2 Labeled rest

Declaration has `**rest` and no `*rest`.

Accept iff:

```text
P == F
L startsWith K
```

Capture:

```text
labels[K.len..]
corresponding labeled values
```

No extra positional values are accepted.

## 11.3 Split rest

Declaration has both `*rest` and `**rest`.

Accept iff:

```text
P >= F
L startsWith K
```

Capture residual positional and labeled lanes separately.

## 11.4 Complete rest

Declaration has terminal `***rest`.

Accept iff:

```text
P >= F
L startsWith K
```

Capture both residual lanes into one product.

---

# 12. Acceptance implementation requirements

`accepts` must be pure and allocation-free.

Recommended structure:

```rust
pub fn accepts(&self, positional_count: usize, labels: &[Symbol]) -> bool {
    let f = self.fixed_positionals as usize;
    let k = self.fixed_labels.as_ref();

    match self.mode {
        RestMode::Positional { .. } => {
            positional_count >= f && labels == k
        }
        RestMode::Labeled { .. } => {
            positional_count == f && labels.starts_with(k)
        }
        RestMode::Split { .. } | RestMode::Complete { .. } => {
            positional_count >= f && labels.starts_with(k)
        }
    }
}
```

Exact Rust details may vary.

Requirements:

- compare interned `Symbol`s directly;
- do not allocate temporary vectors;
- do not render selector strings;
- do not invoke user equality/hash/string conversion;
- reject quickly on count mismatch before prefix comparison.

---

# 13. Capture products

All rest captures use A.3 Tuple/Unit finalization.

## 13.1 Empty capture

If both residual lanes are empty:

```text
capture == Unit
```

Examples:

```phalcom
method(*args)        // method() -> args == ()
method(**labels)     // no residual labels -> labels == ()
method(***remaining) // no residual values/labels -> remaining == ()
```

Do not allocate an empty Tuple, List, Record, or Map.

## 13.2 Positional capture

`*rest` produces a positional-only Tuple when non-empty.

Conceptually:

```phalcom
method(a, *rest)
method(1, 2, 3)
```

bindings:

```text
a    = 1
rest = (2, 3)
```

One residual positional value yields a singleton Tuple:

```text
(2,)
```

not the bare value.

## 13.3 Labeled capture

`**rest` produces a labeled-only Tuple preserving residual label order.

Conceptually:

```phalcom
method(timeout, **rest)
```

called with a pack whose labeled lane is:

```text
[(#timeout, 1), (#debug, true), (#trace, false)]
```

binds:

```text
timeout = 1
rest    = (debug: true, trace: false)
```

The tuple display syntax may use the language's normal labeled Tuple rendering. The **parameter declaration** itself does not use obsolete trailing-colon syntax.

## 13.4 Complete capture

For a normalized complete-rest declaration with:

```text
F = 1
K = [#fixed]
```

and actual pack:

```text
positionals = [1, 2, 3]
labels      = [#fixed, #x, #y]
values      = [4, 5, 6]
```

bindings are:

```text
fixed positional parameter = 1
fixed labeled parameter     = 4
complete rest               = (2, 3, x: 5, y: 6)
```

Fixed entries are removed before finalizing the residual capture.

---

# 14. Shared capture constructor

Implement one VM helper backed directly by A.3 product finalization:

```rust
fn finish_capture(
    &mut self,
    positional_values: &[Value],
    labels: &[Symbol],
    labeled_values: &[Value],
) -> PhResult<Value>;
```

or an equivalent repository-style signature.

Contract:

```text
no residual lanes -> canonical Unit
otherwise         -> Tuple finalized by A.3 finish_tuple semantics
```

Requirements:

- `labels.len() == labeled_values.len()`;
- do not stage through List;
- do not stage through Map/Record;
- preserve label order;
- perform no user dispatch;
- do not apply send-arity limits to captures themselves.

The helper belongs to the VM/product-construction layer, not compiled user code.

---

# 15. Canonical rest selector representation

Rest-capable method selectors should render structurally for reflection/debugging.

Recommended canonical forms:

```text
sum(*)
format(_,*)
log(_,*,format,**)
options(_,debug,**)
forward(_,timeout,***)
```

Canonical output should omit illustrative spaces and use the same component escaping rules as F.1.

The formatter must include:

- fixed positional slots;
- fixed labeled-prefix slots;
- the correct rest marker.

Do not retain U9's lossy formatting where a declaration with fixed slots is rendered merely as:

```text
name(*)
```

Literal Symbol labels such as `#*`, `#**`, `#***`, and `#_` remain distinct through F.1 escaping.

---

# 16. Rest selector representation is descriptive, not executable pattern syntax

The wildcard selector spelling must never become the runtime matching algorithm.

Specifically, do not:

```text
construct name(*) on exact miss
probe the ordinary method table with progressively synthesized wildcard strings
parse format(_,*) back into RestLayout
```

Rest dispatch uses the secondary class index plus `RestLayout::accepts`.

This keeps formatting/reflection independent from runtime matching internals.

---

# 17. Class rest-family index

Add a class-owned secondary index alongside ordinary method storage:

```rust
rest_methods: HashMap<Symbol, ObjRef>
```

where the key is the interned **base selector family name** and the value references the rest-capable `MethodObject` declared by that class.

The exact source file is the repository-current location of `ClassObject`; F.3 must not move class lookup state into a VM-global registry merely for this feature.

Requirements:

- no linear scan of all methods on every exact miss;
- one entry maximum per base family per class;
- index points to the same `MethodObject` stored by ordinary class method infrastructure;
- reflective installation uses the same invariant-preserving installation path;
- cache/world-version invalidation, if applicable to method installation today, must remain consistent.

---

# 18. One-rest-pattern-per-family rule

Legal within one class:

```text
foo()
foo(_)
foo(_,x)
foo(_,*,x,**)
```

Only the last entry is rest-capable.

Rejected within one class:

```text
foo(*)
foo(_, *)
```

because both are rest-capable members of the same `foo` family.

Likewise, overlapping patterns such as:

```text
foo(*)
foo(*,x)
```

are not assigned a specificity ordering in F.3.

A later selector-dispatch specification may lift this restriction after defining ambiguity/specificity semantics.

---

# 19. Method installation invariants

All method attachment paths must funnel through one invariant-preserving operation or equivalent shared logic.

Conceptually:

```rust
fn install_method(
    &mut self,
    class: ClassId,
    method: ObjRef,
) -> Result<(), MethodInstallError>;
```

For a rest-capable method:

1. perform ordinary exact-selector duplicate/replacement checks;
2. obtain the signature's base family;
3. inspect the class's `rest_methods` entry;
4. reject a conflicting second rest pattern for that family;
5. install/update ordinary method storage according to existing semantics;
6. install/update the rest-family index atomically with it;
7. apply normal method-cache/world invalidation.

Reflective method definition must not bypass steps 2–7.

If the repository supports replacing an existing method with the **same** selector, preserve that existing replacement policy; the rest-family index must be updated to the replacement object rather than creating a second pattern.

---

# 20. Exact lookup precedence

Exact lookup remains the first dispatch pass.

Given receiver-class chain:

```text
Subclass
Superclass
Root
```

and concrete selector `foo(_,x)`:

```text
Pass 1:
    Subclass exact foo(_,x)?
    Superclass exact foo(_,x)?
    Root exact foo(_,x)?

only if all exact lookups miss:

Pass 2:
    Subclass rest family foo accepting this pack?
    Superclass rest family foo accepting this pack?
    Root rest family foo accepting this pack?
```

Therefore:

- same-class exact beats same-class rest;
- inherited exact beats subclass rest;
- subclass accepting rest beats superclass accepting rest;
- subclass non-accepting rest allows superclass rest to be considered.

Do not perform:

```text
Subclass exact
Subclass rest
Superclass exact
...
```

That would violate F.3 semantics.

---

# 21. Shared rest fallback lookup

Factor a cold helper for the second pass, conceptually:

```rust
fn rest_fallback_lookup(
    &self,
    start_class: ClassId,
    base_name: Symbol,
    positional_count: usize,
    labels: &[Symbol],
) -> Option<ObjRef>;
```

Algorithm:

```text
class = start_class
while class exists:
    if class.rest_methods[base_name] exists:
        method = entry
        layout = method.signature.rest (must exist)
        if layout.accepts(positional_count, labels):
            return method
    class = superclass(class)
return None
```

The first **accepting** entry wins.

A present-but-non-accepting entry is a pattern miss, not an arity exception.

---

# 22. Access control and compiler-internal authority

Rest fallback must preserve the same access/visibility semantics as exact method selection.

The lookup helper may locate the first accepting method, but invocation must use the ordinary authorization policy.

Do not let rest dispatch bypass private/internal method rules.

For F.2 `InvokePack` with:

```rust
PackAccess::CompilerInternal
```

compiler authority applies to the selected exact or rest method during lookup/authorization exactly as it does for existing compiler-internal sends.

Authority must not leak into the callee frame beyond the existing permitted lifetime.

An authorization failure for the selected accepting rest method is not a reason to silently continue searching superclasses unless ordinary exact lookup already has that behavior.

---

# 23. Static `Invoke` integration

Ordinary static sends retain the existing fast exact path.

Only after exact lookup misses everywhere should static dispatch obtain the concrete call shape and invoke `rest_fallback_lookup`.

The shape required is:

```text
base_name
positional_count
ordered labels
```

Do not parse human-formatted selector strings on every miss.

Preferred sources, in order:

1. existing structured selector/signature metadata already associated with the selector constant;
2. a cached structured selector decode stored at selector interning/constant creation;
3. only if unavoidable during migration, the repository's canonical selector decoder — never ad-hoc string parsing.

F.3 should not add a per-successful-Invoke branch merely to precompute rest data.

---

# 24. F.2 `InvokePack` integration

F.2 assembles a completed dynamic builder whose parts are already available before/while the opcode flattens the call window:

```text
positionals
labels
labeled_values
```

After F.3, `InvokePack` dispatch order is:

1. validate completed builder and dynamic arity as F.2 specifies;
2. derive the original concrete selector;
3. exact lookup across the applicable class chain;
4. on exact miss, call shared rest-family fallback with:
   - F.2 base name;
   - `positionals.len()`;
   - ordered `labels`;
5. if a rest method is selected, repack the flat call window for that declaration;
6. otherwise forward ordinary dNU with the original concrete selector and original flat arguments.

The old F.2/U9 `name(*)` variadic probe is removed.

Dynamic and static calls with the same concrete pack shape must resolve to the same method.

---

# 25. Super integration

## 25.1 Static super

If static `SuperSend` can target a rest-capable family, exact lookup begins above the defining class.

After exact miss, rest fallback begins at that same superclass start point.

## 25.2 Dynamic super

For `SuperSendPack`, use F.2's existing defining-class semantics.

Both passes begin above the defining class:

```text
exact super lookup
then rest-family super fallback
```

Never restart rest fallback from the receiver's dynamic class.

---

# 26. Reflective ordinary send

If a reflection API models an ordinary language-level message send, it must use the same two-pass semantics:

```text
exact
rest
then dNU/error according to ordinary send behavior
```

Do not create a reflection-only rest matcher with different precedence.

Direct invocation of a specific `MethodObject` is different: if that API bypasses selector lookup and the provided arguments do not satisfy the method's required shape, use the repository's existing `Arity`/ArgumentError convention rather than searching another family member.

---

# 27. dNU semantics

If no exact method and no accepting rest fallback exists, dNU receives exactly what was sent:

- the original concrete selector;
- the original flat positional argument values;
- the original flat labeled argument values;
- the normal decoded concrete label information used by existing dNU machinery.

Never forward:

```text
sum(*)
foo(_,*,x,**)
```

or any wildcard/rest selector merely because it was probed as a fallback.

Do not rewrite the argument window destructively before the rest candidate has been selected.

---

# 28. Rest-aware frame binding

Before entering a selected rest method's bytecode frame, the VM has the original flat call window:

```text
receiver
actual positional values...
actual labeled values...
```

and the concrete shape:

```text
P
L
```

Repack that window into declaration-local parameter order before pushing the ordinary closure frame.

The compiled body must see normal parameter locals only.

---

# 29. Binding layouts by mode

Use `RestLayout` parameter indices or equivalent declaration-order metadata to place captures exactly.

Conceptually:

## 29.1 Positional

Declaration shape:

```text
fixed positionals
*positional_rest
fixed labels
```

Frame arguments:

```text
receiver
fixed positional values
positional capture Tuple/Unit
fixed labeled values
```

## 29.2 Labeled

Declaration shape:

```text
fixed positionals
fixed labels
**labeled_rest
```

Frame arguments:

```text
receiver
fixed positional values
fixed labeled values
labeled capture Tuple/Unit
```

## 29.3 Split

Declaration shape:

```text
fixed positionals
*positional_rest
fixed labels
**labeled_rest
```

Frame arguments:

```text
receiver
fixed positional values
positional capture Tuple/Unit
fixed labeled values
labeled capture Tuple/Unit
```

## 29.4 Complete

Declaration shape:

```text
fixed positionals
fixed labels
***complete_rest
```

Frame arguments:

```text
receiver
fixed positional values
fixed labeled values
complete capture Tuple/Unit
```

Exact physical order must follow the compiler's actual local-slot declaration order. Parameter indices in `RestLayout` prevent the VM from guessing based on source text.

---

# 30. Frame repacker design

Factor one helper, conceptually:

```rust
fn repack_rest_call_window(
    &mut self,
    receiver_idx: usize,
    method: ObjRef,
    positional_count: usize,
    labels: &[Symbol],
    source_range: SourceRange,
) -> PhResult<()>;
```

or a shape-equivalent internal API.

Responsibilities:

1. assume the selected layout already accepts the concrete shape;
2. identify fixed positional values;
3. identify fixed labeled values;
4. identify residual lane slices;
5. construct required capture products;
6. rewrite the receiver/argument window into declaration-local order;
7. leave exactly the arguments expected by the ordinary bytecode method frame entry.

It must not:

- redo inheritance lookup;
- parse selector strings;
- invoke user code;
- stage through List/Map;
- alter dNU state on a non-selected candidate.

---

# 31. GC and allocation safety during capture construction

Capture construction can allocate Tuple objects.

F.3 relies on the current VM invariant already used by F.2: collection does not run at arbitrary mid-opcode points while temporary Rust locals are being rearranged.

Nevertheless, prefer this safe ordering:

```text
1. keep the original flat call window intact/rooted;
2. construct required capture value(s);
3. only then destructively rewrite the operand window;
4. push the callee frame.
```

In Split mode, two capture products may be required.

If the collector architecture later gains mid-helper/mid-opcode safepoints, capture temporaries must be explicitly rooted on the VM stack or through the collector's root API before any subsequent allocation.

F.3 must document this assumption where the repacker is implemented.

---

# 32. U9 call-prologue retirement

Delete the old call-prologue behavior conceptually equivalent to:

```rust
if signature.variadic {
    let rest = args.split_off(fixed);
    let list = heap.alloc_list(rest);
    ...
}
```

Rest packing belongs only to the F.3 rest-aware repacker.

After F.3:

```text
rest binding type = Unit or Tuple
```

never List.

Existing U9 fixtures asserting List identity must be migrated.

---

# 33. Signature metadata migration

Preferred final `Signature` state:

```rust
pub struct Signature {
    // existing exact selector/signature fields
    ...
    pub rest: Option<RestLayout>,
}
```

`positional_arity` continues to report fixed/minimum positional arity for compatibility.

Migration sequence:

```text
1. add RestLayout/Signature.rest
2. migrate compiler construction
3. migrate VM readers to Signature.rest
4. migrate reflection/arity readers
5. remove old List prologue
6. remove U9 name(*) fallback/cache
7. delete SignatureKind::Variadic
8. delete Signature.variadic storage
9. delete ParameterDef.is_rest boolean once all AST users are migrated
```

A temporary accessor such as:

```rust
fn is_rest_capable(&self) -> bool {
    self.rest.is_some()
}
```

is acceptable.

A second stored boolean is not.

---

# 34. Reflection compatibility

Until the separate reflection/type specification lands:

```text
Method#arity
```

continues to mean the fixed/minimum positional arity.

F.3 does not introduce public APIs such as:

```text
restMode
ArgumentPackType
fixedLabelPrefix
restParameterIndex
```

The method's printable selector may become structurally more accurate through the canonical wildcard formatting described above.

Literal label `#*` remains distinguishable from the positional-rest marker through F.1 escaping.

---

# 35. Native methods

Unless a pre-existing primitive has an explicitly ratified compatible ABI, reject native methods carrying `Signature.rest`.

Reason:

F.3 defines the bytecode-method frame ABI as declaration-repacked fixed slots plus Tuple/Unit captures. It does not define whether a native primitive should receive:

```text
raw actual arguments
or
already-repacked declaration arguments
```

That is a future primitive-ABI design decision, not an implicit F.3 behavior change.

---

# 36. Error taxonomy

Add structured compiler/install diagnostics following repository conventions.

Recommended categories:

```rust
DuplicateRestFamilyDeclaration {
    class: ...,
    base_family: ...,
    first_selector: ...,
    second_selector: ...,
}

DuplicatePositionalRestParameter { ... }
DuplicateLabeledRestParameter { ... }
DuplicateCompleteRestParameter { ... }

ParameterAfterLabeledRest { ... }
ParameterAfterCompleteRest { ... }

ConflictingCompleteAndPositionalRest { ... }
ConflictingCompleteAndLabeledRest { ... }

NativeRestMethodUnsupported { ... }
```

Exact enum names should match repository style.

Diagnostics should:

- point at the second/conflicting declaration or parameter;
- render human-readable class and selector names;
- never expose raw Symbol IDs;
- avoid generic string-only errors when a structured variant fits existing conventions.

A rest-family entry that simply does not accept a call is **not** an error. It is a lookup miss.

---

# 37. Performance requirements

Rest fallback is a miss-path feature, but it is still dispatch and should remain lean.

Required properties:

- successful exact static `Invoke` incurs no wildcard selector construction;
- successful exact static `Invoke` incurs no rest-family hash lookup;
- rest-family lookup is O(class depth) with O(1)-average per-class base-name lookup;
- acceptance allocates nothing;
- fixed-label prefix matching compares Symbol identities;
- no method-table scan;
- no wildcard selector parsing;
- no user hash/equality/string conversion;
- capture allocation occurs only after a rest method has actually been selected;
- exact dNU behavior remains unchanged on total miss.

F.2 dynamic sites remain uncached in their initial implementation. F.3 does not introduce a new dynamic/rest inline cache unless profiling separately justifies it.

---

# 38. Source/component changes

Exact file paths must follow repository HEAD. The expected components are:

## 38.1 Method metadata

Likely area:

```text
phalcom-core/src/method/
```

Modify:

- parameter/rest metadata used by compiler;
- `Signature`;
- selector formatting/reflection helpers;
- remove U9 `SignatureKind::Variadic` by completion.

## 38.2 Class object / universe

Modify the repository-current `ClassObject` definition:

```rust
rest_methods: HashMap<Symbol, ObjRef>
```

Update:

- allocation/default construction;
- GC tracing only if the current class object requires explicit tracing of method refs;
- class debug/display if exhaustive;
- method installation;
- reflective method definition;
- cache/world invalidation hooks.

## 38.3 Compiler

Likely areas:

```text
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/scope.rs
```

Implement:

- lane-aware rest binder parsing/normalization from the current AST;
- ordering validation;
- `RestLayout` construction;
- canonical structural rest selector formatting;
- duplicate rest-family validation within one class declaration;
- native-rest rejection if method kind is known at compile time.

## 38.4 VM dispatch

Likely area:

```text
phalcom-core/src/vm/mod.rs
```

Implement/factor:

- shared rest fallback lookup;
- shared capture constructor;
- rest call-window repacker;
- static exact-miss integration;
- `InvokePack` exact-miss integration;
- static/dynamic super integration;
- reflective ordinary-send integration if applicable;
- U9 List-prologue removal;
- U9 wildcard-probe/cache removal.

## 38.5 Errors

Likely area:

```text
phalcom-core/src/error.rs
```

Add structured F.3 declaration/install diagnostics.

## 38.6 Tests

Add focused F.3 fixtures/unit tests in the repository's current test layout.

---

# 39. Implementation Phase 0 — prerequisite and repository-state audit

Before editing:

- confirm amended F.2 has landed;
- identify actual `ClassObject` source file;
- identify the single authoritative method-install path;
- identify reflective method-definition path;
- identify current `Signature` and `SignatureKind` users;
- identify every `is_rest`/`variadic`/`Variadic(...)` reader;
- identify U9 `name(*)` cache/probe;
- identify current call-prologue List packing;
- identify selector structured metadata/decoder available to static `Invoke` miss handling;
- identify `SuperSend` and `SuperSendPack` start-class helpers;
- confirm A.3 `finish_tuple` API;
- confirm GC safepoint behavior used during VM opcode execution;
- confirm current block syntax remains separate and make no block-rest edits.

Do not copy obsolete function signatures from earlier plans if HEAD differs.

---

# 40. Implementation Phase 1 — metadata model

Implement:

```text
RestParameterKind
RestMode
RestLayout
Signature.rest
```

Add unit tests for:

- normalization to Positional/Labeled/Split/Complete;
- invalid combination rejection helpers;
- `accepts` for every mode;
- ordered prefix behavior;
- zero-allocation acceptance where test infrastructure can reasonably assert it.

At this phase, old U9 fields may temporarily remain only to keep the tree compiling while readers migrate.

They must no longer be authoritative after compiler construction switches to `Signature.rest`.

---

# 41. Implementation Phase 2 — selector formatting

Replace lossy U9 rest selector rendering with the structural formatter.

Test at minimum:

```text
sum(*)
format(_,*)
log(_,*,format,**)
options(_,debug,**)
forward(_,timeout,***)
```

using the repository's canonical whitespace-free output.

Also test escaped literal labels corresponding to:

```text
#*
#**
#***
#_
```

No runtime fallback should depend on these strings.

---

# 42. Implementation Phase 3 — compiler validation and RestLayout construction

Update class/method declaration compilation.

Implement:

- lane-aware rest binder extraction;
- fixed positional count calculation;
- fixed ordered label-prefix extraction;
- rest parameter local-slot/index capture;
- mode normalization;
- ordering diagnostics;
- complete-vs-split conflict diagnostics;
- one-rest-family-per-class declaration check;
- native-rest rejection.

Ensure examples/tests use current no-colon parameter syntax.

---

# 43. Implementation Phase 4 — class rest index and installation path

Add `rest_methods` to `ClassObject`.

Refactor method installation only as much as necessary so ordinary and reflective installs share the invariant.

Tests:

- first rest method installs;
- exact methods in same family still install;
- conflicting second rest pattern rejects;
- different base family installs;
- same selector replacement follows existing repository replacement rules;
- reflective install cannot bypass conflict detection;
- superclass and subclass may each own their own rest entry for the same base family.

---

# 44. Implementation Phase 5 — capture constructor and repacker

Implement `finish_capture` first.

Test directly:

- zero lanes -> Unit;
- one positional -> singleton Tuple;
- multiple positionals -> Tuple;
- labeled-only -> labeled Tuple in order;
- both lanes -> combined Tuple in canonical product order.

Then implement the rest call-window repacker.

Test each mode with fixed slots and residual captures before wiring lookup.

Do not reuse U9 List packing internally as a temporary staging representation.

---

# 45. Implementation Phase 6 — shared rest fallback lookup

Implement the class-chain helper independently of send opcodes.

Tests:

- no entry -> None;
- accepting local entry -> local method;
- non-accepting local entry -> accepting superclass entry;
- accepting local entry -> superclass not consulted;
- no accepting entry -> None;
- label-order mismatch -> miss;
- exact label-prefix Symbol identity/order respected.

No selector strings should be synthesized in these tests.

---

# 46. Implementation Phase 7 — static exact-miss integration

Wire ordinary static `Invoke` miss handling:

```text
existing exact lookup
-> shared rest fallback
-> existing dNU
```

Preserve the successful exact hot path.

If the static selector representation does not already expose base name/count/labels efficiently, add or reuse a structured selector decode/cache at an appropriate metadata boundary rather than reparsing display strings on every miss.

Tests:

- exact same-class beats rest;
- inherited exact beats subclass rest;
- exact miss reaches rest;
- total miss reaches unchanged dNU.

---

# 47. Implementation Phase 8 — F.2 dynamic dispatch migration

Update `InvokePack` from the temporary F.2/U9 semantics:

```text
exact
old variadic probe
DNU
```

to:

```text
exact
shared F.3 rest fallback
DNU
```

Use builder parts already available in F.2 for `P` and ordered `L`.

Do not rebuild/decode wildcard selector patterns.

Tests:

- dynamic equivalent of a static call resolves the same exact method;
- dynamic exact miss resolves the same rest method as static call;
- dynamic labels preserve order during rest acceptance;
- dynamic total miss sends original concrete selector to dNU;
- dynamic compiler-internal access semantics remain intact.

---

# 48. Implementation Phase 9 — super integration

Wire static and dynamic super exact-miss fallback where applicable.

Tests:

- exact super method wins;
- rest lookup starts above defining class;
- receiver subclass's own rest entry is not reconsidered;
- non-accepting immediate superclass rest allows higher superclass rest;
- total miss follows existing super/dNU behavior.

---

# 49. Implementation Phase 10 — reflection integration

Update reflection-facing method metadata:

- `Method#arity` remains fixed/minimum positional arity;
- selector text uses structural rest formatting;
- literal star-like labels remain distinct.

If reflective ordinary send exists, route it through the same exact/rest dispatch seam.

If reflective method installation exists, confirm index updates and duplicate-family rejection.

Do not add new public rest-introspection APIs in F.3.

---

# 50. Implementation Phase 11 — U9 deletion

Once all F.3 paths are green, remove:

```text
ParameterDef.is_rest boolean semantics
SignatureKind::Variadic
Signature.variadic storage
variadic_selector_cache
name(*) fallback probe
List rest call-prologue packing
U9-only tests and helpers
```

Search repository-wide for:

```text
is_rest
variadic
Variadic
name(*)
alloc_list(rest
```

and classify every remaining occurrence deliberately.

No compatibility branch may remain in production dispatch after F.3 completion.

---

# 51. Implementation Phase 12 — diagnostics, documentation, regression/performance

Finish:

- structured compile/install errors;
- source spans;
- disassembler/reflection output;
- method/signature documentation;
- any normative memory/dispatch docs;
- full test suite;
- static-send performance comparison.

Confirm F.3 did not add work to successful exact static sends beyond unavoidable code-layout effects.

---

# 52. Test plan — capture values

Required cases:

```text
method(*args) called with 0 residual positionals
    -> args == Unit

method(*args) called with 1 residual positional
    -> singleton Tuple

method(*args) called with many
    -> positional Tuple

method(**labels) with zero residual labels
    -> Unit

method(**labels) with residual labels
    -> labeled Tuple preserving order

method(***remaining) with zero residual values/labels
    -> Unit

method(***remaining) with both residual lanes
    -> one Tuple preserving both lanes
```

Also assert captures are not List.

---

# 53. Test plan — matching

Required cases:

- positional rest enforces minimum `F`;
- positional rest rejects unexpected labels;
- labeled rest requires exact positional count `P == F`;
- labeled rest accepts fixed label prefix plus extras;
- split accepts residuals in both lanes;
- complete accepts residuals in both lanes;
- wrong fixed label order does not match;
- missing fixed label prefix entry does not match;
- extra positional values do not match labeled-only rest;
- no user equality/hash/string method is invoked during matching.

---

# 54. Test plan — dispatch precedence

Required cases:

- exact same-class selector beats same-class rest fallback;
- inherited exact selector beats subclass rest fallback;
- subclass accepting rest beats superclass rest;
- subclass non-accepting rest allows superclass rest;
- no rest fallback -> ordinary dNU;
- same concrete pack resolves identically through static `Invoke` and dynamic `InvokePack`;
- exact search is demonstrably complete before rest search begins.

A useful regression is:

```text
Subclass:   rest foo(...)
Superclass: exact foo(concrete-shape)
```

The superclass exact method must win.

---

# 55. Test plan — declarations and installation

Required compiler/install failures:

- two rest methods same base/class;
- duplicate positional rest binder;
- duplicate labeled rest binder;
- duplicate complete rest binder;
- positional + complete rest;
- labeled + complete rest;
- split + complete rest;
- parameter after `**rest`;
- parameter after `***rest`;
- unsupported native rest method;
- reflective second rest-family installation.

Diagnostics should name class/base family and both rest selectors for the family conflict.

---

# 56. Test plan — U9 migration

Required regressions:

- old rest binding is no longer List;
- `sum(*numbers)` still computes the same result when the implementation works with Tuple/Unit;
- fixed + positional rest works;
- fixed labeled prefix + labeled rest works;
- split rest works;
- complete rest works;
- exact fixed overload still wins;
- old `name(*)` probe is not used;
- no U9 variadic cache remains active.

---

# 57. Test plan — dynamic F.2 interaction

Required cases:

- `InvokePack` exact hit remains exact;
- dynamic positional expansion selects positional rest when exact misses;
- dynamic labeled expansion selects labeled/split/complete rest according to shape;
- ordered dynamic labels are not reordered for matching;
- dynamic dNU sees the concrete selector;
- F.2 subscript send kinds do not accidentally enter method-family rest fallback unless the language explicitly defines rest-capable subscript methods; preserve current selector-kind eligibility rules;
- dynamic send arity limit remains F.2's responsibility before dispatch;
- rest capture Tuple size is not constrained by an additional F.3 `u8` check.

---

# 58. Test plan — super

Required cases:

- static super exact hit;
- static super exact miss -> accepting rest above defining class;
- dynamic super exact miss -> accepting rest above defining class;
- current receiver class rest entry is ignored for super fallback;
- non-accepting superclass entry allows next superclass;
- total miss preserves current super/dNU behavior.

---

# 59. Test plan — reflection

Required cases:

- rest method `arity` == fixed/minimum positional arity;
- selector text includes fixed positional slots;
- selector text includes fixed labeled-prefix slots;
- selector text includes unambiguous `*`/`**`/`***` marker;
- literal `#*` selector label remains distinct from positional rest marker;
- reflective method installation updates `rest_methods`;
- reflective ordinary send, if supported, follows exact-before-rest semantics.

---

# 60. Test plan — GC/stress

Under repository GC stress facilities if available:

- residual values remain live while capture Tuple is allocated;
- split mode safely constructs two captures;
- call-window rewrite does not lose object roots;
- nested allocation during invocation does not expose a partially rewritten frame;
- abandoned invocation after an error leaves no leaked private staging object, noting F.3 itself adds no pack-builder object beyond F.2.

---

# 61. Test plan — static performance regression

Benchmark representative successful exact sends before/after F.3:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
```

Expected:

```text
no rest-family lookup on exact hit
no wildcard selector rendering
no capture allocation
no method-table scan
```

Rest fallback cost belongs only to exact misses.

---

# 62. Verification commands

Use repository-current commands, approximately:

```bash
cargo fmt --check
cargo test -p phalcom-core
cargo test -p phalcom-ast
cargo test -p phalcom-core -- rest
cargo test -p phalcom-core -- f3
```

If the repository uses an integration fixture:

```bash
cargo test -p phalcom-core --test f3_rest_capture
```

or the current equivalent.

Also run the repository-standard clippy/lint target used by CI.

Disassemble representative exact/rest methods and static/dynamic calls.

Search for stale U9 implementation references before completion.

---

# 63. Manual verification matrix

Manually inspect these behaviors:

```text
1. sum(*numbers) still computes correctly.
2. Rest binding is Tuple/Unit, never List.
3. Exact selector in same family wins.
4. Inherited exact selector wins over subclass rest.
5. Wrong label order fails rest acceptance.
6. Dynamic and static equivalent calls resolve identically.
7. dNU receives the original concrete selector.
8. Super rest starts above defining class.
9. Literal #* label remains distinct from wildcard marker.
10. No block-rest syntax/runtime behavior has been introduced.
```

---

# 64. Correctness invariants

The implementation must satisfy all of these:

- [ ] `Signature.rest == None` is the only ordinary no-rest runtime state.
- [ ] Rest-capable signatures use Positional/Labeled/Split/Complete metadata.
- [ ] `is_rest: bool` is no longer semantically authoritative.
- [ ] `SignatureKind::Variadic` is removed by F.3 completion.
- [ ] `Signature.variadic` storage is removed by F.3 completion.
- [ ] Rest captures are Unit or Tuple only.
- [ ] Fixed labeled parameters match an ordered prefix.
- [ ] Matching never reorders/searches labels by name.
- [ ] Matching uses no argument value types.
- [ ] Matching allocates nothing.
- [ ] Exact lookup completes across inheritance before rest lookup begins.
- [ ] One rest pattern per base family per class is enforced.
- [ ] Reflective install cannot bypass that invariant.
- [ ] Subclass non-accepting rest allows superclass fallback.
- [ ] Static and dynamic Method sends share the same fallback algorithm.
- [ ] Non-Method selector kinds do not accidentally enter rest-family fallback.
- [ ] Super fallback starts at the exact same superclass boundary as exact super lookup.
- [ ] Rest selector strings are not parsed/probed for dispatch.
- [ ] U9 `name(*)` fallback/cache is gone.
- [ ] Old List call-prologue packing is gone.
- [ ] dNU receives the original concrete selector and original flat args.
- [ ] Successful exact static calls perform no rest lookup.
- [ ] Native rest methods are not silently accepted.
- [ ] Public primitive-floor delta remains 0.
- [ ] Block rest remains unimplemented in F.3 and reserved for F.4.

---

# 65. Completion checklist

F.3 is complete when:

- [ ] F.2 amended dynamic-pack path is the active prerequisite.
- [ ] Parameter AST/compiler representation distinguishes `*`, `**`, and `***` rest binders.
- [ ] `RestMode` and `RestLayout` are implemented.
- [ ] `Signature.rest` is the single runtime source of truth.
- [ ] Invalid rest combinations/orderings produce structured diagnostics.
- [ ] Structural rest selector formatting is canonical and unambiguous.
- [ ] `ClassObject` has a rest-family index.
- [ ] Normal and reflective method installation maintain the index.
- [ ] One-rest-pattern-per-family is enforced per class.
- [ ] `RestLayout::accepts` implements the four exact predicates.
- [ ] Shared rest fallback lookup walks inheritance deterministically.
- [ ] Exact lookup always precedes rest fallback across the full chain.
- [ ] `finish_capture` finalizes Unit/Tuple directly through A.3 semantics.
- [ ] VM rest-call repacker rewrites into declaration-local order before frame entry.
- [ ] Eligible static Method `Invoke` exact miss uses shared rest fallback.
- [ ] Eligible dynamic Method `InvokePack` exact miss uses shared rest fallback.
- [ ] Super exact miss uses correct above-defining-class rest fallback.
- [ ] Reflective ordinary send is consistent if present.
- [ ] dNU receives the original concrete call.
- [ ] `Method#arity` remains fixed/minimum positional arity.
- [ ] Native rest methods are rejected unless separately ratified.
- [ ] U9 List packing is removed.
- [ ] U9 wildcard probe/cache is removed.
- [ ] `SignatureKind::Variadic` is removed.
- [ ] `Signature.variadic` storage is removed.
- [ ] Static exact-send performance shows no unexplained regression.
- [ ] No type-based dispatch is introduced.
- [ ] No block-rest implementation is included.
- [ ] Public primitive-floor delta is 0.

---

# 66. Decision record summary

| Decision | Ruling |
|---|---|
| F.2 prerequisite | Must be fully landed before F.3 |
| Block rest | Deferred explicitly to F.4 |
| Current block syntax | `|x| { ... }`; unchanged by F.3 |
| Labeled parameter declaration syntax | No trailing colon |
| Runtime no-rest representation | `Signature.rest == None` |
| Rest modes | Positional, Labeled, Split, Complete |
| Rest metadata owner | `Signature` |
| Rest selector text | Reflection/debug description, not fallback key |
| Rest captures | Unit/Tuple only |
| Label matching | Ordered prefix, Symbol identity |
| Type-based dispatch | None |
| Rest patterns per base/class | At most one |
| Rest index owner | `ClassObject` |
| Exact vs rest | Full exact pass first, then full rest pass |
| Subclass non-accepting rest | Continue to superclass |
| Static hot path | Keep exact-hit path materially unchanged |
| Dynamic F.2 fallback | Replace U9 variadic probe with F.3 rest fallback |
| Rest-eligible selector domain | Ordinary Method families only in F.3 |
| Super fallback start | Above defining class |
| dNU | Original concrete selector + original flat args |
| Capture construction | Shared VM helper via A.3 `finish_tuple` |
| Frame binding | VM repacks before bytecode frame entry |
| Native rest methods | Unsupported in F.3 |
| `Method#arity` | Fixed/minimum positional arity |
| `SignatureKind::Variadic` | Remove by F.3 completion |
| `Signature.variadic` | Remove by F.3 completion |
| U9 List packing | Remove |
| Public primitive bindings | None |

---

# 67. Non-goals and follow-ups

F.3 does not include:

- block/closure rest parameters — reserved for F.4;
- labeled or complete block-rest syntax;
- native/primitive rest ABI design;
- multiple rest patterns in one base family/class;
- wildcard specificity or ambiguity ranking;
- public rest-signature reflection APIs beyond existing `arity` and selector text;
- argument-pack type reflection;
- type-based overload selection;
- dynamic/rest inline caching;
- widening VM send arity;
- changes to F.2 pack-builder syntax/assembly semantics;
- new public primitive methods.

Future work should remain separate:

```text
F.4:
    block/closure rest support, when explicitly specified

primitive ABI follow-up:
    native rest methods

selector-dispatch follow-up:
    multiple rest patterns + specificity/ambiguity rules

reflection/type follow-up:
    structured rest-signature introspection

performance follow-up:
    dynamic/rest dispatch cache only if profiling warrants it
```

---

# 68. Final implementation guidance

F.3 should preserve a strict separation of responsibilities:

```text
F.2 outgoing pack assembly:
    evaluate source arguments
    preserve lexical timing
    produce canonical lanes

selector exact lookup:
    retain existing fast path

F.3 rest fallback:
    inspect only base family + canonical call shape
    walk class rest-family index
    apply allocation-free RestLayout acceptance

F.3 binding:
    construct residual Tuple/Unit captures
    repack into declaration-local parameter order
    enter ordinary method frame

reflection/formatting:
    render structural wildcard selector text
    never drive runtime matching
```

The critical architectural rule is:

```text
exact dispatch and rest-pattern dispatch are two phases of the same message-send semantics,
not two competing dispatch systems.
```

F.3 therefore adds a narrow exact-miss fallback and a pre-frame argument repacker while leaving successful exact dispatch materially untouched.

That is the intended implementation boundary.
