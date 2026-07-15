# `@native` — a `.ph` source anchor for a Rust primitive

- Status: **Specified — not built.** No `native` row exists in
  `AttributeRegistry::new`; `@native` today raises `attr.unknown`.
- Date: 2026-07-16
- Evidence: none in `phalcom-core/src` — this file specifies work, it does not
  record it. The mechanism it mirrors is real: `InvariantExpander`
  (`compiler/attributes.rs` L456-478) is a registered no-op, and `@invariant`'s
  actual effect lives in `expand_class_attributes` (L1548), not in its expander.
- Depends on: [README.md](README.md) (the tier model) ·
  [annotations-core.md](../experimental/annotations-core.md) (the `@` mechanism,
  registry, phase pipeline)
- Related: [ignore.md](ignore.md) (the sanctioned drop; `@native` borrows its
  mechanism provisionally) ·
  [core/floor-census.md](../core/floor-census.md) (the native binding census this
  attribute's invariant test cross-checks) ·
  [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)
  (the primitive floor)

## What it is

`@native` marks a `.ph` member whose **real implementation is a Rust primitive**.
The `.ph` body is source that exists *only* so tooling has somewhere to land — LSP
go-to-definition, documentation, and a reader asking "what does `List#toString`
actually do?". The compiler **drops the member**; the native binding installed at
bootstrap stays the live one.

```phalcom
class List {
  // Never runs. `list_to_string` (primitive/list.rs) is the live binding.
  // This body exists so `go-to-definition` on `.toString` lands somewhere
  // readable instead of nowhere.
  @native toString => "[" + self.joined(", ") + "]"
}
```

The motivating case: every core type should be able to *show its source*, even
when that source is Rust. Today a user who jumps to `Number#toString` finds
nothing at all.

## What it is **not**

**It is not a protection mechanism.** Verified 2026-07-15 against a running
interpreter: `class List { toString { "x" } }` successfully hijacks the native
binding, and `"\(…)"` picks the override up. Reopening a core class **works**.
`@native` never guards a binding, and must never be motivated as if it did — it is
purely ergonomic, for humans and tooling.

**It is not a binding directive.** `@native` does not tell the compiler *which*
Rust function to install, and does not cause anything to be installed. The native
binding is installed by `universe/primitives.rs` at bootstrap, entirely
independently, exactly as it is today. `@native` only says "do not compile this
body, and assert a native exists".

## Semantics

1. The member is **parsed and validated** like any other. A syntax error in a
   `@native` body is a compile error. This is deliberate: an anchor that does not
   parse is not indexable, so the parse is what makes the anchor real rather than
   a comment in disguise.
2. Attribute **legality is still checked** (`attr.illegal_target`).
3. The member is then **dropped** from `ClassDef::members` before compilation. No
   bytecode is emitted; no method is installed; the method table is untouched.
4. The class's live behaviour is therefore whatever bootstrap installed.

**The dropping is provisional.** It is a borrow of [`@ignore`](ignore.md)'s
mechanism, standing in until `@native`'s own mechanics are decided (see N-2).
`@ignore` is the attribute whose *meaning* is "the compiler ignores this";
`@native` currently has the same effect for want of a better one.

## Legality

| | |
|---|---|
| Legal targets | `Method`, `Getter`, `Setter`, `Construct` |
| Illegal targets | `Class`, `Field` (`attr.illegal_target`) |
| Arguments | none — `@native`, never `@native(...)` |

**`Getter` is load-bearing, not an afterthought.** `toString` is a
`SignatureKind::Getter`, so the motivating case targets `Target::Getter`. A
`legal_targets()` of `[Target::Method]` alone would reject the exact member this
attribute exists for. (`toString` and `toString()` are *different selectors* — see
ADR-0022's CB-1 amendment.)

A `@native` member whose class+selector has **no** installed native binding is
**not** a compile error. The compiler has no view of the binding table, which is
built at bootstrap. This is caught by the invariant test below, not by the
compiler — and that is the correct division, not a gap.

## Implementation

### 1. Registration

`compiler/attributes.rs`, `AttributeRegistry::new` (the registry block currently
holding ten rows):

```rust
expanders.insert("native".to_string(), Box::new(NativeExpander));
```

Registration is **required even though the expander does nothing**. The registry
is also the legality gate: an unregistered name raises `attr.unknown` unless it
resolves to a user `Attribute` subclass (M-ATTR-ROOT's
`resolves_to_attribute_class`). `InvariantExpander`'s own doc states this
precedent outright — its `expand` is "a deliberate no-op: the registry/
legality-check machinery in `expand_class_attributes` requires every registered
name to have an `AttributeExpander` row".

```rust
/// Registry entry for `@native`. Its [`AttributeExpander::expand`] is a
/// deliberate no-op — the member this attribute marks is removed wholesale by
/// [`expand_class_attributes`], which is the only code that owns the
/// [`ClassDef`] and can therefore remove from it. This row exists so the name
/// is legal and its targets are checked.
struct NativeExpander;

impl AttributeExpander for NativeExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Method, Target::Getter, Target::Setter, Target::Construct]
    }
    fn expand(&self, _ctx: &mut ExpandCtx, _member: &mut ClassMember, _args: &[Expr])
        -> Result<(), CompilerError> { Ok(()) }
}
```

### 2. Suppression lives in the driver, not the expander

This is the one structural fact that shapes the whole implementation:

```rust
fn expand(&self, ctx: &mut ExpandCtx, member: &mut ClassMember, args: &[Expr])
    -> Result<(), CompilerError>;
```

`expand` receives `&mut ClassMember` — it can *mutate* a member but **cannot
remove it** from `ClassDef::members`. So `@native` is not expressible as an
ordinary expander. The removal must happen in `expand_class_attributes`, which
owns the `ClassDef` by value.

There is precedent, and it is exact: `@invariant` is *also* a registered no-op
whose real work is a driver special-case (`if attr.name == "invariant" {
validate_purity(…); class_invariants.push(…) }`). `@native` follows the same
shape.

```rust
// In `expand_class_attributes`, BEFORE any member-attribute expansion:
class.members.retain(|m| !member_has_attr(m, "native") && !member_has_attr(m, "ignore"));
```

`member_has_attr` must read the attribute list of each `ClassMember` variant
(`Method`/`Getter`/`Setter`/`Construct` each carry their own `*Def`), so it needs
a small match — there is no uniform accessor today.

### 3. Ordering is load-bearing

Drop **before** member-attribute expansion, for three reasons:

- A dropped body's `@requires`/`@ensures` must not be woven into a body that is
  about to be discarded.
- The `@invariant` weave runs **last, across every member**; it must never see a
  dropped member.
- `@get`/`@set`/`@variant` derives must not generate members from a corpse.

Dropping *after* expansion would be silently wasteful in the best case and would
weave contracts into vanishing code in the worst.

## The invariant test — mandatory, and the reason this attribute is dangerous

**A `@native` body is a second source of truth that looks like executable code
but never runs.** Nothing forces it to agree with the Rust it claims to mirror.

This is precisely the failure mode CB-2 documented: the floor census sat five
amendments behind the code because a document asserted something the machine
never checked. `@native` is *worse than a stale doc* — readers trust code more
than prose, and this is prose wearing code's clothes.

The mitigation is the same discipline that fixed CB-2 —
`invariants.rs::floor_census_matches_installed_bindings` as source of record:

```
for every (class, selector) carrying @native in core.ph:
    assert an installed NATIVE binding exists for that class + selector
```

- **Subset, not bijection.** The reverse direction ("every native binding has an
  anchor") would redden the tree instantly — 136 floor bindings against ~0
  anchors. Bijection is a goal to grow toward, never a gate.
- **Catches:** an anchor with no implementation behind it; an implementation
  deleted with its anchor left standing.
- **Does not catch:** body drift — an anchor that describes behaviour the Rust
  does not have. Nothing cheap does. Recorded, not solved.

Without this test, `@native` is a drift generator with a friendly face. The test
is not optional polish; it is the attribute's licence to exist.

## Which members can legally carry `@native` today

Only four native `toString` bindings exist (`universe/primitives.rs`):

| Class | Rust fn | Site |
|---|---|---|
| `Object` | `object_to_string` | primitives.rs:43 |
| `Number` | `number_to_string` | primitives.rs:121 |
| `Symbol` | `symbol_tostring` | primitives.rs:171 |
| `List` | `list_to_string` | primitives.rs:298 |

Corrections, verified against the tree 2026-07-16 — each contradicts a plausible
assumption:

- **`Bool` has its own `.ph` `toString`** (core.ph:428,
  `self.ifTrue({"true"}, ifFalse:{"false"})`). It is *not* a native binding and
  does *not* inherit `Object#toString`. `@native toString` on `Bool` would fail
  the invariant test — correctly.
- **`Nil`** has no reopen and no own `toString`.
- **`Map`/`Set`/`Tuple`/`Range`** have real `.ph` `toString`s derived in CB-1a.
  They are live and they send per element. Converting them to native + anchor
  would be four floor additions (136 → 140, plus amendment, ADR, census, test
  constant) for zero behavioural gain. Not planned.

**Land one exemplar first: `List#toString`.** `Object`/`Number`/`Symbol` are
mechanical follow-ups once the mechanism and its test are proven.

## Tier placement

Compile tier, `runtime: false`. It is the first **subtractive** Compile-tier
decorator — every built decorator today adds members (`@construct`, `@data`,
`@get`/`@set`, `@variant`) or wraps bodies (`@requires`/`@ensures`/`@invariant`);
`@native` and `@ignore` remove one.

This does **not** require a sixth tier (which [README.md](README.md) "What this
precludes" forbids): removal is an AST→AST transform in the generate phase, which
is exactly what the Compile tier is. It does require acknowledging that the
`AttributeExpander` trait cannot express subtraction — see §2.

## Test strategy

Positive lane (stdout-exact):

- A `@native` member is **not** installed: the native still answers, and the
  anchor body's text never appears in output.
- An anchor body whose text differs visibly from the native's output proves the
  drop (if the anchor ran, the test fails loudly).

Negative lane (`compile-errors/`, substring match):

- `@native class Foo` → `attr.illegal_target`.
- `@native` on a field → `attr.illegal_target`.
- A `@native` body with a syntax error → a parse error (proving the body is still
  parsed, not skipped).

Rust:

- The `@native`-anchors ⊆ installed-native-bindings invariant, in
  `phalcom-core/tests/invariants.rs`.

Every fixture must be **mutation-tested** — corrupt its `.expected`, confirm the
suite reddens *and names that fixture*, restore. The harness bails at the first
mismatch, so mutate one at a time. A silently-skipped fixture is indistinguishable
from a passing one.

## Hazards

- **Silent go-live.** Delete `@native` from a member and its body becomes live,
  shadowing the native binding with no diagnostic. The invariant test cannot catch
  this — removing the attribute removes the member from the checked set.
  Unmitigated. Recorded.
- **Body drift** (above). The anchor may lie; only its *existence* is checked.
- **A plausible-but-wrong anchor is worse than no anchor.** An empty
  `go-to-definition` teaches a reader nothing. A confidently wrong body teaches
  them something false.

## Open questions

| # | Question |
|---|---|
| N-1 | **Resolved, recorded.** Should a `@native` member with no native binding be a compile error? It cannot be — the compiler has no view of the binding table, which bootstrap builds. The invariant test is the only possible check. |
| N-2 | **The provisional core.** How does LSP go-to-definition find an anchor the compiler has discarded? The LSP would need to index the AST *before* the drop, or the drop would need to record spans somewhere. Undecided — this is exactly what "`@native`'s mechanics are provisional" names. Until it is answered, `@native` and `@ignore` are behaviourally identical, which is unsatisfying but honest. |
| N-3 | Should the anchor's `SignatureKind` be checked against the native's? Would catch `@native toString()` (a `Method(0)`) written for a native `toString` (a `Getter`) — a real and easy mistake given CB-1's selector distinction. Cheap to add to the invariant test. Deferred only because the test's shape should settle first. |
| N-4 | Class-level `@native class List`? "This whole class is native" is tempting and would cut annotation noise, but it makes the invariant test's subject ambiguous (every selector? only unbound ones?). Out of scope. |
| N-5 | `@native` + `@ignore` on one member — error, or redundant-but-legal? Leaning error (`attr.redundant`): they mean different things and stacking them signals confusion. Not decided. |

## What this precludes

- **Making `@native` a real binding directive** ("install *this* Rust fn for
  *this* signature") without revisiting the drop semantics — the two readings are
  not compatible, and this file commits to the drop.
- **Using `@native` as a sealing or protection mechanism.** Reopens work
  regardless; an anchor changes nothing about dispatch. Any future design that
  leans on `@native` for protection is building on sand.
- **Treating an anchor as documentation of record.** Only the anchor's existence
  is machine-checked; its body is unverified prose. The Rust remains the truth.
