# New proposals originating in this tree

- Status: **Proposed — nothing here is ratified.** Each entry is a PDR
  candidate with its analysis done; the PDR can be short. Entries proposed
  inside family files are indexed at the bottom so the candidates live in
  one list.

## `@override` — class-definition-time override check (recommended)

**The problem it solves is real and current.** Overriding is silent: a
subclass method with a typo'd selector (`toString()` for `toString`,
`draw(at)` for `draw(at:)` — selector identity makes these *different
methods*) installs a new member and the parent's stays live. No diagnostic
exists or can exist without a declared intent. This is the C++ pre-`override`
world, and the fix is the same declared-intent marker C++11, Java
(`@Override`), Kotlin (mandatory `override`), and Swift all converged on —
rare unanimity among precedents, and the consequence everywhere was the same:
a whole bug class became a compile error.

**Why it is *sound* in Phalcom now — and wasn't before 2026-07-20.** The
check "does an inherited member with this exact selector exist?" needs a
stable superclass chain and a complete parent method set at check time. Both
became true only recently: superclass sealed at definition (ADR-0026/0041),
classes closed — no post-definition reopening (PDR-0001), and definition
order guarantees the parent's members are installed before a subclass's
class-definition executes. The check runs at class-definition time (when the
resolved parent exists), implemented as a driver-level validation against
`lookup_method_in_hierarchy` starting at the superclass.

**Semantics.**

- `@override` on method/getter/setter: legal iff the superclass chain defines
  the identical selector; otherwise `attr.no_override_target`, naming the
  nearest-selector candidates (the near-miss list is the point — it catches
  the typo).
- Absence of `@override` on an actual override: legal, no diagnostic (v0.2).
  A future lint ("override without marker") is the Kotlin endgame, gated on
  the warn tier; making the marker *mandatory* is a breaking change this
  proposal explicitly does not include.
- Interaction with `@class`: checks the *metaclass* chain (class-side
  overrides are overrides too — the tower makes this one rule, not two).
- Tier: Compile-registered builtin, lowercase; fires as class-def-time
  validation like the `@On` checks. `runtime: false`, zero retention.

**Cost:** one hierarchy lookup per marked member at class-definition time.
Nothing per-send, nothing retained.

**What it precludes:** nothing. It is a pure check; deleting it restores
today's semantics. (That reversibility is why it can be a short PDR.)

## PDR-candidate index (proposals made across the tree)

| Candidate | Where argued | Size |
|---|---|---|
| Naming convention: lowercase builtin vs Capitalized `Attribute` class | [README.md](README.md), [mechanism.md](mechanism.md) | small |
| COLL-3 suffix resolution (`@Name` → `NameAttribute` then `Name`) | [mechanism.md](mechanism.md) | small |
| Labeled attribute arguments (DEF-3 fix — unblocks ratified `@On` surface) | [mechanism.md](mechanism.md) plan §1 | medium |
| Registry-wide `AttrArity` validation (DEF-6) | [mechanism.md](mechanism.md) plan §4 | small |
| `@ensures` `result` substitution (DEF-1) | [contracts.md](contracts.md) plan §1 | small |
| ADR-0052 guard implementation (DEF-2 — ratified, so a unit, not a PDR) | [contracts.md](contracts.md) plan §2 | medium |
| Contract inheritance ruling (covariant-replacement gap) | [contracts.md](contracts.md) | needs real design |
| `attr.set_on_const` (DEF-8) | [derives.md](derives.md) | small |
| Reject-don't-ignore `@get(priv)` (DEF-7) | [derives.md](derives.md) | small |
| `@native`+`@ignore` stacking = `attr.redundant` (N-5/I-4) | [subtractive.md](subtractive.md) | trivial |
| N-2 pre-drop harvest table (LSP anchor + invariant test, one producer) | [subtractive.md](subtractive.md) | medium |
| `attr.native_outside_core` | [subtractive.md](subtractive.md) | small |
| U-METHOD-REIFY floor amendment (`Method.fromBlock`/`invokeOn`/`defineMethod`) | [behavioral.md](behavioral.md) plan §1 | **the big one** — floor PDR |
| U-LAYOUT-SLOTS (reserved hidden slots; `@lazy`/`@synchronized`/`@observable` gate) | [behavioral.md](behavioral.md) plan §4 | medium |
| ADR-0053 guard-bit build (ratified → unit) | [interception.md](interception.md) plan | medium |
| `@Deprecated(reason:, since:)` | [metadata-and-docs.md](metadata-and-docs.md) | small |
| `@suspends` advisory marker (ship only with a consumer) | [concurrency.md](concurrency.md) | small, gated |
| `@override` | this file | small |
| Framework renames (COLL-2/COLL-4) | [frameworks.md](frameworks.md) | small, editorial until the families build |

## Deliberately not proposed

Recorded so their absence reads as decision, not oversight: `@async`/`@await`
([concurrency.md](concurrency.md)); performance hints and `@cfg`/`@intrinsic`
([compiler-directives.md](compiler-directives.md)); `@effect` before R-5
([reactive.md](reactive.md)); `@pure` without an effect system
([contracts.md](contracts.md)); doc-content decorators
([metadata-and-docs.md](metadata-and-docs.md)); `@Timed` as a separate
decorator ([behavioral.md](behavioral.md)); scheduling/priority decorators
([concurrency.md](concurrency.md)).
