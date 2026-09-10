# LANG004.C1.P2 — Selector Shapes, Setter Value Lanes, Callable References, and Family Activation

> **Implementation status:** PLAN ONLY — this document does not implement the patch.
>
> **Repository:** `aureat/phalcom-lang`
>
> **Prepared against remote `main` HEAD:** `347b4d7bf2505176a861feea84828256abe2a129` — `docs(implementation): organize CONC002 and LANG004 plans`
>
> **Repository-state limitation:** investigation used the GitHub-visible repository state. It cannot establish the user's current local working tree, uncommitted edits, or unpublished worktrees. C0 therefore begins with a local drift check before any code edit.
>
> **Program placement:** follow-on plan under `docs/implementation/LANG004-associated-lookup-and-logical-paths/C1-associated-lookup/`, after `LANG004.C1.P1`.
>
> **Recommended state file:** `docs/implementation/LANG004-associated-lookup-and-logical-paths/C1-associated-lookup/selector-shapes-and-family-activation-implementation-state.md`.

---

## 1. Program completion claim

This plan completes the selector/reference/family surface that P1 deliberately left open. It replaces the synthetic setter label `put` with a structural, exactly-one-value setter lane; makes exact getter/setter, subscript, operator, selector-pattern, and whole-family reference syntax coherent under prefix `&`; completes subscript-family activation; adds property-style and explicit getter/setter APIs to `Family`; and makes exact singleton-variant references such as `&Option::None` reify as getter-shaped family capabilities that return the canonical singleton through `get()` / `value`.

The dominant law is:

```text
ordinary access/invocation shape + prefix & = reference to that exact shape
postfix ...                          = explicit whole-base family capture
```
Representative end state:

```phalcom
object.name
&object.name                  // exact Getter

object.name()
&object.name()                // exact nullary Method

object.name(x)
&object.name(_)               // exact unary Method

&object.name(...)             // Method-kind structural pattern
&object.name...               // Getter | Setter | Method, same base

object.name = x
&object.name=(_)              // exact Setter
&object.name=                 // Getter | Setter

object[i, debug: flag]
&object[_, debug]             // exact SubscriptGet

object[i, debug: flag] = x
&object[_, debug]=(_)         // exact SubscriptSet

&object[...]                  // all SubscriptGet shapes
&object[...]=(_)              // all SubscriptSet shapes
&object[...]=                 // SubscriptGet | SubscriptSet

&object.+(_)                  // exact operator Method
&object.+(...)                // operator Method pattern
&object.+...                  // complete `+` base family

Option::Some(42)
&Option::Some(_)              // exact payload constructor reference

Option::None
const none = &Option::None    // exact getter-shaped singleton constructor capability
none.get()
none.value
```

The implementation is complete only when selector identity, AST/parser syntax, semantic analysis, lowering, compiler/VM behavior, reflection, LSP output, fixtures, examples, and current normative documentation all agree on these laws; `=(put)` is no longer a production/current-spec authority; and all checkpoint evidence has been executed or explicitly classified.

---

## 2. Repository-grounded architecture

### 2.1 Ownership map

| Concern | Authoritative location | Primary symbols |
|---|---|---|
| Structural selector identity | `phalcom-common/src/selector.rs` | `Selector`, `SelectorBase`, `SelectorKind`, `SelectorSlot`, `SelectorPattern`, `SelectorKindPattern` |
| Source selector syntax/ranges | `phalcom-ast/src/ast.rs` | `SelectorSpecSyntax`, `ExactSelectorSyntax`, `SelectorPatternSyntax`, `CallableReferenceExpr`, `CallableReferenceTarget`, `IndexAccessor`, `SetIndexExpr` |
| Grammar | `phalcom-ast/src/parser.rs` | `parse_callable_reference_selector`, `parse_index_member`, property-setter branch, associated suffix parsing |
| Callable IDs/signatures | `phalcom-semantic/src/checker/declaration_signature.rs` | `CallableSyntaxRef`, `callable_id_for_syntax`, `semantic_signature_for_syntax` |
| Reference/family semantics | `phalcom-semantic/src/checker/expression.rs`, `checker/associated.rs` | `synthesize_callable_reference`, `BehavioralFamilySpec`, `CallableReferenceResolution` |
| Family semantic shapes | `phalcom-semantic/src/types/family.rs` | `FamilyOperationShape`, `FamilyMemberTypeKind` |
| Semantic → executable projection | `phalcom-core/src/modules/semantic_lowering.rs` | `CallableReferenceLoweringSpec`, `ExecutableFamilyDescriptor`, `ExecutableFamilyEntry`, `ExecutableFamilyTarget` |
| Bound Family runtime | `phalcom-core/src/vm/send.rs`, heap family types | `FamilyInvocationKind`, `activate_family_with_kind`, `FamilySpec` |
| Associated Family runtime | `phalcom-core/src/heap/associated.rs`, `vm/dispatch.rs`, `vm/associated.rs` | `AssociatedFamilyObject`, associated-family bytecodes/target activation |
| Static/dynamic subscript sends | `phalcom-core/src/compiler/lib/expr.rs`, `vm/dispatch.rs` | `Expr::SetIndex`, `ArgumentPackBuilderObject`, `InvokePack`, `PackSendKind` |
| Public `Family` API | `phalcom-core/core/universe/src/callable/family.ph`, `phalcom-core/src/primitive/family.rs` | `Family`, `family_get`, `family_set` |
| Reflection | `phalcom-core/src/primitive/selector.rs`, `method/mod.rs` | `Selector.from`, `Selector::kind`, `decode_selector` |
| LSP presentation | `phalcom-lsp/src/selectors.rs`, `completion.rs`, hover/token consumers | selector formatting/completion helpers |
| Current normative docs | `docs/spec/current/selectors.md`, `docs/spec/callables/family.md`, `docs/spec/adts.md`, ADR-0060 | selector/family laws |

The shared structural selector is already the correct authority. `Selector::setter(name)` stores **zero ordinary slots**, and `Selector::subscript_set(slots)` stores only bracket/index slots. `SignatureKind::Setter` derives runtime arity 1; `SignatureKind::SubscriptSet(n)` derives `n + 1`. The repository therefore already contains the right semantic distinction: the `put` spelling is a surface/transport leak, not a reason to flatten the assigned value into the selector tuple.

### 2.2 End-to-end flow

```text
source declaration/reference/send
    ↓
AST range-rich selector syntax
    ↓
phalcom-common Selector / SelectorPattern
    ↓
semantic CallableId / FamilyOperationShape / family resolution
    ↓
ModuleLoweringSemantics
    ↓
MakeFamily | MakeAssociatedFamily | constructor thunk | SetIndex lowering
    ↓
VM Family / AssociatedFamily / message dispatch
    ↓
selected current Method or canonical singleton Value
```

No later layer should re-derive selector semantics from punctuation when structural data is available.

### 2.3 Sources of truth

**Selector identity**

```text
Source of truth:
    phalcom_common::selector::{Selector, SelectorPattern}

Derived:
    encoded Symbol
    SignatureKind bridge
    semantic CallableId
    runtime method-table key
    reflection/LSP text

Forbidden competing authority:
    suffix parsing
    punctuation heuristics
    synthetic `put` label
```

**Setter value lane**

```text
Source of truth:
    SelectorKind::Setter / SelectorKind::SubscriptSet
    + one dedicated callable value parameter

Ordinary selector slots:
    property setter: []
    subscript setter: bracket tuple slots only

Invocation values:
    selector-shaped values + exactly one trailing setter RHS
```

**Family representation**

```text
Object::Family
    bound receiver + live ordinary dispatch

Object::AssociatedFamily
    frozen semantic descriptor + associated target execution

Public activation kind:
    Method | Getter | Setter | SubscriptGet | SubscriptSet
```

These two runtime family representations may share a public API but must not become one semantic authority.

---

## 3. Ratified semantic invariants

### I-01 — Canonical setter selectors

```text
property=(_)
[_, _, debug]=(_)
```

`property=(put)` and `[...]=(put)` are obsolete current spellings.

### I-02 — Exactly one setter value lane

A named setter has exactly one value lane; a subscript setter has exactly one value lane. These are invalid:

```text
property=(_, _)
property=(_, _, param)
[_, _]=(_, _)
[_, _]=(_, _, param1, param2)
```

### I-03 — Setter RHS is not a selector slot

`[_, _, debug]=(_)` is structurally:

```rust
Selector {
    base: SelectorBase::Subscript,
    kind: SelectorKind::SubscriptSet,
    slots: [Positional, Positional, Label("debug")],
}
```

The final `(_)` is implied by `SubscriptSet` and never appears in `slots`.

### I-04 — Normal positional-before-label law is unchanged

The final setter RHS is a distinguished value lane outside the bracket tuple. It is **not** permission for ordinary calls/tuples to place a positional after a labeled lane.

### I-05 — Evaluation order

For:

```phalcom
receiver()[index1(), index2(), debug: flag()] = rhs()
```

observable order is:

```text
receiver → index1 → index2 → flag → rhs → setter body
```

Each executes exactly once. RHS is the last operand evaluation.

### I-06 — Indexed assignment result

Preserve the repository's current indexed-assignment law: the assignment expression evaluates to the original RHS independent of the underlying setter body's return value.

### I-07 — Named selector/family algebra

```text
property             exact Getter
property=(_)         exact Setter
property=            Getter | Setter
property()           exact nullary Method
property(_)          exact Method
property(_, label)   exact Method
property(...)        Method-kind pattern
property(_, ...)     Method-kind pattern
property...          Getter | Setter | Method for that base
```

### I-08 — Subscript algebra

```text
[shape]              exact SubscriptGet
[shape]=(_)          exact SubscriptSet
[...]                any SubscriptGet
[...]=(_)            any SubscriptSet
[...]=               SubscriptGet | SubscriptSet
```

### I-09 — Operators and special named bases

```text
+()       exact nullary Method
+(_)      exact unary Method
+(from)   exact labeled Method
+(...)    Method pattern
+...      complete base family
```

`+=(_)` must not become a property setter. `==(_)`, `>=(_)`, `<=(_)`, etc. remain Method selectors.

### I-10 — Prefix `&` references the written selector shape

```text
&receiver.name        exact Getter
&receiver.name=(_)    exact Setter
&receiver.name=       Getter | Setter
&receiver.name(...)   Method pattern
&receiver.name...     entire named family

&receiver[_, debug]       exact SubscriptGet
&receiver[_, debug]=(_)   exact SubscriptSet
&receiver[...]            SubscriptGet pattern
&receiver[...]=(_)        SubscriptSet pattern
&receiver[...]=           all subscript accessors
```

The reference target is the rightmost selector component. The receiver prefix is evaluated normally exactly once.

### I-11 — Exact getter reference activation remains Family-style

```phalcom
const getter = &object.value
getter.get()
getter.value
```

`getter()` remains Method-kind invocation and must not fall back to Getter.

### I-12 — Named accessor API aliases

```text
family.value       ≡ family.get()
family.value = rhs ≡ family.set(rhs)
```

### I-13 — Subscript Family APIs

```phalcom
family[key, debug: mode]
family[key, debug: mode] = rhs

family.get((key, debug: mode))
family.set((key, debug: mode), rhs)
```

The first argument to explicit subscript APIs is a real Tuple/product preserving positional/labeled layout. Setter RHS remains outside that product.

### I-14 — Singleton variant references

```phalcom
Option::None            // canonical singleton value
const none = &Option::None
none.get()              // same canonical singleton
none.value              // same canonical singleton
```

`&Option::None` is a getter-shaped capability, not the singleton itself and not a nullary Method closure. Payload variants remain Method-shaped constructor references.

### I-15 — Family construction is non-probing

Creating a bound reference/family does not invoke the member, call `doesNotUnderstand`, or require the live bound target to exist at construction time. Associated exact references may use the existing semantic resolution authority.

---

## 4. Tempting wrong fixes — forbidden

1. Do not append a subscript RHS to `ArgumentPackBuilderObject.positionals` after labels.
2. Do not keep `put` as an invisible runtime label while merely changing printed syntax.
3. Do not turn `Setter` into unary `Method`.
4. Do not relax global positional-before-label validation.
5. Do not make `family()` fall back to Getter or `family(x)` fall back to Setter.
6. Do not leave bare `&receiver.name` as whole-family capture; whole family is `&receiver.name...`.
7. Do not make `&Option::None` evaluate the singleton during reference construction.
8. Do not make `&Option::None` a zero-argument Method closure.
9. Do not route `AssociatedFamily` through ordinary live receiver-family lookup.
10. Do not replace LSP `=(put)` suffix checks with `=(_)` suffix checks when structural `Selector` data exists.
11. Do not create another selector/pattern type in semantic/core/LSP.
12. Do not model `property=` as a fake slot ellipsis.
13. Do not infer setter semantics from an operator base ending in `=`.
14. Do not change indexed-assignment result semantics while repairing lowering.
15. Do not rewrite historical/as-built documents as though old syntax never shipped.

---

## 5. Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| **C0 — Revision and migration baseline** | 1–2 | Local execution state is reconciled with pinned remote HEAD; old mechanisms and current tests are inventoried. | revision/status; migration searches; focused baseline if needed | workspace test/clippy |
| **C1 — Canonical selector identity** | 3–5 | Shared selector layer owns `=(_)`, accessor kind sets, and unchanged slot ordering. | `phalcom-common` tests; core bridge compile; hostile operator cases | parser/semantic/runtime |
| **C2 — Source grammar and AST** | 6–10 | Parser/AST express all exact/pattern/family shapes and new setter declarations. | AST integration + syntax hostile matrix | semantic behavior |
| **C3 — Semantic family/reference model** | 11–15 | All source forms resolve to correct structural family operations; setter RHS is dedicated. | semantic family/associated tests | codegen/runtime |
| **C4 — Setter send ABI and evaluation order** | 16–19 | Static/dynamic SetIndex separate index shape from RHS with no `put`. | focused index/pack/runtime tests | Family subscript API |
| **C5 — Complete Family activation surface** | 20–24 | Five activation kinds + `value` aliases + direct/explicit subscript APIs work. | family runtime/API tests; native census | singleton integration |
| **C6 — Exact singleton associated references** | 25–27 | `&Option::None` is getter-shaped AssociatedFamily capability returning canonical singleton. | semantic/lowering/runtime ADT tests | tooling/docs |
| **C7 — Reflection and editor consistency** | 28–30 | Reflection/LSP/source index present the same structural selector truth. | reflection + targeted LSP tests | broad delivery |
| **C8 — Fixtures/docs/governance/deletion** | 31–34 | Current examples/specs/governance are migrated and old authority cannot silently run. | language corpus + negative gates | workspace gates |
| **Final Gate** | — | Cross-crate delivery readiness and complete evidence ledger. | format/check/package/workspace gates | none |

---

# Checkpoint C0 — Revision and migration baseline

### Tasks
- Task 1 — Reconcile repository state and create P2 implementation state.
- Task 2 — Inventory production/current-spec consumers and establish baseline evidence.

### Why this is a checkpoint

The current GitHub state contains the recently landed P1 callable-reference work, while P1's state document records historical worktree/baseline details. This patch must not confuse those records with the user's current local checkout. C0 creates the evidence ledger and finite migration set before code changes begin.

### Entry conditions
- Local checkout contains or intentionally supersedes remote commit `347b4d7bf2505176a861feea84828256abe2a129`.
- P1 `Expr::CallableReference`, semantic resolution, and lowering products are present.

### Working set
**Primary:** C1 checkpoint/state docs, repository status/log, searches for `=(put)`, `CallableReferenceTarget`, `FamilyInvocationKind`, `PackSendKind::SubscriptSet`, `IndexAccessor::Set`.

**Secondary:** only files surfaced by those searches.

**Out of scope:** code edits, unrelated concurrency/fiber work, unrelated baseline repairs.

### Semantic contract
- Local revision/drift is explicitly known.
- Pre-existing failures are separable from P2 regressions.
- The old spelling and old P1 bare-reference rule have a finite migration inventory.

### Risks
- stale branch assumptions;
- hidden local partial implementation;
- historical/generated docs mistaken for normative production sources.

### Hostile cases
- local branch already contains partial selector changes;
- remote HEAD differs from local main;
- dated docs intentionally retain old syntax.

### Required evidence

```bash
git rev-parse HEAD
git status --short
git log -8 --oneline

rg -nF '=(put)' phalcom-common phalcom-ast phalcom-semantic phalcom-core phalcom-lsp docs/spec docs/adr docs/pdr
rg -n 'CallableReferenceTarget|FamilyInvocationKind|PackSendKind::SubscriptSet|IndexAccessor::Set' phalcom-ast phalcom-semantic phalcom-core phalcom-lsp
```

If local drift makes baseline attribution necessary, run only the nearest existing P1 slices first.

### Do not run yet
`cargo test --workspace --all-targets` or full workspace clippy.

### Escalate immediately if
- P1 reference/lowering architecture is absent;
- selector authority has moved out of `phalcom-common`;
- local partial work contradicts §3 semantics.

### Completion
- [ ] local revision/status recorded;
- [ ] migration inventory categorized;
- [ ] baseline evidence recorded where needed;
- [ ] P2 state file created;
- [ ] no unresolved PLAN DRIFT incident.

## Task 1 — Reconcile state and create the implementation ledger

**Purpose:** establish an auditable resume point.

**Risk:** Semantic LOW; fanout local.

**Owned files:** `.../C1-associated-lookup/CHECKPOINT.md`; new P2 state file.

**Source of truth:** actual local checkout.

**Edit operations:**
1. Add `LANG004.C1.P2` to C1's plan table.
2. Create the state file with sections `Repository state`, `Established invariants`, `Decisions`, `Evidence ledger`, `Deferred gates`, `Active incident`, `Next resume action`.
3. Seed I-01 through I-15 from this plan.
4. Record actual SHA/branch/worktree and pre-existing edits.

**Testing classification:** no standalone behavioral test.

## Task 2 — Inventory consumers and baseline

**Purpose:** bound implementation fanout before editing.

**Risk:** Semantic LOW; fanout cross-crate search.

**Inspect:** exact old-spelling hits; common selector encoder/decoder; setter parser branches; `decode_selector`; dynamic SetIndex; Family runtime; LSP suffix handling; current docs.

**Must not:** turn every historical `docs/implementation/**` or `docs/wiki/raw/**` hit into a rewrite requirement.

**Edit operations:** categorize hits as production, parser/AST, LSP, current normative docs, governance docs, historical/generated docs. Record nearest existing tests for each ownership layer.

**Testing classification:** investigation only.

---

# Checkpoint C1 — Canonical selector identity

### Tasks
- Task 3 — Migrate exact selector encode/decode to `=(_)`.
- Task 4 — Add named-accessor and subscript-accessor kind-set patterns.
- Task 5 — Remove synthetic `put` from the runtime signature bridge and lock hostile selector cases.

### Why this is a checkpoint

Every later layer consumes structural selectors. Parser, semantic, VM, reflection, and LSP must not independently decide what `property=`, `[...]=`, or `=(_)` means.

### Entry conditions
- C0 COMPLETE.
- `phalcom-common::selector` remains authoritative.

### Working set
**Primary:** `phalcom-common/src/selector.rs`, `phalcom-core/src/method/mod.rs`.

**Secondary:** exhaustive enum consumers that compilation exposes.

**Out of scope:** AST grammar, Family APIs, VM subscript activation.

### Semantic contract
- `Selector::setter("x").encode() == "x=(_)"`.
- subscript setter encoding ends with `]=(_)`.
- setter RHS never appears in `Selector.slots`.
- selector patterns represent exact kind, all named, named accessors, and all subscripts.
- ordinary slot ordering remains positional-then-labeled.

### Risks
- decoding setter as Method;
- allowing arbitrary setter slots;
- named patterns matching subscript kinds;
- operator bases misclassified as setters.

### Hostile cases
`x=(_,_)` rejected; `[_,debug]=(_,_)` rejected; `==(_)` remains Method; `x(...)` excludes accessors; `x...` includes named accessors/methods; `x=` excludes methods; `[...]=` excludes named members.

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-common
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

The first proves structural selector laws; the second is only exhaustive/caller compilation evidence.

### Do not run yet
AST/semantic/LSP behavioral suites.

### Escalate if
- exact selector strings are persisted under a compatibility contract;
- any production structure stores setter RHS in `Selector.slots`.

### Completion
- [ ] exact encoding/decoding green;
- [ ] new kind patterns green;
- [ ] operator hostile cases green;
- [ ] core bridge compiles;
- [ ] state updated.

## Task 3 — Migrate exact selector encoding/decoding

**Purpose:** make `=(_)` the sole current exact setter spelling.

**Risk:** Semantic HIGH; fanout multi-file.

**Owned symbols:** `Selector::encode`, `Selector::try_decode_exact`, runtime-form decoder, exact selector tests.

**Current:** named/subscript setter encode as `=(put)`.

**Target:** named/subscript setter encode as `=(_)`; parsed `_` after `=` never enters `slots`.

**Edit operations:**
1. Replace encode branches.
2. Rewrite exact decoder recognition for `name=(_)` and `[slots]=(_)`.
3. Preserve Getter/Setter zero-slot validation.
4. Investigate any demonstrated persistence compatibility before retaining a legacy input decoder; canonical output must always be new syntax.
5. Add round-trips for empty/positional/labeled subscript shapes.

**Code instruction — STRUCTURAL:**

```rust
(SelectorBase::Named(name), SelectorKind::Setter) => format!("{name}=(_)")
(SelectorBase::Subscript, SelectorKind::SubscriptSet) => format!("[{slots}]=(_)")
```

**Testing:** C1 boundary.

## Task 4 — Generalize selector kind patterns

**Purpose:** represent `property=` and `[...]=` without fake slots.

**Risk:** Semantic HIGH; fanout cross-crate enum.

**Target — STRUCTURAL:**

```rust
pub enum SelectorKindPattern {
    Exact(SelectorKind),
    AnyNamed,        // Getter | Setter | Method
    NamedAccessors,  // Getter | Setter
    AnySubscript,    // SubscriptGet | SubscriptSet
}
```

**Edit operations:** update validation, `matches`, encode/decode, ordering/hash derives, tests. Relax `SelectorPattern::new`'s current `has_gap` requirement only enough for kind-set patterns such as `property=`; do not manufacture an ellipsis lane.

**Must not:** expose arbitrary unsupported selector-kind sets merely because a bitset is convenient internally.

**Testing:** focused common-selector tests at C1.

## Task 5 — Remove synthetic `put` from `SignatureKind` bridge

**Purpose:** prevent runtime selector decomposition from resurrecting `put`.

**Risk:** Semantic HIGH; fanout core bridge.

**Owned symbols:** `SignatureKind`, `encode_selector`, `decode_selector` in `phalcom-core/src/method/mod.rs`.

**Current hazard:** `decode_selector` fabricates `vec![Some("put")]` for a setter.

**Target:** Getter/Setter have no selector labels. Runtime arity continues to come from `Signature::new`.

**Edit operations:** remove the synthetic label injection; update comments; audit every `decode_selector` consumer that expected `put`; branch on `SignatureKind` instead. Add operator/setter collision tests.

**Testing:** C1.

---

# Checkpoint C2 — Source grammar and AST

### Tasks
- Task 6 — Migrate property and subscript setter declaration grammar.
- Task 7 — Generalize callable-reference target syntax beyond named-only.
- Task 8 — Implement exact getter/setter and whole named-family parsing.
- Task 9 — Implement operator/subscript reference parsing and patterns.
- Task 10 — Migrate AST consumers mechanically and lock hostile syntax.

### Why this is a checkpoint

C2 makes the complete desired surface representable with precise ranges before semantic meaning is attached. Parser tests can prove that exact selectors, kind-set projections, structural patterns, whole families, operators, and subscripts are distinct syntax categories without depending on type checking or runtime dispatch.

### Entry conditions
- C1 COMPLETE.
- shared selector algebra compiles.

### Working set
**Primary:** `phalcom-ast/src/ast.rs`, `phalcom-ast/src/parser.rs`, `phalcom-ast/tests/family_selector_syntax.rs`, parser integration tests.

**Secondary — compile fanout only:** `phalcom-semantic/src/advisory/analyzer.rs`, `phalcom-semantic/src/source_index/{builder,occurrence}.rs`, compiler/LSP AST walkers.

**Out of scope:** semantic family membership, bytecode, runtime activation.

### Semantic contract
The AST distinguishes exact selector, accessor kind-set, slot pattern, and whole family. Setter declarations accept exactly one positional value binder after `=`.

### Risks
- postfix `...` consumed as an expression ellipsis instead of reference-family marker;
- `name=(_)` parsed as a Method base ending in `=`;
- subscript reference syntax collides with normal `IndexExpr` parsing;
- AST generalization loses source ranges used by source index/LSP.

### Hostile cases
Must reject:

```phalcom
name=(_ x, _ y) { ... }
name=(label x) { ... }
[_ i]=(_ x, _ y) { ... }
[_ i]=(label x) { ... }
&object.name=(_, _)
&object[_, debug]=(_, _)
```

Must distinguish:

```phalcom
&object.name
&object.name()
&object.name...
&object.name(...)
&object.name=
&object.name=(_)
```

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

The AST suite proves source classification/ranges. The semantic check proves mechanical exhaustive-callsite migration only.

### Do not run yet
Semantic behavioral suites and core runtime tests.

### Escalate immediately if
- subscript references require changing ordinary indexing precedence rather than adding a reference-context production;
- generalized `CallableReferenceTarget` cannot preserve the selector/name ranges source index already consumes;
- operator reference parsing would make prefix `&` ambiguous with the overloadable `&` operator outside reference context.

### Completion
- [ ] setter declarations migrated;
- [ ] named exact/kind-set/pattern/whole-family forms parse;
- [ ] operator forms parse;
- [ ] subscript forms parse;
- [ ] illegal setter-value shapes reject at parse time;
- [ ] AST consumers compile;
- [ ] state updated.

## Task 6 — Migrate setter declaration grammar

**Purpose:** replace the mandatory textual `put` role with one positional setter-value binder.

**Risk:** Semantic MEDIUM; fanout AST/parser + declaration consumers.

**Owned files/symbols:**
- `phalcom-ast/src/parser.rs` — property setter branch and `parse_index_member`.
- `phalcom-ast/src/ast.rs` — `SetterDef`, `IndexAccessor`.

**Current implementation:** both setter productions require literal identifier `put`; subscript AST stores `IndexAccessor::Set { put: Box<ParameterDef> }`.

**Target declaration forms:**

```phalcom
property=(_ value) { ... }
[_ index, debug mode]=(_ value) { ... }
```

**Edit operations:**
1. Extract one parser helper if both setter branches duplicate the same grammar.
2. Require `(`, positional marker `_`, local binder, optional annotation, `)`.
3. Reject label syntax, rest syntax, commas, or additional parameters.
4. Rename `IndexAccessor::Set { put }` to `Set { value }` (or another neutral repository-consistent name) and update exhaustive callers.
5. Keep `SetterDef.param` as the one dedicated value parameter.
6. Update ranges/comments from “put role” to “setter value lane”.

**Code instruction — STRUCTURAL:**

```rust
fn parse_setter_value_parameter(&mut self) -> ParserResult<ParameterDef> {
    // exactly `(_ local [: Type])`
    // label == None
    // rest_mode == None
}
```

If the current parser has a reusable positional-binder helper that can be constrained to exactly one lane, reuse it instead.

**Testing classification:** C2 syntax matrix.

## Task 7 — Generalize callable-reference target syntax

**Purpose:** remove P1's named-only reference target limitation while preserving receiver-once semantics and source ranges.

**Risk:** Semantic MEDIUM; fanout cross-crate exhaustive enum.

**Owned symbol:** `CallableReferenceTarget` and constructors/consumers.

**Current:** `BoundNamed` / `AssociatedNamed`.

**Target — STRUCTURAL:** prefer one shared selector-member representation over proliferating `BoundOperator`, `BoundSubscript`, etc. A likely shape is:

```rust
pub enum CallableReferenceTarget {
    Bound {
        receiver: Box<Expr>,
        member: CallableReferenceMemberSyntax,
        separator_range: SourceRange,
    },
    Associated {
        receiver: Box<Expr>,
        member: CallableReferenceMemberSyntax,
        separator_range: SourceRange,
    },
}

pub enum CallableReferenceMemberSyntax {
    Named { /* base/ranges/spec */ },
    Operator { /* structural selector spec */ },
    Subscript { /* structural selector spec */ },
}
```

Reuse `ExactSelectorSyntax`, `SelectorPatternSyntax`, or `AssociatedMemberSyntax` where they already express the required range-rich structure. Do not duplicate the selector model just to avoid a local AST refactor.

**Edit operations:** update semantic checker, compiler, advisory analyzer, source-index builder/occurrence, and LSP/token walkers exhaustively. Preserve the law that everything before the rightmost reference target is an ordinary receiver expression evaluated once.

**Testing:** no standalone behavior; C2 parser + `cargo check`.

## Task 8 — Parse named exact/accessor/whole-family references

**Purpose:** implement the final named reference grammar.

**Risk:** Semantic HIGH; fanout parser/normalized syntax.

**Target mapping:**

```text
&object.name        Exact(Getter)
&object.name()      Exact(Method [])
&object.name(_)     Exact(Method [_])
&object.name=(_)    Exact(Setter)
&object.name=       Pattern(NamedAccessors)
&object.name(...)   Pattern(Method)
&object.name...     Pattern(AnyNamed)
```

The same selector algebra applies after `::` where associated lookup permits the named base.

**Edit operations:**
1. Make bare name explicitly normalize to Getter rather than “selector absent = whole family”.
2. Parse bare trailing `...` as whole-family `AnyNamed`.
3. Parse trailing `=` with no value pattern as `NamedAccessors`.
4. Parse `=(_)` as exact Setter only.
5. Keep parentheses without `=` in Method-kind space.
6. Preserve selector labels as labels (`debug`), not destructuring syntax (`debug: _`).

**Must not:** use expected type to disambiguate bare getter vs family.

**Testing:** C2.

## Task 9 — Parse operator and subscript references

**Purpose:** complete the reference surface across selector bases.

**Risk:** Semantic HIGH; fanout parser/AST.

**Inspect before editing:** `parse_property_name`, `parse_method_name`, associated operator/subscript parsing, normal `IndexExpr` postfix parsing.

**Target forms:**

```phalcom
&object.+()
&object.+(_)
&object.+(...)
&object.+...

&object[_, debug]
&object[_, debug]=(_)
&object[...]
&object[_, ...]
&object[..., debug]
&object[...]=(_)
&object[_, ...]=(_)
&object[...]=
```

**Implementation guidance:** adapt the existing associated `Operator(ExactSelectorSyntax)` / `Subscript(ExactSelectorSyntax)` machinery so bound and associated reference targets normalize through the same common selector pipeline.

**Must not:** parse `+=(_)` as a property setter; operator base identity remains authoritative.

**Testing:** focused parser regression at C2.

## Task 10 — Migrate AST consumers without adding semantics

**Purpose:** complete the structural refactor while keeping ownership boundaries clean.

**Risk:** Semantic LOW; fanout cross-crate.

**Owned consumers:** advisory analyzer, source-index builder/occurrence, compiler AST matches, LSP semantic-token walkers.

**Changes:** preserve receiver visitation, selector/name/bracket source ranges, and target occurrences. Do not perform selector matching or resolution in these walkers.

**Testing:** no standalone test; C2 compile and C7 integration prove this work.

---

# Checkpoint C3 — Semantic family/reference model

### Tasks
- Task 11 — Normalize every reference form to canonical family specs.
- Task 12 — Extend family member/type matching for accessor kind sets and subscripts.
- Task 13 — Keep setter RHS as a dedicated semantic application role.
- Task 14 — Represent exact singleton associated references as getter-shaped associated-family capabilities.
- Task 15 — Update denotation/fingerprint/source products and semantic hostile tests.

### Why this is a checkpoint

Parser distinctions matter only if they resolve to the intended family operations and types. C3 is the semantic source-of-truth checkpoint; lowering must consume these products rather than re-read source punctuation.

### Entry conditions
- C2 COMPLETE.
- all reference forms have precise AST classifications.

### Working set
**Primary:** `phalcom-semantic/src/checker/expression.rs`, `checker/associated.rs`, `types/family.rs`, `types/denotation.rs`.

**Secondary:** semantic snapshot/fingerprint and source-index publication only where changed products require it.

**Tests:** `phalcom-semantic/tests/semantic/families/*`, `associated/*`, generic ADT associated tests.

**Out of scope:** bytecode, runtime Family methods, LSP formatting.

### Semantic contract
- each reference syntax maps to one structural `Selector`/`SelectorPattern`;
- whole family, callable pattern, accessor kind-set, exact getter/setter, and subscript kinds remain distinct;
- setter RHS binds to the dedicated setter parameter but not selector shape;
- exact singleton associated reference denotes a Family capability, not a singleton value.

### Risks
- `property=` includes Method members;
- `name(...)` includes Getter/Setter;
- RHS contributes a positional selector slot;
- generic setter inference binds the RHS against an index parameter;
- `&Option::None` collapses to `ExactValue`.

### Hostile cases
Use one class defining Getter `x`, Setter `x=(_)`, Method `x()`, and Method `x(_)`. Assert member/operation kinds for `&obj.x`, `&obj.x=`, `&obj.x(...)`, and `&obj.x...`. Add a subscript getter/setter pair with mixed positional/labeled index shape. Verify `Option::None` and `&Option::None` have different semantic products.

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic 'families::' -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic 'associated::' -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic 'integration::generic_adts' -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic --no-run
```

Use nearest valid module filters if names drift.

### Do not run yet
Full semantic suite, core runtime, LSP.

### Escalate immediately if
- `FamilyOperationShape` cannot represent all five selector kinds without adding another selector authority;
- generic callable application fundamentally requires every application positional to contribute a selector positional slot;
- exact singleton reference cannot be represented as a one-entry associated family without violating current family typing.

### Completion
- [ ] semantic reference matrix passes;
- [ ] setter RHS binding passes;
- [ ] singleton exact ref is capability, not value;
- [ ] semantic products publish/fingerprint deterministically;
- [ ] no error/Unknown was weakened to Dynamic;
- [ ] state updated.

## Task 11 — Normalize new reference syntax to `BehavioralFamilySpec`

**Purpose:** make structural common selectors the only semantic authority for bound references.

**Risk:** Semantic HIGH; fanout semantic checker.

**Owned symbols:** `synthesize_callable_reference`, helper converting selector syntax to `BehavioralFamilySpec`, `resolve_bound_behavioral_family`.

**Exact semantic mapping:**

```text
bare named       → Exact(Selector::getter(base))
named =(_)       → Exact(Selector::setter(base))
named =          → Pattern(NamedAccessors)
named (...)      → Pattern(Exact Method kind)
named ...        → Pattern(AnyNamed)

subscript exact  → Exact(SubscriptGet/SubscriptSet)
subscript gap    → Pattern(exact subscript kind)
subscript [...]= → Pattern(AnySubscript)

operator exact/pattern/whole → named-base Method/family algebra,
                               never implicit Setter
```

**Edit operations:** replace the P1 assumption that `selector == None` implies whole family. Ensure normalized structural spec is stored in `CallableReferenceResolution`, not source punctuation.

**Testing:** C3 semantic matrix.

## Task 12 — Extend family member/type matching

**Purpose:** ensure selected family types contain exactly the members denoted by the selector pattern.

**Risk:** Semantic HIGH; fanout semantic family/type code.

**Owned symbols:** family candidate resolution in `checker/associated.rs`; `FamilyOperationShape`; member filtering/type construction.

**Changes:** teach matching about `NamedAccessors`/`AnySubscript`; preserve selector kind in `FamilyOperationShape`; keep whole named families disjoint from subscript families; keep `name(...)` Method-only; preserve existing Method rest-pattern semantics.

**Hostile test:** same base has getter, setter, nullary method, unary method; assert exact member-kind sets for all projections.

## Task 13 — Model setter RHS as dedicated semantic application role

**Purpose:** preserve normal call-shape validation while type-checking the final setter value.

**Risk:** Semantic HIGH; fanout callable application.

**Owned symbols:** `synthesize_set_property`, `synthesize_set_index_expr`, and only the necessary `checker/call.rs` argument/binding types.

**Current useful behavior:** `synthesize_set_index_expr` already computes selector slots from `index_arguments` only and separately appends the RHS to application arguments.

**Target:** preserve that split through binding. If generic `ApplicationArgument::Positional` layout validation rejects a setter RHS after labels, introduce either a dedicated `SetterValue` role or a setter-specific apply helper. Choose the smallest architecture-consistent change; do not weaken general call ordering.

**STRUCTURAL option:**

```rust
enum ApplicationArgument<'a> {
    Positional { ... },
    Labeled { ... },
    SetterValue { expression: &'a Expr, range: SourceRange },
}
```

Use only if keeping RHS outside `ApplicationArgument` would duplicate canonical callable-application logic.

**Testing:** focused generic/labeled setter semantic regression.

## Task 14 — Exact singleton associated reference semantic product

**Purpose:** distinguish direct singleton value lookup from reference to the singleton-producing Getter.

**Risk:** Semantic HIGH; fanout semantic/lowering dependency.

**Current:** `Option::None` resolves as `AssociatedResolutionKind::ExactValue`; associated family descriptors already represent singleton targets.

**Target:** in callable-reference context, the same singleton variant becomes a one-member associated Family resolution whose operation is Getter and whose member target is the singleton variant. Direct `Option::None` remains `ExactValue`.

**Must not:** mutate ordinary associated lookup semantics to make reference lowering work.

**Testing:** C3 + C6.

## Task 15 — Publish/fingerprint the new semantic products

**Purpose:** keep snapshot/incremental products deterministic after enum/spec changes.

**Risk:** Semantic MEDIUM; fanout semantic internals.

**Owned files:** denotation/fingerprint modules if exhaustive matches change; snapshot publication; source-index reference products.

**Changes:** update exhaustive hashing/ordering; preserve source ranges for exact selector target components; add a cold/rebuilt-snapshot equality assertion where practical for the new structural selector/reference product.

**Testing:** C3 focused semantic suites.

---

# Checkpoint C4 — Setter send ABI and evaluation order

### Tasks
- Task 16 — Preserve static SetIndex as index shape + trailing value lane.
- Task 17 — Replace dynamic synthetic-`put` pack encoding with a separate setter-value channel.
- Task 18 — Update VM dynamic selector derivation/dispatch.
- Task 19 — Add exactly-once/evaluation-order/result hostile fixtures.

### Why this is a checkpoint

This is the highest-risk runtime change. Static and dynamic subscript assignment must converge on one structural selector while the RHS remains evaluated last and the private dynamic argument pack remains valid under ordinary positional-before-label rules.

### Entry conditions
- C3 COMPLETE.
- semantic selector formation never includes RHS.
- current indexed-assignment result law recorded as “original RHS”.

### Working set
**Primary:** `phalcom-core/src/compiler/lib/expr.rs`, `heap/pack_builder.rs`, `bytecode.rs`, `vm/dispatch.rs`, `method/mod.rs`.

**Tests:** current indexed assignment and argument-expansion/dynamic-pack tests.

**Secondary:** opcode-name/disassembler tests if a bytecode is added.

**Out of scope:** Family API and singleton references.

### Semantic contract

```text
static SetIndex:
    receiver → index values → rhs → SubscriptSet(index shape)

dynamic SetIndex:
    receiver → build index-only pack in source order → rhs → SubscriptSet(pack shape)
```

No synthetic `put` exists; RHS never joins ordinary pack positionals.

### Preferred backend design

Add a dedicated setter-pack bytecode, naming adapted to local conventions, e.g.:

```rust
InvokeSubscriptSetPack { access: PackAccess }
```

with an explicit stack contract equivalent to:

```text
... receiver, index_pack, rhs
```

This is preferable to a hidden kind-dependent extra operand on generic `InvokePack`. An explicit redesign of `InvokePack` is acceptable only if the value lane remains mechanically separate.

### Risks
- RHS evaluated early or twice;
- index label order altered;
- assignment result becomes setter return;
- static and dynamic selector derivation diverges;
- pack expansion creates a hidden post-label positional.

### Hostile cases
A setter returning an unrelated sentinel; side-effecting receiver/index/labeled/RHS producers; a bracket label literally named `put`; a dynamic computed-label/expansion case.

### Required evidence
Run exact regression, then existing indexed/pack modules, then `--no-run` core integration. Do not start with full core.

### Do not run yet
Full core/workspace suite.

### Escalate immediately if
- private pack representation is externally persisted/exposed;
- preserving assignment RHS requires reintroducing a synthetic label;
- dynamic complete expansion exposes a separate pre-existing ordering bug outside this setter change.

### Completion
- [ ] no synthetic `put` lowering;
- [ ] static/dynamic selector identity agrees;
- [ ] evaluation-order hostile fixture passes;
- [ ] original RHS result preserved;
- [ ] `put` usable as ordinary bracket label;
- [ ] opcode tables updated if needed;
- [ ] state updated.

## Task 16 — Static SetIndex lowering

**Purpose:** keep the fast static path structurally pure.

**Risk:** Semantic MEDIUM; fanout local compiler.

**Owned symbol:** `Compiler::compile_expr_want` branch `Expr::SetIndex`.

**Edit operations:** compile receiver; compile bracket arguments in lexical order; derive selector slots from bracket args only; compile RHS last; emit SubscriptSet with index arity; preserve hidden RHS value needed for assignment-expression result.

**Must not:** include RHS in the selector labels/slot builder.

**Testing:** C4.

## Task 17 — Dynamic SetIndex ABI

**Purpose:** delete `PackReserveStaticLabel("put")` and keep the RHS outside the argument pack.

**Risk:** Semantic HIGH; fanout compiler/bytecode/VM.

**Current:** index pack is followed by a synthetic reserved `put` label filled with RHS.

**Target:** index pack contains only source bracket contributions; RHS is a separate value operand evaluated after pack assembly.

**Edit operations:**
1. Remove `put_idx`/reservation/fill logic.
2. Root receiver and index pack across RHS evaluation.
3. Evaluate RHS exactly once after all index contributions.
4. Emit dedicated subscript-set pack opcode or explicit equivalent.
5. Preserve RHS for expression result without re-evaluation.
6. Update bytecode name/index/exhaustive match tables.

**Testing:** C4 high-risk evidence.

## Task 18 — VM dynamic subscript-set dispatch

**Purpose:** derive selector from index pack only and append RHS only to the activation value window.

**Risk:** Semantic HIGH; fanout VM.

**Owned symbols:** `InvokePack`/new opcode handler; `dynamic_pack_selector`; pack extraction.

**Target flow:**

```text
index pack
→ validate finished builder
→ derive bracket slots from pack positionals + labels
→ Selector::subscript_set(slots)
→ activation values = index values + dedicated rhs
→ dispatch SignatureKind::SubscriptSet(index_slot_count)
```

**Must not:** search for or remove a label named `put`.

## Task 19 — Evaluation-order/result hostile fixture

**Purpose:** defeat the easiest incorrect stack/pack implementations.

**Risk:** Semantic HIGH; fanout tests.

**Fixture requirements:** static positional; static positional+labeled; dynamic computed/expanded shape; bracket label named `put`; setter returns value different from RHS; side-effect log proves receiver → every index contribution → RHS → setter body; assignment result is original RHS.

**Testing:** exact new test first, then nearest existing indexed-assignment and argument-expansion modules.

---

# Checkpoint C5 — Complete Family activation surface

### Tasks
- Task 20 — Extend `FamilyInvocationKind` to all five selector kinds.
- Task 21 — Implement bound Family subscript activation.
- Task 22 — Add `value` / `value=(_)` aliases for named accessors.
- Task 23 — Add `get(Tuple)` / `set(Tuple,value)` subscript APIs.
- Task 24 — Route the public API correctly for `AssociatedFamily` and update native/floor census.

### Why this is a checkpoint

After C5, every captured family can be activated through a surface that reflects selector kind. Ordinary function-style application remains Method-only; `get()` / `set(_)` and `value` / `value=(_)` remain named Getter/Setter gateways; bracket syntax and the Tuple overloads remain SubscriptGet/SubscriptSet gateways. The checkpoint integrates public API, runtime route selection, exact/pattern family validation, and core bootstrap declarations before evidence is meaningful.

### Entry conditions
- C4 COMPLETE.
- runtime dynamic setter dispatch contains no synthetic `put` role.

### Working set

**Primary:**
- `phalcom-core/src/vm/send.rs` — `FamilyInvocationKind`, `activate_family_with_kind`.
- `phalcom-core/src/heap/selector_pattern.rs` — `RuntimeSelectorPattern`, subscript runtime base.
- `phalcom-core/src/primitive/family.rs` — public native Family gateways.
- `phalcom-core/core/universe/src/callable/family.ph` — authored Family class surface.
- existing associated-family activation helpers in `vm/dispatch.rs` / `vm/associated.rs`.

**Secondary:** native metadata/generated registration and floor-census code only as required by adding authored primitives.

**Out of scope:** changing immutable `MethodFamily` reflection snapshots; generalizing arbitrary List values into selector-shape tuples.

### Semantic contract established

```text
family(args...)            → Method
family.get()               → Getter
family.set(rhs)            → Setter
family.value               → Getter
family.value = rhs         → Setter
family[index-shape]        → SubscriptGet
family[index-shape] = rhs  → SubscriptSet
family.get(shapeTuple)     → SubscriptGet
family.set(shapeTuple,rhs) → SubscriptSet
```

Exact family specs must reject activation through the wrong selector kind even when arity happens to coincide. Pattern families must derive candidate selectors only within the requested invocation kind.

### Semantic risks
- exact Getter accidentally callable with `family()`;
- exact Setter accidentally selected by `family(x)`;
- direct subscript family activation routes through ordinary Method `call`;
- Tuple labels reordered during explicit API activation;
- setter RHS accidentally included in Tuple shape;
- `AssociatedFamily` is routed through live bound-family lookup;
- captured base names `get`, `set`, or `value` shadow the API gateway.

### Hostile cases
1. One family base defines Getter, Setter, `name()`, and `name(_)`; each gateway must choose the correct kind.
2. A family base itself named `value`; `family.value` must invoke the Family API gateway, not recursively look up captured `value` as a property on the Family object.
3. Exact subscript getter rejects setter activation and vice versa.
4. Mixed Tuple `(row, column, debug: true)` preserves positional/labeled shape exactly.
5. Associated family uses the same API surface but only its frozen descriptor candidates.

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_family_runtime -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <family_accessor_api_filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <family_subscript_api_filter> -- --nocapture
```

Then run the repository's exact native-binding/floor-census invariant tests touched by new Family primitives. Do not guess the final count; derive it from installed/generated metadata.

### Do not run yet
Full workspace test/clippy.

### Escalate immediately if
- normal method dispatch on `Family` cannot reliably reach its API members before the captured-family routing layer;
- Tuple/product extraction would require stringifying/reparsing selectors;
- `AssociatedFamily` activation can only be made to work by discarding its descriptor restriction.

### Completion
- [ ] five invocation kinds implemented;
- [ ] direct Family brackets work;
- [ ] Tuple explicit APIs preserve shape;
- [ ] `value` aliases equal get/set semantics;
- [ ] AssociatedFamily routes through associated authority;
- [ ] core/native census updated and verified;
- [ ] state updated.

## Task 20 — Extend `FamilyInvocationKind`

**Purpose:** make selector-kind selection explicit for subscript families.

**Risk:** Semantic HIGH; fanout VM exhaustive matches.

**Owned files/symbols:** `phalcom-core/src/vm/send.rs` — `FamilyInvocationKind`, exact and pattern branches of `activate_family_with_kind`.

**Inspect before editing:** every constructor/call of `activate_family_with_kind`; `RuntimeSelectorPattern::matches_call`; `SignatureKind` conversion.

**Target — EXACT unless local drift renamed the enum:**

```rust
pub(crate) enum FamilyInvocationKind {
    Method,
    Getter,
    Setter,
    SubscriptGet,
    SubscriptSet,
}
```

**Edit operations:**
1. Add subscript variants.
2. Extract one exhaustive kind-compatibility helper used by exact-family activation.
3. For patterns, reject named base for subscript activation and subscript base for named Method/Getter/Setter activation before method lookup.
4. Preserve Setter/SubscriptSet value arity as invocation data, not pattern slots.

**Must not:** infer invocation kind solely from argument count.

**Testing classification:** C5 runtime evidence.

## Task 21 — Implement bound Family subscript activation

**Purpose:** remove the current runtime dead end where subscript selector patterns exist structurally but Family activation rejects them.

**Risk:** Semantic HIGH; fanout VM/runtime pattern.

**Owned symbols:**
- `activate_family_with_kind`;
- `RuntimeSelectorBase::Subscript` handling;
- `RuntimeSelectorPattern::matches_call` or equivalent shape matcher;
- compiler family-application detection for direct `family[...]` / `family[...] = rhs` if required.

**Current implementation:** runtime selector-pattern representation already anticipates subscript families, but generic Family activation reports that subscript patterns require index activation.

**Target:** exact and pattern subscript Families perform live lookup on the stored bound receiver using a structural SubscriptGet/SubscriptSet selector. Direct bracket activation and explicit Tuple API must converge on the same internal route selector.

**Edit operations:**
1. Add exact subscript kind checks.
2. Add subscript pattern candidate construction from the incoming index shape.
3. Preserve current access authority / dNU behavior of bound Families.
4. Route setter RHS separately from index shape.

**Testing:** C5.

## Task 22 — Add property-style `value` aliases

**Purpose:** provide natural property invocation on top of explicit getter/setter methods.

**Risk:** Semantic MEDIUM; fanout core Universe + primitive runtime.

**Owned files:** `phalcom-core/core/universe/src/callable/family.ph`, `phalcom-core/src/primitive/family.rs`.

**Target authored surface — adapt annotations only to existing conventions:**

```phalcom
@native
value -> Dynamic

@native
value=(_ newValue: Dynamic) -> Dynamic
```

**Target runtime law:** `family.value` and `family.get()` call the same internal Getter activation helper; `family.value = rhs` and `family.set(rhs)` call the same internal Setter helper. Do not implement one alias by sending the other public selector—avoid avoidable recursion/dispatch overhead and keep error provenance consistent.

**Hostile case:** captured family base is itself `value`.

**Testing:** C5.

## Task 23 — Add explicit Tuple subscript APIs

**Purpose:** expose programmatic subscript-family invocation without requiring dynamically generated bracket syntax.

**Risk:** Semantic HIGH; fanout Family primitive + product/Tuple representation.

**Target authored surface:**

```phalcom
@native
get(_ shape: Tuple) -> Dynamic

@native
set(_ shape: Tuple, _ value: Dynamic) -> Dynamic
```

**Conceptual law:**

```phalcom
get(_ shape: Tuple) { subscript[***shape] }
set(_ shape: Tuple, _ value) { subscript[***shape] = value }
```

**Implementation boundary:** use existing Tuple/product layout helpers directly. The first argument supplies the complete subscript positional/labeled shape. For setter, the second argument is the dedicated RHS and must never be folded into that Tuple.

**Edit operations:**
1. Validate the first argument as Tuple/product using existing runtime type helpers.
2. Extract positionals and labels without converting to text.
3. Build/borrow an `ArgumentView` or equivalent structural call view.
4. Activate `SubscriptGet` / `SubscriptSet` through the same internal family selector as direct bracket syntax.
5. Preserve label order exactly.

**Must not:** accept List as shape unless a separate language decision adds it.

**Testing:** C5 high-risk API fixture.

## Task 24 — Route `AssociatedFamily` through the common public Family API

**Purpose:** make the public `Family` contract truthful for both heap representations while preserving separate semantic authorities.

**Risk:** Semantic HIGH; fanout runtime + bootstrap/native metadata.

**Current gap:** `primitive/family.rs` recognizes both `Object::Family` and `Object::AssociatedFamily` as Family-class values, but `activate_family_with_kind` currently extracts the ordinary bound Family path.

**Target flow:**

```text
public Family gateway
    ├─ Object::Family
    │    → bound receiver + FamilySpec
    │    → live ordinary dispatch
    │
    └─ Object::AssociatedFamily
         → ExecutableFamilyDescriptor
         → descriptor-restricted associated target selection
```

**Edit operations:**
1. Split `activate_family_with_kind` into a representation dispatcher plus bound/associated internal helpers, or add the equivalent explicit dispatch at the primitive gateway.
2. Reuse existing associated family target execution; do not duplicate variant/behavioral execution code.
3. Install new core native members `value`, `value=(_)`, `get(_)`, `set(_,_)`.
4. Run/regenerate native metadata tables as repository workflow requires.
5. Recompute/update floor census and binding-count assertions from actual installation.

**Testing:** C5.

---

# Checkpoint C6 — Exact singleton associated references

### Tasks
- Task 25 — Project an exact singleton reference as a one-entry associated family.
- Task 26 — Activate singleton targets through Getter gateways.
- Task 27 — Lock singleton identity and payload-constructor non-regression.

### Why this is a checkpoint

Singleton variant references cross semantic associated resolution, immutable lowering descriptors, ADT registry singleton storage, and Family activation. Isolating the feature prevents a syntactically small change from accidentally changing allocation or constructor/value identity semantics.

### Entry conditions
- C5 COMPLETE.
- `AssociatedFamily` supports Getter activation via `get()` / `value`.

### Working set
**Primary:** `phalcom-semantic/src/checker/expression.rs`, `phalcom-core/src/modules/semantic_lowering.rs`, `compiler/lib/associated.rs`, associated-family VM dispatch, ADT registry singleton path.

**Tests:** semantic associated/family tests; `associated_lowering.rs`; `associated_reification.rs`; `native_adt_runtime.rs`; general enum runtime tests.

**Out of scope:** changing physical singleton `Value` representation; interning the Family capability itself; new VariantConstructor heap class.

### Semantic contract

```text
Option::None
    → direct canonical singleton Value

&Option::None
    → Family-class associated capability
       operation = exact Getter
       target = Singleton(VariantId)

ref.get() / ref.value
    → canonical singleton Value
```

For a general enum singleton, repeated activation returns the same canonical singleton immediate/descriptor value. For native `Option::None`, activation returns the existing dedicated None representation.

### Risks
- eager singleton load during reference construction;
- singleton represented as Method closure;
- `ConstructVariant` allocates a case object for a singleton;
- generic outer parameters become specialized incorrectly;
- payload constructor references regress.

### Hostile cases
- direct `Option::None` identity equals repeated `(&Option::None).get()`/`.value` results;
- same for a non-native enum singleton;
- `(&Option::None)()` fails Method-kind activation rather than acting as getter;
- `&Option::Some(_)` still yields a callable constructor reference.

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic <singleton_reference_filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <associated_lowering_filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <associated_reification_filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test native_adt_runtime
```

### Do not run yet
Full workspace; C7 tooling remains stale.

### Escalate immediately if
- associated family descriptors cannot represent a singleton value member under Getter operation;
- calling such a member requires treating it as Method-shaped;
- canonical singleton lookup is unavailable from the associated-family target executor.

### Completion
- [ ] singleton callable reference projects successfully;
- [ ] lowering emits associated family capability, not closure/value load;
- [ ] get/value return canonical identity;
- [ ] payload constructor references unchanged;
- [ ] state updated.

## Task 25 — Project singleton reference as one-entry associated-family descriptor

**Purpose:** reuse the repository's existing `ExecutableFamilyTarget::Singleton` instead of adding a new runtime object.

**Risk:** Semantic HIGH; fanout semantic-to-lowering bridge.

**Owned symbols:** `project_callable_reference_resolution`, `project_associated_resolution`, the callable-reference semantic product.

**Current issue:** direct singleton lookup projects as `AssociatedLoweringSpec::SingletonLoad`, which is not a valid callable-reference lowering product. `project_callable_reference_resolution` currently only accepts resolved bound method, variant constructor thunk, or associated family.

**Target:** callable-reference semantic resolution for singleton produces or is projected into a one-entry `ExecutableFamilyDescriptor`:

```rust
ExecutableFamilyEntry {
    operation: /* Getter, zero selector slots */,
    member_kind: FamilyMemberTypeKind::Value,
    target: ExecutableFamilyTarget::Singleton { variant },
}
```

Use the actual `FamilyOperationShape` getter constructor/API available at implementation time. Then reuse `CallableReferenceLoweringSpec::MakeAssociatedFamily`.

**Must not:** add `MakeSingletonClosureThunk` unless repository evidence proves the existing singleton family target cannot satisfy Getter activation.

**Testing:** C6.

## Task 26 — Execute singleton target through Getter API

**Purpose:** return the canonical ADT singleton when a one-entry associated family is activated as Getter.

**Risk:** Semantic HIGH; fanout associated-family VM.

**Target:** descriptor candidate selection matches exact Getter operation and calls the existing singleton-load/registry path. There is no fresh `AdtCaseObject` allocation and no `ConstructVariant` instruction for the singleton activation.

**Testing:** C6 runtime identity tests.

## Task 27 — Singleton/payload constructor regression matrix

**Purpose:** prove the final distinction rather than only successful execution.

**Risk:** Semantic HIGH; fanout tests.

**Required assertions:**

```phalcom
const none = &Option::None
const a = none.get()
const b = none.value
const c = none.get()

a === Option::None
b === Option::None
c === Option::None
```

Also test a user/general enum singleton and preserve:

```phalcom
const some = &Option::Some(_)
some(42)
```

---

# Checkpoint C7 — Reflection and editor consistency

### Tasks
- Task 28 — Migrate `Selector` reflection.
- Task 29 — Remove LSP string-based setter authority and update completion/hover/tokens.
- Task 30 — Add cross-consumer source-index/LSP reference consistency evidence.

### Why this is a checkpoint

After runtime semantics stabilize, user-facing introspection and editor presentation must expose the same canonical structural identities. This is the seam where stale `=(put)` text and P1's bare-family rule are most likely to survive without breaking execution tests.

### Entry conditions
- C6 COMPLETE.

### Working set
**Primary:** `phalcom-core/src/primitive/selector.rs`, `phalcom-lsp/src/selectors.rs`, `completion.rs`, `hover.rs`, `semantic_tokens.rs`, semantic source-index reference consumers.

**Out of scope:** new unrelated LSP capabilities; changing `Selector` object identity representation.

### Semantic contract
- reflection parses/renders `=(_)`;
- setter `Selector.slots` excludes RHS;
- LSP formatting/completion branches on structural selector kind where available;
- new operator/subscript/accessor references receive correct source indexing/tokens;
- whole-family and Method-pattern displays are distinct.

### Risks
- `Selector::kind` still prints old `put` spelling;
- LSP switches to `strip_suffix("=(_)")` rather than structural kind;
- source index loses operator/bracket ranges after AST refactor;
- pattern values are forced into exact `Selector` reflection.

### Hostile cases
selector base contains `=`; subscript setter has ordinary label `put`; operator selector; `property=` kind-set has no exact selector; exact and whole-family reference hovers differ.

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <selector_reflection_filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test integration <selector_completion_filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test integration semantic_tokens_current_syntax::operator_symbols_and_named_family_references_follow_parser_syntax -- --nocapture
```

Rename/broaden the existing semantic-token test if its old name no longer reflects coverage.

### Do not run yet
Workspace-wide gates; docs/fixtures still need migration.

### Escalate immediately if
- LSP receives only encoded selector strings at a layer where no structural selector can be threaded without architectural expansion;
- changing AST reference targets breaks semantic target identity rather than only occurrence spans.

### Completion
- [ ] reflection round-trips new exact setter syntax;
- [ ] reflection slots retain dedicated RHS invariant;
- [ ] no LSP production suffix parser for setter identity;
- [ ] reference source ranges/tokens correct;
- [ ] targeted LSP integration green;
- [ ] state updated.

## Task 28 — Migrate Selector reflection

**Purpose:** align first-class selector objects with current canonical syntax.

**Risk:** Semantic MEDIUM; fanout local runtime reflection.

**Owned symbols:** `construct_selector`, `selector_kind`, `selector_slots`, `selector_to_string`.

**Changes:** `Selector.from` accepts canonical `=(_)`; `toString`/symbol round-trip emits `=(_)`; setter/subscript-setter kind display no longer contains `(put)`; `selector_slots` continues to return only actual selector slots, never setter RHS.

**Testing:** C7 reflection tests.

## Task 29 — LSP structural selector migration

**Purpose:** prevent editor semantics from becoming a second punctuation parser.

**Risk:** Semantic MEDIUM; fanout LSP.

**Current issues:** `selectors.rs` has a textual fallback for `=(put)`; `completion.rs` strips the old suffix to produce assignment snippets.

**Target:** use `Selector.kind` / structural AST selector products. Setter completion is selected because kind is Setter, not because encoded text happens to end in `=(_)`. Canonical selector text is used only for display.

**Edit operations:** update selector formatting, completion insertion text, hover/signature rendering, semantic token traversal for generalized reference targets, tests.

**Testing:** C7.

## Task 30 — Cross-consumer reference consistency

**Purpose:** prove AST → semantic → source index → LSP agreement after reference-target generalization.

**Risk:** Semantic MEDIUM; fanout semantic/LSP tests.

**Required invariant:**

```text
source reference selector/range
    == normalized semantic selector or pattern
    == source-index target/range
    == LSP displayed selector/reference form
```

Prefer one integration matrix containing named Getter/Setter/family, operator, and subscript reference over duplicating the same semantic fact in many tests.


---

# Checkpoint C8 — Fixtures, examples, normative docs, governance, and deletion migration

### Tasks
- Task 31 — Update language fixtures, corpus, and examples.
- Task 32 — Rewrite current selector/family/ADT documentation.
- Task 33 — Amend ADR-0060 and PDR-0032 governance for the setter-lane change.
- Task 34 — Execute deletion gates and close the implementation state.

### Why this is a checkpoint

The migration is incomplete if execution works but the current language corpus, selector documentation, accepted index-selector decision, or LSP-facing examples still teach the old model. `=(put)` was explicitly ratified historically, and P1 explicitly documented bare `&name` as the whole family while excluding exact getter/operator/subscript references. C8 makes the new model the sole **current** authority while retaining historical documents as historical evidence where repository conventions require it.

### Entry conditions
- C7 COMPLETE.
- all code paths use structural `=(_)` semantics.

### Working set

**Primary normative/current:**
- `docs/spec/current/selectors.md`
- `docs/spec/callables/family.md`
- `docs/spec/adts.md`
- `docs/spec/current/lexical-structure.md`
- `docs/spec/primitives/Selectors I.md`
- `docs/spec/primitives/Selectors II.md`
- `docs/spec/current/core/core-classes.md`
- `docs/spec/current/core/floor-census.md`
- `docs/adr/accepted/0060-index-operator-as-real-selector.md`
- `docs/pdr/0032-transition-1-language-surface-convergence.md`
- LANG004 C1 checkpoint/plan/state files.

**Fixtures/examples:**
- `phalcom-ast/tests/family_selector_syntax.rs`
- core language corpus family/index/ADT fixtures
- associated reification/lowering fixtures
- public examples found by exact migration searches.

**Historical/generated — inspect before editing:** dated `docs/implementation/**`, `docs/superpowers/**`, `docs/wiki/raw/**`.

### Semantic contract
Current documentation and executable examples teach the same selector algebra as the implementation. Historical syntax may remain only in explicitly historical/as-built material. No old production/current-spec authority can silently run.

### Risks
- blind textual replacement changes examples that intentionally meant a whole family into exact Getter references;
- accepted ADR is silently contradicted instead of amended;
- generated docs are hand-edited instead of regenerated;
- old P1 statements remain in current specs.

### Hostile cases
- every old bare `&x.name` occurrence must be classified by intent before migration;
- every `=(put)` production/current-spec hit must either disappear or be an explicit historical quotation;
- docs must distinguish `name...` from `name(...)` and `[...]=` from `[...]=(_)`.

### Required evidence

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <language_corpus_filter> -- --nocapture
```

Then run the negative/deletion searches in Task 34. If the repository has a docs/link checker or generated-census verification script, run its focused target here.

### Do not run yet
The full workspace delivery gate belongs after C8 is COMPLETE.

### Escalate immediately if
- governance convention forbids amending accepted ADR text in place and requires a new superseding ADR/PDR;
- a generated source-of-truth file is discovered where this plan assumes a derived document;
- old setter spelling is part of a serialized compatibility format rather than merely source syntax.

### Completion
- [ ] executable fixtures/examples migrated;
- [ ] current specs migrated;
- [ ] ADR/PDR amendment explicit;
- [ ] floor/native references reflect actual generated census;
- [ ] old production spelling absent;
- [ ] stale P1 current prose absent;
- [ ] C0–C8 state ledger complete;
- [ ] no active INCIDENT.

## Task 31 — Update fixtures, corpus, and examples

**Purpose:** migrate executable examples deliberately and add a compact public end-to-end demonstration.

**Risk:** Semantic LOW for mechanical spelling changes, MEDIUM where bare-reference intent changes; fanout test corpus.

**Owned files:** current AST family-selector tests, family runtime fixtures, indexed-assignment fixtures, ADT associated-reference tests, examples returned by exact searches.

**Current implementation:** P1 tests intentionally treat bare `&object.method` as whole family; selector literal tests include `#name=(put)`; setter declarations use `put` binder marker.

**Target implementation:**
- declaration `=(put value)` → `=(_ value)`;
- exact selector literal `#name=(put)` → `#name=(_)`;
- old **whole-family** `&obj.name` → `&obj.name...`;
- new bare `&obj.name` tests assert exact Getter;
- add `&obj.name=`, `&obj.name=(_)`, operator refs, subscript refs;
- add `family.value`, setter, direct brackets, Tuple API, and singleton ref examples.

**Must not:** perform an unreviewed global `&x.name` → `&x.name...` replacement. Determine whether each old site intended exact getter, exact associated behavior, or whole family.

**Testing classification:** validated by C8 language corpus and already-owned checkpoint tests.

## Task 32 — Rewrite current selector/family/ADT specifications

**Purpose:** document the conceptual model, not only a syntax diff.

**Risk:** Semantic MEDIUM; fanout documentation.

**Required concepts:**
1. selector kind is part of identity;
2. setter RHS is one dedicated value lane outside selector slots;
3. normal positional-before-label law remains unchanged;
4. subscript evaluation order is left-to-right with RHS last;
5. exact / kind-set / slot-pattern / whole-family distinction;
6. `&` “reference the written shape” law;
7. exact Getter uses `.get()` / `.value`, not ordinary `()`;
8. Family named and subscript API table;
9. associated singleton reference behavior;
10. receiver-prefix evaluated once;
11. `name(...)` versus `name...` and `[...]=(_)` versus `[...]=` distinctions.

Include a normative table equivalent to:

| Reference | Selection |
|---|---|
| `&x.name` | exact Getter |
| `&x.name=(_)` | exact Setter |
| `&x.name=` | Getter \| Setter |
| `&x.name(...)` | Method pattern |
| `&x.name...` | complete named family |
| `&x[...]` | SubscriptGet pattern |
| `&x[...]=(_)` | SubscriptSet pattern |
| `&x[...]=` | all subscript accessors |

**Testing classification:** docs review + C8 deletion gates.

## Task 33 — Amend selector governance

**Purpose:** explicitly supersede the accepted `put` decision rather than allowing implementation/spec divergence.

**Risk:** Semantic MEDIUM; fanout ADR/PDR/current spec.

**Owned documents:** ADR-0060 sections defining `[...]=(put)`; PDR-0032 fixed setter-role text.

**Implementation boundary:** preserve historical rationale, then append/date an amendment or create a new superseding decision according to repository governance convention. The amendment must say that the value remains a fixed semantic role but its selector notation becomes the distinguished positional `(_)`, not an external label `put`.

**Required amendment law:**

```text
old:  [index-shape]=(put)
new:  [index-shape]=(_)

The new `_` is not appended to index selector slots.
It denotes the unique setter value lane.
```

Also amend named setters from `name=(put)` to `name=(_)`.

**Testing classification:** no executable test; negative/current-doc gates.

## Task 34 — Deletion gates and implementation-state closure

**Purpose:** prove replacement exclusivity and make the plan resumable/reviewable at completion.

**Risk:** Semantic LOW; fanout repository audit.

**Required production/current searches:**

```bash
# Expected: zero selector-semantic hits in current production/core surface.
rg -nF '=(put)' \
  phalcom-common/src phalcom-ast/src phalcom-semantic/src \
  phalcom-core/src phalcom-core/core phalcom-lsp/src

# Expected: zero old setter-role plumbing hits. Inspect every result; unrelated
# uses of the English word "put" are allowed.
rg -n 'put_idx|put_label|IndexAccessor::Set \{ put|setter parameter must start with.*put' \
  phalcom-common/src phalcom-ast/src phalcom-semantic/src phalcom-core/src phalcom-lsp/src

# Expected: zero old LSP suffix authority.
rg -n 'strip_suffix\("=\(put\)"\)' phalcom-lsp/src phalcom-core/src

# Expected: zero stale current-spec P1 claims.
rg -n 'bare named form captures the whole named family|reference syntax does not add.*exact.*getter' \
  docs/spec/current docs/spec/callables

# Inspect any remaining punctuation-authority heuristics.
rg -n 'ends_with\(.*=\(|strip_suffix\(.*=\(|contains\(.*\.\.\.' \
  phalcom-core/src phalcom-lsp/src phalcom-semantic/src
```

Historical documents may retain `=(put)` only when clearly dated/as-built/historical and excluded from current authority. List retained categories in the state file; no unexplained occurrence is acceptable.

**State closure:** update checkpoint table to COMPLETE/PENDING Final Gate, record all evidence commands/results, negative searches, changed symbols, unexpected findings, and remaining broad gates.

---

# 14. Patch-grade test design matrix

Tests are scheduled by semantic risk, not by task count. Existing ownership-layer suites should be extended rather than duplicated.

| Risk | Ownership layer | Minimum evidence |
|---|---|---|
| setter encoding becomes unary Method | `phalcom-common` | exact round-trip + kind assertion |
| setter RHS leaks into selector slots | common + reflection | setter slots empty/index-only |
| `property=` matches Method | common/semantic | mixed-base family member-kind assertion |
| `property(...)` matches Getter | semantic | same-base hostile class |
| `property...` misses Getter/Setter | semantic | same-base hostile class |
| operator confused with setter | common/parser | `==(_)`, `+(_)`, invalid setter projection |
| illegal setter value arity accepted | parser | negative declaration/reference matrix |
| subscript RHS evaluated before a label | compiler/runtime | side-effect log |
| dynamic pack still smuggles `put` | compiler/runtime | legal `[put: key] = rhs` + negative search |
| assignment result changes | runtime | setter returns sentinel, assignment `=== rhs` |
| `family()` invokes Getter | runtime | same base Getter + nullary Method |
| `family(x)` invokes Setter | runtime | same base Setter + unary Method |
| Tuple subscript labels reorder | runtime | `(row, column, debug: true)` shape test |
| AssociatedFamily routed as live Family | runtime | frozen descriptor/candidate hostile case |
| singleton ref eager-loads value | semantic/lowering | reference product is Family, not ExactValue load |
| singleton ref allocates a case | runtime | identity equals direct singleton across activations |
| LSP owns punctuation semantics | LSP | structural completion test + negative source search |

---

# 15. Recommended end-to-end language fixture

Add one readable fixture under the existing family/index language-corpus organization rather than inventing another harness. Use the repository's actual assertion/output convention; the following is semantic pseudocode where necessary:

```phalcom
class Probe {
  @constructor
  new() { _value = 10 }

  value { _value }

  value=(_ next) {
    _value = next
    #setterReturn
  }

  value() { #nullary }
  value(_ x) { x }

  [_ key] { key }

  [_ row, debug flag]=(_ next) {
    #subscriptSetterReturn
  }
}

const p = Probe.new()

const getter = &p.value
Assert.equal(getter.get(), 10)
Assert.equal(getter.value, 10)

const setter = &p.value=(_)
setter.set(20)
setter.value = 30

const accessors = &p.value=
Assert.equal(accessors.value, 30)
accessors.value = 40

const all = &p.value...
Assert.equal(all.get(), 40)
Assert.equal(all(), #nullary)
Assert.equal(all(7), 7)

const indexes = &p[...]=
Assert.equal(indexes[3], 3)
indexes[1, debug: true] = 99
Assert.equal(indexes.get((3,)), 3)
indexes.set((1, debug: true), 99)

const none = &Option::None
Assert.truth(none.get() === Option::None)
Assert.truth(none.value === Option::None)

const some = &Option::Some(_)
Assert.equal(some(42), Option::Some(42))
```

The fixture should also contain a side-effect trace for setter evaluation order rather than relying only on result values.

---

# 16. Verification scheduling and failure protocol

## 16.1 Smallest-first order

For each checkpoint:

```text
exact regression
    ↓
focused existing semantic/runtime module
    ↓
affected crate compile/test
    ↓
dependent integration layer
    ↓
workspace at Final Gate only
```

Do not rerun a passing broad suite after every mechanical task.

## 16.2 What compile checks prove

`cargo check -p <crate>` is useful immediately after enum/API fanout changes because it proves exhaustive caller migration and Rust-level API consistency. It does **not** prove selector semantics.

Behavioral evidence must come from the checkpoint's focused tests.

## 16.3 Incident protocol

If required checkpoint evidence fails, set that checkpoint to `INCIDENT`; dependent checkpoints do not proceed. Record:

1. exact command and failing test;
2. important error/assertion output;
3. direct fixture → parser/semantic/compiler/runtime path;
4. one nearby passing comparator;
5. classification: `PRODUCT`, `FIXTURE`, `DEPENDENCY/PUBLICATION`, `BACKEND/HARNESS`, `BASELINE`, or `PLAN DRIFT`;
6. narrow allowed repair boundary;
7. tempting broad repairs that remain forbidden.

Example:

```text
C4 — INCIDENT

Failure:
    dynamic labeled SetIndex evaluates RHS before computed label.

Path:
    fixture
    → Expr::SetIndex
    → dynamic pack compiler branch
    → setter-pack opcode
    → VM handler

Comparator:
    static labeled SetIndex preserves expected order.

Classification:
    PRODUCT

Allowed repair:
    compiler/lib/expr.rs dynamic SetIndex scratch/stack sequence.

Do not:
    relax pack ordering;
    change parser source order;
    move label evaluation into VM;
    change assignment result semantics.
```

---

# 17. Repository drift protocol

Before every checkpoint:

1. verify primary files and symbols still exist;
2. inspect effects of earlier checkpoint diffs on the same APIs;
3. search for new exhaustive consumers when enums changed;
4. adapt local mechanics to repository drift;
5. do not silently alter §3 semantic decisions.

Full repository re-investigation is unnecessary unless a required symbol disappears, a new authority replaces the planned one, or evidence contradicts the plan.

---

# 18. Implementation-state protocol

After every checkpoint, record concise facts rather than scratch reasoning:

```md
## Established invariants
- I-01: ...

## Decisions
- D-01: ...

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative/deletion evidence
- ...

## Deferred gates
- command → destination

## Unexpected findings
- ...

## Active incident
None.

## Next resume action
Begin C<N+1>, Task <N>.
```

At checkpoint completion, produce a short supervisor report:

```text
Checkpoint C<N> COMPLETE

Established:
    <dominant semantic claim>

Changed:
    <file> — <symbol/responsibility>

Evidence:
    <command> — PASS

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — <result>

Deferred:
    <gate> → <destination>

Unexpected findings:
    none | <fact>

Next:
    C<N+1> — <boundary>
```

---

# 19. Final broad delivery gates

After C8 is COMPLETE, run broad delivery evidence once, in this order. Adapt toolchain wrappers only to repository convention; never mark a gate successful without observing it.

```bash
cargo fmt --all -- --check
```

**Proves:** formatting compatibility only.

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
```

**Proves:** cross-crate API/exhaustive-match/build integration.

Run the full affected ownership suites:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-common
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test integration
```

**Proves:** all affected package-level semantics remain integrated after every migration.

Then:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
```

**Proves:** workspace compatibility. P1 historically recorded unrelated `phalcom-repl` failures; if they still reproduce unchanged, classify them as baseline evidence rather than attributing them to P2 or claiming the gate green.

Finally:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

**Proves:** lint delivery readiness. P1 historically recorded unrelated `phalcom-modules` warnings; if they remain, classify and record them as baseline/release blockers or an explicit waiver. Do not silently suppress them in P2 code.

If `scripts/verify.sh --full` remains the repository's canonical full verification gate and contains additional generated/census/docs checks, run it once here after inspecting what it covers.

---

# 20. Final negative/deletion gates

Run these after all code/docs migration:

```bash
rg -nF '=(put)' \
  phalcom-common/src phalcom-ast/src phalcom-semantic/src \
  phalcom-core/src phalcom-core/core phalcom-lsp/src
```

Expected: zero selector-semantic hits.

```bash
rg -n 'put_idx|put_label|IndexAccessor::Set \{ put|setter parameter must start with.*put' \
  phalcom-common/src phalcom-ast/src phalcom-semantic/src phalcom-core/src phalcom-lsp/src
```

Expected: zero old setter-role plumbing hits.

```bash
rg -n 'strip_suffix\("=\(put\)"\)' phalcom-lsp/src phalcom-core/src
```

Expected: zero.

```bash
rg -n 'bare named form captures the whole named family|reference syntax does not add.*exact.*getter' \
  docs/spec/current docs/spec/callables
```

Expected: zero current-spec hits.

```bash
rg -n 'ends_with\(.*=\(|strip_suffix\(.*=\(|contains\(.*\.\.\.' \
  phalcom-core/src phalcom-lsp/src phalcom-semantic/src
```

Expected: no unexplained selector-authority heuristics. Some unrelated punctuation checks may remain; inspect and justify each relevant hit.

Historical/as-built documents may retain old spelling only if explicitly classified as historical. The final state file must list intentionally retained categories.

---

# 21. Deferred-evidence audit

Before delivery, enforce:

```text
No deferred test/check remains without one of:
- executed successfully;
- explicitly removed from scope with justification;
- recorded as a known baseline/release blocker.
```

No checkpoint may remain `INCIDENT`.

---

# 22. Suggested commit groups

| Group | Suggested commit | Scope |
|---|---|---|
| C1 | `feat(selectors): canonicalize setter value lanes` | common selector algebra + method bridge |
| C2 | `feat(ast): complete selector-shaped callable references` | setter grammar + AST/reference syntax |
| C3 | `feat(semantic): resolve exact accessor and subscript families` | semantic specs/types/reference resolution |
| C4 | `fix(runtime): separate subscript setter rhs from argument packs` | compiler/bytecode/VM SetIndex |
| C5 | `feat(callables): complete Family accessor and subscript activation` | Family runtime/public API |
| C6 | `feat(adts): reify singleton constructor getter references` | associated singleton semantic/lowering/runtime |
| C7 | `feat(tooling): align selector reflection and lsp syntax` | reflection/LSP/source index |
| C8 | `docs(lang): ratify selector and family convergence` | fixtures/examples/spec/ADR/PDR/state |

Do not force one commit per mechanical task. Keep commits semantically coherent and bisectable.

---

# 23. Known scope exclusions

This plan does **not**:

- redesign ordinary message-send positional/labeled ordering;
- introduce default arguments;
- change immutable `MethodFamily` snapshot semantics;
- change physical singleton `Value` representation;
- introduce a separate `VariantConstructor` heap class;
- change associated namespace `::` semantics;
- change class-side dispatch from ordinary `.`;
- redesign generic getter/setter type theory beyond using existing generic machinery correctly;
- expose arbitrary selector-kind set syntax;
- accept Lists as subscript-shape tuples;
- alter indexed-assignment RHS-result semantics;
- resolve unrelated `phalcom-repl` / `phalcom-modules` baseline failures;
- rewrite historical implementation plans as though the old syntax never existed.

---

# 24. Checkpoint evidence summary template

Populate during implementation; no checkpoint is pre-complete.

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | revision/migration baseline | local SHA/status + inventories | PENDING |
| C1 | canonical selector identity | common tests + core check | PENDING |
| C2 | complete source grammar/AST | AST integration | PENDING |
| C3 | semantic family/reference model | focused semantic suites | PENDING |
| C4 | setter ABI/evaluation order | indexed/pack runtime tests | PENDING |
| C5 | complete Family activation | family runtime/API tests | PENDING |
| C6 | singleton exact reference | associated/ADT tests | PENDING |
| C7 | reflection/LSP consistency | reflection + LSP integration | PENDING |
| C8 | fixtures/docs/deletion | corpus + negative gates | PENDING |
| Final | delivery readiness | affected full suites + workspace gates | PENDING |

---

# 25. Release-complete criteria

The implementation is complete only when:

- [ ] C0 through C8 are `COMPLETE`.
- [ ] `=(_)` is the only current canonical setter spelling.
- [ ] named Setter `Selector.slots` is empty; subscript setter slots contain only bracket shape.
- [ ] `property=` means Getter|Setter and `property...` means Getter|Setter|Method.
- [ ] `[...]`, `[...]=(_)`, and `[...]=` have the ratified kind scopes.
- [ ] operator references use the same family algebra without accidental setter interpretation.
- [ ] `&receiver.name` is exact Getter and `&receiver.name...` is whole family.
- [ ] `&Option::None` is a getter-shaped Family capability and returns the canonical singleton through `get()` / `value`.
- [ ] exact getter Family activation remains `get()` / `value`, never fallback `()`.
- [ ] `family.value` / assignment aliases are equivalent to explicit get/set.
- [ ] direct Family bracket activation works.
- [ ] `get(Tuple)` / `set(Tuple,rhs)` preserve tuple labels/order and separate RHS.
- [ ] dynamic SetIndex contains no synthetic `put` label.
- [ ] subscript setter RHS evaluates after all bracket arguments and exactly once.
- [ ] indexed assignment still returns the original RHS regardless of setter return.
- [ ] reflection, semantic products, lowering, runtime, and LSP agree on selector identity.
- [ ] current fixtures/examples/specs are migrated.
- [ ] ADR-0060/PDR-0032 old setter-role clauses are explicitly amended/superseded.
- [ ] final negative searches have no unexplained production/current-spec hits.
- [ ] all deferred gates are executed, explicitly classified as baseline, or recorded as release blockers.
- [ ] final state file contains no unresolved INCIDENT.
- [ ] format/check/full affected package suites pass.
- [ ] workspace test/clippy outcomes are recorded accurately.

---

# 26. Final supervisor-facing report template

```text
Implementation Program LANG004.C1.P2 COMPLETE

Base revision:
    <sha>

Established:
    structural setter value lane; complete selector-shaped references;
    five-kind Family activation; singleton getter references; tooling/docs convergence.

Checkpoint evidence:
    C0 ...
    C1 ...
    ...

Negative gates:
    old `=(put)` production search — zero relevant hits
    synthetic `put` plumbing search — zero relevant hits
    stale bare-family current-spec search — zero relevant hits

Broad gates:
    fmt — ...
    workspace check — ...
    affected package suites — ...
    workspace tests — ...
    workspace clippy — ...

Known baseline blockers/waivers:
    none | <explicitly classified list>

Unexpected findings:
    none | <facts>

Active incident:
    None.

Next:
    review scoped diff; commit/push only when explicitly authorized by execution workflow.
```
