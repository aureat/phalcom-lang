# U-BINDINGS — Implementation spec

Companion to [`plan.md`](plan.md). Governed by
[ADR-0064](../../../adr/accepted/0064-let-const-bindings-and-field-mutability.md)
(**Accepted**, supersedes ADR-0014).

> **ADR-0061 ruled 2026-07-19 — ratified, with its bare-`_` rejection scoped to field
> position.** A leading-`_` identifier is required *where a field is being declared*; bare `_`
> at binding position stays legal, so the corpus's 13 `let _ = …` throwaway reads are
> unaffected. This unblocks the unit. **The ADR file itself has not been re-filed yet** — see
> §12D.1 for why that edit was held.

**Status: READY to dispatch.** User rulings **L-1…L-9** are locked in
[§12A](#12a-locked-decisions-user-ruling-2026-07-19) and extend ADR-0064 with enforcement it
does not itself specify; [§12B](#12b-investigation-why-global-and-local-disagree) is the
investigation behind L-3.

Every precondition in `plan.md` was re-verified against the tree at `de49d3a` on 2026-07-19
before this spec was written. **Concurrent sessions moved `main` to `a65d499` during
authoring** (three commits: learn Doc 6, a testing spec, a course tracker) — none touch
`phalcom-ast/src`, `phalcom-core/src/compiler/`, or `core.ph`, so every anchor below still
holds, but recount the codemod figures (§0 P3) before starting. A **locked** worktree also
exists at `.claude/worktrees/repl-cell-model` (branch `worktree-repl-cell-model`, `301044e`)
— a live session; do not touch it, and exclude it from every corpus scan or the counts
double.

Five `plan.md` claims did not survive that verification. They are corrected in
[§1](#1-corrections-to-planmd-read-this-first), and the corrections change the build order,
the codemod strategy, and the definition of the green gate. **Read §1 before §2.**

---

## 0. Preconditions — re-verified 2026-07-19 at `de49d3a`

`plan.md` says "verify on HEAD — do not trust this list". Done. Results:

| # | Claim | Verdict | Evidence |
|---|---|---|---|
| P1 | `let`/`var` are keywords; no `Token::Const` | ✅ holds | `phalcom-ast/src/lexer.rs:262-263`; `Token::Let`/`Token::Var` at `phalcom-ast/src/token.rs:32,39` |
| P2 | `const` unused in the corpus | ✅ holds | exactly 1 hit, and it is inside a *comment* — `tools/vsphalcom/manual-test/01-syntax-highlighting.ph:6` calls `const` a "dead keyword" |
| P3 | 352 `var` / 728 `let` across 395 files | ⚠️ **drifted** | now **360 `var` / 743 `let` / 401 files**. Recount before the codemod; do not reuse the plan's figures |
| P4 | `let` on a field is unenforced | ✅ **reproduced** | the `clobber` probe prints `99`, no error, exactly as `plan.md` predicts |
| P5 | `let x` (no init) rejected; `var x` reads `None` | ✅ holds | both probed; exact messages in §2.3 |
| P6 | `FieldDef.mutable` comes from `let`/`var` | ✅ holds | `phalcom-ast/src/ast.rs:300-303`; set at `parser.rs:1154` |
| P7 | fields are implicitly declared by assignment | ✅ holds | a class with zero field declarations compiles and runs |

`plan.md` calls P4 and P7 load-bearing. Both hold, so §3 of the plan is neither
already-done nor under-specified. **But P4's motivating defect has zero live instances**
— see §1.5.

---

## 1. Corrections to `plan.md` — read this first

### 1.1 There is no `FIELD` token, so the plan's disambiguation argument is void

`plan.md` §3 justifies the bare-`_x` declaration form like this:

> Bare `_x` as a declaration is **unambiguous**: fields lex as `FIELD`, getters as `IDENT`,
> and `getter_decl` requires a `method_body` (`=>` or a block). A `FIELD` token alone in a
> class body can only be a field declaration.

**No `FIELD` token exists.** `phalcom-ast/src/token.rs` has no `Field` variant at all.
Field names are ordinary `Token::Identifier`s that happen to start with `_`; the
leading-underscore convention is applied *downstream* (`parse_primary` routes a leading-`_`
identifier to `Expr::Field`), not by the lexer.

The conclusion is still reachable, but it needs a different mechanism and it is not free.
See §4.

### 1.2 `_x = e` at class-body position is already a valid production — and it crashes

This is the sharpest finding. `parse_class_member` (`phalcom-ast/src/parser.rs:1269-1282`)
contains:

```rust
let has_equal = self.eat(&Token::Equal);
if has_equal && name.starts_with('_') {
    // → ClassMember::Getter { body: vec![Statement::Expr { … }] }
}
```

So `_x = 5` in a class body **parses today**, as a *getter* whose body is the expression.
ADR-0064 assigns that exact syntax a different meaning (a mutable field with a default).
The two collide head-on.

Worse, the existing production is broken — and the breakage is wider than the `= e` form.
Probed at `de49d3a`:

| Class body | Result |
|---|---|
| `_x = 5` | `Internal error: Static field slot 0 out of bounds` |
| `_x => 5` | `Internal error: Static field slot 0 out of bounds` |
| `_x { return 5 }` | `Error: Invalid field initializer body` |
| `static _c = 7` | ✅ works |

The `_x => 5` case still crashes when nothing ever reads `_x` (a class whose only other
member returns a constant), so **the failure is at class-definition time, not at call
time**. Every non-static class member whose name begins with `_` is being routed into
field-initializer machinery; the `=>` form reaches an internal error, the block form
reaches a real diagnostic, and only the `static` path survives.

The practical consequence for §4: **there are no working leading-underscore getters
today.** Reinterpreting that syntax as a field declaration cannot regress any behavior
that currently works.

**Corpus exposure is exactly one line**, found by a brace-depth scan (a plain regex
overcounts by ~157× because it catches `_name = name` *statements* inside constructor
bodies):

```
phalcom-core/tests/lang/classes/class_static_field_shared_state.ph:9:  static _count = 0
```

That one site is the `static` form, which works and must keep working. The broken
non-static form is exercised by **zero** files.

**Ruling for this unit:** the field grammar in §4 *replaces* the `parser.rs:1269-1282`
branch for the non-static case. Deleting it is safe (nothing uses it) and incidentally
closes a live internal-error crash. Preserve the `static _x = e` path unchanged — U-CTOR
later renames `static` to `@class`, and that is not this unit's business.

### 1.3 The "byte-identical golden stdout" gate is false as written

`plan.md` lists as a hard gate: "**byte-identical golden stdout across the codemod**".
It cannot be. Two compiler diagnostics name the keywords in user-facing text:

```
phalcom-core/src/compiler/lib/error.rs:26
  "Cannot reassign immutable `let` binding '{0}'; declare it with `var` to allow mutation."
phalcom-core/src/compiler/lib/error.rs:36
  "`let` binding '{0}' requires an initializer; use `var {0}` for an uninitialized binding."
```

Both must be reworded, and one golden pins the text:

```
phalcom-core/tests/lang/compile-errors/compile_error_let_reassignment.expected
  Cannot reassign immutable `let` binding
```

**Corrected gate:** byte-identical stdout across the codemod **except** for
`compile_error_let_reassignment.expected`, whose new text is pre-registered in §9 *before*
the codemod runs. Pre-registration matters — an unregistered "expected" diff in a
1000-file rebaseline is indistinguishable from a real regression and will be
rubber-stamped.

### 1.4 The codemod is not the risk the plan says it is

`plan.md` states "The risk here is entirely in the **1080-site codemod**" and recommends an
AST-driven rewriter, calling a regex pass a fallback that "will misfire". Measured, the
position-dependent portion is **16 sites**:

| Rewrite | Sites | Position-dependent? |
|---|---|---|
| `let x` → `const x` (statement) | 713 | no — blanket |
| `let (a,b)` / `let [a,*r]` → `const …` | 15 | no — blanket |
| `let _x` → `const _x` (field) | **0** | n/a |
| `var x` → `let x` (statement) | 343 | no — blanket |
| `var _x` → `_x` (field, drop keyword) | **16** | **yes — the only hard case** |

`let` at field position is **zero occurrences**, so the `_`-prefix proxy the plan distrusts
has no false-positive source on the `let` side. `let` → `const` is a blanket rename at
every position. Only `var` diverges by position, and only 16 sites are class-body. They are
enumerated in §6.2 and are hand-reviewable in one sitting.

**The AST-driven rewriter is not required.** It is also actively worse in one respect: an
AST printer rewrites declaration *nodes* and leaves comments untouched, so every
`// U14: \`let (a, b) = point\`` comment silently becomes wrong. A textual pass updates
comments too. `plan.md` does not mention comment text at all; it is part of the job either
way.

### 1.5 The spec docs are already migrated — they describe behavior that does not exist

`plan.md`'s write-set lists the spec grammar, keyword lists, `classes.md` §2, and the
ADR-0014 status flip as work to do. **All of it is already done**, verified 2026-07-19:

| Doc | Current text |
|---|---|
| `docs/spec/v0.2/syntax/grammar.md:60` | `binding := ( "let" \| "const" ) IDENT [ "=" expr ]` |
| `docs/spec/v0.2/syntax/grammar.md:160` | `keyword := "let" \| "const" \| "class"` |
| `docs/spec/v0.2/syntax/lexical.md:89,301` | already lists `let`/`const` as the live pair |
| `docs/spec/v0.2/syntax/statements-and-declarations.md:28` | `binding := ("let" \| "const") IDENT [ "=" expr ]` |
| `docs/spec/v0.2/syntax/statements-and-declarations.md:42-43` | "`var` is **not** a keyword." |
| `docs/spec/v0.2/classes.md:8,134,143` | cites ADR-0064, uses `let` for mutable |
| `docs/adr/accepted/0014-…:3` | already `Superseded by ADR-0064` |
| `docs/adr/STATUS.md:44` | already `Superseded by 0064` |

**This is a status/reality gap running the opposite direction from the usual one.** The
normal Phalcom failure is an ADR left Proposed after the code shipped (0028/0036/0037/0040,
and 0056 today). Here the *spec* is ahead of the *code*: anyone reading
`docs/spec/v0.2/syntax/` today will conclude `let`/`const` is implemented. It is not — the
lexer still has `Token::Var` and no `Token::Const`. Until U-BINDINGS lands, the v0.2 syntax
spec is not a description of Phalcom.

Two consequences for this unit:

1. **§7's doc rows shrink to verification**, not authoring. Do not rewrite what is already
   correct; diff it against §3/§4 and fix only divergence.
2. `statements-and-declarations.md:81` writes the field production as
   `field_decl := [ "const" ] FIELD [ "=" expr ]`. **That notional `FIELD` nonterminal is
   almost certainly the origin of `plan.md`'s false "fields lex as `FIELD`" claim** (§1.1) —
   a grammar-level metavariable was read as a lexer token. When implementing §4, treat
   `FIELD` as "identifier with a leading underscore", which is what the tree actually has.

### 1.6 One ADR index row was missed by the otherwise-complete flip

`docs/adr/README.md:57` still lists ADR-0014 as **Accepted**, while the ADR file itself and
`STATUS.md` both say Superseded. **ADR-0064 has no row in that index at all.** Two-way ADR
status sync is a standing convention here; this is a third place that needs to agree, and
it is the one that got missed. Fix in step 4.

### 1.7 P4's defect has no live instances

`plan.md` §Role motivates the unit partly on the unenforced-`let`-field hole. The hole is
real (P4 reproduces). But `let` at field position occurs **0 times** in the corpus, so
nothing today depends on, or violates, the broken promise. The migration's actual
justification is the keyword ergonomics (743 immutable vs 360 mutable — the common case
carries the odd keyword) plus closing the hole prospectively. Worth knowing when weighing
this unit against others: **no user-visible bug is being fixed by §5**, one is being
prevented.

---

## 2. Lexer and tokens

### 2.1 Token set

| Token | Change |
|---|---|
| `Token::Const` | **added**; `"const"` joins the keyword table |
| `Token::Let` | retained, **meaning inverted** — now mutable |
| `Token::Var` | **deleted** at the end of step 2 (see §8 for why not step 1); `"var"` then lexes as `Token::Identifier` |

`phalcom-ast/src/lexer.rs:262-263` is the keyword-table site. P2 confirms `const` is not
in use as an identifier anywhere, so adding it as a keyword breaks nothing.

### 2.2 AST

`phalcom-ast/src/ast.rs`:

- `BindingKind` keeps its two variants. **Rename them** — `Let`/`Var` → `Mutable`/`Immutable`
  (or `Let`/`Const`), because leaving a variant named `Var` after `var` is deleted is a
  trap for the next reader. The doc comments at `ast.rs:407-419` state ADR-0014's
  semantics verbatim and must be rewritten, not just re-labelled.
- `FieldDef.mutable` keeps its type. Its meaning flips from "declared `var`" to "declared
  *without* `const`". Doc comment at `ast.rs:300-303` must say so.
- `LetBinding` keeps its shape. Its doc comment (`ast.rs:421-437`) references ADR-0014 and
  the `let`/`var` split; rewrite against ADR-0064.

No structural AST change. This is a rename-and-reinterpret unit at the AST layer.

### 2.3 Diagnostics

Both existing messages are rewritten (see §1.3). Two new ones are added:

| Key | Message |
|---|---|
| `binding.const_requires_initializer` | ``` `const` binding '{0}' requires an initializer; use `let {0}` for an uninitialized binding. ``` |
| `field.no_mutable_keyword` | ``` mutable fields take no keyword; write `{0}` instead of `let {0}`. ``` |
| `field.const_write` | ``` cannot assign to `const` field '{0}' outside a constructor. ``` |
| `binding.redeclared` | ``` '{0}' is already declared in this scope; use assignment, or declare it in a nested scope to shadow. ``` (**L-3**) |

Rewritten:

| Site | New message |
|---|---|
| `error.rs:26` | ``` Cannot reassign immutable `const` binding '{0}'; declare it with `let` to allow mutation. ``` |
| `error.rs:36` | *(deleted — replaced by `binding.const_requires_initializer`)* |

---

## 3. Binding grammar (statement position)

```
binding := ("let" | "const") pattern [ "=" expr ]
```

| Form | Rule |
|---|---|
| `let x` | mutable, uninitialized → reads `None` (ADR-0007, carried over verbatim) |
| `let x = e` | mutable |
| `const x = e` | immutable |
| `const x` | error: `binding.const_requires_initializer` |
| `const x = 1` then `const x = 2` **in the same scope** | error: `binding.redeclared` (**L-3**) |
| `const x = 1` then `let x = 2` **in the same scope** | error: `binding.redeclared` — a redeclaration cannot release immutability (**L-3**) |
| `const x = 1` then `x = 2` | error: `AssignToImmutable` (already enforced) |
| `const x = 1`, captured by a block that writes `x` | error: `AssignToImmutable` (**L-3** — currently unguarded, see §12B.4) |
| the same name declared in a **nested** scope | legal — shadowing is specified and tested, and stays |
| `const (a, b) = e` | immutable destructuring — pattern rules unchanged (U14/ADR-0046) |
| `let (a, b) = e` | mutable destructuring |
| `const (a, b)` | error — a destructuring pattern always required an initializer *regardless of kind*, and still does |

**Destructuring is untouched.** `Pattern` and every U14 production are outside this unit's
write-set; only the leading keyword changes. `FieldDef.name` is a `String`, not a
`Pattern`, so field destructuring does not exist and cannot be affected.

The immutability check itself already exists (it enforces old-`let` today) and simply
follows `const`. `Compiler::immutable_globals` (`compiler/lib/mod.rs:51`) is the existing
mechanism.

**Safety property worth stating:** the `let` → `const` direction cannot break existing code,
because `let x` with no initializer is *already* a compile error (P5). There is no
uninitialized `let` in the corpus to migrate into an illegal `const x`.

---

## 4. Field grammar (class-body position)

```
field_decl := { attribute } [ "const" ] IDENT_leading_underscore [ "=" expr ]
```

| Form | Rule |
|---|---|
| `_x` | mutable, no default |
| `_x = e` | mutable, default `e` |
| `const _x = e` | immutable, defined at declaration |
| `const _x` | immutable, assignable **only inside a constructor body** |
| `let _x` | error: `field.no_mutable_keyword` (**L-2** — there is no third field form) |
| `var _x` | error — `var` does not exist (**L-1**) |

### 4.1 How to disambiguate without a `FIELD` token

Per §1.1 there is no lexical marker. `parse_class_member`
(`phalcom-ast/src/parser.rs:1234`) must decide between a field declaration and a
getter/method on lookahead. The rule:

> At class-body position, an identifier with a leading `_` that is **not** followed by
> `(`, `=>`, or `{` is a field declaration.

Concretely, in dispatch order:

1. `Token::Const` → field declaration (unambiguous, new keyword).
2. `Token::Let` → **error** `field.no_mutable_keyword` (do not silently accept).
3. `Token::LBracket` → subscript member (U-INDEX, unchanged, must stay ahead of the name branches).
4. `Token::Construct` → constructor (unchanged).
5. Identifier starting with `_`, with lookahead ∈ {newline, `}`, EOF, `=`} → field declaration.
6. Otherwise → existing `parse_method_name` path.

Rule 5 subsumes and replaces the `parser.rs:1269-1282` non-static getter branch (§1.2).
`parse_field_decl` (`parser.rs:1153`) currently does `self.advance(); // 'let' or 'var'`
unconditionally — that advance becomes conditional on a `const` having been seen.

### 4.2 What must not regress

- `static _x = e` keeps working (the one corpus site, `class_static_field_shared_state.ph:9`).
  This is the **only** leading-underscore class-body form that functions at `de49d3a`.
- Rule 5 requires the lookahead to *not* be `=>`/`{`, so `_x => 5` and `_x { … }` still
  route to the getter path. Per §1.2 that path is already broken for underscore names, so
  this preserves nothing that works — it merely avoids *changing* the failure mode inside
  this unit. **Fixing the underscore-getter crash is explicitly out of scope here**; if it
  is fixed separately, rule 5's lookahead set is the contract to re-check.
- Attributes precede the optional `const`, so `@get @set _x` and `@class const _limit = 10`
  both parse with no further grammar change. `plan.md` §"not preclude" asks for the second
  form to be verified before U-CTOR starts — do it here, in a fixture.

---

## 5. `const` field enforcement — syntactic, and it needs new compiler state

> A write to a `const` field outside a constructor body is `field.const_write`.

Keyed on **which member the write appears in**. No flow analysis — Phalcom has none, by
ADR-0052's precedent.

**Gap in the plan:** it says "the compiler knows the enclosing member". It does not.
`Compiler` (`phalcom-core/src/compiler/lib/mod.rs:39-80`) carries `module`,
`immutable_globals`, `current_class`, `is_static_context`, `loop_contexts`,
`deopt_fallback_depth` — there is **no** current-member-kind field. One must be added,
modelled directly on `is_static_context` (`mod.rs:55`):

```rust
/// Whether the member currently being compiled is a constructor body.
/// Gates `const`-field writes (ADR-0064 §3) — the only place they are legal.
in_constructor: bool,
```

Set it where `ClassMember::Construct` is compiled (`compiler/lib/class_decl.rs:104,242,597`
— three sites, all must be covered) and clear it on exit. The check goes at the field-write
emission site, `compiler/lib/expr.rs:329,333` (`Bytecode::SetField`), where the slot is
already resolved.

### 5.1 Accepted gaps — do not try to close these

Per ADR-0064 §3:

- two writes to the same `const` field inside one constructor: **not caught** (needs
  definite assignment).
- a constructor that never assigns a `const _id`: **not caught** → reads `None` forever.

The second is reachable via ADR-0063 §7 (`Factory.new()` bypasses constructors) and is
**specified behavior, not a bug**. Fixture it (§9) so it stays specified rather than
drifting into a bug report later.

---

## 6. The codemod

### 6.1 Single pass, always

`var` → `let` followed by `let` → `const` turns every original `var` into `const`. It is a
swap, so it must be one pass — one alternation, or a placeholder round-trip:

```sh
sed -E -i '' 's/\blet\b/@@CONST@@/g; s/\bvar\b/let/g; s/@@CONST@@/const/g' <files>
```

This correctly handles all 1071 blanket sites (713 + 15 + 343) **and** comment text. It
does *not* handle the 16 field sites — see §6.2.

Recount before running; §0 P3 shows the figures already drifted once.

### 6.2 The 16 position-dependent sites

Every `var _x` in the corpus, verified 2026-07-19 to be class-body field declarations (all
at class-body indentation, several carrying attributes). Each becomes bare `_x`:

```
benchmarks/annotations/showcase.ph:69          @get @set var _x
benchmarks/annotations/showcase.ph:71          @get @set var _y
benchmarks/annotations/showcase.ph:114         @observable var _items
phalcom-core/core/core.ph:1641                 var _targets
phalcom-core/core/core.ph:1642                 var _tier
phalcom-core/tests/lang/compile-errors/attr_missing_hook.ph:9                  var _cache
phalcom-core/tests/lang/compile-errors/annotation_data_eq_hash_collision.ph:9  var _cents
phalcom-core/tests/lang/decorators/decorators_attribute_retention.ph:12        var _name
phalcom-core/tests/lang/decorators/decorators_attribute_retention.ph:19        var _label
phalcom-core/tests/lang/runtime-errors/runtime_attribute_store_frozen.ph:10    var _name
phalcom-core/tests/lang/errors/annotation_data_derive_full.ph:11               var _cents
phalcom-core/tests/lang/errors/annotation_data_derive_full.ph:12               var _currency
phalcom-core/tests/lang/errors/annotation_construct_own_fields.ph:9            var _x
phalcom-core/tests/lang/errors/annotation_construct_own_fields.ph:10           var _y
phalcom-core/tests/lang/errors/annotation_data_with_shallow_copy.ph:12         var _items
phalcom-core/tests/lang/errors/annotation_data_with_shallow_copy.ph:13         var _owner
```

Note the concentration: 11 of 16 are annotation/decorator fixtures, and `core.ph:1641-1642`
is the `On` attribute class. The `@get @set var _x` and `@observable var _items` forms are
the ones that prove attributes-before-`const` parses (§4.2).

### 6.3 Gate

The golden corpus is the oracle. Capture full suite stdout *before* the codemod, diff
after. One pre-registered exception (§1.3). Anything else that moves is a regression.

---

## 7. Write-set

| File | Change |
|---|---|
| `phalcom-ast/src/token.rs` | `Token::Const` in; `Token::Var` out (step 2) |
| `phalcom-ast/src/lexer.rs:262-263` | keyword table: add `"const"`, drop `"var"` (step 2) |
| `phalcom-ast/src/ast.rs:296-303,405-437` | `BindingKind` variant rename + doc rewrite; `FieldDef.mutable` doc; `LetBinding` doc |
| `phalcom-ast/src/parser.rs:675` | `parse_binding`: `const`-without-initializer rejection; `BindingKind` mapping flip |
| `phalcom-ast/src/parser.rs:1153` | `parse_field_decl`: conditional keyword advance, `const` handling |
| `phalcom-ast/src/parser.rs:1234-1282` | `parse_class_member`: new dispatch (§4.1); delete the non-static `_x = e` getter branch |
| `phalcom-core/src/compiler/lib/mod.rs:39-80` | add `in_constructor: bool` |
| `phalcom-core/src/compiler/lib/class_decl.rs:104,242,597` | set/clear `in_constructor` at all three `Construct` sites |
| `phalcom-core/src/compiler/lib/expr.rs:329,333` | `field.const_write` check at `SetField` emission |
| `phalcom-core/src/compiler/lib/error.rs:22-36` | rewrite two diagnostics, add three |
| `phalcom-core/core/core.ph:1641-1642` | codemod (field sites) + 111 blanket sites |
| corpus `.ph` ×401 files | the codemod, 1087 sites |
| `phalcom-lsp/src/semantic_tokens.rs:157-158` | keyword-classification arm: drop `Token::Var`, add `Token::Const` |
| `phalcom-lsp/src/hover.rs:125-130` | `KEYWORD_DOCS`: delete the `var` entry; **the `let` blurb is now backwards** (says immutable) — rewrite as mutable; add a `const` entry |
| `phalcom-lsp/src/hover.rs:185-186` | `keyword_spelling`: drop `Token::Var`, add `Token::Const` |
| `phalcom-lsp/src/hover.rs:605` | coverage test list: `"var"` → `"const"` |
| `phalcom-lsp/src/hover.rs:703-709` | test `phaldoc_attaches_to_a_top_level_var_binding` uses `var total = 0` — rename + respell, or split into mutable/immutable cases |
| `phalcom-lsp/src/{backend,completion,index,hover}.rs` | doc-comment prose naming the old pair — `backend.rs:256,258,326`, `completion.rs:91,294`, `index.rs:463,470`, `hover.rs:53-58,393,397,429` |
| **`phalcom-core/bin/gen-core-table/main.rs:44-47`** | `KEYWORDS` array — **the generator, and the real source of truth**. Has `var`, is missing `let` entirely. Edit here, then regenerate |
| `tools/vsphalcom/src/generated/core-table.json:2-20` | regenerated output of the above — do not hand-edit |
| `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json:121` | TextMate keyword regex: drop `var`, add `let`/`const` (`let` is absent today — pre-existing highlighting gap this fixes) |
| `tools/vsphalcom/manual-test/01-syntax-highlighting.ph` | lines 35,41-43,49-50,56-57,63,93 use `var`; line 88 keyword-coverage comment; line 6 calls `const` a dead keyword |
| `tools/vsphalcom/manual-test/03-diagnostics-error.ph:5,13` | `let x = ;` broken-initializer fixture + its comment |
| `tools/vsphalcom/manual-test/CHECKLIST.md:34-36,56,58` | **asserts `const` must NOT be colored** ("dead 2023 keyword") — now exactly backwards |
| **`phalcom-repl/src/common.rs:11-14`** | `KEYWORDS` — has `let`, missing `var`; add `const`. Feeds the completer *and* `highlighter.rs:19` |
| `phalcom-repl/src/completer.rs:113` | duplicate of the above array (its own comment admits the dupe) |
| `phalcom-repl/src/rustyline/completer.rs:73` | **third** independent copy, doesn't import `common::KEYWORDS` |
| `fuzz/phalcom.dict:8,80` | `"let"` entries stay valid; add `"const"` |
| `docs/spec/v0.2/syntax/{grammar,lexical,statements-and-declarations}.md`, `classes.md` | ⚠️ **already migrated (§1.5) — verify only, do not rewrite** |
| `docs/adr/accepted/0014-*.md`, `docs/adr/STATUS.md` | ⚠️ **already flipped (§1.5) — no action** |
| `docs/adr/README.md:57` | ADR-0014 row still says Accepted; **ADR-0064 has no row at all** (§1.6) |
| ~30 further `docs/spec/v0.2/**` files | still describe the old pair — `implementation-status.md:26-27,60`, `destructuring.md` (title included), `values-and-absence.md`, `modules.md`, `iteration.md:50`, `selectors.md:236-239`, `is-tests.md:122`, `open-questions.md`, `deferred-work.md`, plus most of `decorators/` and `drafts/` and `experimental/`. **Scope decision required — see §12.5** |

---

## 8. Build order

`plan.md`'s order leaves the tree red in the middle: step 1 deletes `Token::Var`, but the
16 class-body `var _x` sites have no valid spelling until step 2's grammar exists. Bare
`_x` is a parse error today (probed). Corrected:

**Step 1 — bindings, `var` retained as a deprecated alias.**
Add `Token::Const`, invert `Token::Let` to mutable, **keep `Token::Var`** still parsing to a
mutable binding/field. Rewrite the two diagnostics and rebaseline the one pre-registered
`.expected`. Run the blanket codemod over all 1071 statement sites, **excluding** the 16
field sites (§6.2). Green.

**Step 2 — field grammar, then `var` dies.**
Add bare `_x` and `const _x` (§4.1), reject `let _x`, delete the non-static `_x = e` getter
branch (§1.2). Rewrite the 16 field sites. Only now delete `Token::Var` and drop `"var"`
from the keyword table. Green.

**Step 3 — `const` field enforcement.**
Add `in_constructor`, wire the three `Construct` sites, add the `field.const_write` check.
Green.

**Step 4 — tooling and docs.**
Not "docs" as `plan.md` framed it — the syntax spec and the ADR-0014 flip are already done
(§1.5). What actually remains:

- **Keyword lists in four independent copies.** `gen-core-table/main.rs:44-47` (regenerate
  `core-table.json` after), the tmLanguage regex, and `phalcom-repl`'s three duplicated
  arrays. None of them currently agree with each other — `let` is missing from the grammar
  and the generator, `var` is missing from the repl. This unit is the natural moment to
  collapse the duplication, but that is a judgment call, not a requirement.
- **LSP hover blurbs**, where the `let` description is not merely stale but *inverted*.
- **`CHECKLIST.md:34-36`**, which currently instructs a human tester to verify that `const`
  is **not** highlighted. Left unfixed, it will produce a false bug report on the first
  post-migration dev-host run.
- **`docs/adr/README.md:57`** (§1.6).
- The ~30 remaining spec files, per the §12.5 scope decision.

Commit per green step. Verify each commit from a **clean throwaway worktree at the SHA**,
not in-tree — an in-tree gate hides partial-stage commits. Main has live concurrent
sessions: commit narrow paths directly on `main`, do **not** branch, and never `git add -A`.

---

## 9. Tests

**Positive lane** (`tests/lang/bindings/`):

- `bindings_let_mutable.ph` — `let x = 1  x = 2` → `2`
- `bindings_let_uninit_reads_none.ph` — `let x` → `None` (ADR-0007 carried over)
- `bindings_const_immutable.ph` — `const x = 1`, read it
- `bindings_const_destructure.ph` — `const (a, b) = (1, 2)` (pattern rules unchanged)
- `bindings_let_destructure_mutable.ph` — `let (a, b) = (1, 2)` then reassign `a`
- `field_mutable_no_keyword.ph` — bare `_x` declaration + mutation
- `field_mutable_with_default.ph` — `_x = 10` (the form that replaces the deleted getter branch)
- `field_const_declared_default.ph` — `const _x = 10`
- `field_const_written_in_ctor.ph` — `const _id` + `construct new(v) { _id = v }`
- `field_attributes_before_const.ph` — `@get @set const _x = 1` (proves §4.2, unblocks U-CTOR)

Do **not** write a `_x => 5`-is-still-a-getter fixture — that form is broken at `de49d3a`
(§1.2) and such a fixture would be red on arrival.

**Negative lane** (`tests/lang/compile-errors/`):

- `binding_const_no_initializer.ph` — `const x` → `binding.const_requires_initializer`
- `binding_const_reassigned.ph` — `const x = 1  x = 2`
- `field_const_write_outside_ctor.ph` — **the P4 defect** — `clobber(v) { _n = v }` on `const _n` → `field.const_write`
- `field_let_keyword_rejected.ph` — `let _x` → `field.no_mutable_keyword`
- `binding_const_redeclared_same_scope.ph` — `const x = 1` · `const x = 2` → `binding.redeclared` (**L-3**)
- `binding_const_redeclare_as_let_rejected.ph` — `const x = 1` · `let x = 2` → `binding.redeclared`; immutability is not releasable (**L-3**)
- `binding_const_captured_write_rejected.ph` — the §12B.4 upvalue hole: a block writing a captured `const` → `AssignToImmutable`. **Closes DEFERRED #13** — strike that entry when this goes green

**Must stay green (shadowing is legal and specified):**

- `binding_let_shadow_inner_block.ph`, `binding_let_shadow_in_loop_body.ph` — existing
  fixtures. L-3 bans *same-scope* redeclaration only; nested shadowing is untouched. If
  either reddens, the same-name check was keyed on name instead of name-at-depth.

**Positive control:**

- `binding_var_is_not_a_keyword.ph` — `let var = 1` must now parse; `var` is an ordinary identifier

**Specified-sharp-edge lane:**

- `field_const_never_assigned_reads_none.ph` — a `const _id` no constructor assigns, reached
  via the bare allocator → `None`. Pins ADR-0064 §3's second accepted gap as *intended*.

**Regression guard for §1.2:**

- `class_static_field_shared_state.ph` (existing) must stay green — it is the only corpus
  user of `static _x = e`.

**Fixtures must be proven wired.** The harness runs one test per *lane*, iterating the
directory, so a new file can be silently skipped. Corrupt each `.expected`, confirm the
suite reddens, restore.

---

## 10. Gates

- full `cargo test` green
- golden stdout byte-identical across the codemod, **except** the single pre-registered
  `compile_error_let_reassignment.expected` (§1.3)
- `cargo clippy --workspace` — 13 pre-existing, none new
- `cargo doc` clean (all touched public items keep full rustdoc)
- `graphify update .`
- ADR-0014 and `STATUS.md` agree after the flip

---

## 11. What must this not preclude

- **Definite-assignment analysis** — §5 is deliberately weaker than "a `const` field is
  always assigned". If flow analysis ever lands, it tightens with no surface change.
- **ADR-0011 read-before-write** — orthogonal (that is *any* write anywhere; this is *where*
  a write may appear). Both must pass together; add a fixture combining them.
- **U-CTOR's `@class` on fields** — the optional `const` sits after attributes, so
  `@class const _limit = 10` needs no further grammar change. §9 fixtures this.
- **`const` on parameters or methods** — the keyword is not spent elsewhere.
- **ADR-0061** (Proposed, underscore prefixes) — §4.1's rule 5 keys on a leading `_`, which
  is exactly what ADR-0061 would formalize. If 0061 lands first, rule 5 must satisfy its §6
  rather than duplicate it. **These two units should be sequenced deliberately, not raced.**

---

## 12A. Locked decisions (user ruling, 2026-07-19)

These are **ruled, not proposed**. They extend ADR-0064 — which is spelling-only — with
enforcement it does not currently specify. An implementer may not reopen them.

| # | Ruling |
|---|---|
| **L-1** | **`var` does not exist after this unit.** Not deprecated, not an alias — removed from the token set, the keyword table, and every keyword list in the tree. The step-1 alias in §8 is a transient build device that must be gone by the end of step 2. |
| **L-2** | **Fields are `_x` or `const _x`. Nothing else.** No `let _x`, no `var _x`, no third form. `let _x` is a hard error (`field.no_mutable_keyword`). |
| **L-3** | **`const` is enforced on every path**, not just direct assignment: assignment, reassignment through a captured upvalue, and same-scope redeclaration are all errors. See §12B for the four paths and which are currently unguarded. |
| ~~**L-4**~~ | ~~Duplicate field declarations are an error.~~ **WITHDRAWN 2026-07-19 — owned by [decision 0065](../../../decisions/0065-classes-are-closed.md) instead.** Its decision item 2 already rules redefinition within a module a compile error, explicitly covering "a duplicate member within one body — a field or a method", with the diagnostic `X is already defined` carrying both spans. Two units had independently ruled the same thing on the same day with conflicting diagnostic names. **U-BINDINGS does not implement this**; do not add a duplicate-field check here. Corpus cost is zero either way (a brace-depth scan finds 0 duplicate field declarations), so nothing is lost by deferring it to 0065's unit. |
| **L-6** | **All implicit bindings are immutable.** Method parameters, block parameters, and the `for` loop variable may not be reassigned. Today params are mutable and the loop variable is not, with nothing declaring or documenting the split. **Corpus cost: zero** — an AST-shaped scan finds **0** methods that assign to their own parameter. Fills the gap ADR-0064 §167-168 explicitly left open (`const` on parameters "not spent on any other position"). To vary a parameter, declare a local from it. `add_local`'s synthetic receiver/parameter slots (`scope.rs:114`) must stop passing `is_mutable: true`. |
| **L-7** | **The ~30 stale spec files are in scope** (resolves the former §12.5). The same single-pass rewrite covers `.md` and `.ph`. Includes the three contradictions below, which are *not* mere spelling drift: `open-questions.md` Q1 (`:28-33`, `:202`) still states the pre-ADR-0064 mapping as RESOLVED with **zero** references to ADR-0064 — a reader following Q1 alone writes both keywords wrong; `deferred-work.md` frames its live re-opening concern in `var` terms; `values-and-absence.md:8` cites ADR-0014 as governing **via a broken relative path** (`../../adr/0014-…`, missing `accepted/`) with no supersession note. |
| **L-8** | **Field names must begin with `_` + a letter.** `class Z { var foo = 1 }` is accepted today and declares an **unreachable slot** — verified: `foo` in a method body resolves as a variable, giving `Undefined variable 'foo'`, while the declaration silently consumes a slot. The new grammar (§4.1 rule 5) closes this by construction, but only if the leading-`_` requirement is *enforced* rather than conventional. This is ADR-0061's field-name tightening, ratified 2026-07-19 **scoped to field position** — bare `_` remains a legal *binding* name, so the 13 corpus `let _ = …` throwaway reads are untouched. Enforce the rule in `parse_field_decl`, **not** in the lexer or `parse_primary`, or the scoping is lost. |
| **L-9** | **Destructuring scratch locals get unique names** — `$destructure` becomes per-claim unique (e.g. depth- or counter-indexed), so **L-5 ships with no `$`-prefix exemption**. Chosen over reclaiming the slots, which is deferred to [`docs/deferred/destructuring-scratch-slots.md`](../../../deferred/destructuring-scratch-slots.md). Rationale: the scratch names are **write-only** (`patterns.rs` never resolves them; `compile_pattern_bind_from_slot` takes an explicit `value_slot`), so the rename is semantically inert — while reclamation touches slot-allocation order, which this unit is *already* rewriting for L-3/L-5. **Nesting makes a single reusable slot impossible**: `let ((a, b), c) = …` holds two scratches at once, because the `Pattern::Tuple` arm re-enters itself while the outer scratch is still live. The generator must therefore produce a fresh name per claim, not per statement. |
| **L-5** | **Same-scope redeclaration is an error for `let` as well as `const`** (user ruling, 2026-07-19, resolving §12A.1). One name, one declaration, per scope. Nested-scope shadowing stays legal. **Corpus cost: one fixture** — `ic_global_cache_shadow_invalidates.ph:34`'s third case (idempotent re-declare) must be restructured; see §12A.1. This also deletes the global-vs-local divergence in §12B by construction: if a name cannot be redeclared, the two mechanisms can no longer disagree about what a redeclaration means. |

### 12A.1 One sub-case the evidence decides differently

L-3 bans same-scope redeclaration of **`const`**. Whether to also ban it for mutable `let`
is a separate question, and the corpus answers it with exactly two data points:

**Site 1 — deliberate, and it is mutable.**
`phalcom-core/tests/lang/ic/ic_global_cache_shadow_invalidates.ph:34`, `var List = 7`
(→ `let List = 7` after the rename). Its own comment specifies the semantics under test:

> Re-declaration returns the existing slot (declare is idempotent), so this is a plain
> write, not a new binding.

This is one of three cases that fixture uses to exercise global-cache invalidation
(fresh-slot declare, plain assignment, idempotent re-declare). Banning mutable
redeclaration deletes the third case.

**Site 2 — accidental, and it is immutable.**
`phalcom-core/tests/lang/iteration/new_capabilities_goldens.ph:38-39`, `let r = …` twice
(→ `const r` after the rename). The intervening comment is unedited authoring
debris — "*Wait, is inclusive true start and end inclusive? Let's check…*" — with a
first attempt left in place above the corrected line. **This is precisely the bug L-3
catches**, sitting in the committed test corpus today, invisible because the compiler
accepts it.

So L-3 as written costs nothing and catches the one real defect. Extending the ban to
mutable `let` costs one deliberate test case.

**RULED 2026-07-19 — extend the ban (L-5).** Mutable redeclaration is *already* incoherent
across scopes: at global scope it reuses the slot (idempotent write), at local scope
`add_local` unconditionally pushes a **new** slot (§12B.2), so the same source construct
means two different things depending on where it appears. Keeping it legal would mean
*specifying* that divergence; banning it deletes the divergence by construction.

Consequence to schedule: `ic_global_cache_shadow_invalidates.ph`'s third case must be
restructured. Its first two cases (fresh-slot declare, plain assignment) are unaffected.
The idempotent-declare path still exists in the VM — it simply becomes unreachable from
same-scope source, so the case moves to a compile-error fixture or is dropped.

## 12B. Investigation: why global and local disagree

Requested follow-up. Verified against the tree, then re-derived from source rather than
inferred from black-box behavior.

### 12B.1 Observed

| Program | Result |
|---|---|
| top level: `let a = 1` · `var a = 2` · `a = 3` | ❌ rejected |
| method body: `let z = 1` · `var z = 2` · `z = 3` | ✅ `3` |
| top level: `var a = 1` · `let a = 2` · `a = 3` | ❌ rejected |

At global scope immutability can be **acquired but never released**. At local scope a
redeclaration supersedes cleanly. Same source construct, two semantics.

### 12B.2 Mechanism

**Globals — a monotonic name set.** `immutable_globals: HashSet<Symbol>`
(`compiler/lib/mod.rs:51`) has **two insert sites and one read, and no `remove` anywhere**:

| Site | Role |
|---|---|
| `compiler/lib/patterns.rs:47` | insert on an immutable binding |
| `compiler/lib/mod.rs:420` | insert on `import … as Name` |
| `compiler/lib/expr.rs:302` | `contains` → `AssignToImmutable` |

There is no record of a global's *declared kind* — only a set of names that are immutable.
Once a name enters, nothing takes it out, so a later mutable redeclaration writes the value
(`DefineGlobal` is idempotent on the slot) while the immutability flag survives.

**Locals — a stack with last-wins lookup.** `add_local`
(`compiler/lib/scope.rs:114`) pushes a fresh `Local { name, depth, is_captured, is_mutable }`
and **never checks for an existing binding of the same name**. `resolve_local_in`
(`scope.rs:131`) scans `(0..num_locals).rev().find(…)` — reverse, last-wins. So the
redeclaration's own `is_mutable` governs.

That asymmetry is the whole defect. It is two independent mechanisms that were never
reconciled, not a rule anyone wrote down.

### 12B.3 A second consequence: local redeclaration leaks frame slots

Because `add_local` always pushes, `let a = 1` three times in one scope allocates **three
slots** and raises `max_slots`. `max_slots` sizes the fiber stack frame, so redeclaration
inflates every activation of that function forever. Minor today (corpus redeclarations: 2),
but it is a second reason the local path needs the same-name check L-3 requires anyway.

### 12B.4 The four immutability paths — three guarded, one not

| Path | Bytecode | Guarded? |
|---|---|---|
| direct local write | `SetLocal` | ✅ rejected |
| global write | `SetGlobal` | ✅ rejected |
| **captured write through a closure** | `SetUpvalue` | ❌ **allowed** |
| same-scope redeclaration | `DefineGlobal` / `add_local` | ❌ **allowed** |

The upvalue hole is real and reproduces:

```phalcom
class M {
  run {
    let a = 1
    let f = { a = 2 }
    f.call()
    return a        // → 2
  }
}
```

`compiler/lib/expr.rs:293-298` carries an explicit NOTE admitting it and pointing at
`DEFERRED.md`. **That claim checks out** — `DEFERRED.md` entry **#13** describes it
accurately, and already anticipates this unit:

> filed against ADR-0014's `let`/`var` spelling; ADR-0064 supersedes that spelling but keeps
> every rule, so the hole is unchanged — it is now a hole in `const`. Re-verify against
> U-BINDINGS when it lands.

So DEFERRED #13 is not a stale entry; it is a correctly-parked precondition, and L-3 is the
thing that closes it. Fixing it means walking the enclosing function-states in the
assignment path instead of stopping at the current function.

### 12B.5 Work implied by L-3 and L-5

Four changes, all in the compiler, none in the codemod:

1. **Give globals a declared kind.** Replace `immutable_globals: HashSet<Symbol>` with a map
   from name to kind, so a redeclaration can be diagnosed rather than silently merged. A
   `remove` is *not* the fix — that would make `const` releasable, which L-3 forbids.
2. **Reject same-scope redeclaration.** Globals: on the second `DefineGlobal` for a name in
   the same module scope. Locals: `add_local` checks `func.locals` for the same name at
   `func.scope_depth` before pushing. **Nested-scope shadowing must stay legal** — it is
   specified and covered by `binding_let_shadow_inner_block.ph` and
   `binding_let_shadow_in_loop_body.ph`, both of which pass today and must keep passing.
3. **Guard the upvalue path** (`expr.rs:293-298`), closing DEFERRED #13.

Diagnostics to add: `binding.redeclared`, plus reusing
`AssignToImmutable` for the upvalue path.

## 12C. Removed from scope: DEFERRED #17 — `class None` reopen clobbers its global

**This unit no longer fixes #17.** It was folded in on 2026-07-19 and pulled back out the
same day, once [decision 0065](../../../decisions/0065-classes-are-closed.md) (*Classes are
closed: remove class reopening*, Accepted 2026-07-19) surfaced. 0065 lists `DEFERRED.md` #17
under its own Related set.

> **Correction 2026-07-19** (this paragraph previously read "the clobber is unreachable, so a
> fix here would be dead code"). The fix is **not** dead code, and #17 does not close. Under
> 0065 the defect dissolves only as a *user-reachable* bug — ruling 3 reserves kernel names, so
> nobody outside core can write `class None`. It **survives as a bootstrap task**:
> `vm/bootstrap.rs:262-265` inserts the `None` class row expressly so `core.ph` **can** complete
> that stub, and ruling 4 sanctions exactly that. The guard is the prerequisite for `None` ever
> carrying members or `@sealed` (`DEFERRED` #35). Ownership is now explicit:
> [`U-CLASSCLOSE`](../U-CLASSCLOSE/plan.md) §3.5 lands it, because that unit already edits the
> `Statement::Class` lowering path. Giving `None` a body is separately deferred —
> [`docs/deferred/class-sealing-followups.md`](../../../deferred/class-sealing-followups.md)
> item 3.

**Carried forward for whoever implements 0065**, because it is easy to mis-test:

- The defect: `Statement::Class` emits `DefineGlobal` unconditionally, including on reopen.
  `None`'s global is deliberately bound to the singleton *instance*, not the class
  (`VM::install_core`), so reopening `class None` rebinds it to the class object.
- **The obvious test does not detect it.** `class None { … }` then `var x = None;
  x == None` reports `true` either way — both sides read the same clobbered binding. The
  comparison must use a *genuinely produced* `None`:
  `Some.new(5).filter { x => false } == None` → `true` before, `false` after.
- `isNone` keeps answering correctly throughout; only the *binding* moves. That asymmetry is
  what keeps it quiet.
- #17's own line numbers predate U-REOPEN-FIX (`e85f31a`) and should not be trusted.

**One interaction survives into this unit.** L-5 rejects same-scope redeclaration at the
`DefineGlobal` site. Until 0065 lands, class reopening still exists and a reopen must **not**
be treated as a redeclaration — `core.ph` completes kernel stubs through that path and
bootstrap fails immediately otherwise. Note that stub completion and true reopen are already
distinct branches (`class_decl.rs:332` vs `:308`, discriminated by a `field_layouts` miss vs
hit), so the exemption has a precise predicate rather than needing a heuristic.

## 12D. Issues flagged for the user

1. **ADR-0061's ratification is not yet filed, deliberately.** The ruling is recorded here
   and in the banner, but the ADR file still sits in `docs/adr/proposed/` marked Proposed.
   The edit was held because the repo is mid-convention-change:
   [`docs/decisions/README.md`](../../../decisions/README.md) (uncommitted, authored by a
   concurrent session) establishes a **flat** directory where *status is never encoded in a
   path*, precisely because "a status change is a file move" guarantees drift. So the old
   convention's `git mv proposed/ → accepted/` is the exact operation the new one exists to
   eliminate, while leaving an Accepted ADR inside `proposed/` violates the old one. Filing
   it also means editing `docs/adr/` structure while another session works there
   uncommitted. **The minimal non-colliding action** — flip the status line in place, update
   `docs/adr/STATUS.md`, fix `docs/adr/README.md`'s rows (0056/0059 point at `proposed/`;
   0061 and 0064 have no rows at all), and append the field-position amendment — is correct
   under both conventions and awaits a go-ahead. **Same holds for the ADR-0056 and ADR-0059
   ratifications ruled the same day.**

2. **Field-name shadowing across a hierarchy stays silent — ruled 2026-07-19.** A subclass
   declaring a field whose name matches a parent's gets its own separate slot (verified:
   parent reads `1`, subclass reads `9`). No diagnostic, no work: ADR-0011 makes fields
   private and non-inherited, so the names collide but the slots were never related. Recorded
   so it is not re-litigated as a bug.

3. **§1.2 is a latent crash, and it is broader than the field grammar.** *Every* non-static
   leading-underscore class member is broken at `de49d3a` — `_x = 5` and `_x => 5` both
   reach `Internal error: Static field slot 0 out of bounds` at **class-definition time**,
   and `_x { … }` reaches `Invalid field initializer body`. U-BINDINGS' field grammar
   absorbs the `_x`/`_x = e` forms and so fixes those two incidentally, but it does **not**
   fix the `=>`/block getter forms — those stay broken unless someone scopes a separate fix.
   Corpus exposure is zero, so there is no urgency, but this is a real internal error
   reachable from ordinary-looking source and it deserves its own entry in `DEFERRED.md`
   whether or not U-BINDINGS runs.

4. **The plan's risk framing was inverted** (§1.4). If U-BINDINGS was deprioritized on the
   strength of "1080-site AST-driven codemod", that estimate was wrong by roughly the ratio
   1071 blanket : 16 hard. The unit is smaller than it reads.
5. **No user-visible bug is fixed by §5** (§1.7). Weigh accordingly against units that do.

6. **Keyword lists are duplicated four ways and already disagree.** `let` is missing from
   the tmLanguage grammar and from `gen-core-table`; `var` is missing from all three
   `phalcom-repl` copies. So Phalcom's own editor tooling has never highlighted the full
   keyword set correctly. This unit must touch all four regardless; whether to also
   de-duplicate them into one source is a separate, optional cleanup.
</content>
