# Phalcom F.2/F.3 Completion — Implementation Specification

> Baseline: `aureat/phalcom-lang` `main` at commit `a88bdfee1571022a033e91ad05662a50b9094087`
>
> Prepared after re-auditing the 12 commits added after the previous F.2/F.3 review.
>
> This specification is intentionally pinned to that commit. Rebase/re-audit the named hunks if `main` moves before applying the companion patch.

## 1. Goal

Close the remaining F.2/F.3 implementation and evidence gaps without reopening the language semantics that are already implemented correctly.

The implementation must:

1. preserve the existing F.2 dynamic-pack semantics and static fast path;
2. preserve F.3 exact-before-rest lookup and lane-aware capture;
3. repair the native-rest bootstrap inconsistency introduced by the callable-runtime commits;
4. centralize runtime method-table/rest-index mutation so duplicate-family rejection cannot leave the two indexes out of sync;
5. replace generic rest-declaration compiler/parser messages with stable structured diagnostics;
6. add direct regressions for subclass non-acceptance fallback and `super` rest lookup boundaries;
7. graduate the dynamic-send E.3 unbounded-spread fixture out of the ignored positive-only pending lane;
8. implement the repository's already-specified `PHALCOM_GC_STRESS` safepoint stress facility;
9. update the stale F.2/F.3 closure records to reflect the callable-runtime amendments; and
10. leave performance as an explicit measured acceptance gate rather than inventing benchmark results.

No new public Phalcom primitive binding is required.

---

## 2. Fresh audit findings after the 12 new commits

### 2.1 The previous F.2 semantic-gap list is stale

`docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send-amended.md`, sections 36–37, still contains an all-unchecked invariant list even though its completion checklist records the corresponding implementation/tests as complete.

The only F.2 closure items that remain genuinely external to the semantic implementation are:

- every-safepoint GC stress;
- an actual static-send benchmark measurement.

Do not change F.2 pack semantics, selector assembly, arity semantics, subscript `put` ordering, hidden-local rooting, or generic-`*` lowering as part of this patch.

### 2.2 Callable work ratified native rest after the original F.3 plan

The current F.3 amended plan still says:

- “Native rest methods are not silently accepted.”
- “Native rest methods are rejected unless separately ratified.”
- “Native rest methods | Unsupported in F.3.”

Those statements are no longer current.

The new normative callable documentation requires native and bytecode Methods to share rest acceptance/capture semantics and defines `PrimitiveFn::Shape` as the shape-aware ABI used by native rest gateways.

Current native rest gateways include:

- `Object#perform(_,***)`
- `Method#invokeOn(_,***)`
- `Function#call(***)`

The F.3 closure document must be amended to record this later ratification instead of treating native rest as forbidden.

### 2.3 Confirmed bug: `primitive_rest!` does not populate the ordinary method dictionary

File: `phalcom-core/src/primitive/mod.rs`

Current `primitive_rest!` creates a shape-aware primitive and inserts it only through:

```rust
$vm.heap.class_mut($class).add_rest_method(base_symbol, method_id);
```

It does **not** also perform:

```rust
$vm.heap.class_mut($class).add_method(symbol, method_id);
```

That is inconsistent with bytecode rest installation, where a rest method lives in both:

1. `ClassObject.methods`, keyed by its structural selector; and
2. `ClassObject.rest_methods`, keyed by base-name family.

This is observable outside fallback dispatch:

- `Behavior#methods` enumerates `ClassObject.methods`;
- `VM::finalize_class_base_names` rebuilds the `::` family index from `ClassObject.methods`;
- exact structural reflection expects the structural Method to be a direct definition.

Therefore a native rest gateway can be callable through rest fallback while being absent from direct method reflection and open-family discovery.

This is a runtime/object-model consistency bug introduced by the callable-native-rest work.

### 2.4 Runtime bytecode method installation still mutates the two indexes separately

File: `phalcom-core/src/vm/dispatch.rs`

`Bytecode::Method` currently:

1. calls `validate_method_installation`;
2. assigns holder/access metadata;
3. calls `add_method`;
4. conditionally calls `add_rest_method`;
5. increments `world_version`.

The validation itself is correct, including duplicate rest-family rejection, but the mutation seam is distributed. There is no public reflective method installer today, so this is not a presently exploitable language bug. It is nevertheless a real implementation-quality gap because future runtime installation must not be able to update one index without the other.

Centralize the validated table/index mutation now.

### 2.5 Structured F.3 declaration diagnostics remain incomplete

Files:

- `phalcom-ast/src/error.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-core/src/compiler/lib/error.rs`
- `phalcom-core/src/compiler/lib/class_decl.rs`

The parser currently routes rest-ordering/combination failures through `SyntaxErrorKind::Message`, and the defensive compiler validation still uses `CompilerError::Message` for invalid combinations and duplicate rest families.

This violates the F.3 completion requirement that invalid rest declarations have structured diagnostics with stable categories/spans.

### 2.6 Hierarchy implementation is correct but under-proven

The runtime algorithm already does the desired thing:

- exact selector lookup walks the complete inheritance chain before rest lookup;
- a non-accepting rest method on a child does not terminate the walk;
- `super` begins both exact and rest lookup at the superclass of the defining class.

The missing work is dedicated acceptance evidence for:

1. child rest family exists but rejects the call shape, parent rest accepts;
2. `super` exact lookup beats an ancestor rest fallback;
3. `super` skips a non-accepting parent rest and continues to an accepting grandparent rest;
4. static and dynamic-pack forms behave identically.

### 2.7 E.3 pending fixture is still in the wrong harness

The ignored `collections_pending` lane calls `support::check_pending("collections")`, which expects pending fixtures to succeed.

Yet:

`phalcom-core/tests/lang/collections/pending/boundedness_spread_unbounded_rejected.ph`

is an intentional negative case:

```phalcom
foo(*(0..))
```

The collection-literal negative lane already has an unbounded collection-spread test, but the pending case is distinct: it proves F.2 dynamic-send spread invokes E.3's provably-unbounded rejection.

Promote this send-spread case into `collections/negative/`; do not create a mixed positive/negative pending harness.

### 2.8 General GC stress is specified but not built

File: `docs/work/testing/10-gc-stress.md`

The repository already defines the intended environment variable:

```text
PHALCOM_GC_STRESS
```

The current runtime has one legal collection site:

`VM::service_gc_safepoint`, called at the dispatch-loop safepoint before an opcode starts.

Implement stress at that existing safepoint. Do **not** collect from `Heap::alloc`.

The existing testing document proposes making every allocation latch. This implementation spec tightens the mechanism to match the document's stronger observable promise literally:

- `PHALCOM_GC_STRESS=1`: collect at every dispatch safepoint;
- `PHALCOM_GC_STRESS=N`: collect at every Nth dispatch safepoint;
- unset or `0`: normal threshold-driven collection only.

This avoids overloading `next_gc` (a live-object threshold) with a safepoint cadence and makes the `N` behavior well-defined.

The corpus child process already inherits its parent's environment by default; `tests/support/mod.rs` does not call `env_clear`. No explicit `Command::env` forwarding code is necessary.

### 2.9 Stray source-tree file must be removed

File: `phalcom-core/src/heap/F.md`

At the pinned baseline the file contains only an accidental handoff prompt, not source documentation. Delete it.

---

## 3. Design decisions

### D1 — F.2 semantics are closed

Do not redesign F.2. All patch work touching F.2 is evidence/infrastructure/documentation only.

### D2 — A rest Method has one canonical binding represented in two indexes

For every rest-capable Method owned by a class:

```text
ClassObject.methods[structural_selector] == method
ClassObject.rest_methods[base_name]       == method
```

`rest_methods` is a secondary dispatch index, not an alternative ownership/dictionary.

Native and bytecode rest methods obey the same invariant.

### D3 — Runtime method installation is validated then committed through one helper

Introduce a VM helper in `phalcom-core/src/vm/dispatch.rs`:

```rust
fn install_method_binding(
    &mut self,
    target_class: ClassId,
    selector: Symbol,
    method: ObjRef,
) -> PhResult<()>
```

Responsibilities:

1. call `validate_method_installation` before mutation;
2. install/update `ClassObject.methods`;
3. if rest-capable, install/update `ClassObject.rest_methods`;
4. increment `world_version` exactly once.

It does **not** set holder/access/lexical metadata. `Bytecode::Method` already knows the correct lexical/static owner and must prepare that metadata before calling the helper.

Bootstrap primitive macros remain a distinct trusted path, but `primitive_rest!` must explicitly maintain the same dictionary/index invariant.

### D4 — One rest family per base name per class remains the rule

No wildcard specificity ranking is added.

If a class already owns rest family `foo`, installing a structurally different rest Method for `foo` fails before either index is changed.

Replacing the same structural selector remains compatible with the current validation behavior.

### D5 — Structured diagnostics use typed reason enums

Parser and compiler diagnostics should carry a structured “rest declaration” category rather than a free-form `Message`.

Parser-side reason enum:

```rust
pub enum RestParameterErrorKind {
    AfterTerminal,
    PositionalAfterLabeled,
    DuplicatePositional,
    DuplicateLabeled,
    CompleteConflict,
    PositionalAfterLabeledOrRest,
    UnsupportedInSubscript,
}
```

`SyntaxErrorKind` gets:

```rust
RestParameter(RestParameterErrorKind)
```

with stable `syntax.rest.*` codes.

Compiler-side reason enum:

```rust
pub enum RestDeclarationErrorKind {
    DuplicatePositional,
    DuplicateLabeled,
    DuplicateComplete,
    CompleteConflict,
    TerminalRestNotLast,
}
```

`CompilerError` gets:

```rust
InvalidRestDeclaration {
    kind: RestDeclarationErrorKind,
    span: SourceRange,
}
```

and a dedicated duplicate-family variant carrying both declaration spans.

The compiler variants are intentionally defensive: parser-valid source normally cannot reach several of these states, but attribute expansion/compiler-synthesized AST must not fall back to opaque strings.

### D6 — GC stress is safepoint cadence, never mid-allocation collection

Add stress state to `Heap` because `Heap::new` is the runtime creation point already named by the testing specification:

```rust
gc_stress_interval: Option<usize>,
gc_stress_safepoints: usize,
```

Add:

```rust
pub(crate) fn gc_due_at_safepoint(&mut self) -> bool
```

which returns true when either:

- ordinary `gc_pending` is set; or
- the configured stress safepoint interval expires.

`VM::service_gc_safepoint` becomes:

```rust
if self.heap.gc_due_at_safepoint() {
    self.force_gc();
}
```

`Heap::collect` keeps the ordinary threshold growth policy unchanged. Stress cadence is independent of ordinary allocation thresholds.

Invalid non-numeric/negative `PHALCOM_GC_STRESS` values should fail fast with an explicit configuration panic/message during VM construction rather than silently disabling the test mode.

### D7 — No benchmark result is fabricated

The patch may preserve/add benchmark programs, but F.2/F.3 closure can only mark the performance row complete after a same-machine measurement is actually run and recorded.

The existing:

- `bare_send`
- `rest_fallback_send`

Criterion cases are sufficient.

Do not add a rest inline cache merely to close the checklist.

---

## 4. Implementation tasks

## Task 1 — Repair native rest dictionary/index consistency

### Files

Modify:

- `phalcom-core/src/primitive/mod.rs`
- `phalcom-core/tests/invariants.rs`

### Production change

In `primitive_rest!`, immediately after allocating `method_id`, insert the structural selector into the ordinary method dictionary before adding the rest-family index:

```rust
$vm.heap.class_mut($class).add_method(symbol, method_id);

let base_symbol = $vm.interner.intern($base);
$vm.heap.class_mut($class).add_rest_method(base_symbol, method_id);
$vm.world_version += 1;
```

Do not create a second Method object.

Do not change selector encoding.

Do not add a public primitive.

### Regression test

Add a focused invariant test for all current native rest gateways:

```text
Object   perform(_,***)   base perform
Method   invokeOn(_,***)  base invokeOn
Function call(***)        base call
```

For each row assert:

1. the structural selector is present in `ClassObject.methods`;
2. the same Method handle is present in `rest_methods[base]`;
3. the finalized `base_names[base]` bucket contains that structural selector.

This test must fail against the pinned baseline because condition 1/3 is false for `primitive_rest!`.

It also prevents future “dispatch works, reflection disappears” regressions.

---

## Task 2 — Centralize validated bytecode method installation

### File

Modify:

- `phalcom-core/src/vm/dispatch.rs`

### New helper

Place next to `validate_method_installation`:

```rust
fn install_method_binding(
    &mut self,
    target_class: ClassId,
    selector: Symbol,
    method: ObjRef,
) -> PhResult<()> {
    let rest_base = self.validate_method_installation(target_class, selector, method)?;
    self.heap.class_mut(target_class).add_method(selector, method);
    if let Some(base) = rest_base {
        self.heap.class_mut(target_class).add_rest_method(base, method);
    }
    self.world_version += 1;
    Ok(())
}
```

Import/use the already-available `ClassId`, `ObjRef`, `Symbol`, and `PhResult` names according to the file's current imports.

### `Bytecode::Method` rewrite

Keep the existing static/instance metadata work, but replace duplicated table/index/version mutation with:

```rust
self.install_method_binding(target_class, selector, method_id)?;
```

For the instance side, call `universe.note_method_installed` after successful installation exactly as today.

Required ordering:

```text
derive target class
prepare Method holder/access metadata
prepare Closure.lexical_class
validated atomic binding helper
sacred-selector notification (instance side only)
```

A duplicate rest-family error must occur before either dictionary/index mutation.

### Unit test rewrite

Replace the current `duplicate_rest_family_is_rejected_before_index_mutation` setup that manually calls `validate_method_installation` with a test of `install_method_binding`.

The test should:

1. install the first rest Method through the helper;
2. attempt a structurally different second rest Method in the same base family;
3. assert an error;
4. assert `methods[first_selector]` still points to the first Method;
5. assert `rest_methods[base]` still points to the first Method;
6. assert `second_selector` was not inserted.

This turns the existing evidence from “validator is non-mutating” into “the actual mutation seam is atomic with respect to validation.”

---

## Task 3 — Add structured rest parser diagnostics

### Files

Modify:

- `phalcom-ast/src/error.rs`
- `phalcom-ast/src/parser.rs`

### `phalcom-ast/src/error.rs`

Add the typed reason enum and error messages.

Recommended exact surface:

```rust
#[derive(Debug, Clone, Copy, Error, Eq, PartialEq)]
pub enum RestParameterErrorKind {
    #[error("no parameter may follow **rest or ***rest")]
    AfterTerminal,
    #[error("*rest must precede labeled parameters")]
    PositionalAfterLabeled,
    #[error("at most one *rest parameter is allowed")]
    DuplicatePositional,
    #[error("at most one **rest parameter is allowed")]
    DuplicateLabeled,
    #[error("***rest cannot coexist with *rest or **rest")]
    CompleteConflict,
    #[error("positional parameters must precede labeled parameters and *rest")]
    PositionalAfterLabeledOrRest,
    #[error("rest parameters are not supported in subscript declarations")]
    UnsupportedInSubscript,
}
```

Give each reason a stable code, for example:

```text
syntax.rest.after_terminal
syntax.rest.positional_after_labeled
syntax.rest.duplicate_positional
syntax.rest.duplicate_labeled
syntax.rest.complete_conflict
syntax.rest.positional_after_rest_or_label
syntax.rest.unsupported_subscript
```

Add to `SyntaxErrorKind`:

```rust
#[error("{0}")]
RestParameter(RestParameterErrorKind),
```

and delegate `SyntaxErrorKind::code()` to the nested reason.

### `phalcom-ast/src/parser.rs`

Import `RestParameterErrorKind`.

Replace the corresponding `SyntaxErrorKind::Message(...)` sites in `parse_selector_params` and the subscript-rest check with `SyntaxErrorKind::RestParameter(...)`.

Keep existing ranges exactly where they are already computed.

Do not change valid split-rest grammar:

```phalcom
split(_ fixed, *tail, timeout, **extra)
```

In particular, fix the rustdoc above `parse_selector_params`: `*rest` is **not** required to be the final parameter. The real rule is:

- `*rest` may be followed by fixed labeled parameters and an optional terminal `**rest`;
- `**rest` is terminal;
- `***rest` is terminal and exclusive.

### Parser tests

Add parser unit coverage (in the existing parser test module or the nearest AST test file) that checks the typed kind/code for at least:

- duplicate `*rest`;
- duplicate `**rest`;
- `***rest` combined with another rest lane;
- parameter after `**rest`;
- positional parameter after `*rest`;
- subscript rest.

Also keep one positive split-rest parse case.

---

## Task 4 — Add structured compiler-side rest diagnostics

### Files

Modify:

- `phalcom-core/src/compiler/lib/error.rs`
- `phalcom-core/src/compiler/lib/class_decl.rs`

### New compiler reason enum

Add outside `CompilerError`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestDeclarationErrorKind {
    DuplicatePositional,
    DuplicateLabeled,
    DuplicateComplete,
    CompleteConflict,
    TerminalRestNotLast,
}
```

Implement `Display` or derive a display form appropriate to the project's error style.

### `CompilerError` variants

Add:

```rust
#[error("invalid rest declaration: {kind}")]
InvalidRestDeclaration {
    kind: RestDeclarationErrorKind,
    span: SourceRange,
},
```

and:

```rust
#[error(
    "class.duplicate_rest_family: class '{class}' already defines rest family '{base}' \
     (first declared at {first_line}:{first_col})."
)]
DuplicateRestMethodFamily {
    class: String,
    base: String,
    span: SourceRange,
    first_span: SourceRange,
    first_line: usize,
    first_col: usize,
},
```

Exact field syntax may be adjusted to satisfy `thiserror`, but both spans and the first location must be retained.

### `validate_rest_usage`

Replace generic:

```rust
CompilerError::Message("invalid rest parameter combination".into())
```

and:

```rust
CompilerError::Message("**rest and ***rest must be terminal".into())
```

with typed `InvalidRestDeclaration` variants using the offending parameter's range.

The order of checks should make the diagnostic deterministic:

1. duplicate positional;
2. duplicate labeled;
3. duplicate complete;
4. complete/rest coexistence;
5. terminal labeled/complete rest not last.

### Duplicate family scan

Current code uses:

```rust
rest_families.insert((method.is_static, method.name.clone()), member_name_range).is_some()
```

Change it to retain the first span:

```rust
if let Some(first_span) =
    rest_families.insert((method.is_static, method.name.clone()), member_name_range)
{
    let source = self.source_text();
    let (line, col) = crate::diagnostics::line_col(&source, first_span.start);
    return Err(CompilerError::DuplicateRestMethodFamily { ... });
}
```

The duplicate structural-selector path remains governed by `DuplicateSelector`; this new variant is specifically for *different structural rest selectors in the same base family*.

---

## Task 5 — Add hierarchy and `super` F.3 acceptance fixtures

### Files to create

- `phalcom-core/tests/lang/rest/rest_inherited_nonaccepting_fallback.ph`
- `phalcom-core/tests/lang/rest/rest_inherited_nonaccepting_fallback.expected`
- `phalcom-core/tests/lang/rest/rest_super_boundary.ph`
- `phalcom-core/tests/lang/rest/rest_super_boundary.expected`

The existing `rest_dispatch()` test in `phalcom-core/tests/lang.rs` already runs every `.ph` file in `tests/lang/rest`; no new Rust harness test is required.

### Fixture A — child non-acceptance continues upward

Use this shape:

```phalcom
class ParentRest {
  route(*items) { return 100 + items.size }
}

class ChildRest is ParentRest {
  route(_ fixed, **extra) { return 200 + extra.size }
}

const child = ChildRest.new()
System.print(child.route(1, 2))

const positional = (1, 2)
System.print(child.route(*positional))

System.print(child.route(1, debug: 2))
```

Expected:

```text
102
102
201
```

Why it is discriminating:

- two positionals do not satisfy child's labeled-rest layout (exactly one fixed positional required), so lookup must continue to parent positional rest;
- dynamic `*positional` must take the same path;
- one positional + one label *does* satisfy the child rest and proves the child family itself is live.

### Fixture B — `super` exact/rest boundary

Use three levels:

```phalcom
class GrandRest {
  choose(*items) { return 300 + items.size }
  fallback(*items) { return 400 + items.size }
}

class ParentRestBoundary is GrandRest {
  choose(_ left, _ right) { return 500 }
  fallback(_ fixed, **extra) { return 600 + extra.size }
}

class ChildRestBoundary is ParentRestBoundary {
  choose(*items) { return 700 + items.size }
  fallback(*items) { return 800 + items.size }

  exactSuper() { return super.choose(1, 2) }
  exactSuperDynamic() {
    const args = (1, 2)
    return super.choose(*args)
  }

  fallbackSuper() { return super.fallback(1, 2) }
  fallbackSuperDynamic() {
    const args = (1, 2)
    return super.fallback(*args)
  }
}
```

Expected:

```text
500
500
402
402
```

This proves:

- `super` skips the child's own rest family entirely;
- parent exact wins before grandparent rest;
- non-accepting parent rest continues to grandparent rest;
- static and dynamic-pack super sends use the same boundary.

If the current compiler rejects local `const` in that exact method body form, hoist the Tuple to a local using the repository's accepted local binding syntax; do not weaken the semantic assertions.

---

## Task 6 — Promote the dynamic-send E.3 negative fixture

### Files

Move/rename:

```text
phalcom-core/tests/lang/collections/pending/boundedness_spread_unbounded_rejected.ph
→ phalcom-core/tests/lang/collections/negative/boundedness_send_spread_unbounded_rejected.ph
```

and the `.expected` sidecar likewise.

Update the fixture header to state:

```text
area: collections
spec: E.3 boundedness + F.2 outgoing generic *
status: NEGATIVE
```

Keep the dynamic-send source shape:

```phalcom
class Sink {
  take(*items) { return items.size }
}
Sink.new().take(*(0..))
```

If the current pending fixture uses an undefined `foo`, prefer a real receiver/method so the test cannot accidentally pass/fail for an unrelated undefined selector before boundedness analysis.

Expected diagnostic substring:

```text
cannot exhaust a provably unbounded source
```

The negative harness matches substrings, so do not over-pin source-span formatting.

Delete the old pending pair.

Update the `collections_pending` ignore comment only if necessary; do not add negative semantics to `check_pending`.

---

## Task 7 — Implement `PHALCOM_GC_STRESS`

### Files

Modify:

- `phalcom-core/src/heap/mod.rs`
- `phalcom-core/src/vm/gc.rs`
- `docs/work/testing/10-gc-stress.md`

Optionally add focused unit tests in `phalcom-core/src/heap/mod.rs`'s `#[cfg(test)]` module if one exists; otherwise use `phalcom-core/tests/gc.rs` only for public behavior and keep cadence tests private to the heap module.

### Heap state

Add:

```rust
gc_stress_interval: Option<usize>,
gc_stress_safepoints: usize,
```

Initialize in `Heap::new`.

Parse the environment once:

```text
unset / "" / "0" => None
"1"                => Some(1)
"N" where N > 1    => Some(N)
anything else      => fail fast with a clear PHALCOM_GC_STRESS configuration message
```

### Safepoint predicate

Add:

```rust
pub(crate) fn gc_due_at_safepoint(&mut self) -> bool
```

Pseudo-code:

```rust
let stress_due = match self.gc_stress_interval {
    None => false,
    Some(interval) => {
        self.gc_stress_safepoints += 1;
        if self.gc_stress_safepoints >= interval {
            self.gc_stress_safepoints = 0;
            true
        } else {
            false
        }
    }
};

self.gc_pending || stress_due
```

`force_gc`/`Heap::collect` continues to clear ordinary `gc_pending` as today.

### VM change

In `VM::service_gc_safepoint`:

```rust
if self.heap.gc_due_at_safepoint() {
    self.force_gc();
}
```

Do not add any collection call to `Heap::alloc`.

Do not add a safepoint inside an opcode.

### Tests

Add cadence unit tests that directly configure the private heap stress fields or test a pure parser helper:

- disabled: stress never independently requests collection;
- interval 1: every safepoint due;
- interval 3: false, false, true, repeat;
- ordinary `gc_pending` still requests collection before a stress interval expires.

The broad acceptance commands are:

```bash
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test lang
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test f2_pack_gc
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test outgoing_packs
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test outgoing_packs_completion
```

Use the exact integration-test target names present in the checkout if Cargo reports a renamed target.

The metamorphic requirement is unchanged: outputs/errors must match ordinary mode.

### Documentation correction

Update `docs/work/testing/10-gc-stress.md` section 2.

Remove the “set `next_gc = 0`” mechanism. State instead that stress mode augments the existing safepoint predicate with a cadence counter.

Also remove the claim that `tests/support/mod.rs` must explicitly propagate the environment variable: child processes inherit environment by default.

---

## Task 8 — Update F.2/F.3 closure records

### F.2 file

Modify:

`docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send-amended.md`

Section 36:

- mark the semantic invariants supported by the current implementation/evidence as complete;
- do not leave the entire list unchecked when section 37 already records them as implemented.

Section 37:

- change the GC stress row to checked only after Task 7's implementation/tests are run;
- leave the benchmark row unchecked until a measurement is actually recorded.

Recommended wording:

```text
[x] Every-safepoint PHALCOM_GC_STRESS infrastructure is implemented; stress corpus gate passes.
[ ] Static send benchmark shows no unexplained regression; requires recorded same-machine measurement.
```

If the patch cannot execute the stress corpus in its environment, do **not** mark the “passes” clause complete; distinguish “infrastructure implemented” from “nightly/full corpus observed green.”

### F.3 file

Modify:

`docs/work/pending/collections/F.3-rest-capture-and-rest-pattern-dispatch-amended.md`

Update these stale decisions:

```text
Native rest methods are not silently accepted.
Native rest methods are rejected unless separately ratified.
Native rest methods | Unsupported in F.3
```

to the post-callable-amendment rule:

```text
Shape-aware native rest Methods are ratified by the callable runtime.
They use the same RestLayout acceptance as bytecode Methods.
Native activation receives the shaped call window through PrimitiveFn::Shape;
bytecode activation materializes Unit/Tuple declaration-local captures.
Legacy fixed-arity primitives cannot become rest Methods implicitly.
```

Update the reflective-install wording because no public reflective mutator exists today:

```text
All runtime method-installation entry points maintain both indexes;
no public reflective method installer currently exists.
```

After Tasks 1–7 and fresh test execution, the following rows can be closed:

- subclass non-accepting rest allows superclass fallback;
- super fallback boundary;
- normal/native rest installation maintains both indexes;
- structured rest diagnostics;
- GC/stress infrastructure.

Primitive-floor row:

- this patch adds zero new primitive bindings;
- `primitive_rest!` adding an existing structural selector to `methods` does not create a new Method/binding; it makes the dictionary/index representation consistent;
- run `floor_census_matches_installed_bindings` as the authoritative gate.

Performance row remains measurement-gated.

### Delete accidental file

Delete:

`phalcom-core/src/heap/F.md`

---

## 5. Tests and expected red/green sequence

Because the changes include bug fixes/refactoring, implement in test-first order.

### Cycle 1 — native rest dictionary bug

RED:

Add the invariant test requiring `methods`, `rest_methods`, and `base_names` consistency for native rest.

Expected pinned-baseline failure:

```text
Object/Method/Function native rest structural selector absent from ClassObject.methods
and/or base_names
```

GREEN:

Patch `primitive_rest!`.

Run:

```bash
cargo test -p phalcom-core --test invariants native_rest
cargo test -p phalcom-core --test invariants floor_census_matches_installed_bindings
```

### Cycle 2 — atomic method installation seam

RED/refactor guard:

Rewrite the existing duplicate-rest-family unit test to exercise `install_method_binding`; it will not compile until the helper exists.

GREEN:

Add helper and route `Bytecode::Method` through it.

Run:

```bash
cargo test -p phalcom-core duplicate_rest_family_is_rejected_before_index_mutation
cargo test -p phalcom-core --test lang rest_dispatch
```

### Cycle 3 — structured parser diagnostics

RED:

Add typed-kind assertions.

GREEN:

Add reason enum/`SyntaxErrorKind` variant and replace `Message` sites.

Run:

```bash
cargo test -p phalcom-ast
```

### Cycle 4 — structured compiler diagnostics

Add unit/integration assertions for the new `CompilerError` variants where a compiler-synthesized AST path is testable.

Run:

```bash
cargo test -p phalcom-core
```

### Cycle 5 — inheritance regressions

Add the two `.ph`/`.expected` pairs.

Run:

```bash
cargo test -p phalcom-core --test lang rest_dispatch
```

### Cycle 6 — E.3 fixture promotion

Move/rewrite the dynamic-send boundedness fixture.

Run:

```bash
cargo test -p phalcom-core --test lang collections_literals_negative
cargo test -p phalcom-core --test outgoing_packs_completion
```

### Cycle 7 — GC stress

Add cadence tests first, then production state/predicate.

Run ordinary:

```bash
cargo test -p phalcom-core --test gc
cargo test -p phalcom-core --test f2_pack_gc
cargo test -p phalcom-core --test outgoing_packs
cargo test -p phalcom-core --test outgoing_packs_completion
cargo test -p phalcom-core --test lang rest_dispatch
```

Run stress:

```bash
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test lang
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test f2_pack_gc
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test outgoing_packs
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test outgoing_packs_completion
```

Run cadence middle gear:

```bash
PHALCOM_GC_STRESS=100 cargo test -p phalcom-core --test lang rest_dispatch
```

### Full verification

```bash
cargo fmt --check
cargo test -p phalcom-ast
cargo test -p phalcom-core
cargo clippy -p phalcom-ast -p phalcom-core --all-targets --all-features -- -D warnings
```

Use the repository's actual CI clippy flags if they differ.

---

## 6. Benchmark acceptance procedure

Do not make this a semantic blocker and do not claim a number without running it.

Existing harness:

`phalcom-core/benches/vm_bench.rs`

Relevant cases:

- `bare_send`: exact static dispatch cost;
- `rest_fallback_send`: rest-family miss/fallback baseline.

Required measurement:

```bash
cargo bench -p phalcom-core --bench vm_bench -- bare_send
cargo bench -p phalcom-core --bench vm_bench -- rest_fallback_send
```

For the “no unexplained static regression” row, compare `bare_send` against a recorded same-machine/toolchain/profile baseline. A direct `bare_send` versus `rest_fallback_send` ratio is not evidence for static regression because they measure different mechanisms.

If an unexplained regression appears, profile before adding a rest inline cache.

---

## 7. Primitive-floor acceptance

Run:

```bash
cargo test -p phalcom-core --test invariants floor_census_matches_installed_bindings
```

The floor census already recognizes native rest primitives by reading `rest_methods` in addition to `methods`. Task 1 must not add a new Method object or a new public selector; it only makes the ordinary dictionary contain the same existing Method handle.

Expected feature-local floor delta from this completion patch:

```text
0 public primitive bindings
```

---

## 8. Files changed by the companion patch

Expected modifications:

```text
docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
docs/work/pending/collections/F.3-rest-capture-and-rest-pattern-dispatch-amended.md
docs/work/testing/10-gc-stress.md

phalcom-ast/src/error.rs
phalcom-ast/src/parser.rs

phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/gc.rs

phalcom-core/tests/invariants.rs
phalcom-core/tests/lang.rs                       # comments only if needed

phalcom-core/tests/lang/rest/rest_inherited_nonaccepting_fallback.ph
phalcom-core/tests/lang/rest/rest_inherited_nonaccepting_fallback.expected
phalcom-core/tests/lang/rest/rest_super_boundary.ph
phalcom-core/tests/lang/rest/rest_super_boundary.expected

phalcom-core/tests/lang/collections/negative/boundedness_send_spread_unbounded_rejected.ph
phalcom-core/tests/lang/collections/negative/boundedness_send_spread_unbounded_rejected.expected
```

Expected deletions:

```text
phalcom-core/tests/lang/collections/pending/boundedness_spread_unbounded_rejected.ph
phalcom-core/tests/lang/collections/pending/boundedness_spread_unbounded_rejected.expected
phalcom-core/src/heap/F.md
```

The patch should not modify unrelated callable, Option, collection-literal, or type-system semantics.

---

## 9. Completion definition

The implementation can be called complete only when fresh execution proves all of the following:

- native rest gateways are present in `methods`, `rest_methods`, and `base_names`;
- exact-before-rest semantics remain unchanged;
- duplicate rest-family installation cannot partially mutate either index;
- parser/compiler rest declaration errors use structured categories;
- child non-acceptance falls through to an accepting ancestor in static and dynamic forms;
- `super` uses the same above-defining-class boundary for exact and rest lookup;
- the dynamic-send unbounded-spread negative is active, not hidden in pending;
- every-safepoint GC stress executes the corpus without changing behavior;
- floor census remains green with zero new public primitive bindings;
- formatter/tests/lints are green;
- static-send performance is either recorded within accepted variance or investigated, rather than assumed.

Until those commands have actually been run, the patch should be described as “constructed against `a88bdfee…` and awaiting repository-side verification,” not as a verified passing implementation.
