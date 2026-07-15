# `@ignore` — the compiler does not compile this member

- Status: **Specified — not built.** No `ignore` row exists in
  `AttributeRegistry::new`; `@ignore` today raises `attr.unknown`.
- Date: 2026-07-16
- Evidence: none in `phalcom-core/src` — this file specifies work, it does not
  record it. The mechanism it uses is real: `InvariantExpander`
  (`compiler/attributes.rs` L456-478) is a registered no-op whose actual effect
  lives in `expand_class_attributes` (L1548).
- Depends on: [README.md](README.md) (the tier model) ·
  [annotations-core.md](../experimental/annotations-core.md) (the `@` mechanism,
  registry, phase pipeline)
- Related: [native.md](native.md) (borrows this attribute's mechanism,
  provisionally)

## What it is

`@ignore` drops a member. The compiler parses it, checks its attribute legality,
and then discards it: no bytecode, no method installed, method table untouched.

```phalcom
class Draft {
  // Parsed. Not compiled. `Draft` has no `halfFinished` method.
  @ignore halfFinished(x) {
    x.someMethodThatDoesNotExistYet()
  }
}
```

**This is the sanctioned drop — the only attribute whose *meaning* is "the
compiler ignores this".** If a future attribute needs a member to vanish, it
either delegates to this mechanism or justifies its own; there is deliberately not
a second, parallel way to make code disappear.

## Relationship to `@native`

[`@native`](native.md) currently **also** drops its member, but that is
**provisional** — a borrow of this attribute's mechanism, standing in until
`@native`'s own mechanics are decided (native.md N-2). Today the two are
behaviourally identical. They are not interchangeable:

| | `@ignore` | `@native` |
|---|---|---|
| Meaning | "this is not code" | "the real implementation is in Rust; this is its source anchor" |
| Effect today | drops the member | drops the member |
| Effect permanence | **permanent** — this is what it means | **provisional** — the drop is a stand-in |
| Asserts anything? | no | yes: that a native binding exists for this class + selector |
| Machine-checked? | nothing to check | **yes** — the anchors ⊆ installed-native-bindings invariant |

**Rule.** If you want "do not compile this", write `@ignore`. Reach for `@native`
only when a Rust primitive really does back the member — it carries an assertion
and a test, and using it as a generic mute makes that test lie.

## Semantics

1. The member is **parsed**. A syntax error inside an `@ignore` body is still a
   compile error. `@ignore` mutes *compilation*, not *lexing and parsing* — it is
   not a comment, and it is not `#if 0`.
2. Attribute **legality is still checked** (`attr.illegal_target`).
3. The member is **dropped** from `ClassDef::members` before compilation.

What it does **not** do: suppress diagnostics, weaken checks on surrounding code,
or defer anything. It removes exactly one member from the class body and nothing
else.

## Legality

| | |
|---|---|
| Legal targets | `Method`, `Getter`, `Setter`, `Construct` |
| Illegal targets | `Class`, `Field` (`attr.illegal_target`) |
| Arguments | none — `@ignore`, never `@ignore(...)` |

`Getter` and `Setter` are included for the same reason as in
[native.md](native.md): `toString` is a `SignatureKind::Getter`, and a member kind
that can be written can be ignored.

`Field` is excluded because dropping a field changes the instance layout
(ADR-0011), which is a materially different act from dropping a method — see I-2.

## Implementation

`@ignore` and `@native` share one mechanism. The full derivation is in
[native.md](native.md) §Implementation; the short form:

### 1. Registration

```rust
expanders.insert("ignore".to_string(), Box::new(IgnoreExpander));
```

Registration is required even though the expander does nothing — the registry is
also the legality gate, and an unregistered name raises `attr.unknown` unless it
resolves to a user `Attribute` subclass (M-ATTR-ROOT). `InvariantExpander` is the
stated precedent for a registered no-op.

```rust
/// Registry entry for `@ignore`. Its [`AttributeExpander::expand`] is a
/// deliberate no-op — the marked member is removed wholesale by
/// [`expand_class_attributes`], the only code that owns the [`ClassDef`] and
/// can remove from it. This row exists so the name is legal and its targets
/// are checked.
struct IgnoreExpander;

impl AttributeExpander for IgnoreExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Method, Target::Getter, Target::Setter, Target::Construct]
    }
    fn expand(&self, _ctx: &mut ExpandCtx, _member: &mut ClassMember, _args: &[Expr])
        -> Result<(), CompilerError> { Ok(()) }
}
```

### 2. Suppression lives in the driver

`AttributeExpander::expand` takes `&mut ClassMember` — it can mutate a member but
cannot remove it from `ClassDef::members`. The removal therefore happens in
`expand_class_attributes`, which owns the `ClassDef`:

```rust
// BEFORE any member-attribute expansion:
class.members.retain(|m| !member_has_attr(m, "native") && !member_has_attr(m, "ignore"));
```

One `retain` serves both attributes. Ordering is load-bearing — see
[native.md](native.md) §3: drop before expansion, so contracts are never woven
into a body that is about to vanish and the `@invariant` weave (which runs last,
across every member) never sees a corpse.

## Tier placement

Compile tier, `runtime: false`. Subtractive — see [native.md](native.md)
§"Tier placement" for why removal needs no sixth tier but does exceed what the
`AttributeExpander` trait can express.

## Test strategy

Positive lane (stdout-exact):

- An `@ignore` member is not installed — sending its selector raises
  `doesNotUnderstand`, proving absence rather than merely not-observing presence.
- An `@ignore` member whose body would fail at runtime if it ran (or would print)
  proves the drop rather than a coincidence.
- Sibling members of an ignored one still compile and work.

Negative lane (`compile-errors/`, substring match):

- `@ignore class Foo` → `attr.illegal_target`.
- `@ignore` on a field → `attr.illegal_target`.
- An `@ignore` body with a **syntax error** → a parse error. This is the fixture
  that pins "`@ignore` is not a comment", and it is the most valuable one here.

Every fixture must be **mutation-tested** — corrupt its `.expected`, confirm the
suite reddens *and names that fixture*, restore. The harness bails at the first
mismatch, so mutate one at a time. A silently-skipped fixture is indistinguishable
from a passing one.

## Hazards

- **Silent dead code.** An `@ignore`d member compiles clean forever and never
  runs. It is invisible drift by construction — the attribute's whole purpose.
  Unlike `@native`, there is nothing to cross-check it against, because it asserts
  nothing. The mitigation is social: `@ignore` is for work in progress, not for
  parking code indefinitely.
- **Reach-for-the-mute.** The easy failure is using `@ignore` where the honest act
  is deleting the member (git remembers it) or fixing it. Nothing in the compiler
  can distinguish those cases.
- **Not a feature flag.** `@ignore` is unconditional. It has no build-mode axis
  and must not grow one by accident — `CompileMode` (`Debug`/`Release`/
  `Unchecked`) is a separate mechanism that strips *contract guards*, and
  conflating them would be a second, unrelated build-mode dimension (README D-1).

## Open questions

| # | Question |
|---|---|
| I-1 | Should `@ignore` warn? A member that is silently dropped forever is exactly the kind of thing a linter should mention once. Argues for a diagnostic at `warn` level — but Phalcom has no warning tier today, only errors. Deferred until one exists. |
| I-2 | `@ignore` on a `Field`? Dropping a field changes the instance layout (ADR-0011), which is materially different from dropping a method and interacts with `@construct`/`@data`/`@get`/`@set` derives that read the field list. Currently `attr.illegal_target`. Revisit only with a concrete need. |
| I-3 | `@ignore` on a whole class? Same shape as native.md N-4. Out of scope. |
| I-4 | `@native` + `@ignore` on one member — error, or redundant-but-legal? Leaning error (`attr.redundant`). Shared with native.md N-5; decide once, for both. |

## What this precludes

- **A second drop mechanism.** Any future "make this vanish" attribute delegates
  to this one or argues its way past this line. One way to disappear code.
- **`@ignore` as conditional compilation.** It is unconditional by definition;
  making it read a build mode would collapse it into a feature-flag system this
  spec family has not designed and does not want by accident.
- **`@ignore` as a diagnostic suppressor.** It removes a member; it does not mute
  errors about one. A member that does not parse still fails.
