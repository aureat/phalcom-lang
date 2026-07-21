# U-ANNOT-CONTRACTS — Work order: `@` core mechanism + `@requires`/`@ensures`/`@invariant`

_Self-contained implementation plan for **one** implementer. Compiler/AST unit — touches
`phalcom-ast/src/ast.rs`, `phalcom-ast/src/parser.rs`, and `phalcom-core/src/compiler/` (new
module). **Reviewer ON** (spine files `parser.rs`/`compiler/lib.rs`) — hand the diff to
`phalcom-reviewer`; do not self-approve. Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean. Grounded in **[ADR-0054](../../../adr/0054-two-speed-ratification-annotation-decorator-tiers.md)**
(ratifies this tier), **[ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)**
(Fix 1 — receiver-scoped invariant guard, built in from the start here, not retrofitted), and
normative **[annotations-core.md](../../../design/experimental/v0.2/annotations-core.md)**,
**[annotations-legality-grammar.md](../../../design/experimental/v0.2/annotations-legality-grammar.md)**,
**[annotations-contracts.md](../../../design/experimental/v0.2/annotations-contracts.md)**,
**[annotations-contract-semantics.md](../../../design/experimental/v0.2/annotations-contract-semantics.md)**,
**[annotations-test-strategy.md](../../../design/experimental/v0.2/annotations-test-strategy.md)**.

> **Grounding correction vs. the source drafts (read this before coding).** Three claims in the
> ratified docs do not match HEAD, verified by direct source read (2026-07-13):
> 1. **`Token::At` already exists and already lexes `@`** (`phalcom-ast/src/token.rs` L216,
>    `phalcom-ast/src/lexer.rs` L618: `b'@' => (1, Token::At)`). `annotations-core.md`'s "Lexer:
>    add `Token::At`" step is **already done** — nothing to build there.
> 2. **The member-compile loop is not 3 variants, it's 4.** `annotations-core.md` L14 states the
>    loop "consumes exactly three member kinds — `ClassMember::{Method,Getter,Setter}`". HEAD's
>    `ClassMember` (`phalcom-ast/src/ast.rs` L140–145) has a **fourth** variant, `Construct`,
>    fully wired end-to-end (`ConstructDef`, `parser.rs` L937 `self.eat(&Token::Construct)`,
>    `compiler/lib.rs` L860 field-collection + L1170–1209 compile/alias-registration) — landed in
>    **U7** (`docs/forge/units/U7/as-built.md`), not "unbuilt" as
>    `annotations-construct.md` §"Prerequisite 2" claims. This unit's `expand_class_attributes`
>    must be written against the real 4-variant `ClassMember`; U-ANNOT-LAYOUT's `@construct`
>    derive emits `ClassMember::Construct(ConstructDef{..})` directly — **not** a `MethodDef`
>    with a nonexistent `is_constructor: bool` field, which is what `annotations-construct.md`'s
>    own pseudocode (incorrectly) shows. Flag both docs for a doc-sync pass in the return
>    contract; do not silently "fix" them as part of this unit's diff (out of write-set).
> 3. **`@invariant`'s own worked example doesn't parse under the stated grammar.**
>    `annotations-contracts.md` writes `@invariant => _balance >= 0` with **no following member
>    name** — but `annotations-legality-grammar.md`'s own EBNF (`class-member := attribute*
>    [static] member-decl`) requires attributes to bind to a following `member-decl`, and
>    `parse_method_block`'s `=>`-body form (`parser.rs` L1113–1115) is only reachable *after* a
>    member name is already parsed by `parse_class_member`. There is no name here. See §3.1 for
>    the resolution (DEC-ANNOT-B, resolved by grounding, not blocked).
>
> None of this reopens ADR-0054's ratification — the *design* is settled; these are grammar/impl
> completions the drafts left imprecise, the same class of gap prior units
> (U-ITERABLE §DEC-ITERABLE-A, U7's own construct landing) have resolved by reading HEAD rather
> than re-litigating.

> **Scope correction vs. the stated build order (flag, not silent).** The task ordering groups
> `@get`/`@set` into this ("method-table-macro") tier alongside `@requires`/`@ensures`/
> `@invariant`. **This unit does not build `@get`/`@set`.** Their sole legal target per the
> legality table is `Target::Field` (`annotations-legality-grammar.md`'s table row), and no
> `Field`/`FieldDef` grammar production exists anywhere in `phalcom-ast` today (confirmed: no
> `var`/`let` handling in `parse_class_member`, only in statement-position `parse_statement`,
> `parser.rs` L409–410) — there is **no legal syntax to attach `@get`/`@set` to** until
> `FieldDef` lands. `annotation-paradigm-bridges.md`'s "Unifying finding" table lists `@get`/
> `@set` under "method-table macro / no layout ADR needed", which is true of their *derivation
> logic* (a getter/setter body is trivial to generate) but not of their *grammar dependency* —
> and ADR-0054 §1's own ratification bullet list already groups `@get`/`@set` with
> `annotations-construct.md`, i.e. the layout-tier document, not with `annotations-core.md`/
> `annotations-contracts.md`. This plan follows the grammar-enforced reading: **`@get`/`@set`
> ship in U-ANNOT-LAYOUT**, immediately after `FieldDef` lands there, not in this unit. This
> unit's own scope is the registry/pipeline machinery plus `@requires`/`@ensures`/`@invariant`
> only — all three target `Method`/`Getter`/`Setter`/(`@invariant`) `Class`, member kinds that
> already exist.

## 1. Mission (one sentence)
Land the `@` attribute mechanism's compile-time desugar pass — lexer (already done), AST
(`Attribute` node + `attributes`/`invariants` fields), parser (attribute-collection loop +
newline binding + dangling-attribute diagnostic), and the phase-ordered
(`generate`→`weave`→`finalize`) `AttributeExpander` registry — realized end-to-end through the
three Design-by-Contract attributes `@requires`/`@ensures`/`@invariant`, including ADR-0052's
receiver-scoped, unwind-safe re-entrancy guard and the two-axis (guard/metadata) release-mode
stripping, with **zero instance-layout impact** and **zero new `Value`/opcode surface**.

## 2. Preconditions (verify on actual HEAD — do not assume)
- `Token::At` lexes `@` today (`lexer.rs` L618); unused downstream (no `Token::At` reference
  anywhere in `parser.rs` — confirmed via full-file grep). This unit is the first consumer.
- `construct` keyword, `ConstructDef`, `ClassMember::Construct`, and the compiler's two call
  sites (`compiler/lib.rs` L860, L1170–1209, including `vm.constructor_aliases`/
  `vm.has_new_construct` registration) are fully landed (U7). This unit's expander does not
  touch `Construct` members (no method-table-macro attribute targets `Construct`), but must not
  assume a 3-variant `ClassMember` anywhere in its own types.
- `ClassDef` (`ast.rs` L111–123): `{ name, superclass: Option<SuperclassRef>, members:
  Vec<ClassMember>, range }`. No `attributes` field. `MethodDef`/`GetterDef`/`SetterDef`
  (L178–201) likewise carry no `attributes` field. This unit adds them.
- `Expr` (`ast.rs` L334–359) has `Var{value,range}`, `Block(Box<BlockExpr>)`,
  `MethodCall(Box<MethodCallExpr>)`, `Assignment(Box<AssignmentExpr>)` — the nodes
  `annotations-core.md`/`annotations-contracts.md` assume the expander builds synthetic AST
  from. `Statement` (`ast.rs` L24+) has `Let(LetBinding)`, `Return(ReturnStatement)`,
  `Expr{expr,range}` — confirm `LetBinding`'s exact shape before synthesizing `let __old_0 = …`
  (§3.2).
- **No `Statement::Try`/`Statement::Ensure` AST node exists.** `try`/`on`/`catch`/`ensure` is
  **not** a dedicated statement — `ast.rs` L101–105's own comment states it desugars at parse
  time to a block-chain: `{ P }.on(T) { e => … }.ensure { … }`, i.e. ordinary `Expr::Block` +
  `Expr::MethodCall` sends of the native `Block::ensure(_)` primitive
  (`phalcom-core/src/primitive/block.rs` L277–297, confirmed present). **This is the mechanism
  ADR-0052's unwind-safe epilogue must use**: the invariant-guarded method body is woven as
  `{ <original body> }.ensure({ <cleanup> })`, an `Expr::MethodCall` wrapping an `Expr::Block`
  — not a new Statement variant, not hand-rolled unwind logic. Ground this explicitly before
  writing §3.3; it is the one piece of "builds AST from existing nodes" that isn't obvious from
  the spec prose alone.
- **No fiber-scoped guard state exists anywhere** (`grep checking:\|in_public_call` across
  `vm.rs`/`heap.rs` — zero hits). Wholly new. `FiberObject` (`heap.rs` L237+) already carries
  `stack: Vec<Value>`, `frames: Vec<CallFrame>`, `open_upvalues: BTreeMap<usize,ObjRef>` — all
  swapped in/out on fiber switch (mirrored by `VM::stack`/`frames`/`open_upvalues`). **New
  finding, not stated in ADR-0052**: because an `@invariant`-guarded call can `yield` mid-body
  (the woven prologue/epilogue don't suppress yielding), the `checking` set must be **saved and
  restored on fiber switch exactly like those three fields**, or a suspended fiber's in-flight
  invariant-check bookkeeping silently corrupts/leaks into the resuming fiber. Design decision
  (§3.3): `checking: HashSet<ObjRef>` lives as a field on `FiberObject`, mirrored by
  `VM::checking`, swapped by the same code path that swaps `stack`/`frames`/`open_upvalues` on
  resume/park (`vm.rs`, fiber switch — locate and extend, do not duplicate the swap logic).
- `MethodObject` (`method.rs` L299–306) is a flat 3-field struct (`kind`, `signature`, `holder`)
  — no side-table field for D-contract-1's reflectable `Symbol → [Block]` predicate metadata.
  This unit adds one (a new field, populated only when contracts are present and metadata isn't
  stripped — §3.6).
- `decode_selector`/`encode_selector` (`method.rs` L95, L181) already round-trip a selector
  string ↔ `(name, labels, SignatureKind)` — confirms the reflection-shaped machinery this unit
  leans on for naming already exists; the metadata table itself does not.
- `class Error {}` (`core.ph` L37) is the existing hand-rolled root.
  `PreconditionError`/`PostconditionError`/`InvariantError` are three new `.ph` subclasses
  reopening this exact pattern — zero Rust change, zero new primitive.
- **No compile-mode plumbing exists anywhere** — `grep -rn "release\|unchecked\|CompileMode"`
  across `bin/phalcom/cli.rs` and `compiler/lib.rs` returns nothing. §3.6's `debug`/`release`/
  `unchecked` axis is **wholly new infrastructure**, not a flag this unit merely reads — budget
  for it explicitly (a `CompileMode` enum threaded from `cli.rs` through `Compiler::new`/the
  expander context; default `debug`).
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected
  "parser.rs"` / `graphify affected "compiler/lib.rs"` / `graphify affected "ast.rs"` and check
  for concurrent editors (standing repo hazard — continuous concurrent sessions land on `main`).

## 3. Design (realize the ratified docs — do not re-litigate the model)

### 3.1 Four-layer change, corrected (annotations-core.md, annotations-legality-grammar.md)
- **Lexer**: none — `Token::At` already lexes.
- **AST** (`phalcom-ast/src/ast.rs`):
  ```rust
  /// A `@name(args…)` attribute attached to a class member or, for
  /// `@invariant`, standing alone in a class body (see DEC-ANNOT-B).
  #[derive(Debug, Clone)]
  pub struct Attribute {
      pub name: String,
      pub args: Vec<Expr>,
      pub range: SourceRange,
  }
  ```
  Add `pub attributes: Vec<Attribute>` to `ClassDef`, `MethodDef`, `GetterDef`, `SetterDef`.
  Add `pub invariants: Vec<(Expr, SourceRange)>` to `ClassDef` (DEC-ANNOT-B). **Do not** add
  `attributes` to `ConstructDef` — no method-table-macro or (per U-ANNOT-LAYOUT's own scope)
  layout-tier attribute targets `Construct`.
- **Parser** (`parser.rs`): in `parse_class_body`'s loop (L911–923), before calling
  `parse_class_member`, add an attribute-collection prefix:
  ```rust
  let mut pending_attrs = Vec::new();
  loop {
      self.skip_newlines();
      match self.peek() {
          Token::RBrace | Token::Eof if pending_attrs.is_empty() => break,   // (Eof still errors below)
          Token::RBrace | Token::Eof => return Err(dangling_attribute_error(&pending_attrs)),
          Token::At => {
              let attr = self.parse_attribute()?;   // consumes '@' ident ['(' attr-args? ')']
              self.skip_newlines();                  // newline binding (ADR-0016)
              if attr.name == "invariant" {
                  // DEC-ANNOT-B: standalone class-body item, no following member.
                  class_invariants.push((single_required_arg(&attr)?, attr.range));
                  continue;
              }
              pending_attrs.push(attr);
              continue;
          }
          _ => {}
      }
      let member = self.parse_class_member()?;
      attach_attrs(&mut member, std::mem::take(&mut pending_attrs));
      members.push(member);
  }
  ```
  `attr-args` reuses the existing expression parser (`self.parse_expr()`), comma-separated,
  exactly as `annotations-legality-grammar.md`'s EBNF specifies; a bare identifier not legal as
  a standalone expr in context still parses as `Expr::Var` per that doc's own note — no special
  casing needed, `parse_expr` already produces `Expr::Var` for a bare identifier.
- **Pass**: new module `phalcom-core/src/compiler/attributes.rs`, `expand_class_attributes`
  called at the top of the `Statement::Class` arm (`compiler/lib.rs` L763), **before** either
  member-scan loop (L773 field-collection, L819 reopen-guard) — the expanded `ClassDef` is what
  those loops must see, so pass order is load-bearing:
  ```rust
  pub trait AttributeExpander {
      fn legal_targets(&self) -> &'static [Target];
      /// `ctx` carries the interner/diagnostics sink; `target` is the
      /// already-validated position. Returns members to append (generate) or
      /// mutates the member in place via `ctx` (weave) — see §"Composition".
      fn expand(&self, ctx: &mut ExpandCtx, member: &mut ClassMember, args: &[Expr]) -> Result<(), CompilerError>;
  }
  pub fn expand_class_attributes(class: ClassDef, registry: &AttributeRegistry) -> Result<ClassDef, CompilerError>;
  ```
  Registry rows this unit adds: `"requires"`, `"ensures"`, `"invariant"`. U-ANNOT-LAYOUT adds
  `"construct"`, `"get"`, `"set"`, `"data"`, `"sealed"`, `"variant"` to the **same** registry
  (shared file — see §4.1 collision note).

### 3.2 `@requires`/`@ensures` — per-method weave (annotations-contracts.md verbatim)
Three AST-level steps on `MethodDef.body`/`GetterDef.body`/`SetterDef.body`:
1. **`old(...)` hoist** (only inside `@ensures` args) — each `old(sub)` becomes a synthesized
   `Statement::Let(LetBinding{ name: "__old_N", init: sub, .. })` prepended to the body; the
   occurrence rewrites to `Expr::Var{value:"__old_N",..}`. Confirm `LetBinding`'s exact field
   names before coding (read `ast.rs` around `LetBinding`'s definition — not grepped in this
   plan's own precondition pass, do so first). **`old` on a mutable, non-value operand is
   rejected** (`contract.old_on_mutable`) — annotations-contracts.md's restriction; the
   syntactic check is: operand is not `Expr::Number`/`Expr::String`/`Expr::Boolean` **and** not
   provably a `@data`-tagged type (U-ANNOT-LAYOUT lands later — so for this unit, reject `old`
   on any non-literal operand unconditionally; **loosening this once `@data` exists is
   U-ANNOT-LAYOUT's own concern, not a TODO left dangling here** — state this explicitly in the
   return contract so the follow-on unit knows to revisit the check).
2. **Precondition prologue** — per `@requires(c)`, prepend `c.ifFalse { PreconditionError.raise("<selector> requires …") }` (an `Expr::MethodCall` on `c` sending `ifFalse`, whose block body raises). Multiple `@requires` are independent statements, prepended in **declaration order** — order affects only which failure raises first (annotations-contracts.md: "order-independent... a property of the derive model").
3. **Postcondition epilogue** — bind the body's last expression to `let __result = …`, append each `@ensures(c).ifFalse { PostconditionError.raise(...) }` in declaration order, end with `Expr::Var{"__result"}` to preserve implicit return. **Early `return x` sites**: walk the body for `Statement::Return`, rewrite each to run the postcondition checks against `x` before returning (annotations-contracts.md's stated v1 fix — the pass rewrites each site, does not weave at the compiler's return-emit point).

Predicate purity floor (annotations-contract-semantics.md): reject at expansion time any
`Expr::Assignment` inside a `@requires`/`@ensures`/`@invariant` argument, and any
`Expr::MethodCall` whose selector is a syntactically-known mutator on `self`/a field access
(`_x = `-shaped sends; a fixed small deny-list — `at(_,put:)`, `add`, `remove`, any setter
selector `name=(_)` — is a floor, not a proof, per DEC-C's own precedent). Diagnostic:
`contract.impure_predicate`.

### 3.3 `@invariant` — whole-class weave, receiver-scoped (ADR-0052 Fix 1, verbatim mechanism)
`ClassDef.invariants` (populated in §3.1's parse step) is conjoined in declaration order into
one synthesized private method `__check_invariant()` (a `ClassMember::Method`, `is_static:
false`, selector-unique per class — check for a hand-written `__check_invariant` collision, same
`attr.accessor_collision` diagnostic class as `@get`/`@set`/`@construct` use in U-ANNOT-LAYOUT).
Folded over every **public** (name doesn't start with `_`, per the existing field-privacy
convention already used elsewhere in this codebase — confirm the exact "public" predicate
against how `doesNotUnderstand`/method visibility is determined today before hard-coding it),
**non-static**, **non-constructor** method (`ClassMember::Construct` is checked on exit only —
skip entry-wrap for it, exit-check only, since "object not yet built on entry" per
annotations-contracts.md).

Per ADR-0052, the woven shape (using the `Block::ensure(_)` mechanism from §2's precondition,
not a fabricated Statement):
```
// prologue, prepended:
(checking.contains(self)).ifFalse({
  checking.insert(self)
  self.__check_invariant()
})
// body wrapped:
{ <original body, with early-returns unmodified — ensure covers all exits> }.ensure({
  (checking.contains(self)).ifTrue({    // "this call owned the entry" — see below
    self.__check_invariant()
    checking.remove(self)
  })
})
```
**Ownership tracking correction to ADR-0052's prose**: the ADR's pseudocode gates removal on
"this call owned the entry" without specifying how that's tracked syntactically. Concretely:
entry ownership is `self` **not already being in `checking` at the moment this call's prologue
ran** — capture that as a synthesized local `let __invariant_owner = checking.contains(self).not
` evaluated in the prologue *before* the `insert`, and gate the epilogue's check/remove on
`__invariant_owner` (not on `checking.contains(self)`, which is always true at that point
regardless of ownership once nested calls exist — re-checking membership in the epilogue does
**not** distinguish the owning call from a nested one; only a locally-captured boolean does).
This is a genuine correctness gap in ADR-0052's own pseudocode this unit must not reproduce —
flag it in the return contract as a documented deviation from the ADR's literal pseudocode,
same intent, corrected mechanism.

`checking` is `VM::checking: HashSet<ObjRef>` (or `FiberObject::checking`, swapped on fiber
switch per §2's precondition finding) — surfaced to `.ph` code as a native
zero-argument-ish pair of primitives (`__invariantChecking(self)`/`__invariantEnter(self)`/
`__invariantExit(self)`, small native surface on `Object`/`System`, exact naming is the
implementer's call, document it) rather than a real `Set` instance, to avoid allocating a real
`Set` object per guarded call — this is a **hot-path allocation** hazard on every
`@invariant`-guarded call (see Rubric).

### 3.4 Multiple `@invariant` — conjoined
Already natural: `ClassDef.invariants` is a `Vec`; `__check_invariant` folds them with `and`,
raising on the first failing conjunct with a message naming which one (source-order index or
span).

### 3.5 Contracts are reflectable (D-contract-1)
New `MethodObject` field (behind the metadata-retention axis, §3.6):
```rust
/// Reflectable predicate metadata (D-contract-1) — `Symbol` (e.g. `#requires_0`)
/// → the un-woven predicate as a `Block` `Value`, for property-testing harnesses
/// and `Method>>contracts`. Empty/`None` when contracts are absent or metadata
/// is stripped (annotations-contract-semantics.md's stripping axis).
pub contracts: Option<Vec<(Symbol, Value)>>,
```
Populated by the expander alongside the woven guard, **only** when metadata is retained for the
active `CompileMode` (§3.6) — when stripped, the predicate `Block` is never built in the first
place (never allocated, not allocated-then-freed, per annotations-contract-semantics.md).

### 3.6 Release-mode stripping — new `CompileMode` axis (annotations-contract-semantics.md table)
```rust
pub enum CompileMode { Debug, Release, Unchecked }
```
Threaded `cli.rs` → `Compiler` construction → `ExpandCtx`. Table (verbatim):

| Mode | `@requires` guard | `@ensures` guard | `@invariant` guard | Metadata (default) |
|------|------|------|------|------|
| `debug` (default) | woven | woven | woven | retained |
| `release` | woven | stripped | stripped | retained (opt out `--strip-contract-metadata`) |
| `unchecked` | stripped | stripped | stripped | stripped by default |

Guard stripping = the expander's `weave` step for that attribute is a no-op (emits nothing) in
that mode. Metadata stripping = §3.5's `contracts` field population is skipped. These are
**independent** — implement as two separate boolean checks inside each expander's `expand`,
never coupled.

### Rubric — hazards & preclusion (mandatory)
- **Invariant-set allocation on every guarded call (THE performance hazard this unit must not
  introduce carelessly).** A real `Set`/`Map` instance per call, or a heap-allocated `HashSet`
  cloned per call, taxes every `@invariant`-guarded send. Resolution in §3.3: `checking` is a
  single native `HashSet<ObjRef>` living on `VM`/`FiberObject`, mutated via cheap native
  primitives, never a `.ph`-visible `Set` object reallocated per call. Pin a golden asserting no
  extra heap allocation appears in the disassembly/allocation-count for a guarded call beyond
  the `Block::ensure` closure itself (which the language already pays for `ensure` generally).
- **`ensure` ⊗ non-local return / thrown error (the ADR-0052 §Bug-1 correctness case).**
  `Block::ensure(_)` (already native, `primitive/block.rs`) is confirmed to fire on every unwind
  path (return/throw/fiber abort) — this is the whole reason §3.3 routes through it instead of a
  hand-rolled epilogue. Verify this primitive's actual semantics (read `block_ensure`,
  `primitive/block.rs` L277–320ish) before relying on it — if it does **not** already cover
  fiber-abort unwind, that is a **precondition gap this unit must surface, not silently work
  around**.
- **Ownership-tracking bug in ADR-0052's own pseudocode** (§3.3) — corrected here; do not
  implement the ADR's literal snippet verbatim, it double-checks membership instead of tracking
  per-call ownership.
- **Fiber-switch ⊗ `checking` set (new finding, §2)** — must swap with `stack`/`frames`/
  `open_upvalues`, not be VM-global-only. Pin a golden: a fiber that yields from inside an
  `@invariant`-guarded call, resumed later, still correctly completes its own exit check and
  does not leak into a *different* fiber's invariant bookkeeping running concurrently
  (cooperative, so "concurrently" means interleaved, not parallel — still a real hazard if the
  set is VM-global and not fiber-swapped).
- **`@invariant`'s parse-time exception to newline-binding is a one-off carve-out.** Every other
  attribute in this repo's grammar binds to a following member; `@invariant` alone does not.
  Keep this carve-out textually isolated in the parser (one `if attr.name == "invariant"`
  branch, well-commented with a pointer to DEC-ANNOT-B) so a future attribute added to the
  registry doesn't accidentally inherit it.
- **Predicate purity is a floor, not a proof** (DEC-C precedent, per the interaction-hazard
  catalog's "Enforcement without static analysis" entry) — document this explicitly next to
  `contract.impure_predicate`'s implementation; do not oversell it as sound.
- **Representation/dispatch impact:** zero. No `Value` tag, no opcode, no `SignatureKind`
  change. The only new runtime state is `VM`/`FiberObject::checking` (a `HashSet<ObjRef>`,
  native-only, never surfaced as a `Value`) and `MethodObject::contracts` (an `Option<Vec<...>>`
  field, additive).
- **Precedent:** Eiffel's own outermost-boundary invariant rule (annotations-contract-semantics.md
  cites it directly) — do not reopen intra-method checkpoints (explicitly out of scope, per that
  doc's own "What this precludes").

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-ast/src/ast.rs` | `Attribute` struct; `attributes`/`invariants` fields on `ClassDef`/`MethodDef`/`GetterDef`/`SetterDef` | AST |
| `phalcom-ast/src/parser.rs` **(SPINE — reviewer ON)** | attribute-collection loop in `parse_class_body`, `parse_attribute` helper, the `@invariant` carve-out, `attr.dangling` diagnostic | parser |
| `phalcom-core/src/compiler/attributes.rs` **(new file)** | `AttributeExpander` trait, `AttributeRegistry`, `expand_class_attributes`, `Target` enum, legality-table check, the three built-in expanders (`requires`/`ensures`/`invariant`), span-hygiene (D3) plumbing | expander |
| `phalcom-core/src/compiler/lib.rs` **(SPINE — reviewer ON)** | one call to `expand_class_attributes` at the top of `Statement::Class` (L763); `CompileMode` threading into `Compiler`/`ExpandCtx` | compiler wiring |
| `phalcom-core/bin/phalcom/cli.rs` | `--release`/`--unchecked`/`--strip-contract-metadata` flags → `CompileMode` | CLI |
| `phalcom-core/src/method.rs` | `MethodObject::contracts: Option<Vec<(Symbol,Value)>>` field + accessor | reflection |
| `phalcom-core/src/heap.rs` | `FiberObject::checking: HashSet<ObjRef>` field | fiber state |
| `phalcom-core/src/vm.rs` | `VM::checking` mirror + fiber-switch swap (extend the existing `stack`/`frames`/`open_upvalues` swap site) + native `__invariantEnter`/`__invariantExit`/`__invariantChecking`-shaped primitives (exact surface, implementer's naming call) | runtime guard |
| `phalcom-core/core/core.ph` | `PreconditionError`/`PostconditionError`/`InvariantError` (`extends Error`, zero Rust) | error classes |
| `phalcom-core/tests/lang/annotations/` (new label) | AST snapshots (insta) + golden `.ph` corpus per §7 | goldens |
| `phalcom-core/tests/lang.rs` | wire new test fns | test harness |

**Deliberately NOT in scope:** `@get`/`@set`/`@construct`/`@data`/`@sealed`/`@variant` (all
U-ANNOT-LAYOUT — different registry rows in the **same** `attributes.rs` file, see §4.1);
`FieldDef`/field-declaration grammar (U-ANNOT-LAYOUT); the `Attribute`/`AttributeUsage` core
class, `Behavior.defineMethod`, any Install/Dispatch/Runtime-tier surface
(`attribute-classes.md`, gated — ADR-0054 §2(b), A-1–A-6 unresolved); `bytecode.rs`/`class.rs`
(no new `Value`/`Object` variant beyond the additive struct fields above).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`parser.rs`/`compiler/lib.rs` are spine files**, historically the busiest in the tree.
  Confirm no concurrent unit holds them before dispatch (standing hazard, per
  `phalcom-concurrent-session-hazards` memory).
- **`phalcom-core/src/compiler/attributes.rs` is shared with U-ANNOT-LAYOUT** — this unit creates
  the file and the registry/`Target`/`AttributeExpander` machinery; U-ANNOT-LAYOUT adds rows to
  the same registry. **Sequence, do not parallelize**: U-ANNOT-LAYOUT must start only after this
  unit's `attributes.rs` scaffolding lands (registry shape, `Target` enum, `ExpandCtx`,
  span-hygiene helpers) — U-ANNOT-LAYOUT is a strict dependency on this unit, not a parallel
  wave partner.
- **`core.ph`** — never two editors; confirm clean before dispatch (this unit's touch is three
  small `class … extends Error {}` blocks, low collision surface, but `core.ph` itself is a
  single-writer chokepoint per `docs/forge/STATE.md`).

## 5. Build order (small, independently-green diffs)
1. **AST + parser scaffolding, no expansion yet.** `Attribute` struct, the four `attributes`
   fields, the parser's attribute-collection loop + `@invariant` carve-out + `attr.dangling`.
   `expand_class_attributes` exists but is a no-op passthrough (returns `class` unchanged,
   attributes parsed-and-discarded). Green — proves the grammar/parse layer in isolation
   (AST-snapshot tests, no compiler-loop change yet).
2. **Registry + `Target` + legality check, still no real expanders.** `AttributeRegistry`,
   `Target` enum, the legal-target check emitting `attr.unknown`/`attr.illegal_target`. Wire the
   one call site in `compiler/lib.rs`. Green — unknown/misplaced-attribute goldens pass; known
   attributes still no-op.
3. **`@requires`/`@ensures`.** The three-step weave, `old()` hoist, purity floor,
   `PreconditionError`/`PostconditionError` in `core.ph`. Green — `contracts_precondition_pass`/
   `_fail`, `contracts_postcondition_old` goldens.
4. **`@invariant` + ADR-0052 guard.** `checking` state (`FiberObject`+`VM`+fiber-switch swap),
   the corrected ownership-tracking prologue/epilogue via `Block::ensure(_)`,
   `InvariantError`, `__check_invariant` synthesis. Green — the ADR-0052 regression goldens
   (§7) are the gating tests for this step.
5. **`CompileMode` + stripping + reflectable metadata.** CLI flags, the two independent axes,
   `MethodObject::contracts`. Green — stripping goldens.

Each step is a self-verifiable commit; never commit a non-compiling tree.

## 6. Mandatory rules
- **Docs:** `///` on every new type/fn/field (`Attribute`, `AttributeExpander`, `Target`,
  `CompileMode`, `MethodObject::contracts`, `FiberObject::checking`, the fiber-switch swap
  extension) citing the ADR/spec § it realizes. `cargo doc --workspace --no-deps` clean.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; no `unsafe`.
- **Reviewer ON** (spine files `parser.rs`, `compiler/lib.rs`) — `phalcom-reviewer` gates the
  diff; writer never self-approves. Every diagnostic in §7's catalog must recover (miette,
  multi-error, ADR-0016), never panic on malformed input.

## 7. Test strategy (extends annotations-test-strategy.md's existing catalog — do not invent a parallel plan)
**AST snapshots (insta, new `phalcom-core/tests/` or `phalcom-ast/tests/snapshots/` fixtures,
per the doc's own table):**
- `expand__requires_prologue`, `expand__ensures_old_hoist`, `expand__ensures_result_bind`,
  `expand__invariant_wrap` — exactly as named in annotations-test-strategy.md's table, plus a
  new `expand__invariant_ownership_tracking` snapshot asserting the corrected §3.3 prologue
  (captures `__invariant_owner`, not a re-check of `checking.contains(self)`) is what's emitted
  — this is new relative to the doc's table because it pins **this plan's correction**, not the
  ADR's literal (buggy) pseudocode.

**Golden `.ph` corpus (`phalcom-core/tests/lang/annotations/`, reusing the doc's exact case
names where listed):**
- `contracts_precondition_pass.ph` / `contracts_precondition_fail.ph`
- `contracts_postcondition_old.ph`
- `contracts_invariant_reentrancy.ph`
- `contracts_invariant_cross_receiver.ph` — **the ADR-0052 Bug-1 regression case**: object `A`'s
  public method calls object `B`'s public method; assert `B`'s own `@invariant` still fires.
- `contracts_invariant_survives_throw.ph` — **the ADR-0052 unwind-safety case**: a thrown error
  inside an `@invariant`-checked call; assert the guard is not permanently inflated (a
  subsequent call on the same or a different receiver still checks correctly).
- `contracts_release_stripped.ph` / `contracts_unchecked_metadata_stripped.ph`
- `annotation_unknown_error.ph`
- **New, not in the doc's table but required by this plan's own findings:**
  `contracts_invariant_fiber_yield.ph` — a fiber that `yield`s from inside an
  `@invariant`-guarded call; resumed later, completes its own exit check correctly and does not
  corrupt a second fiber's `checking` state (the §2/§3.3 fiber-switch finding's regression
  guard — **the single most load-bearing new test in this unit**, since it guards a hazard the
  ADR itself never named).
  `contracts_invariant_no_alloc.ph` / a disasm-shaped probe — asserts no `Set`/`Map` allocation
  per guarded call (the Rubric's allocation hazard).

**Diagnostics catalog (miette, exactly the doc's table plus `contract.impure_predicate`,
`contract.old_on_mutable`, `attr.unknown`, `attr.illegal_target`, `attr.dangling`,
`attr.accessor_collision`)** — every code recovers, never panics; multi-error batches (ADR-0016)
where the source has more than one malformed attribute in a single class body.

**`verify_invariants()` extension** (annotations-test-strategy.md's own item): after bootstrap,
assert woven `__check_invariant` methods exist exactly on classes declaring `@invariant` and
nowhere else.

**NEGATIVE:** malformed `@requires`/`@ensures`/`@invariant` args (non-boolean-shaped predicate —
not statically checkable, so this is a *runtime* dNU/type-error on the `ifFalse` send, not a
compile error; pin one golden showing it fails at the guard call site with an ordinary runtime
diagnostic, not a panic).

## 8. Decisions flagged
| ID | Decision | Resolution |
|---|---|---|
| **DEC-ANNOT-A** — resolved by grounding | `@get`/`@set` in this unit or U-ANNOT-LAYOUT? | **U-ANNOT-LAYOUT** — no legal target exists without `FieldDef` (§ scope-correction banner). |
| **DEC-ANNOT-B** — resolved by grounding | `@invariant`'s grammar (no following member, per its own worked example)? | **Parenthesized-call form**, `@invariant(<expr>)`, standalone class-body item, no following member required (the one carve-out in the newline-binding rule). Flag `annotations-contracts.md`'s `=>` example for doc-sync. |
| **DEC-ANNOT-C** — resolved by grounding | ADR-0052's epilogue ownership-tracking pseudocode double-checks `checking.contains(self)` instead of tracking per-call ownership — bug or intentional? | **Bug in the ADR's literal pseudocode** — implement the corrected `__invariant_owner`-local version (§3.3), not the literal snippet. Flag in the return contract; this is a small ADR erratum, not a new design axis, so not BLOCKED. |
| **DEC-ANNOT-D** — flagged, not resolved | Exact native selector names for the `checking`-set primitives (`__invariantEnter`/`__invariantExit`/`__invariantChecking` are placeholders). | Implementer's naming call within this unit; no design-axis ambiguity, just needs a decision recorded in the return contract. |

No item here is **BLOCKED-ON-DECISION** in the "needs the user" sense — ADR-0054/ADR-0052 already
resolved the design; DEC-ANNOT-A/B/C are architect-resolved-by-grounding (reviewer-visible),
DEC-ANNOT-D is an implementation-naming note.

## 9. Must-not-preclude check
- **`@get`/`@set`/`@construct`/`@data`/`@sealed`/`@variant` (U-ANNOT-LAYOUT):** *served, not
  precluded* — the registry/`Target`/`ExpandCtx`/span-hygiene machinery built here is exactly
  the seam U-ANNOT-LAYOUT adds rows to; nothing here special-cases "there are only three
  attributes."
- **Install/Dispatch/Runtime tiers (`attribute-classes.md`, gated, A-1–A-6 open):** *not
  touched, not complicated.* This unit's `AttributeExpander` is a Rust-side compiler trait, not
  a user-facing `Attribute` class — it shares nothing with the future `Attribute` root's
  `expand(_)`/`wrap(_)`/`aroundSend(_)` hook-selector model beyond the name "expander". No
  `Behavior.defineMethod`, no reflective retained-instance store, no `_attributes` slot on
  `Method`/`ClassObject` is introduced here — when Install/Dispatch/Runtime eventually ratifies,
  `attribute-classes.md`'s own bootstrap section can add that store independently; this unit's
  `MethodObject::contracts` field is a **different, narrower** thing (predicate metadata only,
  not the general retained-attribute-instance list) and should not be conflated with it later —
  flag this distinction explicitly so a future implementer doesn't try to unify them without
  re-checking the shapes match.
- **Contracts as gradual-typing substrate (D-contract-2, deferred not built):** not precluded —
  `MethodObject::contracts` retains reflectable predicate `Block`s exactly as D-contract-1
  requires; a future `amount @ Number` desugar to `@requires(amount.isA(Number))` slots into the
  same expander registry as a new row, no reshape needed.
- **`@sealed`/`@data` exhaustiveness (U-ANNOT-LAYOUT, downstream):** not precluded — this unit's
  `expand_class_attributes` signature (`ClassDef -> Result<ClassDef, CompilerError>`) is generic
  over which registry rows exist; U-ANNOT-LAYOUT's `generate`-phase expanders (member-adding, not
  just weaving) fit the same trait without a signature change.
- **Fiber-scheduling / U-FIBER-REFLECT work:** not precluded — `FiberObject::checking` is an
  additive field alongside the existing `stack`/`frames`/`open_upvalues`, following their exact
  swap discipline; no change to fiber creation/resume/park signatures.

## 10. Return contract (report to `phalcom-reviewer`)
The `Attribute` AST node + the four `attributes` fields + `ClassDef.invariants` · the parser's
attribute-collection loop and the `@invariant` carve-out (with a pointer to DEC-ANNOT-B) ·
`attributes.rs`'s registry/`Target`/`ExpandCtx` shape (the seam U-ANNOT-LAYOUT depends on —
confirm it doesn't need to change once that unit starts) · the three built-in expanders ·
the corrected (not ADR-0052-literal) invariant ownership-tracking mechanism, explicitly flagged
as a deviation-with-reason · `FiberObject::checking`/`VM::checking` + the fiber-switch swap site
· `MethodObject::contracts` + confirmation it is scoped narrower than any future
Install-tier `_attributes` retention store (do not let a later unit silently merge them without
re-verifying the shapes) · `CompileMode` + the two independent stripping axes · all AST-snapshot
+ golden-corpus + diagnostic results from §7, especially the two new fiber-yield/no-alloc
goldens · confirmation of **zero `Value`/opcode/`SignatureKind` change** · a flagged doc-sync
list for the reviewer/orchestrator: `annotations-core.md` (Lexer step already done; loop is 4
variants not 3), `annotations-construct.md` (Prerequisite 2 already landed via U7),
`annotations-contracts.md` (`@invariant`'s `=>` example → parenthesized form) —  this unit does
not edit those docs itself (out of write-set), only flags them · `verify.sh` + `cargo doc` tails.
