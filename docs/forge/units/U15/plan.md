# U15 — Work order: modules & imports — open-Q8

_Self-contained plan for **one** `phalcom-implementer` agent. Grounds in open-question **Q8**
([open-questions.md](../spec/open-questions.md#L67)), the `Module` class ([object-model.md §4
"Runtime & namespaces"](../spec/object-model.md)), and the overlay's security note ("no
external-bytecode loader yet — required before any `import` of compiled units"). The `import`
token **exists** (`phalcom-ast/src/token.rs:62` `Token::Import`) but has **no parser, no AST, no
runtime semantics**. This is the **largest, greenfield** unit in the cluster and is
**BLOCKED-ON-DECISION** on the module-resolution model._

---

## 0. Mission (one sentence)
Give the existing `import` token meaning: define how a Phalcom program names, resolves, loads,
and binds another compilation unit, realized as a `Module` namespace object populated by
compiling a resolved `.ph` source file exactly once (memoized), with cyclic-import detection —
without introducing an external compiled-bytecode loader (which would require a verifier Phalcom
does not have yet).

## 1. Hard guardrails
- **Source-only imports in Draft 0.1.** Import resolves to and compiles a `.ph` **source** file.
  Do **not** load precompiled bytecode units — the overlay flags that a bytecode *verifier* is a
  prerequisite for that and none exists; loading unverified bytecode is a security hole. Compiled
  units are explicitly out of scope → DEFERRED.
- **A module is a `Module` object** (object-model §4) — a first-class namespace value, not a
  compiler-only construct. Its members are reached by ordinary sends (`math.pi`), consistent with
  "everything is a message."
- **Compile each unit exactly once.** A module registry (in `Universe`) memoizes by canonical
  path; a second `import` of the same unit returns the same `Module` object (identity-stable).
- **No global-namespace pollution.** An `import` binds a name in the *importing* scope; it does
  not dump the imported unit's globals into the caller. Kernel/core classes remain globally
  available (they are the bootstrap, not a module).
- Stay inside the write-set (§3).

## 2. Preconditions (verify first)
- `./scripts/verify.sh` green.
- `graphify explain "Module"` + `graphify explain "Universe"` — locate `ModuleObject`
  (`phalcom-core/src/module.rs`), the globals/registry in `universe.rs`, and how top-level source
  is compiled + run today (`compiler/lib.rs`, the CLI entry `bin/phalcom/main.rs`).
- Confirm how the CLI currently loads `core.ph` at startup — the import loader should reuse that
  compile-a-file-into-the-VM path, not fork a second one.
- `Token::Import` present but unused past the lexer — confirm the parser has no `import`
  production yet.

## 3. Confirmed write-set (validate with `graphify affected "ModuleObject"` on HEAD)
| File | Why |
|---|---|
| `phalcom-ast/src/ast.rs` | `Statement::Import { path, binding }` (+ selective form if ruled). **Contended (`phalcom-ast`)** — serialize. |
| `phalcom-ast/src/parser.rs` | Parse `import "path"` / `import "path" as name` / (selective) `import { a, b } from "path"` per DEC-U15. |
| `phalcom-core/src/module.rs` | Extend `ModuleObject` into a real namespace: a member table (name → `Value`); accessor protocol. |
| `phalcom-core/src/universe.rs` | Module **registry** keyed by canonical path; the in-progress set for cycle detection. **Contended** — serialize. |
| `phalcom-core/src/compiler/lib.rs` | Compile-an-imported-unit-into-a-fresh-Module path; emit the import-bind opcode/prologue. **Contended** — serialize. |
| `phalcom-core/src/vm.rs` | Execute the import: resolve path → registry hit or compile+run the unit's top level into its Module → bind. **Contended** — serialize. |
| `phalcom-core/src/primitive/module.rs` | `Module` member-access / reflection primitives. |
| `phalcom-core/bin/phalcom/*` | Only if import resolution needs the entry file's directory (base path for relative resolution). |
| `phalcom-core/tests/lang.rs` (+ multi-file fixtures) | Import corpus (§6). |
| `docs/adr/00XX-module-import-model.md` | New ADR — provisional number, grab next-free. |
| `docs/spec/open-questions.md` Q8 + a new `docs/spec/modules.md` | Flip Q8 to RESOLVED; write the module spec part. |

## 4. Design decision — **BLOCKED-ON-DECISION (DEC-U15)**
**Question:** what is the module-resolution + binding model? Three coupled sub-choices:

**(1) Resolution — how a path maps to a file.**
| Option | Form | Note |
|---|---|---|
| **A — relative file path** | `import "./geometry/point"` resolves relative to the importing file; `.ph` appended | simplest; no registry config; matches "scripts + a lib dir" |
| **B — logical module names + search path** | `import geometry.point` resolved against a configured root list | needs a package/config notion Phalcom doesn't have yet |

**(2) Binding — what `import` introduces.**
| Option | Form | Selector shape |
|---|---|---|
| **A — whole-module binding** | `import "point" as Point` → `Point` is a `Module`; members via `Point.distance(…)` | one binding, members are sends |
| **B — selective binding** | `import { distance, Point } from "point"` → names bound directly | more ergonomic; needs the unit to declare/expose members |
| **C — both** | whole + selective | largest surface |

**(3) What a `.ph` unit exports.** Everything top-level, or an explicit `export`/visibility marker?

**Architect recommendation:** **(1) A relative file path + (2) A whole-module binding
(`import "path" as Name`) + (3) "everything top-level is a member, no explicit export in Draft
0.1."** This is the smallest model that is coherent, needs no package manager, and reuses the
existing "compile a file into the VM" path. Selective import (2B) and explicit `export` are a
clean follow-up once the whole-module form works — pre-reserve the grammar (`from`/`export` are
not yet keywords; adding them later is additive). **Do not pick** — the binding form and the
export policy shape the language's surface; the user should rule.

**Architect-owned once the model is ruled:**
- **Cycle detection:** an "in-progress" set of canonical paths; re-entering a path already
  in-progress returns its *partially-built* `Module` (so mutual imports resolve to the same object
  and complete) — do **not** infinite-loop or silently duplicate. Document the partial-init
  hazard (a name used before the cyclic dependency finished defining it reads `None`/errors).
- **Evaluation:** an imported unit's top level runs **once**, at first import, in its own Module
  scope; side effects at import time are the author's responsibility (like most languages).
- **Kernel visibility:** `Object`, `Number`, etc. are visible in every module without import.

## 5. Risk
- **Path canonicalization is a security + correctness surface:** two spellings of the same file
  must canonicalize to one registry key, or a unit compiles twice and its classes become two
  distinct identities (breaking `==`/`isA`). Canonicalize before the registry probe. Also reject
  path traversal outside an allowed root if/when sandboxing matters (note for the security ADR).
- **Cyclic imports** are the classic subtle failure — the in-progress set + partial-Module return
  is the only sound approach; a naive "compile fully before binding" deadlocks on cycles.
- **Bootstrap ordering:** imports must run *after* the kernel/`core.ph` is loaded; an import in a
  file that runs during bootstrap would see a half-built universe. Gate import execution to
  post-bootstrap.
- **Standing borrow/heap risk:** the registry lives in `Universe`; compiling a nested import while
  holding a borrow of the registry is a re-entrancy trap — structure the loader so the
  compile-into-Module step doesn't hold a `&mut Universe` across the recursive compile.

## 6. Test strategy (green gate must assert)
- Basic: file `a.ph` `import "b" as B`, `b.ph` defines `answer` → `a` reads `B.answer` == 42.
- Identity/memoization: importing `b` from two files yields the **same** `Module` object; a class
  defined in `b` has one identity across both importers (`isA` holds).
- Cycle: `a` imports `b`, `b` imports `a` — both load, no infinite loop; a name defined before the
  cyclic edge resolves; a name used across the not-yet-complete edge fails cleanly (documented).
- Resolution: relative path resolves against the importing file's directory; a missing file
  raises a clean error with the attempted path, not a panic.
- Isolation: a top-level `var` in `b` does **not** leak into `a`'s global scope (only `B.x` sees
  it, per the whole-module binding).
- Kernel access: an imported unit uses `Number`/`Object`/`List` without importing them.

## 7. Forward-looking — must NOT preclude
- **Compiled-unit imports + a bytecode verifier (deferred):** keep the loader's "resolve → obtain
  a compiled chunk → instantiate Module" seam abstract enough that a future verified-bytecode
  source slots in behind it. Note the verifier as a hard prerequisite in the ADR + DEFERRED.
- **Selective import / `export` (2B/3):** reserve `from` and `export` as future keywords; the
  whole-module AST node should carry an optional selective list so adding it is additive.
- **U13 (hierarchy):** a class imported from another module obeys the same sealing/mutability
  policy — modules add no new class-graph mutation path. If U13 rules mutability in, cross-module
  `superclass=` must still invalidate ICs (coordinate).
- **Concurrency (concurrency.md):** module load is synchronous and happens before/outside fiber
  scheduling; do not make `import` itself suspendable. A `Module` is shared across fibers via the
  handle heap — under cooperative scheduling that is race-free, but keep module init single-shot so
  no two fibers half-initialize the same unit.
- **U16 (Family/`::`):** `Module::member` / reflective access should compose with the Family
  machinery later; keep `Module` member lookup a normal send so `::` works on module members for
  free.

## 8. Mandatory rules
- `///` on `Statement::Import`, the registry, the loader, `Module` member accessors; `//!`
  refreshed; cite the `Module` object-model row + the new ADR. `cargo doc` clean.
- Green gate = `./scripts/verify.sh` exits 0. This is greenfield + touches `vm.rs`/`universe.rs`;
  recommend reviewer **ON**.
- Own isolated worktree off `main`.

## 9. Return contract
Report: the DEC-U15 resolution model implemented (all three sub-choices) · the canonicalization +
cycle-detection scheme · confirmation module identity is memoized (one object per unit) · that
kernel classes are visible without import · files changed · new multi-file fixtures · `verify.sh`
+ `cargo doc` tails · DEFERRED entries (compiled-unit loader + verifier, selective import,
`export`, sandboxing/path-traversal policy).
