# 45. `import` resolves by relative file path and binds a whole `Module`; amend the frozen floor +1 (`Module#doesNotUnderstand`)

- Status: Accepted
- Date: 2026-07-12
- Supersedes (partially): [ADR-0027](0027-modules-as-files-with-public-by-default-imports.md)
  — replaces its resolution mechanism (§2/§4: logical-name resolver) and import-form
  grammar (§3: qualified/selective/aliased) with the smaller Draft-0.1 surface below.
  ADR-0027's "core is the auto-imported module" framing and "file = module" premise
  are **kept unchanged**.
- Amends: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (the frozen floor);
  amendment precedent: [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md),
  [ADR-0037](0037-amend-floor-admit-error-root.md),
  [ADR-0038](0038-amend-floor-admit-block-on-ensure.md),
  [ADR-0039](0039-amend-floor-admit-collection-container-primitives.md)
- Related: `docs/spec/v0.2/open-questions.md` Q8; `docs/spec/v0.2/object-model.md` §4
  (`Module` catalog row); `docs/forge/units/U15/plan.md`; `docs/forge/STATE.md`
  (DEC-U15 ruling, 2026-07-12); `phalcom-core/src/module.rs`,
  `phalcom-core/src/interpret.rs::VM::import_module`,
  `phalcom-core/src/primitive/module.rs`

## Context

The `import` token has existed in the lexer since the front-end's earliest commits
but carried no parser production, no AST node, and no runtime semantics —
open-question **Q8**. [ADR-0027](0027-modules-as-files-with-public-by-default-imports.md)
answered Q8 at the *design* level: file = module, top-level names public by default
(`_`-prefix private), three import forms (qualified, selective, aliased) over a
**logical-name resolver** (`import geometry` → some search-path lookup, not a raw
filesystem path in source).

Dispatching the **implementation** unit (U15, `docs/forge/units/U15/plan.md`) surfaced
that ADR-0027's full surface is **BLOCKED-ON-DECISION**: a logical-name resolver needs
a package/search-path notion Phalcom does not have yet (no `phalcom.toml`, no
installed-package directory), and a three-form grammar (qualified + selective +
aliased) is the *largest* defensible surface, not the smallest. The plan's architect
recommendation, put to the user as **DEC-U15**, was the smallest coherent model:
relative file-path resolution + whole-module binding only, with everything top-level
public (no `export` keyword, no `_`-prefix privacy enforcement yet). The user's
autonomous-authority ruling (`docs/forge/STATE.md`, "Design forks — RESOLVED", session
2026-07-12) accepted the recommendation: **DEC-U15 → A + A**.

Separately, once "how does a message like `math.pi` find `pi` in an imported unit's
top level" is worked out, the mechanism turns out to fail
[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) §1's **derivability test**: a
module's members live in [`ModuleObject`](../../phalcom-core/src/module.rs)'s own
`globals`/`name_to_slot` table, which is Rust-internal state with no existing `.ph`-
reachable accessor (unlike a class's `methods` table, which the ADR-0023 reflection
surface already exposes). Reaching it from a member-access send therefore needs at
least one new native primitive — a floor amendment, not an ordinary commit — exactly
as ADR-0038 found for the error-handling catch protocol.

## Decision

### Part 1 — supersede ADR-0027's resolution + binding forms (Q8, DEC-U15 = A + A)

1. **Resolution: relative file path (ADR-0027 alternative "Filesystem-path imports",
   now adopted).** `import "./geometry/point"` resolves relative to the **importing
   file's own directory** (not the process's current directory, not the program
   entry's directory), with a `.ph` extension appended if none is written. The
   resolved path is canonicalized (`std::fs::canonicalize`) before any registry probe
   — two spellings of the same file must resolve to the *same* canonical key, or the
   same unit could compile twice under two different `Module` identities (breaking
   `isA`/class identity across importers). There is **no logical-name resolver, no
   search path, no package notion** — Draft 0.1 is "scripts plus a lib directory".
2. **Binding: whole-module only (ADR-0027's "aliased" form; qualified/selective
   dropped for now).** `import "path" as Name` binds `Name` to the imported unit's
   `Module` object in the **importing scope** as an ordinary immutable global
   (`let`-shaped, [ADR-0014](0014-let-and-var-bindings.md)) — never a namespace merge
   into the importer's own globals. `as Name` is **mandatory**; there is no bare
   `import "path"` (ADR-0027's qualified form) and no `import a, b from "path"`
   (ADR-0027's selective form) yet. `from`/`export` are **not** lexed as keywords —
   reserved for the future selective-import/explicit-export grammar the plan's §7
   flags as a clean follow-up.
3. **Exports: everything top-level, no `export`, no `_`-privacy enforcement yet.**
   ADR-0027's "public by default, `_`-prefix private" convention is **not enforced**
   in Draft 0.1: every name a module's top level declares (`var`/`let`/`class`) is an
   ordinary member, reachable via `Module.member`. A future `export`/privacy pass can
   narrow this without changing the resolution or binding mechanism.
4. **A module is a first-class `Module` object** (object-model.md §4, unchanged from
   ADR-0027): member access is an **ordinary send** — `math.pi` sends the zero-arg
   getter selector `pi` to the `Module` value, `math.distance(1, 2)` sends the
   two-arg selector `distance(_:_:)` — never a compiler-only qualified-name rewrite.
   This is what makes `::`/reflective access (U16) compose with module members for
   free (plan §7).
5. **Compile-once, memoized by canonical path.** A [`Universe::module_registry`](../../phalcom-core/src/universe.rs)
   (`HashMap<String, ObjRef>`, canonical path → `Module`) is probed **before**
   compiling; a hit — whether the unit finished loading or is still mid-load —
   returns the *same* `ObjRef`. There is no separate "in-progress" set: the module is
   inserted into the registry the moment it is **allocated**, before it is compiled
   or run, so a re-entrant probe (a second `import` of the same file, or a cyclic
   import re-entering mid-load) always finds it and never recompiles or loops. A name
   read across a not-yet-complete cyclic edge is the ordinary "member not found"
   `doesNotUnderstand` miss on the still-partially-populated `Module` — a documented
   hazard, not a hang or a silent duplicate.
6. **Evaluation: once, at first import, re-entrantly.** `VM::import_module`
   (`phalcom-core/src/interpret.rs`) mirrors [`VM::send_dynamic`](../../phalcom-core/src/vm.rs)'s
   re-entrant `run_until` pattern — push one fresh frame for the imported unit's top
   level, drain exactly that activation — rather than `VM::run_in_module`'s
   stack-clearing (which assumes it is the *outermost* program entry and would
   destroy the importer's own live frames on a nested import).
7. **Source-only; no compiled-bytecode loader.** `import` resolves to and compiles a
   `.ph` **source** file. Loading a precompiled bytecode unit is explicitly **out of
   scope and DEFERRED** — Phalcom has no bytecode verifier, and loading unverified
   bytecode is a security hole (`docs/forge/DEFERRED.md`).
8. **Kernel visibility unaffected.** `Object`, `Number`, `List`, … remain visible in
   every module without `import` — they are the bootstrap (ADR-0027's already-settled
   "core is the module you never have to name"), not a module a unit opts into.

### Part 2 — amend the frozen floor +1: `Module#doesNotUnderstand(_:)`

**Amend [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) to admit exactly one
native primitive** (`floor-census.md` **+1**; landed at **111 → 112**):

- **`Module#doesNotUnderstand(_:)`** — overrides `Object`'s default miss handler
  (`primitive::module::module_does_not_understand`, `phalcom-core/src/primitive/module.rs`).
  Decodes the reified `Message`'s selector to a bare member name, probes the
  receiver's own `ModuleObject::get`, and either returns the value directly (a
  zero-arg `Getter` selector, `math.pi`) or forwards to it via the matching-arity
  `call(...)` selector (any other shape, `math.distance(1, 2)`) — falling through to
  `Object`'s default `MessageNotUnderstood` raise on a genuine miss. This is the
  **only** way to reach a `ModuleObject`'s `globals`/`name_to_slot` table from a
  message send; nothing in the existing reflection surface (ADR-0023) exposes it, so
  it fails ADR-0019 §1's derivability test exactly as ADR-0038 found for the
  error-handling catch protocol.

Everything else in the import surface is **compiler lowering over existing floor
opcodes**: `Bytecode::Import` (resolve/registry-probe/compile/run, `vm.rs`) followed
by the same `Bytecode::DefineGlobal` an ordinary top-level `let` already emits for
the `as Name` binding — no new opcode class beyond the one dispatch-loop hook, and no
further floor growth. Net floor delta = **+1**.

## Consequences

- **Unblocks U15.** The `import` token gets real semantics; a class or value defined
  in an imported unit is reachable, identity-stable, and visible only through the
  bound name — no global-namespace pollution.
- **ADR-0027 is not fully retired.** Its "file = module", "core is an auto-imported
  module", and "first-class `Module` object" framing all stand. Only its resolution
  mechanism and import-form grammar are narrowed for Draft 0.1; a future unit can
  layer logical-name resolution, selective import, and `export`/`_`-privacy on top of
  this same `Module`/registry substrate without another supersession — the plan's §7
  keeps the loader's resolve-seam abstract enough for a verified-bytecode source to
  slot in later, and the AST's whole-module node can grow an optional selective list
  additively.
- **`floor-census.md` (+1; 111 → 112) must be updated in the same change that installs
  the primitive** (R-INV-0.1), and `tests/invariants.rs::floor_census_matches_installed_bindings`'s
  `bindings` vector + `NEW_*` sum gain a `Module#doesNotUnderstand(_:)` row.
- **Path canonicalization is the correctness-critical step.** Skipping it (or probing
  the registry with an uncanonicalized path) would let the same file compile under
  two `Module` identities, silently breaking `isA` for any class it defines — the
  precise failure mode [ADR-0009](0009-handle-arena-heap.md)'s handle-identity
  discipline exists to prevent elsewhere in the object graph.
- **Cyclic-import partial-init is documented, not solved.** A name read across a
  not-yet-complete cyclic edge fails with an ordinary `doesNotUnderstand`, matching
  how most languages handle circular imports (a load-order hazard the *author*
  manages, not a compiler proof obligation) — consistent with ADR-0027's original
  "circular-import policy is an implementation detail of the module unit, not a
  language-level decision" stance.

## Alternatives considered

- **Keep ADR-0027's logical-name resolver + three import forms as the v1 target.**
  Rejected for Draft 0.1 — needs a search-path/package notion the language has no
  other machinery for yet, and the three-form grammar is strictly larger than what a
  single "read a value from another file" use case needs. Not foreclosed: this ADR
  narrows the *implementation*, not the long-run design ADR-0027 sketched.
- **Selective import (`import a, b from "path"`) instead of, or alongside, whole-module
  binding.** Rejected for now — needs the unit to declare/expose members explicitly
  (or bind every top-level name individually), which the "everything top-level is a
  member" Draft-0.1 model does not yet distinguish. Reserved (`from` unlexed) for a
  clean follow-up once whole-module binding is exercised.
- **Split the floor amendment into `Module#rawGet(_:)`/`Module#rawHas(_:)` plus a
  `.ph`-defined `doesNotUnderstand` override (mirroring `List`'s `rawAt`/`at`
  split).** Rejected — two floor primitives instead of one for no behavioral gain;
  `doesNotUnderstand`'s selector-to-member-name decoding
  (`crate::method::decode_selector`) is not itself expressible in `.ph` (no surface
  reflection over a raw selector string exists), so splitting would not actually move
  any logic above the floor, only add a second binding.
- **Compiled-bytecode `import` (loading a `.phc`-style precompiled unit).** Rejected
  — no bytecode verifier exists; loading unverified bytecode is a security hole.
  Explicitly DEFERRED, not designed against — the resolve → obtain-a-compiled-chunk →
  instantiate-`Module` seam in `VM::import_module` stays abstract enough for a future
  verified-bytecode source to slot in behind it.
