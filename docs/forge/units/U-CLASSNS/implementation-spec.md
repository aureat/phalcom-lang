# U-CLASSNS — Implementation spec

Companion to [`plan.md`](plan.md). Governed by
[PDR-0001](../../../pdr/0001-classes-are-closed.md) ruling 1 (**Accepted**), as
amended by [PDR-0002](../../../pdr/0002-class-declarations-join-the-binding-namespace.md).

> **STATUS: SHIPPED 2026-07-20.** `ClassKey` and all four re-keyed tables are on `main`
> (`vm/mod.rs:74`, `:173`, `:238`, `:296`), plus follow-up `14cdfb9`. Unit B shipped after it
> (`7c2cfab`). **This document is the record of intent, not of the tree** — read it for *why*,
> and verify any file:line against `main` before acting.
>
> **One §11 item never landed:** the two invariant tests pinning `sealed_classes`
> (`key.module == value`, and kernel sealing resolving to the core module). `rg sealed_classes
> phalcom-core/tests/invariants.rs` returns nothing. Tracked as
> [`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) item 4.

Unit **A** of two. Unit **B**
([`U-CLASSCLOSE`](../U-CLASSCLOSE/implementation-spec.md)) landed after this one — its
redefinition error is undecidable without `(module, name)` identity.

`plan.md`'s preconditions were re-verified 2026-07-19 against the tree. **Every line number in
`plan.md` has drifted** (U-BINDINGS rewrote `vm/mod.rs` and `class_decl.rs` this week), and the
census found **two more tables and one compiler field** that `plan.md` does not name and that
cannot be left behind. Both changes are in [§1](#1-corrections-to-planmd--read-this-first).
**Read §1 before §2.**

> **Tree state.** Written against `77b7030`. `d14599b` (PDR-0002) plus two `docs(learn)`
> commits sit between it and the SHA `plan.md` cites; all three are documentation-only, so every
> code anchor below is against live source, re-read at `77b7030`. Main has **live concurrent
> sessions** — re-verify before starting, commit narrow paths on `main` itself, never
> `git checkout -b`, never `git add -A`.

---

## 0. Preconditions — re-verified 2026-07-19 at `77b7030`

| # | `plan.md` claim | Verdict | Evidence |
|---|---|---|---|
| P1 | Four VM tables are name-keyed, VM-global | ✅ holds, **line numbers wrong** | `classes` `vm/mod.rs:107` (plan says :101), `field_layouts` `:157` (:136), `class_parents` `:193` (:172), `sealed_classes` `:215` (:194) |
| P2 | Bindings are already module-scoped | ✅ holds | `define_global`, `vm/api.rs:128` |
| P3 | The two-module `Point` bug reproduces | ✅ holds | unchanged since 0065's repro; re-run it as the unit's before/after |
| P4 | `file = module` (ADR-0045), one import form | ✅ holds | `phalcom-ast/src/parser.rs:511` hard-requires `as` |
| P5 | Two runtime readers of `vm.classes` | ✅ holds | `dispatch.rs:768` (reopen fallback, unit B deletes), `dispatch.rs:885` (`SuperSend`) |
| P6 | `sealed_classes` ownership carries no conflict — bootstrap's `m` and the compiler's `self.module` are the same handle | ✅ holds, **do not re-derive** | `bootstrap.rs:175` → `run_core_module` → `get_module_from_str` → `api.rs:117` `modules.get(&sym).copied()` |
| P7 | LSP keeps its own name-keyed index and never resolves `import` | ✅ holds | `ClassMap` `phalcom-lsp/src/index.rs`; `Statement::Import` is a no-op in every walker |
| P8 | ~40 call sites | ❌ **wrong — 62 across the four tables**, plus the additions in §1.2/§1.3 | full census, §1.1 |

---

## 1. Corrections to `plan.md` — read this first

### 1.1 The site count is 62, not ~40, and it is concentrated in three files

Full workspace census (production + tests + benches + fuzz + tools):

| Table | Sites | Where |
|---|---|---|
| `classes` | 24 | `vm/` 10 · `compiler/` 3 · `tests/` 11 |
| `field_layouts` | 13 | `vm/` 4 · `compiler/` 7 · `tests/` 2 |
| `class_parents` | 13 | `vm/` 3 · `compiler/` 10 |
| `sealed_classes` | 12 | `vm/` 7 · `compiler/` 5 |
| **total** | **62** | |

**Zero sites in `phalcom-repl`, `phalcom-lsp`, `fuzz/`, `benchmarks/`, `phalcom-core/bin/`.**
The LSP work in §8 is therefore *entirely independent* of the VM work — `phalcom-lsp` has no
`phalcom-core` dependency at all, deliberately (ADR-0056 §2, stated in its `Cargo.toml`). Order
the two halves however is convenient; they cannot conflict.

> **A grep for `classes` overcounts by 15×.** `vm.universe.classes` is a `CoreClasses` *struct*
> of named `ClassId` fields (`universe/mod.rs:33`), not a map. A bare `\bclasses\b` returns 369
> workspace hits; the actual VM table is 24. Any sizing derived from a naive grep is wrong.

### 1.2 There are six tables, not four — and the two extra ones are load-bearing

`plan.md` names four. Two more are keyed on class-name `Symbol` and are walked **interleaved
with `class_parents` in the same loop bodies**:

```rust
// vm/mod.rs:172
pub constructor_aliases: HashMap<(Symbol, Symbol), Symbol>,
// vm/mod.rs:180
pub has_new_construct: std::collections::HashSet<Symbol>,
```

Both are read inside the ctor-inherit chain-walks at `class_decl.rs:924-930` and `:958-964`,
alongside `class_parents`. If `class_parents` becomes `ClassKey`-keyed and these stay
`Symbol`-keyed, **those two walks stop compiling** — the loop variable's type changes underneath
them. Their re-key treatment is ruled in [§5](#5-the-two-interleaved-tables).

This is the single largest divergence from `plan.md`'s sizing. Budget **+8–10 sites**.

### 1.3 `Compiler::current_class` is the seventh site and `plan.md` misses it

```rust
// compiler/lib/mod.rs — set at class_decl.rs:438, read at expr.rs:255 and :313
current_class: Option<Symbol>,
```

It is the **lookup key** for `field_layouts` at `expr.rs:258` and `:316`. Once `field_layouts`
is `ClassKey`-keyed, `current_class` has nothing to look up with. It must become
`Option<ClassKey>` in the same commit as `field_layouts`, or step 1 of the build order does not
compile.

### 1.4 `ClassLayout` has exactly one construction site, and no `Default`

Good news `plan.md` does not claim. `ClassLayout` (`vm/mod.rs:27`) derives only
`#[derive(Debug, Clone)]` — **no `Default` impl, no helper constructor**, and exactly one
construction anywhere in the workspace:

```
compiler/lib/class_decl.rs:427 — struct literal, all fields named, no `..Default::default()`
```

So §7's `SourceRange` field touches one site. It also already grew a field this week —
`const_fields: HashSet<Symbol>` landed in `42aafce` (U-BINDINGS) — so the struct is six fields
today, seven after this unit, and the precedent for adding compile-metadata to it is fresh.

### 1.5 The sole `field_layouts` insert moved: `:424` → `:436`

`plan.md`, PDR-0001, and the U-CLASSCLOSE plan all cite `class_decl.rs:424` as the sole
`field_layouts.insert`. U-BINDINGS moved it. Current anchors:

| What | Line |
|---|---|
| `ClassLayout` struct literal | `class_decl.rs:427` |
| `field_layouts.insert` (sole site) | `class_decl.rs:436` |
| `class_parents.insert` (sole site) | `class_decl.rs:406` |
| `sealed_classes.insert` (compiler side) | `class_decl.rs:774` |
| `field_layouts.contains_key` reopen guard | `class_decl.rs:288` |
| reopen branch (layout reuse) | `class_decl.rs:319` |
| sealed cross-unit check | `class_decl.rs:376` |
| `current_class` set | `class_decl.rs:438` |

**Anchor by symbol, not by line.** These moved once this week and will move again — every
citation in this spec names the function or the statement, and the line is a convenience.

### 1.6 `gc.rs` is a compiler-enforced tripwire, not a hazard

`vm/gc.rs:55-95` destructures `self` exhaustively with **no `..` rest pattern**. Every VM field
appears by name. Changing a field's *type* will not break it, but it means:

- the GC file must be in the **same commit** as any field-shape change, and
- it is the one place that hard-errors rather than silently drifting.

Two of the four tables are iterated there for GC roots — `classes.values()` (`:112`) and
`sealed_classes.values()` (`:113`) — both collecting into an unordered `Vec`. **Order-insensitive**,
so the re-key is safe there provided the new key yields the same *value* set. `field_layouts`
(`:79`) and `class_parents` (`:82`) are bound to `_` with an explicit "declared non-root" note;
that stays true.

### 1.7 Nothing prints, serializes, or snapshots these tables

Verified negative, and worth recording because three sites look like hits and are not:

- `phalcom-core/bin/gen-core-table/main.rs:61,405` emits a `"classes"` JSON object — from its
  **own** `BTreeMap` scraped textually out of `core.ph` and `primitive/*.rs`. It never
  constructs a `VM`. Unaffected.
- `phalcom-lsp/src/index.rs` / `core_table.rs` keep an independent `String`-keyed index. VM-free
  by design. §8 changes it for its own reasons, not because of the re-key.
- `bin/phalcom/disasm.rs` prints chunk constants and instructions only.

No `impl Debug for VM`, no `serde`, no `insta` snapshot over any of the six. **There is no
golden-output blast radius in this unit** — a stdout diff means a real behavior change, not a
formatting one.

### 1.8 `create_class`'s real callers are three, not two

`plan.md` treats `create_class`/`create_single_class` as an API-surface problem. Measured:

| Site | Module handle in scope? |
|---|---|
| `vm/api.rs:59`, `:62` | **no** — these are `create_class`'s own internal calls to `create_single_class` |
| `vm/dispatch.rs:786` (`Bytecode::Class` arm) | **yes, derivable** — `closure_id` is in scope; `self.heap.closure(closure_id).module` is the exact expression the `Import` arm already uses at `:800` |
| `tests/invariants.rs:248,249,250,298,299` | **no** — only `vm` |
| `tests/invariants.rs:514,574` | **yes** — local `module`, created at `:498` / `:561` |

`create_single_class` has no callers outside `create_class`. So the signature change reaches
**one production call site** and **seven test call sites**, five of which must create or fetch a
module handle they do not have today. §6 rules how.

### 1.9 `sealed_classes`'s value becomes redundant with its key — deliberately keep it

Today `sealed_classes: HashMap<Symbol, ObjRef>` maps class name → the module handle whose
`Compiler::compile` declared it. Its own doc comment (`vm/mod.rs:194-214`) explains the value's
job precisely: a module `ObjRef` is *"a natural, already-unique-per-compile-unit identifier"*.

Under a `(module, name)` key, **the key's module component is that same compile unit**, and the
check at `class_decl.rs:376` (`sealed_in_module != self.module`) compares a value that is now
derivable from the key. Every writer confirms it: the compiler writes `self.module` while
compiling that class (`:774`), and bootstrap writes `m` for rows keyed to `m`
(`bootstrap.rs:224,229,270`).

**Ruling for this unit: keep the value, do not collapse to `HashSet<ClassKey>`.**

- Collapsing is a *semantic* change disguised as a cleanup — it makes "who sealed it" unstateable
  separately from "who owns it", and this is a correctness gate, not a refactor.
- The redundancy is worth *pinning* rather than exploiting: §11 adds an invariant test asserting
  `key.module == value` for every row. That makes a future collapse safe and mechanical instead
  of a re-derivation.
- `plan.md` §3.1 already rules "do not collapse them" for the same reason. This section only
  records *why* the redundancy appears, so the next reader does not mistake it for a bug.

### 1.10 "Re-key mechanically" is wrong for four of the sites

`plan.md` frames the VM half as a mechanical key swap. It is, for 58 of the 62 sites. Four are
not, and getting them wrong produces **silent slot aliasing** rather than a build failure:
`class_decl.rs:343`/`:440` key on the class *being declared* (no fallback), while `:384`/`:386`
key on the *superclass* (own module, then core). One uniform helper across all four either
silently reopens the kernel or silently mislays every subclass field.

This is the unit's real risk, and `plan.md` does not mention it. [§4](#4-superclass-resolution--the-load-bearing-question)
is the ruling; §11's slot-aliasing fixture is the gate. **Read §4 before writing any code.**

---

## 2. `ClassKey` — the newtype

```rust
/// Identity of a class: the module that declares it, plus its name.
///
/// Class *bindings* have always been module-scoped (`VM::define_global`
/// writes into the module object's own globals), but class *identity* was
/// keyed by bare name VM-wide, so two modules declaring the same class name
/// silently collapsed into one class
/// ([PDR-0001](../../../docs/pdr/0001-classes-are-closed.md)
/// ruling 1). This key restores the symmetry: since file = module
/// (ADR-0045), "same module" and "same file" are the same check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassKey {
    /// The declaring module's [`ModuleObject`](crate::heap::ModuleObject) handle.
    pub module: ObjRef,
    /// The class's name, interned.
    pub name: Symbol,
}
```

Both components are `Copy + Eq + Hash`, so every derive above is free and no manual impl is
needed:

- `ObjRef` is `slotmap::new_key_type!`-generated (`heap/mod.rs:64-73`), which emits
  `#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]`.
- `ClassId` is a plain alias — `pub type ClassId = ObjRef;` (`heap/mod.rs:80`), **not** a
  newtype. Worth knowing: `ClassKey.module` and a `ClassId` are the same Rust type, so the
  compiler will **not** catch a swapped argument. Construct with named fields, never
  positionally, and never add a `ClassKey::new(a, b)` two-positional-arg constructor.

**A bare tuple is rejected** for the reason `plan.md` gives (62+ positional sites), and the
`ClassId`-alias hazard above makes it sharper: `(ObjRef, Symbol)` and a hypothetical
`(ClassId, Symbol)` are indistinguishable to the type system.

Place it in `vm/mod.rs` beside `ClassLayout`. Add a `ClassKey` helper on `Compiler` for the
overwhelmingly common "this module, this name" case:

```rust
/// The [`ClassKey`] for `name` in the module currently being compiled.
fn class_key(&self, name: Symbol) -> ClassKey {
    ClassKey { module: self.module, name }
}
```

Most of the 62 sites become `self.class_key(sym)`, which is why the newtype pays for itself.

---

## 3. The six tables after the re-key

| Table | Before | After |
|---|---|---|
| `classes` | `HashMap<Symbol, ClassId>` | `HashMap<ClassKey, ClassId>` |
| `field_layouts` | `HashMap<Symbol, ClassLayout>` | `HashMap<ClassKey, ClassLayout>` |
| `class_parents` | `HashMap<Symbol, Symbol>` | see §4 |
| `sealed_classes` | `HashMap<Symbol, ObjRef>` | `HashMap<ClassKey, ObjRef>` (value kept — §1.9) |
| `constructor_aliases` | `HashMap<(Symbol, Symbol), Symbol>` | see §5 |
| `has_new_construct` | `HashSet<Symbol>` | see §5 |

Every doc comment on these fields names "class names (by `Symbol`)". **All six doc comments are
rewritten in this unit** — `cargo doc --workspace --no-deps` must be clean and the prose must
stop saying "name".

---

## 4. Superclass resolution — the load-bearing question

**This is the part of the unit that can corrupt memory rather than fail to compile. Read it
before touching a key.**

### 4.1 The hazard

A user module writes `class Sub extends List`. `List` is a kernel class owned by the core module.
Four sites in `class_decl.rs` read the class tables by name, and **they do not all mean the same
name**:

| Site | Keys on | Resolution rule |
|---|---|---|
| `:343` | the class **being declared** | `(self.module, name)` — **no fallback** |
| `:440` | the class **being declared** | `(self.module, name)` — **no fallback** |
| `:384` | the **superclass** | own module, **then core** |
| `:386` | the **superclass** | own module, **then core** |

Apply one uniform helper to all four and a user module's `class List { … }` silently reopens the
kernel `List` — the exact defect this unit exists to remove.

Get the *other* direction wrong and it is worse. `:384`/`:386` are the sole source of
`sc_field_count`, which becomes **every subclass field's slot offset** (`:429-436`). Today they
cannot disagree with the runtime because the tables are module-blind. Re-key them naively and
`(user_module, "List")` misses, so the compiler computes offsets from `sc_field_count = 0` while
the VM wires the real `List` as the parent at runtime (§4.3). That is **silent slot aliasing** —
a subclass field and an inherited field share an offset. It is not a compile error, not a panic,
and not visible in any golden until a field read returns another field's value.

### 4.2 The resolution rule, and why it is complete

The superclass compile-time lookup must be **byte-for-byte the resolution `GetGlobal` performs**:
own module, then core, then error. That is `dispatch.rs:653-668`:

```rust
let (resolved_module, slot) = match self.heap.module(module_id).slot_of(name_sym) {
    Some(slot) => (module_id, slot),
    None => {
        // Not in the current module — try the core module.
        let core_module_sym = self.interner.intern(CORE_MODULE_NAME);
        let core_module = self.get_module(core_module_sym).expect("core module");
        match self.heap.module(core_module).slot_of(name_sym) { /* … */ }
```

Two facts make own-module-then-core the **complete** resolution space, not a heuristic:

1. **A superclass can only ever be a bare identifier.** `SuperclassRef { name: String, range }`
   (`phalcom-ast/src/ast.rs:185-190`), parsed by a single `expect_identifier`
   (`parser.rs:911-914`). `class Sub extends P.Point` is a **parse error** today ("expected
   `{`"). There is no qualified form to resolve.
2. **The compiler performs no binding resolution at all.** `expr.rs:242-251`: not-a-local,
   not-an-upvalue ⇒ emit `GetGlobal` unconditionally, without ever checking whether the global
   exists in any module. So there is no existing resolver to defer to — this unit writes the
   compile-time mirror of the runtime rule.

Add one helper, used **only** at `:384`/`:386`:

```rust
/// Resolve a superclass name to the [`ClassKey`] that actually owns it.
///
/// Mirrors the runtime global-resolution order (`vm/dispatch.rs`,
/// `Bytecode::GetGlobal`): the compiling module first, then the core
/// module. A superclass is always a bare identifier
/// (`phalcom_ast::ast::SuperclassRef`), so those two are the complete
/// resolution space.
///
/// **Not** for the class being declared — see
/// `docs/forge/units/U-CLASSNS/implementation-spec.md` §4.1.
fn resolve_superclass_key(&self, name: Symbol) -> Option<ClassKey> { /* … */ }
```

Name the *other* rule too, so the distinction is stated in code rather than remembered:
`self.class_key(name)` (§2) is the declaring-module key, and its doc comment must say it never
falls back.

> **Rejected: resolving via the binding.** "Ask which module the name resolves to, then key on
> that" is the conceptually right answer and is not implementable — there is no compile-time
> binding resolution to ask (fact 2). The two-step *is* that resolution, written out.

### 4.3 What the runtime does — and why it is already safe

Nothing to change. The superclass object comes **from the stack**, not from a table: the compiler
emits `GetGlobal(sc_name)` (`class_decl.rs:453`) or a `Constant` holding `Object` (`:457`), and
`Bytecode::Class` pops it (`dispatch.rs:744-746`). `FinalizeClass` never touches the tables
either — it reads the class off the stack top and recomputes `base_names` from the already-wired
heap superclass (`dispatch.rs:856-858`).

So the runtime already resolves superclasses correctly under module scoping, **because it goes
through the global-lookup path with its core fallback**. §4.2 exists precisely to make the
compile-time path agree with it. The two must be verified to agree — §11's slot-aliasing fixture
is that verification, and it is the single most important test in this unit.

### 4.4 `class_parents` becomes `ClassKey → ClassKey`

`plan.md` leaves this open. It cannot stay `Symbol → Symbol`:

- its whole purpose is disambiguating classes, so leaving it name-keyed reintroduces exactly the
  collision the unit removes — two modules' `Point` sharing one parent edge;
- the chain-walks at `:927`/`:961` would step from a `ClassKey` cursor into a bare-`Symbol` map,
  and would not compile.

**Store the resolved key** from §4.2's two-step at the sole insert site (`:406`) — not
`self.class_key(sc_name)`. That is what lets a walk cross from a user subclass into its core
parent, which the ctor-inherit guard depends on.

---

## 5. The two interleaved tables

Ruled from the walk sites (`class_decl.rs:918-932` `inherits_new_construct`, `:955-966`
`lookup_constructor_alias`) and the sole populate site (`:640-644`):

| Table | Before | After | Why |
|---|---|---|---|
| `constructor_aliases` | `HashMap<(Symbol, Symbol), Symbol>` | `HashMap<(ClassKey, Symbol), Symbol>` | **partial** — only the first tuple element is a class name |
| `has_new_construct` | `HashSet<Symbol>` | `HashSet<ClassKey>` | **full** — its only element is a class name |

The tuple is two different kinds of thing: `(class name, call-site selector) → installed
selector`. **Selectors stay bare `Symbol`.** They are globally interned with no module scope, and
re-keying them would break the `encode_selector` identity dispatch relies on. This is the most
likely place to over-apply the re-key.

Both are forced by the cursor: `class_sym` in each walk becomes a `ClassKey` the moment
`class_parents` does (`:920`, `:957` test the same cursor the `class_parents.get` at `:923`/`:960`
advances).

Both populate sites (`:641`, `:643`) key off `self.current_class` — which is the second
independent reason that field must become `Option<ClassKey>` (§1.3 is the first).

**Do not harden the walks.** `plan.md`'s rubric and `DEC-CTOR-H` both apply: this guard is
scheduled for deletion in U-CTOR-4. Change the key type and nothing else. If a walk needs more
than a key change, **stop and flag**.

---

## 6. `create_class` gains a module parameter

Signature (`vm/api.rs:39`):

```rust
pub fn create_class(&mut self, module: ObjRef, name: &str, /* … */) -> ClassId
```

The two internal `create_single_class` calls (`api.rs:59,62`) thread it through; the inserts at
`api.rs:76-77` key on `ClassKey { module, name }` for the class and its metaclass.

**Production call site — one.** `dispatch.rs:786`, the `Bytecode::Class` arm. The module is
`self.heap.closure(closure_id).module`, the identical expression the `Import` arm uses at
`:800`. Do not invent a "current module" accessor for this; use the closure's, which is what
"the module this code was compiled in" means everywhere else in the VM.

**Bootstrap sites.** `add_class!` (`bootstrap.rs:187`) and the `None` row (`:266`) run inside
`install_core`, where the core module handle `m` is a local (`:175`). Thread `m` explicitly at
each — do **not** add a `VM::current_module` field or default to "the first module". P6 already
proves `m` is the same handle the compiler holds while compiling `core.ph`, so the rows bootstrap
writes and the rows `core.ph` completes land under the identical key. That equality is the whole
reason stub completion keeps working; it is worth an assertion (§11).

**Test call sites — seven.** Five (`invariants.rs:248,249,250,298,299`) have no module handle.
Give them one from the same helper the other two use (`invariants.rs:498`/`:561` create a
`module` local); do not add a test-only default. A test that does not say which module its class
belongs to is a test that would not have caught the bug this unit fixes.

---

## 7. `ClassLayout` gains a declaration span (DEC-CLASSNS-A, option (i) — ruled)

Add a seventh field:

```rust
/// Source range of the `class` declaration that produced this layout.
///
/// Stored by this unit, consumed by
/// [`U-CLASSCLOSE`](../../../docs/forge/units/U-CLASSCLOSE/implementation-spec.md):
/// PDR-0001 ruling 2's `X is already defined` diagnostic carries **both**
/// spans, and the *first* declaration's location is otherwise unrecoverable —
/// no sibling map records it. Dead until unit B lands; that is intentional,
/// so the struct rewrite happens once.
pub declared_at: SourceRange,
```

Populated at the sole construction site (`class_decl.rs:427`) from `class_def.range`, already in
scope there. **No diagnostic work in this unit.** Unit B reads it.

Rejected alternatives, per `plan.md` §3.4 and PDR-0002: a separate `ClassKey → SourceRange`
map owned by unit B (the same key twice, no separation of concerns, and B would be editing a
struct A just rewrote), and degrading to a single-span diagnostic (contradicts a ruling — and if
that becomes necessary it is an amendment to PDR-0002, not a quiet under-delivery).

Note the field is `SourceRange`, not `Option<SourceRange>`: there is one construction site and it
always has a range, so an `Option` would add a `None` case no code can produce.

---

## 8. LSP — collapse, do not just re-key

Independent of everything above (§1.1: no shared dependency). `ClassMap`'s
`DashMap<String, Vec<ClassEntry>>` exists **solely** to model one class reopened across several
files. Under 0065 that cannot happen, so the `Vec` is not re-keyed — it is **removed**:

```
DashMap<String, Vec<ClassEntry>>   →   DashMap<(Url, String), ClassEntry>
```

`Url` is the correct and only module proxy: the LSP never resolves `import` (`Statement::Import`
is a no-op in every walker), and `ClassEntry` already carries `uri: Url`.

**This fixes two live wrong-answer bugs**, independent of the rest of the unit — they are
reproducible today and worth their own fixture:

- `ClassMap::members` unions members across every file declaring the name, de-duping
  first-seen-wins. `p.<cursor>` on file A's `Point` can offer file B's members.
- `ClassMap::parent` is `.find_map(|e| e.parent.clone())` — returns the first entry that has an
  `extends`, so file B's superclass answers queries about file A's `Point`.

`ClassMap::remove_uri` already filters on `entry.uri != uri`, so invalidation is not a blocker; it
simplifies under the new shape.

`WorkspaceIndex::class_members` / `class_parent` / `has_class` grow a `uri` parameter;
`collect_class_members` (`completion.rs:449-474`) threads it from `Backend::completion`, which
already has the request `uri` in scope.

**Out of scope:** `core_table.rs`'s `classes: HashMap<String, Vec<CoreMember>>`. Kernel classes
only, process-global, and the VM has no per-module identity for them either. Legitimately
name-keyed — leave it.

---

## 9. Write-set

| Path | Change |
|---|---|
| `phalcom-core/src/vm/mod.rs` | `ClassKey` type; six field type changes; six doc-comment rewrites; `ClassLayout.declared_at` |
| `phalcom-core/src/vm/api.rs` | `create_class`/`create_single_class` module param; inserts at `:76-77` |
| `phalcom-core/src/vm/bootstrap.rs` | `add_class!` (`:187`), `None` row (`:266`), sealed rows (`:224,229,270`) — all keyed to `m`; the four `HashMap::new()` at `:36,43,46,47` |
| `phalcom-core/src/vm/gc.rs` | exhaustive destructure (`:55-95`); root iteration `:112-113` |
| `phalcom-core/src/vm/dispatch.rs` | `Bytecode::Class` arm (`:768,786`) re-keyed **in place** — unit B deletes it; `SuperSend` probe (`:885`) |
| `phalcom-core/src/compiler/lib/mod.rs` | `current_class: Option<ClassKey>`; `class_key()` helper |
| `phalcom-core/src/compiler/lib/class_decl.rs` | ~20 sites — the reopen guard, both inserts, superclass reads, sealed check, chain-walks, `current_class` set |
| `phalcom-core/src/compiler/lib/expr.rs` | `field_layouts` reads at `:258`, `:316` |
| `phalcom-core/src/compiler/attributes.rs` | `ExpandCtx` borrowed fields (`:61,82`); `class_parents` walks (`:1465,1473,1705,1800`); `sealed_classes` read (`:1678`) |
| `phalcom-core/tests/invariants.rs` | 11 `classes` reads, 2 `field_layouts` reads, 7 `create_class` calls |
| `phalcom-core/tests/contracts_metadata.rs` | one `classes` read (`:45`) |
| `phalcom-lsp/src/index.rs` | `ClassMap`/`ClassEntry` collapse + 5 inherent methods + 3 pub wrappers + insert/remove |
| `phalcom-lsp/src/completion.rs` | `collect_class_members` signature + 3 call sites |

**Not** in the write-set: `phalcom-core/core/core.ph` (this unit does not touch it — no conflict
with any `.ph`-editing unit in either order), `core_table.rs`, `hover.rs`, `backend.rs` beyond
threading one `uri`, `phalcom-repl`, `fuzz/`, `benchmarks/`.

---

## 10. Build order

Each step is an independently-green commit. Verify each SHA **in a throwaway worktree**, not
in-tree — an in-tree gate hides a partially-staged commit.

1. **`ClassKey` + `resolve_superclass_key` + `field_layouts` + `Compiler::current_class`.** Four
   things, one commit, by necessity: `current_class` is `field_layouts`'s lookup key (§1.3), and
   `field_layouts`'s superclass read at `:384` is one of the two sites that needs the core
   fallback (§4.2) — re-keying it without the resolver is the slot-aliasing bug. Compile-time
   only; no runtime reader of `field_layouts` exists. **Land the slot-aliasing fixture in this
   step**, not at the end.
2. **`classes`, including the Rust-side kernel inserts and `create_class`'s new parameter.**
   `:386`'s fallback and `:343`/`:440`'s no-fallback rule both land here. Green means bootstrap
   survives, which is the real assertion.
3. **`class_parents` (→ `ClassKey → ClassKey`, storing the resolved key) + the two interleaved
   tables (§5) + `sealed_classes`.** Add the sealed-kernel invariant test and the
   `key.module == value` invariant (§1.9) in this step.
4. **`SuperSend` probe** (`dispatch.rs:885`). **The only dispatch-path diff — commit alone**, so
   a perf or correctness regression names its own cause.
5. **LSP collapse + API threading.** Independent of 1–4 (§1.1); may run in parallel or land
   first.

`graphify update . --no-cluster` after the last code commit.

---

## 11. Tests

**The unit's reason to exist** — positive lane, two modules, same class name, no interaction:

```phalcom
import "modp" as P     // class Point { who => "from modp" }
import "modq" as Q     // class Point { who => "from modq" }
System.print(P.Point.new().who)     // from modp
System.print(Q.Point.new().who)     // from modq
System.print(P.Point == Q.Point)    // false
```

All three lines are wrong today (`from modq`, `from modq`, `true`). This is the before/after the
return contract must show.

Further positive lane:

- the importer declaring its **own** `Point` leaves `P.Point` intact;
- two modules each with a `Base`/`Derived` pair of identical names — `SuperSend` resolves the
  right parent (this is the fixture that would catch a mis-keyed `:885` probe, and nothing else
  would);
**The slot-aliasing fixture — the most important test in this unit** (§4.1). A user module
subclassing a **kernel** class, where the subclass declares its own fields and reads both its own
and an inherited one:

```phalcom
class Sub extends List {
  _tag
  construct new(t) { _tag = t }
  tag => _tag
}
```

If §4.2's core fallback is missing, `sc_field_count` is `0`, `_tag` lands on an inherited slot,
and the read returns the wrong value **with no error anywhere**. Assert the field value, not just
that it compiles.

> **Use `List`, not `Option`.** `Option` is registered `@sealed` to the core module at bootstrap
> (`bootstrap.rs:221-225`), so `class Sub extends Option` is *already* rejected today —
> `attr.sealed_violation: 'Sub' extends '@sealed' class 'Option', but was not declared in the
> same compilation unit`. A fixture built on it would pass for the wrong reason and prove
> nothing. `class Sub extends List` compiles and runs at HEAD; verified 2026-07-19.

**Rust-level invariants** (`tests/invariants.rs`):

- kernel sealing still resolves to the core module after the re-key —
  `sealed_classes` for `Option`/`Some`/`None` (per `plan.md`'s rubric);
- **`key.module == value` for every `sealed_classes` row** (§1.9) — pins the redundancy so a
  future collapse is mechanical;
- **bootstrap's `m` and the compiler's `self.module` for `core.ph` are the same handle** (P6) —
  currently a verified-but-unasserted fact, and the fact that makes stub completion work. Assert
  it rather than re-deriving it a third time.

**LSP unit tests** — two same-named classes in two `uri`s, asserting `class_members` returns only
the queried file's members and `class_parent` only the queried file's superclass. `index.rs`'s
existing test module never exercises this case, which is why both bugs are live.

**Negative lane:** nothing new. This unit does **not** error on redefinition — a same-module
duplicate keeps today's behavior until unit B. Resist adding it here; the diagnostic wants both
spans and that is B's design.

Error fixtures go in the **negative** subdir or the suite reddens.

---

## 12. Gates

- `./scripts/verify.sh` exits 0 (build + full `cargo test --workspace` + clippy).
- `cargo doc --workspace --no-deps` clean — all six re-keyed fields, `ClassKey`,
  `ClassLayout.declared_at`, and the changed `create_class` signature carry full rustdoc.
- clippy: no **new** warnings (a pre-existing baseline exists; count it before starting).
- No golden stdout moves — §1.7 establishes there is no formatting blast radius, so **any**
  stdout diff in this unit is a real behavior change and must be explained, not blessed.
- `graphify update . --no-cluster`.

---

## 13. What must this not preclude

- **Unit B** needs `(module, name)` to make its redefinition error decidable, and
  `ClassLayout.declared_at` to render both spans. Both delivered.
- **The reflection layer** (0065 ruling 7) — user classes only. Module-scoped identity is exactly
  what makes "user class" a checkable predicate instead of a naming convention.
- **`SuperSend` `ClassId` stamping** ([`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md)
  item 1) — re-keying the probe must not close it. **Re-key only; do not optimize.** The site's
  comment justifies the name lookup by a future `superclass=` mutation that 0065 makes
  impossible, so the probe *could* become a stamped `ClassId` — that is an unmeasured
  dispatch-path change and this is a correctness gate.
- **Post-bootstrap freeze + H17** (item 2) — out of scope, unmeasured. Promise nothing.
- **`DEFERRED` #35** — P6 resolves its ownership unknown; the sealing-representation unification
  stays reachable, and §1.9's invariant test makes it cheaper.
- **U-CTOR-4** — `class_parents` feeds the ctor-inherit guard chain-walk, already flagged fragile,
  and `DEC-CTOR-H` schedules that guard for **deletion**. Re-key it mechanically. If the walk
  needs more than a key change, **stop and flag** rather than redesigning a guard that is
  scheduled to be deleted.

---

## 14. Issues flagged for the user

1. **The unit is larger than `plan.md` prices it, and the extra work is not the boring kind.**
   62 sites instead of ~40 (§1.1), plus two tables and one compiler field the plan does not name
   (§1.2, §1.3), plus a compile-time resolver that has no existing counterpart to copy (§4.2).
   The sizing miss is not the concern — the two-rule resolution split (§1.10) is. It is the one
   thing in this unit that fails silently.

2. **`SuperSend`'s probe is on the dispatch hot path** (`dispatch.rs:885`). Re-keying it turns a
   `HashMap<Symbol, _>` lookup into a `HashMap<ClassKey, _>` lookup — a wider key, hashed per
   super send. Unmeasured. Step 4 commits it alone specifically so it can be measured in
   isolation if `skynet` moves. **Do not** pre-emptively "fix" it with the stamped-`ClassId`
   optimization that `class-sealing-followups.md` item 1 defers — that is a different change
   with a different risk profile.

3. **`Option` cannot be used in any subclassing fixture** (§11). It is `@sealed` to core, so
   `class Sub extends Option` already errors at HEAD for an unrelated reason. Every doc that
   reaches for a kernel-subclassing example should use `List`. Worth knowing before writing unit
   B's reserved-name fixtures too.

---

## 15. Return contract

Per-step SHAs with `git show --stat`. The two-module reproduction, before and after. Confirmation
that `./scripts/verify.sh` and `cargo doc --workspace --no-deps` are clean **at each SHA, verified
in a throwaway worktree**. The LSP test for the two-same-named-classes case, with the pre-fix
wrong answer quoted. Explicit confirmation that `core.ph` is untouched, that `SuperSend` was
re-keyed and **not** optimized, and that no golden `.expected` was re-blessed.
