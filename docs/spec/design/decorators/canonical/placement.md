# Placement decorators — `@class` and `@constructor`

- Status: **Ratified design (ADR-0063, 2026-07-15, amended DEC-CTOR-F/G/H/I),
  entirely unbuilt.** The tree at HEAD still has `construct`/`static` keywords,
  `ConstructDef`, and the `@construct` derive. Implementation is U-CTOR
  (after U-BINDINGS). This file is the decorator-system view of that ADR —
  read ADR-0063 for the full constructor-model rationale.
- Verification: this family was checked against the metaclass tower (ADR-0002),
  selector identity (ADR-0012), and the `@construct` collision — it is the one
  place the user-facing decorator surface intersects the object model's core.

## What the user's framing gets wrong (and the ADR got right)

The intuitive split — `@construct` for classes, `@constructor` for constructor
methods — is exactly what ADR-0063 DEC-CTOR-E evaluated and rejected: *two
names one character apart with unrelated meanings is a trap.* The ratified
surface is **one target-polymorphic `@constructor`**:

| Applied to | Meaning |
|---|---|
| class header | derive a constructor from declared fields — today's `@construct` job, verbatim behavior |
| method member | this method **is** a constructor |

Same registry-row pattern as `@get`/`@set`: one name, `Target`-dispatched.
`@construct` and `ConstructDef`/`Target::Construct` are deleted when U-CTOR
lands; [derives.md](derives.md) records the residue that transfers (own-fields
limit, keyword-only params).

## `@class` — placement, not storage class

`@class` marks a member as **class-side** (installed on the metaclass), legal
on methods, getters, setters, and fields. It replaced `static` (and the
same-day-deleted `@classField`) because both misname what Phalcom actually
has: per-*declaring*-class storage (`static_slots` indexed by the metaclass
field table, ADR-0017) — a Smalltalk class-instance variable, never
hierarchy-shared. The decorator names *placement*; the storage semantics come
from the tower.

The ratified sharp edge travels with it (DEC-CTOR-A2): an inherited `@class`
method touching a subclass's unset `@class` field raises
`None does not understand` — correct, fixture-pinned, and made a one-word
idiom by the `class` keyword-variable (`class.update(n)` in an inherited
constructor re-runs against the *receiving* class's own slot).

`@class @constructor` on one member is an error, not a redundancy —
constructors are already class-side by construction; stacking the two signals
a misunderstanding worth stopping.

## Why these are decorators and not keywords — the actual argument

This is the strongest philosophy-verification result in the whole tree.
`construct` and `static` were *keywords* carving special member kinds with
special dispatch (`SignatureKind::Initializer`, the super-construct metaclass
hop) — and the overlay records that every constructor bug to date came from
opting out of the metaclass tower's one rule. ADR-0063 dissolves the special
kinds into **ordinary class-side methods**, and once a constructor is an
ordinary method, "this member is a constructor" and "this member is
class-side" are exactly the kind of *member-level, compile-expandable,
semantics-preserving marks* the decorator system exists for. The keywords
weren't wrong syntax; they were wrong *semantics* wearing syntax. Decorators
here aren't sugar preference — they are the visible sign the special cases
died. (Precedent with consequence: Python's `@classmethod`/`@staticmethod`
made the same move — placement as decoration over an ordinary function — and
it is why Python never needed a metaclass-side method grammar.)

## Mechanics (from the ADR, decorator-relevant slice)

- `@class` is a **modifier** (in-place `expand`, sets `is_class_side`);
  `@constructor` is a **derive** (driver-level, 1→2 members): class-side
  `new(x,y) { let instance = self.new_(); instance.«init new»(x,y); return instance }`
  plus instance-side mangled `«init new»(…)` — undeclarable, unoverridable
  (space in the selector), dispatched as a plain method. `new_` is the sole
  primitive allocator, restricted to `native_repr` classes (DEC-CTOR-H2).
- Cost: +1 send per construction vs the fused form — **benchmark-gated**, with
  the fused constructor as recorded fallback. This is ADR-0051 discipline
  applied to a desugar, and the decorator spec inherits the gate: do not
  assume the two-method shape is final until the number exists.
- Both names are lowercase **builtins** (registry rows) under the naming
  convention — they fire at Compile (expand) time and are compiler-owned.

## Implementation plan (decorator-system slice of U-CTOR)

1. Registry: add `constructor` and `class` rows; `legal_targets`:
   `constructor` = `[Class, Method]`, `class` = `[Method, Getter, Setter,
   Field]`. `@class @constructor` co-occurrence check in the driver
   (`attr.conflicting`, or the ADR's named error if it specifies one).
2. Delete `ConstructExpander` + `Target::Construct` with `ConstructDef`'s
   collapse into `MethodDef` (`is_class_side`, `is_constructor` flags) — this
   also dissolves the `attr.dangling`-on-constructor asymmetry
   ([mechanism.md §1](mechanism.md)), so `@native`/`@ignore`/`@requires` on a
   constructor become expressible for free. Update their `legal_targets`
   accordingly in the same change, with fixtures — native.md's "Construct
   listed but unreachable" row finally becomes reachable and must be tested.
3. `derive_constructor` (class-header form) inherits `derive_construct`'s
   body verbatim, then the two recorded residue decisions land here:
   keyword-only vs positional params (field-decorator-followups §3 assigns
   this to U-CTOR-3 explicitly), and the own-fields-only super-chaining limit.
4. Codemod: 148 `construct` + 152 `static` sites, one-shot, no deprecation
   window (DEC-CTOR-D); `member.legacy_keyword` contextual diagnostic guards
   stragglers.
5. Tests: the ADR's fixture list plus decorator-specific ones —
   `@constructor` on class *and* on method, `@class` field storage
   per-declaring-class (`Base.count` set, `Derived.count` `None`),
   `@class @constructor` rejection.

## Hazards

- **Do not harden the current ctor-inherit guard** — the inheritance-aware
  `new`-bare-allocator guard is scheduled for deletion by DEC-CTOR-H; work
  invested there is work thrown away.
- **`@constructor` on a method changes what callers may assume** (`new` no
  longer tombstoned; wrong-arity `Factory.new()` returns an all-`None` object
  *by specification*). The decorator spec must not re-invent the arity guard
  the ADR deliberately deleted; a future *warning* is the sanctioned path.

## What this precludes

- Re-introducing a `@construct`/`@constructor` name pair, or any second
  placement keyword. One name per axis: `@constructor` (kind),
  `@class` (side).
- A hierarchy-shared class-variable decorator (`@shared`) without its own
  storage design — `@class` is per-declaring-class by ratified semantics, and
  overloading it later would repeat the `static` misnaming this family
  exists to fix.
