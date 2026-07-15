# U-CTOR — constructors become ordinary class-side methods: `@constructor`/`@static` decorators, `new_` allocator

Status: **READY** (2026-07-15).
[ADR-0063](../../../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md)
**Accepted**; all seven DEC-CTOR questions ruled (see Decisions). Five sub-units,
landing independently; **U-CTOR-5 is separately gated on a perf measurement**
(DEC-CTOR-C) and may be declined without affecting 1–4. No open blockers.

Not a performance tier — a surface + object-model unit that *removes* machinery.
Net effect on the primitive floor is **−1 fn, −1 binding** (a duplicate dies).

Touches `phalcom-ast/src/{lexer.rs,token.rs,ast.rs,parser.rs}`,
`phalcom-core/src/compiler/{attributes.rs,lib/{class_decl.rs,expr.rs}}`,
`phalcom-core/src/{method/mod.rs,vm/dispatch.rs,value/mod.rs,primitive/{class.rs,object.rs},universe/primitives.rs}`,
`phalcom-core/core/core.ph`, `phalcom-lsp/src/*` (exhaustive-match fixups),
`phalcom-core/tests/invariants.rs`, `docs/spec/v0.2/core/floor-census.md`, plus a
148+152-site `.ph` codemod.

---

## Role

Closes the constructor-dispatch defect class at its root, rather than at the two
symptoms already patched.

The 2026-07-15 partial fix made constructors install under their ordinary selector.
It left standing: the `construct`/`static` keywords, `ConstructDef` (the only
attribute-less `ClassMember`), the `parse_attribute` keyword hack, the
`SignatureKind::Initializer` gate on the super-construct metaclass hop, a duplicated
floor allocator, and — unfixed — the wrong-arity-through-dynamic-receiver hole at
[`DEFERRED.md:29`](../../DEFERRED.md).

---

## Spec anchor

- **[ADR-0063](../../../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md)** — the whole unit. **Accepted** (ratified 2026-07-15). Note U-CTOR-5's desugar is *not* covered by that ratification — DEC-CTOR-C gates it on measurement.
- [ADR-0002](../../../adr/accepted/0002-metaclass-tower-parallel-rule.md) — the parallel tower is the mechanism this stops opting out of.
- [ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md) — `SignatureKind::Initializer` retired (§9).
- [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) — **floor amendment**; census is normative.
- [ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md) — measure-first; the gate on U-CTOR-5.
- `docs/spec/v0.2/classes.md` §1/§3 — **rewritten by this unit** (direction reversed).
- `docs/spec/v0.2/selectors.md` §4 — `@construct` → `@constructor`, "Planned" → shipped.

---

## Preconditions (verify on HEAD — do not trust this list)

| # | Claim | Where | Verify |
|---|---|---|---|
| P1 | `construct`/`static` are keywords | `lexer.rs:284-285` | `rg -n '"static" =>' phalcom-ast/src/lexer.rs` |
| P2 | `ConstructDef` has no `attributes`/`is_static` | `ast.rs:321` | read the struct |
| P3 | `parse_attribute` special-cases `Token::Construct` | `parser.rs:1046` | read |
| P4 | `attach_attributes` errors on constructor | `parser.rs:1113` | read |
| P5 | Constructor installs `Method` selector, `Initializer` kind | `class_decl.rs:640,643` | read |
| P6 | `class_new` ≡ `object_class_new`, byte-identical | `primitive/class.rs:107`, `primitive/object.rs:105` | diff the two bodies |
| P7 | Both registered; `Object class >> new()` shadows `Class >> new()` | `universe/primitives.rs:47,100` | read |
| P8 | `R-INV-0.1` asserts exact floor selector strings | `tests/invariants.rs:616` | read |
| P9 | 148 `construct` / 152 `static` decls | corpus | `rg -c '^\s*construct ' --glob '*.ph'` |
| P10 | 8 non-`new` constructor names, 2 in `core.ph` | corpus | `rg -o --glob '*.ph' '^\s*construct\s+(\w+)' -r '$1' \| sort \| uniq -c` |
| P11 | `new` is **not** a sacred selector | `compiler/inliner.rs` | `rg -n '"new"' phalcom-core/src/compiler/inliner.rs` → expect no sacred-set hit |
| P12 | ADR-0061 not built (leading-`_` rules absent) | `parser.rs:1374` | `rg -n 'fn parse_method_name' -A6` |

**P6/P7 are the load-bearing surprises.** If they do not hold, §U-CTOR-4 is wrong and
the floor accounting in ADR-0063 must be redone before proceeding.

---

## Design

### U-CTOR-1 — `BuiltinAttr` enum, array registry

Mechanical, no behavior change. Lands first so 2/3 have an exhaustive `match` to
extend instead of a stringly `else if` chain.

```rust
// phalcom-core/src/compiler/attributes.rs
pub enum BuiltinAttr { Constructor, Static, Get, Set, Data, Sealed, Variant, Invariant, Requires, Ensures, On }
pub enum AttrKind { Builtin(BuiltinAttr), User(String) }
pub struct Attribute { pub kind: AttrKind, pub args: Vec<Expr>, pub range: SourceRange }
```

`parse_attribute` matches the identifier against a `&'static str` table — no `String`
alloc for known names. Registry becomes an array indexed by discriminant.
`expand_class_attributes`'s `else if attr.name == "construct"` chain
(`attributes.rs:1550-1557`) becomes an exhaustive `match`, so a future unhandled
builtin is a compile error.

> **Honesty note, per ADR-0051.** This is compile-time only, zero runtime effect, and
> almost certainly sub-millisecond across the 666-file corpus. It is justified by
> exhaustiveness and deleting stringly-typed dispatch — **not** by speed. Do not
> benchmark it and do not claim a win.

### U-CTOR-2 — `@static` modifier; drop `Token::Static`

`@static` is a **modifier**: `expand()` sets `is_static = true` in place. One member in,
one out. No codegen change — the bit already exists and already means this.

Parser stops setting `is_static` (`parser.rs:1265`); always `false` until expansion.
`Token::Static` and its lexer row are deleted.

Recovery diagnostic in `parse_class_member`, keyed on identifier `static` followed by a
name token:

> `member.legacy_keyword: 'static foo()' is no longer valid syntax; use the '@static' decorator on the member`

**Static fields.** `static _count = 0` (`class_static_field_shared_state.ph:9`) is a
*field* declaration, not a method — ADR-0017's `static_slots`. `@static` is **not**
**not** legal on `Field` — **ruled** (DEC-CTOR-A). Static fields are per-class
*storage*, not dispatch placement, and get their own decorator:

```phalcom
@classField var _count = 0
```

`@classField` is a **modifier** too (sets the existing static-field bit in place), and
is legal on `Field` only. This sub-unit introduces it, and the codemod must split by
member kind: `static foo()` / `static foo => …` → `@static`, but `static _x = …` →
`@classField var _x = …`. Field declarations are `field_init := FIELD "=" expr` in the
grammar — a distinct production from `method_decl`, so the split is mechanical.

Why the two are different concepts, in one measurement (see DEC-CTOR-A below):
`Derived.count` reads **`None`**, not `Base`'s `2` — storage is per declaring class,
so nothing is "shared" and `@static` would misname it.

### U-CTOR-3 — collapse `ConstructDef`; `@constructor`; drop `Token::Construct`

`ClassMember::Construct` and `ConstructDef` are deleted. `MethodDef` gains
`is_constructor: bool` alongside `is_static`.

`@constructor` is a **derive**, not a modifier — it cannot be done in `expand()`,
which mutates one member in place and cannot append a sibling (this is exactly why
`ConstructExpander`/`GetExpander` are no-ops today). It runs from
`expand_class_attributes` via a new `derive_constructor`, next to `derive_construct`
and `derive_accessors`.

Target-polymorphic, replacing `@construct`:

| Target | Meaning |
|---|---|
| Class header | derive a constructor from declared fields (today's `@construct`, `attributes.rs:729`) |
| Method member | this method is a constructor |

`legal_targets() = &[Target::Class, Target::Method]`. `Target::Construct` deleted.
Header derive runs **first**, emitting a `@constructor`-marked method member that the
member derive then splits — ordering already holds at `class_decl.rs:82`.

Deleted for free: `parse_attribute`'s `Token::Construct` hack (`parser.rs:1046`),
`attach_attributes`'s constructor arm + `attr.dangling` (`parser.rs:1113`).
**Contracts on constructors start working** — add a fixture proving it.

`attributes.rs:729` changes `SignatureKind::Initializer(arity)` → `Method(arity)`.

### U-CTOR-4 — `new_`; re-home `Class >> new()`; tombstone; `duplicate_selector`

**The floor change.** `object_class_new` is deleted as a byte-identical duplicate of
`class_new` (P6). `class_new` → `class_new_`, registered **once**:

```rust
// universe/primitives.rs — replaces BOTH line 47 and line 100
primitive!(vm, class_cls, "new_", SignatureKind::Method(0), class_new_);
```

Instance-side on `Class` — the `Behavior >> basicNew` position, reachable from every
class object through the tower.

`core.ph:34` gains the root default:

```phalcom
class Class {
  new() => self.new_()
}
```

> **Bootstrap gate.** This is the *primitive/library boundary ⊗ bootstrap order*
> hazard. Analysis says safe: every kernel class carries its own `primitive_static!`
> `new`, so none depends on the root default, and nothing before `core.ph:34`
> bare-allocates. **Verify, do not assume** — green bootstrap + `verify_invariants()`
> is a hard gate on this sub-unit.

**Reserved name.** Declaring `new_` in a user class is `selector.reserved_name`.
ADR-0061's ban is *prefix*-keyed and does not cover a trailing underscore, so this is a
new name-keyed check in `parse_method_name`.

**Tombstone.** A class declaring any class-side `new` of arity > 0 and no `new()` gets
a real `new()` method installed on its metaclass at class-definition time, whose body
raises with candidates listed. Ordinary lookup finds it before the root default from
**any** receiver shape. Closes [`DEFERRED.md:29`](../../DEFERRED.md).

Keep the existing compile-time guard (`expr.rs:103`) on top — a compile error beats a
runtime error when the receiver is statically a class. The guard becomes an
optimization over a sound runtime rather than the only defense.

**`class.duplicate_selector`** replaces `ConstructStaticCollision`: two members of one
class body may not install the same selector on the same side, regardless of
decorators. Runs on the **post-expansion** list. Derived members need provenance back
to their source member or the message points at synthesized AST.

### U-CTOR-5 — desugar to two methods (**perf-gated, may be declined**)

```phalcom
@constructor
new(x, y) { _x = x  _y = y }
```
→
```phalcom
@static
new(x, y) { let instance = self.new_()  instance.«init new»(x, y)  return instance }

«init new»(x, y) { _x = x  _y = y  return self }
```

`«…»` is **plan notation, not grammar** — the selector is the string `init new(_,_)`.
`parse_method_name` reads one identifier token, so a name with a space is undeclarable
and unoverridable. Init name derives from the constructor's (`zero()` → `init zero`),
so named constructors never collide.

Deletes: the super-construct metaclass hop (`vm/dispatch.rs`), the
`SignatureKind::Initializer` gate, and the `Initializer` arms of
`encode_selector`/`decode_selector` (`method/mod.rs`).

> **The `Initializer` retirement is a precondition, not a consequence.**
> `encode_selector("init new", …, Method(2))` → `"init new(_,_)"`, but
> `decode_selector("init new(_,_)")` → `Initializer(2)`. Same string, two kinds, and
> `decode_selector` is documented as the exact inverse. **Delete the arms first or the
> round-trip is broken.**

`super.new(x)` inside a `@constructor` body rewrites to `super.«init new»(x)` — an
ordinary instance-side super-send.

> **Gate (ADR-0051).** +1 send per construction, on the hottest path in the language.
> Land only if a construction benchmark shows no regression, or `inliner.rs` folds the
> hop. **If it regresses: decline this sub-unit and keep the fused constructor + the
> `Initializer` gate.** U-CTOR-1..4 do not depend on it.

---

## Write-set

| File | Sub-unit | Change |
|---|---|---|
| `phalcom-ast/src/lexer.rs` | 2, 3 | delete `"static"`/`"construct"` keyword rows (`:284-285`) |
| `phalcom-ast/src/token.rs` | 2, 3 | delete `Token::Static`, `Token::Construct` (`:80,82`) |
| `phalcom-ast/src/ast.rs` | 3 | delete `ClassMember::Construct` (`:197`), `ConstructDef` (`:321`); `MethodDef.is_constructor` |
| `phalcom-ast/src/parser.rs` | 2, 3, 4 | `parse_class_member` (`:1234`) legacy diagnostics + no `is_static`; `parse_attribute` hack out (`:1046`); `attach_attributes` arm out (`:1113`); `parse_method_name` reserved-name check |
| `phalcom-core/src/compiler/attributes.rs` | 1, 2, 3 | `BuiltinAttr`/`AttrKind` (incl. `ClassField`); array registry (`:635`); exhaustive match (`:1550`); `derive_constructor`; `Initializer`→`Method` (`:729`) |
| `phalcom-core/src/compiler/lib/class_decl.rs` | 3, 4, 5 | `Construct` arm → `MethodDef` path (`:615-650`); tombstone install; `duplicate_selector` pre-pass (`:82`+) |
| `phalcom-core/src/compiler/lib/expr.rs` | 4 | keep arity guard (`:103`), re-anchor on the new tables |
| `phalcom-core/src/compiler/lib/error.rs` | 4 | `ConstructStaticCollision` → `DuplicateSelector`; `ReservedName` |
| `phalcom-core/src/method/mod.rs` | 5 | delete `Initializer` arms of `encode_selector`/`decode_selector` |
| `phalcom-core/src/vm/dispatch.rs` | 5 | delete super-construct metaclass hop + `Initializer` gate |
| `phalcom-core/src/primitive/object.rs` | 4 | **delete** `object_class_new` (`:105`) |
| `phalcom-core/src/primitive/class.rs` | 4 | `class_new` → `class_new_` (`:107`) |
| `phalcom-core/src/universe/primitives.rs` | 4 | delete `:47`; `:100` → `new_` |
| `phalcom-core/core/core.ph` | 4 | `class Class { new() => self.new_() }`; codemod 28 sites |
| `phalcom-lsp/src/*` | 3 | 4 `Construct` refs — exhaustive-match fixups |
| `phalcom-core/tests/invariants.rs` | 4 | **R-INV-0.1 selector strings + count (−1)** |
| `docs/spec/v0.2/core/floor-census.md` | 4 | amendment row, net −1 fn/binding |
| corpus `.ph` ×~150 files | 2, 3 | codemod |

---

## Build order

1. **U-CTOR-1** — enum + registry. Green, no behavior change.
2. **U-CTOR-2** — `@static` + `@classField`, codemod 152 sites split by member kind. **Unblocked** (DEC-CTOR-A ruled).
3. **U-CTOR-3** — collapse `ConstructDef`, `@constructor`, codemod 148 sites.
4. **U-CTOR-4** — floor: `new_`, delete the duplicate, re-home `new()`, tombstone, `duplicate_selector`. **Gate: bootstrap + `verify_invariants()` + R-INV-0.1 green.**
5. **U-CTOR-5** — desugar. **Gate: construction benchmark.** Decline if red.

Each step commits green ([[commit-frequently]]). Verify each commit from a **clean
throwaway worktree at the SHA**, not in-tree ([[clean-checkout-verify-each-commit]]).
Main has live concurrent sessions — branch, commit narrow paths, never `git add -a`
([[phalcom-concurrent-session-hazards]]).

---

## Tests / verification

**Positive lane** (stdout-exact) — `tests/lang/classes/`:
- `ctor_decorator_basic.ph` — `@constructor new(x, y)`, literal receiver
- `ctor_dynamic_receiver.ph` — `var C = Point; C.new(1,2)` initializes (the original bug)
- `ctor_named.ph` — `@constructor at(c, r)` + `Ref.at` via variable and module member
- `ctor_super_new.ph` — `super.new(x)` through the desugar (U-CTOR-5)
- `ctor_contracts.ph` — **`@requires` on a constructor** (impossible before U-CTOR-3)
- `ctor_handwritten_smalltalk.ph` — the `person3.ph` pattern with `new_`, working
- `ctor_bare_allocator_still_works.ph` — ctor-less class, both receiver shapes
- `classfield_per_class_storage.ph` — **DEC-CTOR-A2**: `Base.count` → `2`,
  `Derived.count` → `None`. Locks per-declaring-class storage, which nothing tests
  today (`inheritance_static_*.ph` all cover static *methods*)

**Negative lane** — `tests/lang/compile-errors/`:
- `ctor_duplicate_selector.ph` — `@constructor new(x)` + `@static new(x)`
- `ctor_duplicate_static.ph` — `@static new(x)` twice (**catches nothing today**)
- `ctor_reserved_new_underscore.ph` — user declares `new_`
- `ctor_legacy_keyword.ph` — `construct new()` → recovery diagnostic
- `ctor_static_on_field.ph` — `@static` illegal target (pending DEC-CTOR-A)

**Runtime lane** — `tests/lang/runtime-errors/`:
- `ctor_tombstone_arity.ph` — `var C = Point; C.new()` where only `new(_,_)` exists → tombstone raises with candidates. **This is `DEFERRED.md:29`.**
- `classfield_inherited_static_unset.ph` — **DEC-CTOR-A2**: `Derived.bump()` on an
  inherited `@static` method touching an unset subclass `@classField` raises
  `None does not understand '+(_)'`. Ratified as correct, so it is pinned here rather
  than fixed.

**Fixtures must be proven wired**, not assumed: the harness runs one test per *lane*
iterating the directory, so a new file can be silently skipped. Corrupt each `.expected`,
confirm the suite reddens, restore ([[phalcom-golden-test-lanes]]).

**Hard gates:** `cargo test` (26 binaries) · `verify_invariants()` · **R-INV-0.1** ·
`cargo clippy --workspace` (13 warnings pre-existing, none new) · `cargo doc` clean
(every new public item needs rustdoc — [[rust-doc-mandatory]]) · `graphify update .`

---

## Decisions (DEC-CTOR) — **all ruled 2026-07-15**

| # | Question | Ruling |
|---|---|---|
| **A** | Does `@static` cover static fields? | **No — a distinct decorator.** *(against the plan's own recommendation; see below)* |
| **A1** | Its name | **`@classField`** |
| **A2** | Inherited `@static` method reading a subclass's unset `@classField` → `None` | **Working as designed — fixture it** |
| **B** | Ratify ADR-0063 (reverses `classes.md` §1)? | **Ratify in full** — ADR-0063 is **Accepted** |
| **C** | U-CTOR-5 desugar vs +1 send/construction | **Measure, then decide** — gate stands |
| **D** | Codemod one-shot vs deprecation window | **One-shot**, no window |
| **E** | `@construct` vs `@constructor` for the header derive | **Unify on `@constructor`** |

**A is the interesting one, and the plan was wrong.** This doc recommended `@static` on
`Field` — "one decorator for both sides." Measurement says that would have taught the
wrong model:

```phalcom
class Base { @classField var _count = 0
             @static bump() { _count = _count + 1 }
             @static count => _count }
class Derived extends Base {}
Base.bump()  Base.bump()
Base.count      // 2
Derived.count   // None   <- own slot, not Base's
```

Storage is **per declaring class**, exactly as ADR-0011 rules for instance fields —
ADR-0017 is that rule one tower level up. That is a Smalltalk **class-instance
variable**, not a class variable. `static`/`@shared`/`@classvar` all connote
hierarchy-wide sharing that does not exist. `@classField` names it correctly: a field,
obeying the field rule, on the class side.

**A2** ratifies the consequence rather than patching it: `Derived.bump()` raises
`None does not understand '+(_)'`. Untested today — `inheritance_static_*.ph` cover
static *methods* only — and unruled by ADR-0017 (DEC-D settled storage alone). Lock it
with a fixture; do not add a per-class initializer re-run.

**U-CTOR-2 is unblocked**, with scope grown: it now also introduces `@classField` and
must codemod `static _x = …` field declarations separately from `static foo()` methods.

---

## What must this not preclude (P4)

- **Niche/NaN-boxing (ADR-0044/0010)** — no new `Value` arm, no tag. Untouched.
- **U-IC inline caches** — *improved*: constructors become ordinary monomorphic sends.
  Tombstone installs pre-instance (ADR-0053's condition), so no epoch bump.
- **ADR-0043 (no default args)** — nothing varies effective arity; the tombstone is
  arity-0-keyed and adds no arity family. The *identity-dispatch ⊗ optional arity*
  hazard does not fire.
- **ADR-0026/0041 (sealed reparenting)** — the tombstone is computed at
  class-definition time and would need recomputation under reparenting. Already sealed;
  moot. **Re-opening reparenting would reopen this.**
- **ADR-0061** — disjoint (leading vs trailing `_`), but if 0061 lands first, the
  `new_` reserved-name check must compose with its prefix ban rather than duplicate it.
- **A future capability boundary** — retiring `Initializer` removes a *kind*, not a
  hook. A dispatch-level boundary (ADR-0061's deferred idea) stays open.
- **`@constructor` on the class header** — the member derive must not assume a
  hand-written source member; the header derive synthesizes one.

---

## Known deviations this unit does **not** fix

- `implementation-status.md` row 5 is stale independently of this unit ("No `construct`
  token/node", "`ClassMember` = Method/Getter/Setter only" — both false at HEAD). Left
  alone deliberately; fixing it means rewriting a doc that describes a pre-`construct`
  world throughout.
- `class_attribute_construct_get_set.ph` is `status: PENDING` while live and passing.
- The generalized "static receiver, selector matches nothing" diagnostic — the arity
  guard is a special case of it and it would catch typos in *any* static call. Out of
  scope.
