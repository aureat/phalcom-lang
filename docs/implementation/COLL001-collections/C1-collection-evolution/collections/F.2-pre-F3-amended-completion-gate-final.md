# Pre-F.3 Amended Completion Gate
## F.1/F.2 stabilization, migration cleanup, and implementation plan

Status: **final implementation specification — hard prerequisite for F.3-amended**

Repository baseline audited:

```text
repository: aureat/phalcom-lang
branch:     main
commit:     8d6b975bc1413c5a856f1ca9261718bd53bd4e6c
```

Downstream contract:

```text
F.3-rest-capture-and-rest-pattern-dispatch-amended.md
```

This document supersedes the earlier draft:

```text
F.2-pre-F3-blockers-implementation-spec.md
```

It does **not** supersede F.1, F.2 amended, the F.2 completion supplement, or F.3-amended. It closes implementation gaps and records migration decisions required to make those specifications mutually implementable against the audited repository state.

F.3-amended must not begin until every item in the hard completion gate in §18 is satisfied.

---

# 1. Mission

Establish a clean, tested, internally consistent F.1/F.2 baseline before F.3 changes rest capture, method signatures, selector rendering, dispatch fallback, and frame binding.

The required pre-F.3 work is:

1. repair the LSP PackItem migration compile failure;
2. repair Tuple parsing so F.1 expansion syntax works in every legal Tuple position;
3. enforce static duplicate argument-label rejection instead of allowing static sends to bypass builder validation;
4. surface the dedicated F.2 `NonIterableStarOperand` error without corrupting normal iterator exceptions;
5. stop constructor and subscript rest markers from being silently erased by pre-F.3 compilation;
6. align the LSP's current valid U9 positional-rest selector spelling with the runtime until F.3 replaces both;
7. complete the full F.2 behavioral, ordering, fiber, GC, dNU, and static-fast-path regression suite;
8. remove contradictory active documentation and stale implementation comments left by the F.1/F.2 migration;
9. remove unchecked product-count narrowing in static Tuple/Record construction;
10. record the corrected F.3 handoff assumptions so F.3 does not reintroduce obsolete AST migrations.

The goal is not merely "the code compiles." The goal is:

```text
one parser model
one pack AST model
one selector-label model
one outgoing-pack runtime model
one explicit transitional U9 rest model
no silent semantic erasure
complete tests around the seam F.3 is about to replace
```

---

# 2. Authority and specification precedence

For this implementation unit, use the following precedence when repository documents disagree.

## 2.1 Outgoing pack syntax and source phases

Authoritative:

```text
docs/work/pending/collections/F.1-pack-syntax-ast-and-selector-encoding.md
docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
docs/work/pending/collections/F.2-supplement-completion-gaps-before-F.3.md
```

The critical F.1 rules to preserve are:

```text
calls and Tuple construction share pack source phases

positional phase accepts:
    ordinary positional
    *expr
    ***expr

explicit/computed label or **expr starts labeled phase

labeled phase accepts:
    explicit/computed labels
    **expr

after labeled phase begins:
    ordinary positional, *expr, and ***expr are illegal

*** does NOT itself start labeled source phase

multiple *** expansions are legal

* / ** / *** may coexist subject to source-phase rules
```

Older documents that say otherwise are stale and must be amended or marked superseded in §14.

## 2.2 F.2 runtime behavior

F.2 amended and its completion supplement are authoritative.

Retain the landed architecture:

```text
ArgumentPackBuilderObject
PackTryExpandTuplePositionals
PackPushPositional
PackReserveStaticLabel
PackReserveComputedLabel
PackFillReservedLabel
PackExpandLabels
PackExpandComplete
FinishTuplePack
InvokePack
SuperSendPack
```

Do not redesign these paths except where this document identifies a specific correctness defect.

## 2.3 Rest behavior before F.3

The repository is intentionally transitional.

Before F.3:

```text
valid live rest behavior:
    final positional *rest
    SignatureKind::Variadic
    Signature.variadic
    name(*) runtime selector
    trailing positional values captured into List

parsed but not runtime-supported:
    **rest
    ***rest
```

This U9 behavior is **not** to be removed in this pre-F.3 unit.

F.3-amended is responsible for replacing it with:

```text
Signature.rest
RestLayout
Unit/Tuple capture
structured rest-family dispatch
```

## 2.4 F.3-amended is the downstream boundary

This pre-F.3 unit must not partially implement:

```text
RestLayout
Signature.rest
rest_methods class index
rest-family fallback lookup
Tuple/Unit rest frame repacking
structural F.3 wildcard selector identity
U9 deletion
```

Those belong to F.3-amended.

---

# 3. Verified repository state to preserve

The following audited implementation is already sound enough to retain.

## 3.1 Pack AST migration

The live AST uses:

```rust
PackItem
PackLabel
ExpansionMode
TupleLiteralEntry
```

The old call-site `Argument { label, expr }` model is no longer the authoritative representation in core/compiler code.

Do not reintroduce an `Argument` compatibility structure.

## 3.2 Generic positional expansion lowering

Current:

```text
phalcom-core/src/compiler/lib/expr.rs
Compiler::compile_positional_pack_expansion
```

already performs the correct architecture:

```text
E.3 boundedness check
evaluate source once
store source in hidden local
reserve cursor hidden local
PackTryExpandTuplePositionals
generic fallback:
    source.iterate(None)
    source.iteratorValue(cursor)
    source.iterate(cursor)
hidden locals root state across arbitrary sends/fiber suspension
```

Do **not** rewrite this into:

```text
Rust-side iteration
each(...)
native spread helper
re-entrant VM exhaustion
Tuple iteration
```

The only required change to this path is the structured non-Iterable failure boundary defined in §8.

## 3.3 Dynamic SetIndex lowering

Current dynamic SetIndex lowering already:

```text
evaluates receiver once
assembles source index pack
reserves compiler-owned #put before RHS
evaluates RHS once
fills #put
dispatches InvokePack(SubscriptSet)
discards setter result
restores original RHS as assignment-expression result
```

Keep this architecture.

Add tests; do not redesign it.

## 3.4 Dynamic Tuple lowering

Current dynamic Tuple lowering already uses:

```text
ArgumentPackBuilderObject
the normal pack contribution operations
FinishTuplePack
A.3 finish_tuple
```

Keep it.

The parser reachability bug in §6 is what currently prevents all legal F.1 Tuple source forms from reaching this lowering.

## 3.5 Structured F.2 runtime errors already present

Retain:

```rust
ComputedLabelNotSymbol
DuplicateArgumentLabel
InvalidStarStarOperand
NonSymbolMapKeyInExpansion
InvalidStarStarStarOperand
SendArityExceedsLimit
```

Only `NonIterableStarOperand` remains missing.

## 3.6 Selector label escaping

Core and LSP both have the reversible F.1 escape scheme that prevents literal labels:

```text
#_
#*
#**
#***
```

from colliding with structural slot markers.

Do not replace this encoder.

## 3.7 GC/fiber architecture

Retain these assumptions:

```text
pack builder is heap-owned and GC traced
builder Value payloads are traced
compiler hidden locals root source/cursor/builder/RHS
allocation latches GC rather than collecting arbitrarily mid-opcode
fiber stacks/frames participate in tracing
```

F.2 still needs regression tests proving these paths under stress.

---

# 4. Hard decisions for this completion unit

## G1 — U9 stays until F.3

Do not remove:

```rust
SignatureKind::Variadic
Signature.variadic
name(*) probing
List rest packing
```

before F.3.

The pre-F.3 job is to make the outgoing-pack baseline reliable enough that F.3 can remove U9 atomically.

## G2 — Existing AST rest representation is authoritative

Current AST already has:

```rust
pub enum RestMode {
    None,
    Positional,
    Labeled,
    Complete,
}

pub struct ParameterDef {
    ...
    pub rest_mode: RestMode,
}
```

Therefore F.3 must **not** create a second parser-side `RestParameterKind` merely because an older migration description assumed `is_rest: bool`.

The future normalization boundary is:

```text
AST:
    RestMode::None / Positional / Labeled / Complete

F.3 runtime:
    Signature.rest == None
    or RestLayout(...)
```

`RestMode::None` may remain parser-facing while F.3 normalizes it away before runtime `Signature` construction.

## G3 — Subscript rest is illegal

F.3-amended explicitly limits rest fallback to ordinary Method families.

Therefore:

```phalcom
[*rest] { ... }
[**rest] { ... }
[***rest] { ... }
```

and setter equivalents are compile/syntax errors.

Do not silently encode them as fixed subscript parameters.

## G4 — Constructor rest is rejected until explicitly specified

The current F.3-amended document does not define constructor factory/initializer rest semantics sufficiently to justify silently treating constructor rest as ordinary method rest.

Before F.3:

```text
any constructor parameter with rest_mode != None
    -> reject
```

If constructor rest is desired later, amend F.3 explicitly before lifting this guard.

Never erase the marker.

## G5 — Static duplicate labels fail before argument evaluation

For an all-static call:

```phalcom
target(x: sideEffect1(), x: sideEffect2())
```

the compiler must reject the duplicate during compilation.

No user argument expression executes.

Static calls must remain on ordinary `Invoke`; do not allocate a builder merely to obtain duplicate checking.

## G6 — Generic `*` requires the actual cursor protocol

For F.2 positional expansion, a non-Tuple/non-Unit operand is accepted for the generic lane only if its normal method lookup chain contains both:

```text
iterate(_)
iteratorValue(_)
```

The protocol probe:

```text
does not dispatch
does not allocate user objects
does not call doesNotUnderstand
does not authorize access
```

Actual cursor sends still use ordinary `Invoke`, so:

```text
visibility errors
errors thrown inside iterate
errors thrown inside iteratorValue
fiber suspension
user overrides
```

retain normal behavior.

This rule is required to distinguish "operand does not implement the iteration protocol" from "a real iterator implementation ran and threw an error."

## G7 — No new spread-exhaustion opcode

Do not add a second iteration/exhaustion opcode.

Extend the existing `PackTryExpandTuplePositionals` handler narrowly:

```text
Tuple/Unit -> direct-lane success
generic protocol present -> false, compiler enters existing cursor loop
generic protocol missing -> NonIterableStarOperand
```

This preserves the F.2 supplement's compiler-generated cursor requirement.

## G8 — LSP temporarily mirrors current valid U9 selector spelling

Before F.3, valid positional-rest methods still install runtime selector:

```text
name(*)
```

The LSP must temporarily reconstruct that same selector for valid U9 declarations.

F.3 must then migrate core and LSP structural rest selector formatting atomically.

Do not pre-implement F.3 wildcard selector formatting here.

---

# 5. Phase 0 — freeze the baseline before edits

Before changing behavior, record the audited SHA and search the current tree.

Run:

```bash
git rev-parse HEAD

rg -n \
  "PackTryExpandTuplePositionals|compile_positional_pack_expansion|InvokePack|SuperSendPack|FinishTuplePack" \
  phalcom-core

rg -n \
  "rest_mode|RestMode|SignatureKind::Variadic|variadic|name\\(\\*\\)" \
  phalcom-ast phalcom-core phalcom-lsp

rg -n \
  "PackExpansionNotYetSupported|ComputedLabelNotYetSupported|Argument\\b" \
  phalcom-core phalcom-lsp
```

Do not begin F.3 in the same change set.

Recommended change-set boundary:

```text
commit/PR A:
    this pre-F.3 completion gate

commit/PR B:
    F.3-amended
```

---

# 6. Phase 1 — repair Tuple pack parsing

## 6.1 Problem

File:

```text
phalcom-ast/src/parser.rs
```

Function:

```rust
Parser::parse_paren_or_tuple
```

Current structure has two parsing regimes:

```text
A. special branch if the FIRST item is *, **, or ***
B. ordinary tuple/grouping branch otherwise
```

Only branch A recognizes expansion in subsequent entries.

Consequences include rejection/misparse of legal F.1 forms such as:

```phalcom
(1, *xs)
(1, ***pack)
(1, x: 2, **tail)
(1, *a, ***b, *c)
```

## 6.2 Required refactor

Replace the duplicated entry logic with one Tuple-entry parser used after Tuple-ness has been established.

Preserve grouping disambiguation.

Recommended structure:

```rust
fn expansion_mode_at_cursor(&self) -> Option<ExpansionMode>;

fn parse_tuple_entry(
    &mut self,
    labeled_phase: &mut bool,
) -> ParserResult<TupleLiteralEntry>;
```

Exact helper names may follow repository style.

### `parse_tuple_entry` algorithm

At each Tuple entry:

1. try `parse_product_label()`;
2. otherwise inspect `*`, `**`, `***`;
3. otherwise parse an ordinary expression.

For a product label:

```text
parse value
emit TupleLiteralEntry::Labeled
labeled_phase = true
```

For expansion:

```text
*:
    reject if labeled_phase
    emit Expand(Positional)

***:
    reject if labeled_phase
    emit Expand(Complete)
    do NOT set labeled_phase

**:
    emit Expand(Labeled)
    labeled_phase = true
```

For ordinary positional:

```text
reject if labeled_phase
emit Positional
```

## 6.3 Preserve grouping versus Tuple semantics

The outer `parse_paren_or_tuple` must still distinguish:

```phalcom
()       // empty Tuple / Unit form
(x)      // grouping, returns x
(x,)     // singleton Tuple
(x, y)   // Tuple
```

Recommended control flow:

```text
consume '('
skip newlines

if ')':
    return empty Tuple

if first item is a product label:
    Tuple is already unambiguous
    parse it as Tuple entry

else if first item begins *, **, or ***:
    Tuple is already unambiguous
    parse it as Tuple entry
    preserve the repository's required comma grammar for expansion Tuple forms

else:
    parse first ordinary expression
    if no comma:
        consume ')'
        return grouped expression
    else:
        convert first expression to Tuple positional entry

then:
    loop through the same parse_tuple_entry helper for every remaining item
```

Do not preserve the existing separate "first expansion" Tuple parser.

## 6.4 Required source-phase behavior

These must parse:

```phalcom
(*xs,)
(***pack,)
(*a, *b, ***c,)
(1, *xs)
(1, ***pack)
(1, *a, ***b, *c)
(1, label: 2)
(1, label: 2, **tail)
(***first, x, ***second, label: y)
```

These must reject:

```phalcom
(label: 1, 2)
(label: 1, *xs)
(label: 1, ***pack)
(**labels, 1)
(**labels, *xs)
(**labels, ***pack)
```

These must remain legal:

```phalcom
(**a, x: 1, **b)
(***a, ***b, x: 1, **tail)
```

## 6.5 Regression tests

Add parser tests/snapshots for:

```text
empty Tuple
grouped expression
singleton Tuple
trailing comma
newlines between entries
static product label
computed product label
first-item *
first-item **
first-item ***
non-first *
non-first **
non-first ***
multiple ***
mix * and ***
** transition
explicit-label transition
post-transition rejections
```

Do not only add the four originally failing examples.

---

# 7. Phase 2 — repair the LSP PackItem migration

## 7.1 Compile failure

File:

```text
phalcom-lsp/src/index.rs
```

Current helper:

```rust
fn collect_var_occurrences_in_pack_item(
    item: &PackItem,
    names: &HashSet<String>,
    out: &mut Vec<SourceRange>,
)
```

is inconsistent with:

```rust
collect_var_occurrences_in_expr(
    ...,
    out: &mut Vec<(String, SourceRange)>
)
```

## 7.2 Exact edit

Change the helper signature to:

```rust
fn collect_var_occurrences_in_pack_item(
    item: &PackItem,
    names: &std::collections::HashSet<String>,
    out: &mut Vec<(String, SourceRange)>,
)
```

Update every caller.

Do not drop the variable name and do not create an adapter `Vec<SourceRange>`.

## 7.3 Required traversal behavior

Verify occurrences inside:

```text
ordinary positional pack item
static labeled value
computed label expression
computed label value
* expansion operand
** expansion operand
*** expansion operand
Index args
SetIndex args
```

## 7.4 Tests

Add LSP/index tests demonstrating that a top-level binding reference is found inside at least:

```phalcom
target(*xs)
target([labelExpr]: value)
receiver[***indices]
```

The test should assert both:

```text
binding name
SourceRange
```

---

# 8. Phase 3 — implement `NonIterableStarOperand` correctly

## 8.1 Problem

Current runtime errors include all required F.2 pack-specific failures except:

```rust
NonIterableStarOperand
```

Current generic `*` lowering calls ordinary:

```text
iterate(None)
iteratorValue(cursor)
iterate(cursor)
```

If `iterate(_)` does not exist, ordinary send fallback/dNU currently leaks instead of reporting a positional-spread error.

Simply catching "whatever error the first iterate send produces" is incorrect because a valid `iterate` implementation may itself throw an unrelated error.

## 8.2 Add the runtime error

File:

```text
phalcom-core/src/error.rs
```

Add near the other pack errors:

```rust
#[error("* expansion requires Tuple, Unit, or an iterable value; got {found}")]
NonIterableStarOperand {
    found: &'static str,
},
```

Exact wording may be adjusted to repository style, but the variant must remain structured.

Requirements:

```text
report actual operand type
no raw Symbol IDs
no generic ArgumentError(String)
no panic
```

## 8.3 Extend the existing direct-lane probe

File:

```text
phalcom-core/src/vm/dispatch.rs
```

Target handler:

```rust
Bytecode::PackTryExpandTuplePositionals
```

Do not add a new exhaustion opcode.

Change its contract to:

```text
Unit:
    append nothing
    push true

Tuple:
    append positional lane
    push true

other value with real iterate(_) + iteratorValue(_) methods:
    mutate nothing
    push false

other value lacking either protocol method:
    raise NonIterableStarOperand
```

## 8.4 Protocol probe helper

Add a private VM helper near pack dispatch helpers, conceptually:

```rust
fn has_pack_iteration_protocol(&mut self, value: Value) -> bool {
    let iterate_selector = ... encode/intern "iterate(_)" ...;
    let value_selector = ... encode/intern "iteratorValue(_)" ...;

    value.lookup_method(self, iterate_selector).is_some()
        && value.lookup_method(self, value_selector).is_some()
}
```

Use the canonical selector encoder rather than hard-coded selector punctuation if repository convention permits.

The helper must:

```text
perform lookup only
not invoke either method
not call doesNotUnderstand
not allocate a Message
not run user equality/hash/string conversion
not call authorize_method_access
```

Why authorization is deferred:

```text
existence != permission

a private/protected/internal method still means the protocol member exists;
the subsequent ordinary Invoke must surface the normal access violation.
```

## 8.5 dNU ruling

A receiver whose only way to pretend to implement:

```text
iterate(_)
iteratorValue(_)
```

is an overridden `doesNotUnderstand(_)` is **not** considered an F.2 generic spread iterable.

Reason:

```text
the language needs a stable way to classify "non-iterable operand"
without executing arbitrary dNU code merely to discover protocol shape
```

Normal message sends retain dNU semantics.

This ruling is local to protocol conformance for F.2 generic spread.

## 8.6 Preserve the existing compiler loop

Do not modify:

```rust
Compiler::compile_positional_pack_expansion
```

except as needed for tests/comments.

It must continue to emit ordinary cursor protocol sends so:

```text
yield/resume works
user iterator code runs normally
errors thrown by real iterator methods propagate unchanged
```

## 8.7 Tests

Required:

```text
*Unit -> no contribution
*Tuple -> positional-lane projection
*List -> works
*Bytes/String byte view -> works where Iterable
*Range -> works
*Iterator -> works
*user Iterable -> works
*lazy bounded pipeline -> works

*Int/non-iterable -> NonIterableStarOperand

valid iterate(_) implementation throws Error E:
    E propagates unchanged
    NOT NonIterableStarOperand

valid iteratorValue(_) implementation throws Error E:
    E propagates unchanged
    NOT NonIterableStarOperand

protocol member exists but is inaccessible:
    normal access error propagates
    NOT NonIterableStarOperand
```

---

# 9. Phase 4 — static duplicate argument-label validation

## 9.1 Problem

File:

```text
phalcom-core/src/compiler/lib/expr.rs
```

Current:

```rust
Compiler::needs_dynamic_pack
```

chooses the builder only for expansion or computed labels.

Current:

```rust
Compiler::pack_labels
```

collects all-static labels without uniqueness validation.

Therefore:

```phalcom
target(x: 1, x: 2)
```

can remain on static `Invoke` and form an invalid duplicate-label selector rather than failing.

Dynamic packs are already protected by builder reservation checks.

## 9.2 Add a structured compiler diagnostic

File:

```text
phalcom-core/src/compiler/lib/error.rs
```

Add a dedicated variant, conceptually:

```rust
#[error("duplicate argument label `{label}`")]
DuplicateArgumentLabel {
    label: String,
    span: SourceRange,
    first_span: SourceRange,
},
```

Exact field layout may follow existing diagnostics infrastructure.

Do not use:

```rust
CompilerError::Message(format!(...))
```

for the general static duplicate-label invariant.

## 9.3 Replace `pack_labels` collection logic

File:

```text
phalcom-core/src/compiler/lib/expr.rs
```

Target:

```rust
Compiler::pack_labels
```

Replace the `.iter().map(...).collect()` implementation with an explicit loop.

Conceptually:

```rust
let mut labels = Vec::with_capacity(items.len());
let mut seen = HashMap::<String, SourceRange>::new();

for item in items {
    match item {
        Positional => labels.push(None),

        Labeled { Static { text, range }, .. } => {
            if let Some(first) = seen.get(text) {
                return Err(CompilerError::DuplicateArgumentLabel {
                    label: text.clone(),
                    span: *range,
                    first_span: *first,
                });
            }
            seen.insert(text.clone(), *range);
            labels.push(Some(text.clone()));
        }

        Computed label => defensive dynamic-path invariant error,
        Expand => defensive dynamic-path invariant error,
    }
}
```

Use the static label's label span, not the whole call span, for the second/conflicting occurrence.

## 9.4 Coverage of all static send consumers

Every all-static send path that calls `pack_labels` must inherit the same validation.

Audit at minimum:

```text
ordinary receiver.method(...)
implicit-self/unqualified message send
callable .call(...)
static super send
subscript get
subscript set source arguments
pinned selector construction where call labels are reconstructed
```

Do not add per-call-site duplicate loops if they can share `pack_labels`.

## 9.5 Compiler-owned `#put`

Static SetIndex currently has a special collision check for user label:

```text
put
```

Replace its generic string-only diagnostic with the same structured duplicate-label diagnostic.

The collision must be detected before RHS bytecode is emitted/evaluated.

Dynamic SetIndex must keep the existing:

```text
PackReserveStaticLabel(#put)
before RHS evaluation
```

behavior.

## 9.6 Static fast path requirement

After this change:

```text
all-static unique send
    -> no ArgumentPackBuilderObject
    -> ordinary Invoke/SuperSend
```

Do not route static calls through `InvokePack` merely for validation.

## 9.7 Tests

Compile errors:

```phalcom
target(x: 1, x: 2)
self.target(x: 1, x: 2)
callable(x: 1, x: 2)
super.target(x: 1, x: 2)
receiver[x: 1, x: 2]
receiver[put: 1] = rhs
```

Ordering assertion:

```text
duplicate is discovered at compile time;
argument/RHS side effects cannot occur
```

Disassembly assertion for a valid static call:

```text
contains Invoke
does not contain NewArgumentPack
does not contain InvokePack
```

---

# 10. Phase 5 — declaration-label uniqueness

## 10.1 Problem

File:

```text
phalcom-ast/src/parser.rs
```

Function:

```rust
Parser::parse_selector_params
```

tracks:

```text
parameter lane/order
rest placement
```

but does not reject repeated external labels.

A declaration must never produce a selector with duplicate labeled slots.

## 10.2 Parser validation

Track external labels only:

```rust
HashMap<String, SourceRange>
```

When a parameter is normalized to:

```rust
label: Some(external_label)
```

reject a duplicate external label at the second parameter.

Do **not** compare local parameter names for this rule.

Example:

```phalcom
method(x first, x second) { ... }
```

must fail because both external labels are `x`.

Different external labels with the same local name remain subject to the ordinary local-binding rules, not this selector-label check.

## 10.3 Defensive compiler validation

Because compiler/attribute passes can synthesize AST members after parsing, add a defensive validation helper in:

```text
phalcom-core/src/compiler/lib/class_decl.rs
```

before selector construction/member installation.

It should reject duplicate external labels in:

```text
MethodDef.params
IndexMethodDef.params
```

even if a synthetic AST bypassed parser validation.

The normal source path should ordinarily fail earlier in the parser.

## 10.4 Tests

Parser/compiler tests:

```phalcom
class C {
  f(x first, x second) {}
}
```

must fail.

Also verify ordinary unique labels still encode in order.

---

# 11. Phase 6 — stop rest-mode semantic erasure

## 11.1 Current transitional parser behavior

Current `ParameterDef` already stores:

```rust
RestMode::None
RestMode::Positional
RestMode::Labeled
RestMode::Complete
```

Ordinary method compilation currently:

```text
supports only the U9 final positional rest form
rejects labeled/complete rest until F.3
```

Preserve that.

The bugs are declaration kinds that reuse `ParameterDef` but ignore `rest_mode`.

## 11.2 Add one pre-F.3 member-rest validator

File:

```text
phalcom-core/src/compiler/lib/class_decl.rs
```

After attribute expansion and before duplicate-selector canonicalization, validate every member.

Recommended helper:

```rust
fn validate_pre_f3_rest_usage(member: &ClassMember)
    -> Result<(), CompilerError>
```

or a class-level equivalent.

Rules:

### Ordinary non-constructor Method

Preserve current transitional U9:

```text
RestMode::None:
    exact method

one final RestMode::Positional:
    U9 SignatureKind::Variadic

RestMode::Labeled:
    RestModeNotYetSupported

RestMode::Complete:
    RestModeNotYetSupported

positional rest in any non-final position:
    reject
```

### Constructor

Any:

```text
rest_mode != RestMode::None
```

is rejected.

Use a structured/pre-existing "not supported until F.3" compiler diagnostic rather than silently building a constructor selector.

### Index method

Any:

```text
rest_mode != RestMode::None
```

is rejected.

### Getter/Setter/Field/Variant

No rest-bearing parameter list exists.

## 11.3 Parser-side subscript rejection

File:

```text
phalcom-ast/src/parser.rs
```

Function:

```rust
Parser::parse_index_member
```

After parsing the bracket parameters and before constructing `IndexMethodDef`:

```rust
if let Some(rest) = params.iter().find(|p| p.is_rest()) {
    return Err(...);
}
```

Use a source diagnostic such as:

```text
rest parameters are not supported in subscript declarations
```

Point at the rest parameter span.

Keep the compiler validator anyway for synthetic/malformed AST defense.

## 11.4 Constructor parser

Do not contort the parser solely to detect `@constructor` rest.

Attributes are attached around member parsing and compiler expansion can synthesize constructors.

The authoritative rejection belongs in the post-expansion compiler validation pass.

## 11.5 Duplicate-member scan ordering

The rest validator must run **before** the class duplicate-member scan computes selector identity.

Otherwise a constructor/index rest marker could already be erased into an ordinary selector during duplicate-key construction.

Required order:

```text
attribute expansion
pre-F.3 rest-usage validation
declaration-label validation
duplicate member/selector scan
member compilation
```

## 11.6 Tests

Reject:

```phalcom
class C {
  @constructor
  new(*args) {}
}

class C {
  [*indices] {}
}

class C {
  [**labels] {}
}

class C {
  [***pack] {}
}
```

Also create a compiler-unit test with a synthetic `IndexMethodDef` containing rest metadata to prove the compiler defense does not rely solely on the parser.

---

# 12. Phase 7 — LSP selector compatibility before F.3

## 12.1 Problem

File:

```text
phalcom-lsp/src/selectors.rs
```

Current:

```rust
comma_form(name, params)
```

renders every unlabeled parameter as:

```text
_
```

and ignores:

```rust
ParameterDef.rest_mode
```

But valid current U9:

```phalcom
sum(*numbers)
format(_ fmt, *args)
```

installs core runtime selector:

```text
sum(*)
format(*)
```

The U9 runtime selector intentionally loses the fixed-prefix count.

Therefore the LSP and runtime currently disagree on valid code.

## 12.2 Temporary U9 fix

Modify:

```rust
comma_form
```

so a valid final positional-rest parameter uses the current U9 runtime spelling.

Conceptually:

```rust
if params
    .last()
    .is_some_and(|p| p.rest_mode == RestMode::Positional)
{
    return format!("{name}(*)");
}
```

Only apply this shortcut to the valid current U9 shape.

Do not attempt F.3's future structural formatting here.

## 12.3 Invalid future rest syntax

`**rest` and `***rest` are parser-visible F.1 syntax but compile-rejected until F.3.

The LSP may continue best-effort indexing them, but this unit must not invent their final F.3 runtime selector identity.

## 12.4 Stale comment cleanup

Replace the comment that still refers to:

```rust
phalcom_ast::ast::Argument
```

with:

```text
PackItem / call-site static labels
```

or the repository-current terminology.

## 12.5 F.3 handoff

F.3 must update:

```text
phalcom-core selector formatter
phalcom-lsp selector formatter
LSP selector tests
reflection/disassembly expectations
```

in the same phase when structural rest selectors become real.

## 12.6 Tests

Add:

```text
sum(*numbers) -> "sum(*)"
format(_ fmt, *args) -> "format(*)"   // current U9 behavior
ordinary exact method unchanged
```

Do not add F.3 expected strings such as:

```text
format(_,*)
```

until F.3.

---

# 13. Phase 8 — product count narrowing

## 13.1 Problem

File:

```text
phalcom-core/src/compiler/lib/expr.rs
```

Static Tuple lowering currently computes `usize` counts and emits:

```rust
Bytecode::BuildTuple {
    positional: positional as u16,
    labeled: labeled as u16,
}
```

Static Record lowering similarly emits:

```rust
Bytecode::BuildRecord {
    fields: fields as u16,
}
```

Unchecked `as u16` silently truncates oversized source products.

This is inconsistent with nearby List lowering, which explicitly checks `u16::MAX`.

## 13.2 Add a checked product-count helper

File:

```text
phalcom-core/src/compiler/lib/error.rs
```

Add an appropriate structured diagnostic, e.g.:

```rust
#[error("{subject} has {found} entries; bytecode supports at most {limit}")]
ProductCountLimit {
    subject: &'static str,
    found: usize,
    limit: u16,
    span: SourceRange,
},
```

Then add a helper near `checked_send_arity`, conceptually:

```rust
pub(crate) fn checked_product_count(
    subject: &'static str,
    found: usize,
    span: SourceRange,
) -> Result<u16, CompilerError>
```

Do not reuse the send `u8` arity error: product count and send arity are distinct limits.

## 13.3 Tuple lowering

Check both lanes before compiling entries/emitting `BuildTuple`:

```rust
let positional = checked_product_count(
    "Tuple positional lane",
    positional_count,
    tuple_expr.range,
)?;

let labeled = checked_product_count(
    "Tuple labeled lane",
    labeled_count,
    tuple_expr.range,
)?;
```

Then emit the checked values.

## 13.4 Record lowering

Check:

```text
record field count <= u16::MAX
```

before emitting `BuildRecord`.

## 13.5 Dynamic Tuple behavior

Do **not** add this static bytecode count limit to dynamic Tuple pack construction.

`FinishTuplePack` is product assembly, not a send, and should retain the F.2/A.3 semantics already specified.

## 13.6 Tests

At minimum unit-test the checked conversion helper at:

```text
u16::MAX
u16::MAX + 1
```

Avoid generating gigantic source fixtures if the helper can be tested directly.

---

# 14. Phase 9 — documentation and migration-artifact cleanup

This phase is mandatory because the repository currently contains mutually contradictory active specifications.

## 14.1 `docs/spec/collections/06-rest-spread-and-pack-operators.md`

The current file contains obsolete rules including:

```text
value expansion legal only in call argument lists
Tuple expansion invalid
* requires a positional-lane product rather than generic Iterable
***Record accepted
at most one *** provisional
*** cannot mix with * or **
```

Amend it to match F.1/F.2:

```text
Tuple construction supports pack expansion
*Tuple / *Unit direct lane
generic * uses Iterable cursor protocol
** accepts the F.2 labeled-source set
*** accepts Tuple/Unit only
multiple *** legal
expansion-mode mixing legal subject to source phases
** starts labeled source phase
*** does not start labeled source phase
```

Do not change rest capture from List to Tuple/Unit yet in descriptions of the **currently implemented** runtime.

Instead clearly mark U9 capture as transitional and link/point to F.3-amended as the planned replacement.

## 14.2 `docs/spec/collections/05-argument-packs.md`

Remove/mark stale rest-capture statements that imply the final F.3 capture representation may be List/Map/Record or preserve arbitrary structural input kind.

Clarify:

```text
outgoing pack lane model is active now
incoming rest capture remains U9 positional-only implementation until F.3
F.3-amended will canonicalize captures to Unit/Tuple
```

Do not claim F.3 runtime behavior is already implemented.

## 14.3 `docs/spec/current/selectors.md`

Update selector-label documentation to describe F.1's total escaping for arbitrary Symbol labels.

Ensure literal:

```text
*
**
***
_
~
delimiters
Unicode labels
```

cannot be confused with structural slot markers.

Keep U9 `name(*)` documented as transitional current behavior until F.3.

## 14.4 Pending work documents

For old superseded documents such as:

```text
docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send.md
docs/work/pending/collections/F.3-rest-capture-and-dispatch.md
```

do not silently leave them looking equally authoritative.

Add a prominent status banner pointing to the amended successor, or remove them from active indexes while preserving history according to repository documentation policy.

## 14.5 AST/parser comments

File:

```text
phalcom-ast/src/ast.rs
```

Update stale `ParameterDef` documentation that still describes:

```text
labeled declaration syntax as `name:`
only `*name`
rest capture always into List
```

Document the actual transitional state:

```text
declaration labels use current no-colon grammar
rest_mode carries None/Positional/Labeled/Complete
F.1 parses all modes
pre-F.3 compiler only executes U9 final positional rest
F.3 will replace U9 binding semantics
```

File:

```text
phalcom-ast/src/parser.rs
```

Update `parse_selector_params` comments that describe the old boolean/U9 grammar as if it were the final language.

Do not relax ordinary method rest ordering until F.3.

## 14.6 Compiler stale diagnostics

File:

```text
phalcom-core/src/compiler/lib/error.rs
```

Current messages:

```text
"pack expansion is not supported until F.2."
"computed pack labels are not supported until F.2."
```

are no longer truthful user-facing descriptions.

Preferred fix:

1. make dynamic/static routing exhaustive enough that these variants are unreachable for legal F.2 source;
2. replace them with internal/misrouting diagnostics if defensive guards remain.

For example:

```text
internal compiler error: dynamic pack item reached static pack lowering
```

Do not remove:

```text
RestModeNotYetSupported
```

before F.3; that diagnostic still reflects the transitional implementation.

## 14.7 LSP stale comments

Remove remaining references to the old call `Argument` AST and future-tense "until F.2" wording in live pack-aware code.

---

# 15. Phase 10 — complete the F.2 regression suite

F.2 test completion is a hard prerequisite, not optional cleanup.

Use the repository's current test organization. Prefer focused test files/modules rather than one monolithic fixture.

## 15.1 Direct positional lane

Required:

```text
*Unit contributes zero positionals
*Tuple contributes only Tuple.positionals()
Tuple labels are not leaked by *
single positional Tuple
mixed Tuple with labels
```

## 15.2 Generic `*`

Required sources:

```text
List
byte/string sequence type that conforms to Iterable
finite Range
Iterator
SourceIterator
user-defined Iterable
lazy map pipeline
lazy filter pipeline
take(n) bounded pipeline
unknown runtime iterable
```

## 15.3 E.3

Required:

```text
provably unbounded source -> compile error
provably bounded source -> compile
unknown source -> compile
Tuple/Unit direct lane not spuriously rejected
```

## 15.4 Evaluation ordering

Instrument side effects and prove:

```text
receiver evaluated once
pack items evaluated left-to-right
spread operand evaluated once
static/computed label is reserved before corresponding value where required
duplicate label stops later value evaluation
dynamic SetIndex #put collision stops RHS evaluation
```

## 15.5 Non-Iterable and iterator errors

Required:

```text
non-iterable -> NonIterableStarOperand
real iterate error propagates unchanged
real iteratorValue error propagates unchanged
```

## 15.6 Fiber/yield

Create a user iterable whose:

```text
iterate
or iteratorValue
```

yields/suspends a Fiber during generic `*`.

Assert after resume:

```text
source local survives
cursor local survives
builder survives
no contribution duplicated
no contribution skipped
lexical order preserved
```

## 15.7 Dynamic ordinary sends

Required:

```text
receiver.method(*xs)
receiver.method(**labels)
receiver.method(***pack)
mixed dynamic contributions
computed labels
```

## 15.8 Unqualified/callable sends

Required:

```text
implicit self dynamic send
global/local callable dynamic call
bound/open callable where supported
```

Confirm base family for callable dispatch remains:

```text
call
```

## 15.9 Dynamic super

Required:

```text
super.method(*xs)
super.method(**labels)
super.method(***pack)
```

Assert lookup start remains above the defining class.

F.3 will later change exact-miss fallback; this test must pin the F.2 starting semantics.

## 15.10 `**`

Required valid operands:

```text
Unit
Tuple labeled lane
Record
Map with Symbol keys
```

Required failures:

```text
invalid source type -> InvalidStarStarOperand
Map non-Symbol key -> NonSymbolMapKeyInExpansion
duplicate introduced against prior static label
duplicate introduced between expansions
```

Preserve labeled encounter order.

## 15.11 `***`

Required:

```text
Unit
Tuple
multiple *** before labeled phase
mix * + *** before labeled phase
mix *** + explicit label + **
```

Required failure:

```text
Record/Map/other -> InvalidStarStarStarOperand
```

## 15.12 Computed labels

Required:

```text
computed Symbol succeeds
computed non-Symbol -> ComputedLabelNotSymbol
duplicate computed/static collision -> DuplicateArgumentLabel
```

Assert label expression runs before value expression and duplicate reservation prevents value side effects.

## 15.13 Dynamic subscript read

Required:

```text
receiver[*indices]
receiver[***pack]
receiver[label: v, **tail]
computed index labels where grammar permits
```

## 15.14 Dynamic subscript write

Required:

```text
receiver[*indices] = rhs
receiver[***pack] = rhs
receiver[label: v, **tail] = rhs
```

Assert:

```text
implicit #put omitted from bracket selector identity
RHS counted in send arity
RHS is final labeled value
setter return discarded
assignment expression == original RHS
duplicate #put fails before RHS
```

## 15.15 Dynamic Tuple

Required:

```text
(*xs,)
(1, *xs)
(***pack,)
(1, ***pack)
(1, label: 2, **tail)
computed Tuple label
empty expansion -> Unit when final product empty
mixed lanes preserve product order
```

## 15.16 Dynamic send arity

Required:

```text
<= 255 flattened args succeeds
> 255 -> SendArityExceedsLimit
```

Do not apply send arity to Tuple product construction.

## 15.17 dNU forwarding

For dynamic Method send total miss:

```text
dNU receives original concrete selector
dNU receives original flattened values
no wildcard/U9 selector substituted merely because fallback was probed
```

This test becomes critical when F.3 replaces U9 fallback.

## 15.18 Compiler-internal authority

Exercise a dynamic compiler-internal send and assert:

```text
authority applies during selected send
authority does not leak after dispatch
```

## 15.19 GC stress

Under the repository GC stress facility if available:

```text
builder remains live during contributions
spread source remains live
cursor remains live
values appended before allocation remain live
FinishTuplePack survives allocation
dynamic SetIndex RHS survives allocation/dispatch
```

## 15.20 Static regression/disassembly

Representative static calls:

```phalcom
obj.foo()
obj.foo(a)
obj.foo(a, b)
obj.foo(x: a)
```

Assert:

```text
ordinary Invoke
no NewArgumentPack
no InvokePack
no dynamic selector construction on exact static path
```

Representative static Tuple:

```phalcom
(1, 2, x: 3)
```

asserts:

```text
BuildTuple
no pack builder
```

---

# 16. Phase 11 — F.3 handoff corrections

Before declaring this gate complete, annotate the F.3 implementation worklist with these audited facts.

## 16.1 AST migration correction

F.3's conceptual text may discuss replacing:

```text
is_rest: bool
```

with lane-aware rest metadata.

That migration is already effectively done in HEAD.

Actual starting point:

```rust
ParameterDef.rest_mode: phalcom_ast::ast::RestMode
```

F.3 implementation should:

```text
reuse this parser-side representation
relax/replace U9 parser ordering rules
construct RestLayout
store Signature.rest
normalize RestMode::None away at the runtime boundary
```

Do not create a redundant second AST enum without a demonstrated architectural need.

## 16.2 LSP migration seam

F.3 structural selector formatting must update:

```text
phalcom-core/src/method/mod.rs
phalcom-lsp/src/selectors.rs
related selector tests
reflection/disassembly expectations
```

atomically.

The temporary U9 LSP fix in §12 must be removed/replaced in that phase.

## 16.3 Constructor boundary

F.3 must explicitly decide constructor rest before constructor rejection is lifted.

Required question to resolve in F.3 if support is desired:

```text
does rest apply to the class-side factory selector,
the generated instance initializer,
or both as one coupled signature?
```

Do not infer this from ordinary method rest dispatch.

## 16.4 Subscript boundary

Keep the F.3-amended ruling:

```text
rest fallback eligible domain == ordinary Method family
```

Subscript declarations remain non-rest unless a future specification explicitly extends that domain.

## 16.5 U9 deletion remains F.3 work

Only F.3 removes:

```text
SignatureKind::Variadic
Signature.variadic
name(*) runtime fallback/cache
List rest call-prologue packing
U9-only selector decoding behavior
```

---

# 17. Verification commands

After implementing all phases, run from repository root.

## 17.1 Formatting

```bash
cargo fmt --check
```

## 17.2 Compile checks

```bash
cargo check -p phalcom-ast
cargo check -p phalcom-core
cargo check -p phalcom-lsp
```

Prefer also:

```bash
cargo check --workspace
```

if the repository workspace is expected to compile as a whole.

## 17.3 Tests

```bash
cargo test -p phalcom-ast
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

Then:

```bash
cargo test --workspace
```

where supported by the repository's normal CI workflow.

Run focused filters for the new suites, for example:

```bash
cargo test -p phalcom-core -- pack
cargo test -p phalcom-core -- tuple
cargo test -p phalcom-core -- iterable
cargo test -p phalcom-core -- subscript
cargo test -p phalcom-lsp -- selector
```

Use the actual test names created by the implementation.

## 17.4 Lints

Run the repository-standard clippy command. If no narrower CI command exists:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Do not introduce lint suppressions merely to land this gate unless the suppression is independently justified.

## 17.5 Migration searches

Before completion:

```bash
rg -n "phalcom_ast::ast::Argument|\\bArgument \\{" phalcom-core phalcom-lsp

rg -n \
  "pack expansion is not supported until F\\.2|computed pack labels are not supported until F\\.2" \
  phalcom-core phalcom-lsp

rg -n \
  "SignatureKind::Variadic|signature\\.variadic|\\.variadic\\b|name\\(\\*\\)" \
  phalcom-core phalcom-lsp
```

For the U9 search:

```text
remaining U9 references are expected before F.3;
classify them, do not delete them in this unit.
```

Documentation audit:

```bash
rg -n \
  "only in call argument lists|at most one \\*\\*\\*|\\*\\*\\*Record|List.*rest|rest.*List|variadic fallback" \
  docs
```

Every hit must be one of:

```text
correct current transitional description
clearly marked historical/superseded text
F.3-amended future behavior
```

No ambiguous normative contradiction may remain.

---

# 18. Hard completion gate

F.3-amended may start only when all boxes are true.

## 18.1 Parser/AST

- [ ] `parse_paren_or_tuple` uses one expansion-aware Tuple-entry path.
- [ ] `(expr)` grouping behavior is unchanged.
- [ ] `()` / `(expr,)` Tuple behavior is unchanged.
- [ ] `*`, `**`, and `***` work in every legal Tuple source position.
- [ ] multiple `***` Tuple contributions are accepted before labeled phase.
- [ ] `***` does not itself start labeled source phase.
- [ ] `**` and explicit/computed labels do start labeled phase.
- [ ] positional contributions after labeled phase are rejected.
- [ ] declaration external labels are unique.
- [ ] subscript declarations reject rest parameters.
- [ ] existing `ParameterDef.rest_mode` remains the sole parser-side rest-mode field.

## 18.2 Compiler

- [ ] all-static duplicate call labels are rejected before value bytecode is emitted.
- [ ] static SetIndex `#put` collision uses structured duplicate-label validation.
- [ ] ordinary unique static sends remain on `Invoke`/`SuperSend`.
- [ ] constructor rest is explicitly rejected pre-F.3.
- [ ] compiler defensively rejects subscript rest even for synthetic AST.
- [ ] rest validation occurs before duplicate-selector identity computation.
- [ ] static Tuple lane counts use checked `u16` conversion.
- [ ] static Record field count uses checked `u16` conversion.
- [ ] stale F.2 "not yet supported" compiler diagnostics cannot surface for legal F.2 source.

## 18.3 Runtime

- [ ] `NonIterableStarOperand` exists as a structured runtime variant.
- [ ] `PackTryExpandTuplePositionals` keeps Unit/Tuple direct behavior.
- [ ] generic-spread protocol presence is checked without user dispatch.
- [ ] missing iteration protocol raises `NonIterableStarOperand`.
- [ ] real iterator implementation errors propagate unchanged.
- [ ] generic `*` still executes through compiler-generated cursor bytecode.
- [ ] no new Rust exhaustion loop/native spread helper exists.
- [ ] existing F.2 structured pack errors remain intact.
- [ ] dynamic Tuple remains free of send `u8` arity limits.

## 18.4 LSP

- [ ] `phalcom-lsp` compiles.
- [ ] PackItem variable-occurrence traversal carries `(String, SourceRange)`.
- [ ] PackItem occurrence tests cover expansion/computed-label positions.
- [ ] valid U9 positional-rest declarations index as `name(*)`.
- [ ] stale live references to the removed call `Argument` AST are gone.
- [ ] F.3 structural rest selector migration seam is documented.

## 18.5 Tests

- [ ] direct `*Unit`/`*Tuple` tests pass.
- [ ] generic Iterable spread matrix passes.
- [ ] E.3 bounded/unbounded/unknown tests pass.
- [ ] evaluation-order tests pass.
- [ ] non-Iterable structured-error tests pass.
- [ ] iterator user-error propagation tests pass.
- [ ] Fiber yield/resume spread tests pass.
- [ ] ordinary/unqualified/callable dynamic-send tests pass.
- [ ] dynamic super tests pass.
- [ ] `**` valid/error matrix passes.
- [ ] `***` valid/error matrix passes.
- [ ] computed-label tests pass.
- [ ] dynamic subscript read tests pass.
- [ ] dynamic subscript write tests pass.
- [ ] dynamic Tuple tests pass.
- [ ] dynamic arity tests pass.
- [ ] concrete-selector dNU tests pass.
- [ ] compiler-internal authority regression passes.
- [ ] GC stress tests pass where stress mode exists.
- [ ] static disassembly/fast-path regressions pass.

## 18.6 Documentation

- [ ] F.1/F.2 outgoing-pack rules have one authoritative active description.
- [ ] old "call-only expansion" wording is removed/superseded.
- [ ] old "at most one `***`" wording is removed/superseded.
- [ ] old "`***Record` is valid" wording is removed/superseded.
- [ ] selector docs describe total Symbol-label escaping.
- [ ] AST/parser comments describe current no-colon declaration grammar.
- [ ] U9 positional List capture is described only as current transitional behavior.
- [ ] docs do not claim F.3 Tuple/Unit capture is already implemented.
- [ ] superseded F.2/F.3 work documents are visibly marked.

## 18.7 Repository verification

- [ ] `cargo fmt --check`
- [ ] `cargo check -p phalcom-ast`
- [ ] `cargo check -p phalcom-core`
- [ ] `cargo check -p phalcom-lsp`
- [ ] package test suites
- [ ] workspace checks/tests where repository policy expects them
- [ ] repository-standard clippy/lints
- [ ] migration `rg` searches manually classified

---

# 19. Recommended implementation order

Use this order to minimize mixed-state failures.

```text
1. LSP type mismatch
   - restore workspace compile visibility first

2. Tuple parser unification
   - make all F.1 legal Tuple syntax reachable

3. static call duplicate-label validation
   - close the static/builder semantic gap

4. declaration external-label validation
   - close selector-definition duplication

5. NonIterableStarOperand + PackTryExpandTuplePositionals protocol probe
   - finish F.2 structured runtime boundary

6. pre-F.3 rest-usage validation
   - reject constructor/subscript semantic erasure

7. temporary LSP U9 selector compatibility
   - align editor/runtime identity before F.3

8. static Tuple/Record checked count conversions
   - remove silent narrowing

9. full F.2 regression/fiber/GC suite
   - prove the outgoing-pack baseline

10. documentation and stale-comment cleanup
    - establish one active language description

11. full fmt/check/test/lint/search gate

12. only then begin F.3-amended
```

Do not combine step 12 into this implementation unit.

---

# 20. Expected code touch map

Primary files:

```text
phalcom-ast/src/parser.rs
    parse_paren_or_tuple
    parse_selector_params
    parse_index_member
    parser comments/tests

phalcom-ast/src/ast.rs
    stale ParameterDef/rest documentation only
    no new parser-side rest enum

phalcom-core/src/compiler/lib/expr.rs
    pack_labels
    static SetIndex duplicate #put diagnostic
    checked static Tuple counts
    checked Record count

phalcom-core/src/compiler/lib/error.rs
    DuplicateArgumentLabel compiler error
    ProductCountLimit or equivalent
    stale F.2 defensive diagnostics

phalcom-core/src/compiler/lib/class_decl.rs
    post-expansion pre-F.3 rest-usage validation
    defensive declaration-label validation
    ensure validation runs before duplicate-selector scan

phalcom-core/src/error.rs
    NonIterableStarOperand

phalcom-core/src/vm/dispatch.rs
    PackTryExpandTuplePositionals protocol preflight
    small private protocol helper if appropriate

phalcom-lsp/src/index.rs
    PackItem occurrence output type

phalcom-lsp/src/selectors.rs
    current U9 positional-rest selector compatibility
    stale Argument comment
    tests
```

Likely test areas:

```text
phalcom-ast parser tests/snapshots
phalcom-core compiler/unit tests
phalcom-core integration fixtures
phalcom-core VM/dispatch tests
phalcom-lsp selector/index tests
```

Documentation:

```text
docs/spec/collections/05-argument-packs.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
docs/spec/current/selectors.md
docs/work/pending/collections/F.1-pack-syntax-ast-and-selector-encoding.md
    only if wording needs status clarification; do not change ratified semantics
docs/work/pending/collections/F.2-*.md
    amended/superseded status hygiene as needed
docs/work/pending/collections/F.3-*.md
    superseded banner / amended-plan handoff note
```

Use actual HEAD paths if the repository moves a component before implementation.

---

# 21. Explicit non-goals

This completion unit must **not**:

```text
implement RestLayout
add Signature.rest
add RestMode::Split to the parser AST
add ClassObject.rest_methods
implement rest-family fallback
repack rest call windows
change rest capture from List to Tuple/Unit
delete SignatureKind::Variadic
delete Signature.variadic
delete U9 List call-prologue packing
delete U9 name(*) fallback
implement native rest ABI
implement block/closure rest
make subscripts rest-capable
add F.3 reflection APIs
add dynamic/rest inline caches
widen send arity
replace ArgumentPackBuilderObject
replace the 11 landed F.2 pack bytecodes
route static sends through InvokePack
exhaust Iterables in Rust
```

Those are either F.3 work, future work, or regressions.

---

# 22. Final correctness invariants

At the moment F.3-amended begins, the repository must satisfy:

```text
F.1 syntax:
    every legal outgoing pack form parses consistently in calls and Tuples

F.2 compilation:
    every legal outgoing dynamic pack form reaches the builder/cursor machinery

F.2 runtime:
    pack-specific user failures have structured errors

static calls:
    preserve exact Invoke fast path
    statically provable duplicate labels cannot escape validation

dynamic calls:
    preserve lexical evaluation timing and concrete selector identity

Tuple construction:
    static path = BuildTuple
    dynamic path = builder + FinishTuplePack
    no silent u16 narrowing on static product counts

rest transition:
    ordinary U9 positional rest remains explicit and isolated
    labeled/complete rest remain compiler-deferred
    constructors/subscripts never silently erase rest metadata

LSP:
    compiles
    traverses PackItem correctly
    mirrors current valid U9 selector identity

documentation:
    describes one coherent current implementation
    distinguishes current U9 state from future F.3 state

tests:
    prove F.2 behavior before F.3 changes dispatch and frame binding
```

Only after these invariants are demonstrated by the verification gate should F.3-amended begin.

---

# 23. Decision record

| Topic | Final pre-F.3 ruling |
|---|---|
| U9 positional rest | Keep until F.3 |
| Current AST rest field | `ParameterDef.rest_mode` is authoritative |
| New parser-side `RestParameterKind` before F.3 | Do not add |
| Tuple parser | One expansion-aware entry path |
| `***` source phase | Does not start labeled phase |
| Multiple `***` | Legal before labeled phase |
| Static duplicate call labels | Compile error before argument evaluation |
| Dynamic duplicate call labels | Existing builder reservation behavior |
| Non-Iterable `*` | Dedicated `NonIterableStarOperand` |
| How to distinguish non-Iterable | Method-table protocol preflight in existing Tuple/Unit probe |
| dNU-only synthetic iteration | Does not satisfy generic `*` protocol |
| Generic spread exhaustion | Existing compiler-generated cursor bytecode |
| New exhaustion opcode/helper | No |
| Constructor rest pre-F.3 | Reject |
| Subscript rest | Reject |
| Valid U9 LSP selector | Mirror runtime `name(*)` temporarily |
| F.3 selector formatting | Migrate core + LSP atomically in F.3 |
| Static Tuple/Record overflow | Checked `u16` conversion |
| Dynamic Tuple send limit | None |
| F.2 tests | Hard prerequisite |
| Stale normative docs | Amend or mark superseded before F.3 |
| F.3 start | Blocked until §18 passes |

---

# 24. Completion statement

This pre-F.3 unit is complete only when an implementer can state, with test and command output:

```text
1. the workspace compiles, including phalcom-lsp;
2. F.1 Tuple/call pack syntax is parser-consistent;
3. static and dynamic pack validation obey the same language invariants;
4. generic positional spread has the required structured non-Iterable boundary;
5. no declaration kind silently drops rest metadata;
6. all already-landed F.2 machinery is exercised by regression tests;
7. active documentation no longer describes competing pack languages;
8. U9 remains isolated as the explicit transitional input that F.3-amended will replace.
```

Until then:

```text
F.3-amended remains blocked.
```
