# U-CTOR — constructors become ordinary class-side methods: `@construct`/`@constructor`/`@class`, `new_` allocator

> **Revised 2026-07-21** for [PDR-0028](../../../pdr/0028-class-and-constructor-decorator-canon.md).
> `@construct` is class-header-only; `@constructor` is method-only; and `@class`
> owns all class-side placement. Legacy `construct` and `static` declarations remain
> parseable during migration with non-fatal help hints. U-CTOR-4's **tombstone and
> arity guard are deleted** (so this unit **no longer closes `DEFERRED.md:29`** — it
> is dissolved by ruling) and gains a **`native_repr`** category instead; a `class`
> keyword-variable is added.
> **[U-BINDINGS](../../../forge/units/U-BINDINGS/u30-bindings-plan.md) now lands first** — its field grammar is the
> ground this unit's `@class` fields stand on.

Status: **IMPLEMENTATION COMPLETE (CLOSURE WORK PENDING)** (2026-07-22). Closure authority: [`ctor-completion-implementation-spec.md`](../ctor-completion-implementation-spec.md). Final verification is deferred pending explicit instruction.
[PDR-0028](../../../pdr/0028-class-and-constructor-decorator-canon.md) is **Accepted**
and supersedes ADR-0063's target-polymorphic `@constructor` surface. Six sub-units
land independently, but U-CTOR-5's two-method lowering is required semantics, not a
performance option.

Not a performance tier — a surface + object-model unit that *removes* machinery.
Net effect on the primitive floor is **−1 fn, −1 binding** (a duplicate dies).

Touches `phalcom-ast/src/{lexer.rs,token.rs,ast.rs,parser.rs}`,
`phalcom-core/src/compiler/{attributes.rs,lib/{class_decl.rs,expr.rs}}`,
`phalcom-core/src/{method/mod.rs,vm/dispatch.rs,value/mod.rs,primitive/{class.rs,object.rs},universe/primitives.rs}`,
`phalcom-core/core/core.ph`, `phalcom-lsp/src/*` (exhaustive-match fixups),
`phalcom-core/tests/invariants.rs`, `docs/spec/current/core/floor-census.md`, and
compatibility fixtures plus a recommended corpus codemod.

---

## Role

Closes the constructor-dispatch defect class at its root, rather than at the two
symptoms already patched.

The 2026-07-15 partial fix made constructors install under their ordinary selector.
It left standing: the `construct`/`static` keywords, `ConstructDef` (the only
attribute-less `ClassMember`), the `parse_attribute` keyword hack, the
`SignatureKind::Initializer` gate on the super-construct metaclass hop, and a
duplicated floor allocator.

[`DEFERRED.md:29`](../../../forge/DEFERRED.md) (wrong arity through a dynamic receiver) is
**not** fixed by this unit — DEC-CTOR-H rules the behavior correct. That row is
dissolved, not closed; rewrite it to say so.

---

## Spec anchor

- **[PDR-0028](../../../pdr/0028-class-and-constructor-decorator-canon.md)** — governing canon. `docs/spec/current` is authoritative; this pending unit follows it.
- [ADR-0002](../../../adr/accepted/0002-metaclass-tower-parallel-rule.md) — the parallel tower is the mechanism this stops opting out of.
- [ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md) — `SignatureKind::Initializer` retired (§9).
- [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) — **floor amendment**; census is normative.
- [ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md) — measure performance but never make a semantic lowering optional.
- [Classes §1–3](../../../spec/current/classes.md) and [Selectors §4](../../../spec/current/selectors.md) — canonical constructor, placement, and decorator-target contract.

### Current-spec crosswalk (implementation reading order)

Read these as normative constraints on this plan.  They are deliberately linked at
the point of use, rather than treated as background material: implementation work
must reconcile the source, tests, and every row below before changing behavior.

| Surface | Current spec | Plan consequence |
|---|---|---|
| Public constructor contract, `new_`, handwritten construction, field initialization, `@class` fields | [Classes §1–2](../../../spec/current/classes.md) | U-CTOR-2..6; in particular, preserve ordinary lookup for `new`, reserve `new_`, and keep class-side storage per declaring class. |
| Class/metaclass identity, `Behavior`, and parallel class-side lookup | [Object Model §2, §4–5](../../../spec/current/object-model.md) | U-CTOR-3/4 must install and resolve through the parallel metaclass tower; no constructor-only dispatch path may survive. |
| Exact selector identity, declaration grammar, and attribute targets | [Selectors §1, §4](../../../spec/current/selectors.md) · [Messages & Selectors §2–4](../../../spec/current/messages-and-selectors.md) | U-CTOR-3/5 retire `SignatureKind::Initializer` without changing encoded selector identity; duplicate detection keys on side + canonical selector. |
| Ordinary class-side and `super` lookup, DNU behavior | [Method Lookup §1–2](../../../spec/current/method-lookup.md) | U-CTOR-4 preserves inherited `new()`; U-CTOR-5's rewritten `super` send remains an ordinary super-send. |
| Attribute spelling, migration recognition, and reserved declaration names | [Lexical Structure §3, §10](../../../spec/current/lexical-structure.md) · [Syntax grammar](../../../spec/current/syntax/grammar.md) · [Syntax declarations](../../../spec/current/syntax/statements-and-declarations.md) | U-CTOR-2/3/6 own lexer/parser recovery for `static`, `construct`, and the keyword-variable `class`; validate grammar rather than only AST lowering. |
| Decorator tier/order and generated-member collisions | [Decorators overview](../../../spec/current/decorators/README.md) · [`@construct`](../../../spec/current/decorators/construct.md) · [`@constructor`](../../../spec/current/decorators/constructor.md) | U-CTOR-1/3 must preserve class-header derivation ordering, method-only `@constructor`, and same-side selector collisions. |
| Derives and weaves that currently special-case `ConstructDef` | [`@data`](../../../spec/current/decorators/data.md) · [`@get` / `@set`](../../../spec/current/decorators/accessors.md) · [`@requires`](../../../spec/current/decorators/requires.md) · [`@ensures`](../../../spec/current/decorators/ensures.md) · [`@invariant`](../../../spec/current/decorators/invariant.md) · [`@native`](../../../spec/current/decorators/native.md) | U-CTOR-3 migrates every target/attribute path from `Construct` to method + `@constructor`; contracts and native anchors need explicit legality and regression coverage. |
| Core allocator placement and native-class catalog | [Core overview](../../../spec/current/core/overview.md) · [Core classes](../../../spec/current/core/core-classes.md) · [Catalog delta](../../../spec/current/core/catalog-delta.md) | U-CTOR-4 moves the sole generic allocator without changing native constructors; `native_repr` must prevent invalid instance allocation. |
| Tower boot order and invariant gates | [Bootstrap phases](../../../spec/current/core/bootstrap-phases.md) · [Invariant requirements](../../../spec/current/core/invariant-requirements.md) | U-CTOR-4 requires a clean boot plus parallel-tower and floor-census invariants before landing. |
| Primitive floor accounting | [Floor census](../../../spec/current/core/floor-census.md) | U-CTOR-4 updates both census and `R-INV-0.1`; the claimed net −1 is not complete until live bindings agree. |
| Heap edges, `static_slots`, and constructor-era VM metadata | [Memory management §2.1–2.3](../../../spec/current/memory-management.md) | U-CTOR-4's `native_repr` and class-side allocation changes must keep class objects, static slots, and every new heap edge traceable. |
| Uninitialized state / `None` observable after bare allocation | [Values & Absence §3](../../../spec/current/values-and-absence.md) | U-CTOR-4 tests the ruled behavior: a legal bare allocation leaves fields as `None`, rather than inventing an arity/tombstone failure. |
| Hot-path measurement and deoptimization equivalence | [Performance Strategy](../../../spec/current/performance.md) | U-CTOR-5 is required semantics. Benchmark and optimize only without changing its observable two-method behavior. |

### Reconciliation record — resolved by PDR-0028

PDR-0028 settles the five formerly blocking conflicts. Implementation may not reopen
them: `@construct` derives only on class headers; `@constructor` marks only methods;
U-CTOR-5 is semantic lowering; inherited `new()` bare-allocates; `@class` is the only
canonical class-side spelling; and current decorator docs must distinguish canon from
legacy `ConstructDef` implementation until U-CTOR-3 removes it.

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
the floor accounting must be redone before proceeding.

---

## Design

### U-CTOR-1 — `BuiltinAttr` enum, array registry

Mechanical, no behavior change. Lands first so 2/3 have an exhaustive `match` to
extend instead of a stringly `else if` chain.

```rust
// phalcom-core/src/compiler/attributes.rs
pub enum BuiltinAttr { Construct, Constructor, Class, Get, Set, Data, Sealed, Variant, Invariant, Requires, Ensures, On }
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

### U-CTOR-2 — `@class` modifier; retain legacy class-side parsing

`@class` is a **modifier**: `expand()` sets `is_class_side = true` in place. One member
in, one out. No codegen change — the bit already exists and already means this.

Canonical declarations reach the parser with `is_class_side = false` until attribute
expansion. `static` and class-member `class foo(...)` stay recognized only as legacy
declaration forms; each lowers to the same member marked `@class`, preserving meaning
while emitting its hint.

Recovery diagnostic in `parse_class_member`, keyed on identifier `static` followed by a
name token:

> help: `static foo()` is legacy syntax; use `@class foo()`

> help: `class foo()` is legacy syntax; use `@class foo()`

The hint points at `static`, is non-fatal, and does not call the source deprecated.

**`@class` covers fields too** — **ruled** (DEC-CTOR-F). No separate field decorator;
one decorator is legal on `Method`/`Getter`/`Setter`/`Field`:

```phalcom
@class _total = 0        // class-side field (unkeyworded mutable — ADR-0064)
@class update(n) { _total = _total + n }
```

Both are the same bit at different member kinds: on a method it flips the installation
target to the metaclass, on a field it flips storage to the class object's
`static_slots`. `@class` names **placement**, which is what they share — where `@static`
would have implied Java/C# "one shared slot", and Phalcom shares nothing across a
hierarchy (`Derived.count` reads `None`, not Base's `2`).

**Codemod splits by member kind**, mechanical because `field_init` is a distinct
grammar production from `method_decl`:

| Old | New |
|---|---|
| `static foo()` / `static foo => …` | `@class foo()` / `@class foo => …` |
| `static _x = …` | `@class _x = …` |

> **Depends on U-BINDINGS.** The field spelling above (`@class _total = 0`, no `var`)
> is ADR-0064's grammar. U-BINDINGS must land first, or this sub-unit has to emit
> `@class var _total = 0` and then migrate the same lines again.

### U-CTOR-3 — collapse `ConstructDef`; `@construct` header derive; `@constructor` methods

`ClassMember::Construct` and `ConstructDef` are deleted. `MethodDef` gains
`is_constructor: bool` alongside `is_static`.

`@constructor` is a method marker. `@construct` is the class-header derive: it runs
from `expand_class_attributes`, before member decorators, and emits a
`@constructor`-marked method from declared fields. The marker's lowering then splits
that method into the canonical allocation and initializer pair.

| Target | Meaning |
|---|---|
| `@construct` class header | derive a constructor from declared fields |
| `@constructor` method member | mark this method as a constructor |

`@construct` is legal only at `Target::Class`; `@constructor` is legal only at
`Target::Method`. Header derivation runs **first**, emitting a `@constructor`-marked
method member; member lowering follows. `@constructor` on a header and `@construct`
on a member are target errors.

`ConstructDef` and the attribute-less member path are deleted. Legacy `construct
foo(...)` remains parseable, lowers to an equivalent `@constructor` method, and emits
a non-fatal help hint. `Token::Construct`/its lexer spelling remain only as migration
recognition; they are not canonical declaration syntax.

`construct` and `constructor` are reserved declaration names, selector families, and
attribute-class names. These checks must not reject the legacy declaration keyword;
they apply after recognizing that migration form.
**Contracts on constructors start working** — add a fixture proving it.

`attributes.rs:729` changes `SignatureKind::Initializer(arity)` → `Method(arity)`.
Update decorator docs' implementation-status language in the same landing so they no
longer claim legacy `ConstructDef` internals are canonical or implemented behavior.

### U-CTOR-4 — `new_`; re-home `Class >> new()`; `native_repr`; `duplicate_selector`

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
new name-keyed check in `parse_method_name`. Note the trailing `_` itself is **not** a
new convention — `U-NATIVE-MARKER` establishes it as the native-primitive marker per a
user ruling of 2026-07-13; `new_` follows the house rule.

**No tombstone, no arity guard** — **ruled** (DEC-CTOR-H). `new()` is an ordinary
inherited method: a class need not define it, may override it, and may declare other
`new` overloads alongside it. **Delete** the compile-time guard at `expr.rs:103`; it is
now *wrong*, not merely partial, because it rejects a legal send. `Factory.new()` on a
class whose only constructor is `new(n)` returns an object with every field `None`, and
that is **specified**.

> **This unit no longer closes [`DEFERRED.md:29`](../../../forge/DEFERRED.md).** That row is
> **dissolved by ruling** — the behavior it describes is now correct. Rewrite the row
> to say so; do not silently delete it.

**`native_repr` — the one thing that still refuses.** `new_` builds
`InstanceObject::new(class_id, field_count)`, which is meaningful only where instances
*are* `Object::Instance`. Add `native_repr: bool` to `ClassObject`, set at bootstrap;
`class_new_` raises when it is true.

| Group | Members | `new_` |
|---|---|---|
| Immediate-backed | `Number`, `Bool`, `Symbol`, `None` | ✗ type confusion |
| Native-heap-backed | `Str`, `List`, `Map`, `Set`, `Tuple`, `Range`, `Fiber`, `Method`, `Module`, `Block`/`Closure`, `BoundMethod`, `Upvalue`, `Family`, `Class` | ✗ no native payload |
| Instance-backed | user classes, `Error` | ✓ |

Most native rows already register their own `new` and never reach the generic allocator
(verified: `Number.new()` → `0`, `0 + 1` → `1`). The **exposed** ones register no `new`
at all — `Tuple`, `Block`/`Closure`, `BoundMethod`, `Upvalue`, `Family` — and would
otherwise inherit the allocator and hand back broken objects. **New machinery: no such
flag exists today.**

**`class.duplicate_selector`** (`DuplicateSelector`) and **`class.duplicate_field`** (`DuplicateField`) replace `ConstructStaticCollision` and generic member collision: two members of one
class body may not install the same selector on the same side or declare duplicate fields. Runs on the **post-expansion** list. Derived members need provenance back
to their source member or the message points at synthesized AST.

### U-CTOR-5 — desugar to two methods (required semantics)

```phalcom
@constructor
new(x, y) { _x = x  _y = y }
```
→
```phalcom
@class
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

> **Performance discipline (ADR-0051).** Measure the extra send and optimize it only
> with a guard/deopt path that is observably identical to this lowering. A regression
> does not permit retaining a fused constructor or `Initializer` gate: the lowering is
> the [Classes §1](../../../spec/current/classes.md) contract.

---

## Write-set

| File | Sub-unit | Change |
|---|---|---|
| `phalcom-ast/src/lexer.rs` | 2, 3 | retain `static`/`construct` only for legacy-form recognition; emit help hints and lower to canonical decorators |
| `phalcom-ast/src/token.rs` | 2, 3 | retain migration tokens only; remove their role in canonical member representation |
| `phalcom-ast/src/ast.rs` | 3 | delete `ClassMember::Construct` (`:197`), `ConstructDef` (`:321`); `MethodDef.is_constructor` |
| `phalcom-ast/src/parser.rs` | 2, 3, 4 | lower legacy `static`/`construct` forms with non-fatal help hints; parse canonical `@construct`/`@constructor` targets; `parse_method_name` reserved-name check |
| `phalcom-core/src/compiler/attributes.rs` | 1, 2, 3 | `BuiltinAttr`/`AttrKind` (`Construct`, `Constructor`, `Class`); array registry (`:635`); exhaustive match (`:1550`); class-only `derive_construct`, method-only constructor lowering; `Initializer`→`Method` (`:729`) |
| `phalcom-core/src/compiler/lib/class_decl.rs` | 3, 4, 5 | `Construct` arm → `MethodDef` path (`:615-650`); `duplicate_selector` pre-pass (`:82`+) |
| `phalcom-core/src/compiler/lib/expr.rs` | 4 | **delete** the arity guard (`:103`) — DEC-CTOR-H |
| `phalcom-core/src/compiler/lib/error.rs` | 4 | `ConstructStaticCollision` → `DuplicateSelector` / `DuplicateField`; `ReservedName` |
| `phalcom-core/src/method/mod.rs` | 5 | delete `Initializer` arms of `encode_selector`/`decode_selector` |
| `phalcom-core/src/vm/dispatch.rs` | 5 | delete super-construct metaclass hop + `Initializer` gate |
| `phalcom-core/src/primitive/object.rs` | 4 | **delete** `object_class_new` (`:105`) |
| `phalcom-core/src/primitive/class.rs` | 4 | `class_new` → `class_new_` (`:107`); `native_repr` refusal |
| `phalcom-core/src/heap/class.rs` | 4 | **new** `ClassObject.native_repr: bool` |
| `phalcom-core/src/universe/primitives.rs` | 4 | delete `:47`; `:100` → `new_` |
| `phalcom-core/core/core.ph` | 4 | `class Class { new() => self.new_() }`; codemod 28 sites |
| `phalcom-lsp/src/*` | 3 | 4 `Construct` refs — exhaustive-match fixups |
| `phalcom-core/tests/invariants.rs` | 4 | **R-INV-0.1 selector strings + count (−1)** |
| `docs/spec/current/core/floor-census.md` | 4 | amendment row, net −1 fn/binding |
| `docs/spec/current/decorators/{README,construct,constructor}.md` and related decorator docs | 3 | align legacy-internal status/evidence with PDR-0028 and the new AST/lowering path |
| corpus `.ph` ×~150 files | 2, 3 | recommended one-shot codemod; legacy forms remain accepted with help hints |

---

## Build order

1. **U-CTOR-1** — enum + registry. Green, no behavior change.
2. **U-CTOR-2** — `@class` (methods + fields), canonical codemod with legacy `static` hints retained. **Requires U-BINDINGS landed.**
3. **U-CTOR-3** — collapse `ConstructDef`; class-only `@construct`, method-only `@constructor`; canonical codemod with legacy `construct` hints retained.
4. **U-CTOR-4** — floor: `new_`, delete the duplicate, re-home `new()`, `native_repr`, `duplicate_selector`; **delete** the `expr.rs:103` arity guard. **Gate: bootstrap + `verify_invariants()` + R-INV-0.1 green.**
5. **U-CTOR-5** — required desugar; benchmark and optimize without changing semantics.
6. **U-CTOR-6** — `class` keyword-variable, with legacy class-member recovery kept unambiguous.

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
- `ctor_duplicate_selector.ph` — `@constructor new(x)` + `@class new(x)`
- `ctor_duplicate_class_side.ph` — `@class new(x)` twice (**catches nothing today**)
- `ctor_reserved_new_underscore.ph` — user declares `new_`
- `ctor_target_errors.ph` — `@constructor` header / `@construct` member reject with target diagnostics
- `ctor_legacy_keywords.ph` — legacy `construct` / `static` / class-member `class` declarations compile with non-fatal help hints and preserve meaning
- `ctor_reserved_decorator_names.ph` — `construct` / `constructor` cannot be declared as a user name, selector family, or attribute class

**Runtime lane** — `tests/lang/runtime-errors/`:
- `classfield_inherited_class_unset.ph` — **DEC-CTOR-A2**: `Derived.bump()` on an
  inherited `@class` method touching an unset subclass `@class` field raises
  `None does not understand '+(_)'`. Ratified as correct, so it is pinned here rather
  than fixed.

**Fixtures must be proven wired**, not assumed: the harness runs one test per *lane*
iterating the directory, so a new file can be silently skipped. Corrupt each `.expected`,
confirm the suite reddens, restore ([[phalcom-golden-test-lanes]]).

**Hard gates:** `cargo test` (26 binaries) · `verify_invariants()` · **R-INV-0.1** ·
`cargo clippy --workspace` (13 warnings pre-existing, none new) · `cargo doc` clean
(every new public item needs rustdoc — [[rust-doc-mandatory]]) · `graphify update .`

---

## Decisions (DEC-CTOR) — **reconciled 2026-07-21 by PDR-0028**

| # | Question | Ruling |
|---|---|---|
| **A2** | Inherited `@class` method reading a subclass's unset `@class` field → `None` | **Working as designed — fixture it** |
| **B** | Governing constructor canon | **PDR-0028 wins** — `docs/spec/current` is authoritative; ADR-0063's target-polymorphic surface is superseded. |
| **C** | U-CTOR-5 desugar vs +1 send/construction | **Two-method lowering is semantics.** Measure and optimize, but do not decline it. |
| **D** | Codemod one-shot vs compatibility | **Codemod recommended; legacy forms remain parseable with non-fatal help hints.** |
| **E** | `@construct` vs `@constructor` for the header derive | **`@construct` class-only; `@constructor` method-only.** |
| **F** | One decorator for all class-side placement? | **`@class` for both fields and behavior.** |
| **G/G2** | A `class` keyword-variable? | **Yes — dynamic ≡ `self.class`**, legal everywhere |
| **H** | Does declaring `new(n)` drop the inherited `new()`? | **No** — `new()` is an ordinary inherited method; **tombstone + arity guard deleted** |
| **H2** | May any class bare-allocate? | **Only `Object::Instance`-backed** — via a new `native_repr` flag |
| **I/I2** | `let` on fields unenforced | **`let`/`const` rework → [ADR-0064](../../../adr/accepted/0064-let-const-bindings-and-field-mutability.md), [U-BINDINGS](../../../forge/units/U-BINDINGS/u30-bindings-plan.md), lands first** |

### The A → F reversal, and why the measurement mattered twice

This plan first recommended `@static` on `Field` ("one decorator for both sides"). That
was **wrong**, and a measurement proved it:

```phalcom
class Base { @class _count = 0
             @class bump() { _count = _count + 1 }
             @class count => _count }
class Derived is Base {}
Base.bump()  Base.bump()
Base.count      // 2
Derived.count   // None   <- own slot, not Base's
```

Storage is **per declaring class** — ADR-0011's field rule one tower level up, exactly
as ADR-0017 says. A Smalltalk **class-instance variable**, not a class variable. So
`static`/`@shared`/`@classvar` all connote hierarchy-wide sharing that does not exist.

That killed A's *premise* (`@static` misnames storage) but not its *conclusion*, and
DEC-CTOR-A split the decorators. **F then dissolved the split**: `@class` names
**placement**, not sharing, so one decorator covers both without misnaming anything.
The lesson is that the split was a workaround for a bad name, not a real distinction —
one word that tells the truth beats two that route around a lie.

**A2** ratifies the consequence rather than patching it: `Derived.bump()` raises
`None does not understand '+(_)'`. Untested today — `inheritance_static_*.ph` cover
static *methods* only — and unruled by ADR-0017 (DEC-D settled storage alone). Lock it
with a fixture; do not add a per-class initializer re-run. **G makes this edge easier
to hit** — `class.update(n)` in an inherited constructor is a one-word idiom for it.

### U-CTOR-6 — the `class` keyword-variable (DEC-CTOR-G/G2)

`class` ≡ `self.class`, **dynamic**, a value like `self`/`super`. In an instance method
it is the receiver's class; in a `@class` method it is the metaclass (one rule: always
one tower level above `self`).

- Lexer/parser: `class` is already `Token::Class`. Primary position needs **LL(2)** —
  `class` + IDENT is a declaration, `class` + `.` is the variable. Precedent:
  `parse_property_name` already accepts `.class`/`.try` as selector text.
- Lowers to exactly what `self.class` lowers to. **No new opcode, no dispatch change.**
- Fixtures: `class_var_instance_side.ph`, `class_var_metaclass_in_class_method.ph`, and
  `class_var_inherited_ctor_hits_a2.ph` (pins the crash above as intended).

Sugar only — `self.class.update(n)` already works. It earns its place by pairing with
`@class` as vocabulary. **Lexical binding was rejected**: it would resolve to the
defining class and silently skip subclass overrides — PHP's `self::`/`static::` wart.

---

## What must this not preclude (P4)

- **Niche/NaN-boxing (ADR-0044/0010)** — no new `Value` arm, no tag. Untouched.
- **U-IC inline caches** — *improved*: constructors become ordinary monomorphic sends.
  Two-method lowering must remain ordinary sends or use a guarded optimization with
  exact slow-path equivalence; no constructor-only dispatch path or tombstone returns.
- **ADR-0043 (no default args)** — selector identity remains exact. `new()` and
  `new(_)` coexist as ordinary selectors; inherited `new()` must remain reachable.
- **ADR-0061** — disjoint (leading vs trailing `_`), but if 0061 lands first, the
  `new_` reserved-name check must compose with its prefix ban rather than duplicate it.
- **A future capability boundary** — retiring `Initializer` removes a *kind*, not a
  hook. A dispatch-level boundary (ADR-0061's deferred idea) stays open.
- **Distinct decorator targets** — `@construct` stays class-only and `@constructor`
  stays method-only. The header derive synthesizes a method marked `@constructor`.

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
