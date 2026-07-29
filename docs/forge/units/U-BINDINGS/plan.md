# U-BINDINGS — `let`/`const` bindings; unkeyworded mutable fields; `const` fields writable only in constructors

Status: **READY** (2026-07-15). Governed by
[ADR-0064](../../../adr/accepted/0064-let-const-bindings-and-field-mutability.md)
(**Accepted**, supersedes ADR-0014). **Lands before [U-CTOR](../U-CTOR/plan.md)** —
U-CTOR's `@class`-on-field work sits on the field grammar this unit rewrites.

Not a performance unit and not an object-model unit — a **surface + migration** unit.
Semantics are unchanged from ADR-0014 except one genuinely new rule (`const` fields,
§3). The risk here is entirely in the **1080-site codemod**, not the design.

---

## Role

Two problems, one migration:

1. **The common case carries the odd keyword.** 728 `let` (immutable) vs 352 `var`
   (mutable). Every neighbour language spells the ordinary binding `let`.
2. **`let` on a field is decorative.** ADR-0014 promises immutability; fields deliver
   none. Verified on HEAD:
   ```phalcom
   class K { let _n
             @constructor
             new(n) { _n = n }
             clobber(v) { _n = v } }   // → 99, no error
   ```

Doing this *after* U-CTOR would migrate the same declarations twice and force the field
codemod to understand two grammars.

---

## Spec anchor

- **[ADR-0064](../../../adr/accepted/0064-let-const-bindings-and-field-mutability.md)** — the whole unit.
- [ADR-0014](../../../adr/accepted/0014-let-and-var-bindings.md) — **superseded**; flip its status + `STATUS.md` in this unit ([[adr-status-two-way-sync]]).
- [ADR-0007](../../../adr/accepted/0007-option-as-abstract-with-some-none.md) — an uninitialized mutable binding reads `None`; carried over verbatim.
- [ADR-0011](../../../adr/accepted/0011-static-instance-slot-layout.md) — read-before-write; **orthogonal**, must keep passing.
- `docs/spec/current/syntax/statements-and-declarations.md` §2, §5 · `syntax/grammar.md` · `syntax/lexical.md` · `classes.md` §2.

---

## Preconditions (verify on HEAD — do not trust this list)

| # | Claim | Verify |
|---|---|---|
| P1 | `let`/`var` are keywords; no `Token::Const` | `rg -n '"var" =>\|"let" =>\|Const' phalcom-ast/src/token.rs phalcom-ast/src/lexer.rs` |
| P2 | `const` unused in the corpus | `rg -c '\bconst\b' --glob '*.ph'` → expect 0–1 |
| P3 | 352 `var` / 728 `let` across 395 files | `rg -c '\bvar \|\blet ' --glob '*.ph'` |
| P4 | `let` on a *field* is unenforced | run the `clobber` probe above → expect `99` |
| P5 | `let x` (no init) at binding position is rejected; `var x` reads `None` | probe both |
| P6 | `FieldDef.mutable` comes from `let`/`var` at class-body position | read `phalcom-ast/src/ast.rs` `FieldDef` |
| P7 | Fields are also implicitly declared by assignment (no decl needed) | `classes.md` §2; probe a class with no field decls |

**P4 and P7 are load-bearing.** If P4 is false (enforcement already exists) §3 is
mostly done. If P7 is false, the "mutable fields take no keyword" rule needs a
declaration form this plan does not specify.

---

## Design

### 1. Lexer / tokens

- `Token::Var` deleted; `"var"` lexes as `Token::Identifier`.
- `Token::Const` added; `"const"` joins the keyword table.
- `Token::Let` retained, meaning inverted (now *mutable*).

### 2. Binding grammar

```
binding := ("let" | "const") IDENT [ "=" expr ]
```

| Form | Rule |
|---|---|
| `let x` | mutable, uninitialized → `None` |
| `let x = e` | mutable |
| `const x = e` | immutable |
| `const x` | error: `binding.const_requires_initializer` |

`BindingKind` keeps its two-way split; only the token→variant mapping flips. **The
immutability check itself is unchanged** — it already exists for old-`let` and simply
follows `const`.

### 3. Field grammar

```
field_decl := { attribute } [ "const" ] FIELD [ "=" expr ]
```

| Form | Rule |
|---|---|
| `_x` / `_x = e` | mutable |
| `const _x = e` | immutable, defined at declaration |
| `const _id` | immutable, assignable **only in a `@constructor` body** |
| `let _x` | error: `field.no_mutable_keyword` — "mutable fields take no keyword; write `_x`" |

`FieldDef.mutable` is now set by the *presence* of `const`, not by `let` vs `var`.

Bare `_x` as a declaration is **unambiguous**: fields lex as `FIELD`, getters as
`IDENT`, and `getter_decl` requires a `method_body` (`=>` or a block). A `FIELD` token
alone in a class body can only be a field declaration.

### 4. `const` field enforcement — syntactic

> A write to a `const` field outside a `@constructor` body is `field.const_write`.

Keyed on **which member the write appears in** — no flow analysis (Phalcom has none,
by ADR-0052's precedent). Implement in the same pass that already resolves field
writes to slots; the compiler knows the enclosing member.

Accepted gaps, per ADR-0064 §3 — **do not try to close these here**:
- two writes inside one constructor: not caught (needs definite assignment)
- a constructor that never assigns a `const _id`: not caught → reads `None` forever

That second gap is reachable via ADR-0063 §7 (`Factory.new()` bypasses constructors)
and is **specified**, not a bug. Fixture it so it stays specified.

---

## Write-set

| File | Change |
|---|---|
| `phalcom-ast/src/token.rs` | `Token::Var` out, `Token::Const` in |
| `phalcom-ast/src/lexer.rs` | keyword table: drop `"var"`, add `"const"` |
| `phalcom-ast/src/ast.rs` | `FieldDef.mutable` doc; `BindingKind` doc (semantics of the variants flip) |
| `phalcom-ast/src/parser.rs` | `parse_binding`; `parse_field_decl` (bare `FIELD`, `const`, reject `let`); `const`-no-initializer error |
| `phalcom-core/src/compiler/lib/*.rs` | `field.const_write` check at the field-write site; `binding.const_requires_initializer` |
| `phalcom-core/src/compiler/lib/error.rs` | two new diagnostics |
| `phalcom-core/core/core.ph` | codemod (`var _targets` → `_targets`, etc.) |
| `phalcom-lsp/src/*` | keyword lists, highlighting, completion — `var`→`const` |
| corpus `.ph` ×395 files | **the codemod, 1080 sites** |
| `docs/spec/current/syntax/{grammar,lexical,statements-and-declarations}.md` | grammar + keyword lists |
| `docs/spec/current/classes.md` §2 | field mutability + `const` |
| `docs/adr/accepted/0014-*.md` + `docs/adr/STATUS.md` | flip to Superseded, both sides |

---

## The codemod — the actual risk

**It is a swap.** `var`→`let` then `let`→`const` in two passes turns every original
`var` into `const`. **Single pass only.**

**The mapping is position-dependent**, so a textual rewrite is wrong:

| Position | Old | New |
|---|---|---|
| statement | `var x` | `let x` |
| statement | `let x` | `const x` |
| class body | `var _x` | `_x` |
| class body | `let _x` | `const _x` |

Recommended: drive it from the **parser** — walk each file's AST, rewrite declaration
sites by node kind, print with spans preserved. A regex pass keyed on `_`-prefix as a
class-body proxy is the cheap fallback, but it will misfire on any `let _x` local
inside a method body (locals may be `_`-prefixed only by accident — verify against the
corpus before relying on it).

**Gate:** the golden corpus is the oracle. Every `.ph` fixture must produce
byte-identical stdout before and after. Capture the full suite's output *before* the
codemod, diff after.

---

## Build order

1. **Tokens + binding grammar** (`let`/`const`, `var` gone). Codemod statement-position
   sites. Green.
2. **Field grammar** (bare `_x`, `const _x`, reject `let _x`). Codemod class-body sites.
   Green.
3. **`field.const_write` enforcement** + diagnostics. Green.
4. **Docs**: ADR-0014 → Superseded + `STATUS.md`; spec grammar/keyword lists;
   `classes.md` §2.

Commit per green step ([[commit-frequently]]); verify each commit from a **clean
throwaway worktree at the SHA**, not in-tree ([[clean-checkout-verify-each-commit]]).
Main has live concurrent sessions — **commit narrow paths directly on `main`; do not
branch** ([[phalcom-concurrent-session-hazards]] §7).

---

## Tests / verification

**Positive lane** (`tests/lang/`):
- `bindings_let_mutable.ph` — `let x = 1  x = 2` → 2
- `bindings_let_uninit_reads_none.ph` — `let x` → `None` (ADR-0007 carried over)
- `bindings_const_immutable.ph` — `const x = 1` then read
- `field_mutable_no_keyword.ph` — `_x = 0` declaration + mutation
- `field_const_declared_default.ph` — `const _x = 10`
- `field_const_written_in_ctor.ph` — `const _id` + `@constructor new(v) { _id = v }`

**Negative lane** (`tests/lang/compile-errors/`):
- `binding_const_no_initializer.ph` — `const x` → `binding.const_requires_initializer`
- `binding_const_reassigned.ph` — `const x = 1  x = 2`
- `field_const_write_outside_ctor.ph` — **the Context bug** — `clobber(v) { _n = v }` → `field.const_write`
- `field_let_keyword_rejected.ph` — `let _x` → `field.no_mutable_keyword`
- `binding_var_is_not_a_keyword.ph` — *positive* control: `let var = 1` now parses

**Specified-sharp-edge lane:**
- `field_const_never_assigned_reads_none.ph` — a `const _id` no constructor assigns,
  reached via `Factory.new()` (ADR-0063 §7) → `None`. **Pins the H⊗I interaction as
  intended behavior.**

**Fixtures must be proven wired** — the harness runs one test per *lane*, iterating the
directory, so a new file can be silently skipped. Corrupt each `.expected`, confirm the
suite reddens, restore ([[phalcom-golden-test-lanes]]).

**Hard gates:** full `cargo test` · **byte-identical golden stdout across the codemod**
· `cargo clippy --workspace` (13 pre-existing, none new) · `cargo doc` clean
([[rust-doc-mandatory]]) · `graphify update .`

---

## What must this not preclude (P4)

- **Definite-assignment analysis** — §4 is deliberately weaker than "a `const` field is
  always assigned". If flow analysis ever lands, it tightens with no surface change.
- **ADR-0011 read-before-write** — orthogonal (that's *any* write anywhere; this is
  *where* a write may appear). Both must pass together; add a fixture combining them.
- **U-CTOR's `@class` on fields** — the `[ "const" ]` slot sits after attributes, so
  `@class const _limit = 10` needs no further grammar change. Verify this parses before
  U-CTOR starts.
- **`const` on parameters/methods** — the keyword is not spent elsewhere.
- **ADR-0061** (Proposed) — `field_name := "_" [A-Za-z] …` composes with `field_decl`'s
  new shape; if 0061 lands first, the bare-`_x` declaration form must satisfy its §6.

---

## Decisions — all ruled 2026-07-15

| # | Ruling |
|---|---|
| **I** | Enforce `const` fields syntactically (constructor-only writes) |
| **I2** | Own unit + ADR, **before** U-CTOR |
| — | `let` = mutable, `const` = immutable, `var` deleted |
| — | Mutable fields take **no** keyword |
| — | `const _id` may defer its value to a constructor (unlike `const x` at binding position) |
| — | One-shot codemod, no deprecation window (inherits DEC-CTOR-D's reasoning: Phalcom owns 100% of its corpus) |
