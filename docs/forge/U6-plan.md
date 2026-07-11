# U6 — Work order: absence → `Option` + `let`/`var` bindings (dispatch-ready)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Removes surface `nil`
from Phalcom and replaces it with the `Option` type (`Some`/`None`); introduces `let`/`var` binding
forms; wires `??`/`?.`; forbids Option truthiness. **Load-bearing unit → independent
`phalcom-reviewer` gate afterward** (it can corrupt the value model and leak the private VM sentinel
to user code). Grounded in **ADR-0007** (Option as abstract with Some/None), **ADR-0014** (let/var
bindings), and **ADR-0010** (tagged `Value`, private `Nil`); spec = `docs/spec/values-and-absence.md`.
STATE.md ADR mapping is authoritative._

---

## 0. Mission (one sentence)
Make **absence a first-class `Option`** — no surface `nil`, no truthiness — by adding `let`/`var`
bindings (`var x` uninitialized reads `None`), bootstrapping the `Option`/`Some`/`None` kernel classes
on top of U1's **private** `Value::Nil` sentinel (which must never leak to user code), and desugaring
`??`/`?.` to Option sends.

## 1. Hard guardrails (read before writing any code)
- **This is the surface-absence redesign, not a rewrite of the combinator library.** U6 owns the
  *substrate*: binding forms, the private-sentinel→`None` surfacing boundary, the `Option`/`Some`/`None`
  class *existence* + `None` singleton + `Some` construction primitive + the `match` eliminator, and
  the `??`/`?.` desugar. **The rich combinator bodies (`map`/`flatMap`/`filter`/`orElse`/`ifSome`/…)
  are U-STD's job in `core.ph`** (they are Phalcom-level, two method defs each per ADR-0007). Declare
  the class skeletons; do not hand-author the full combinator suite here.
- **The private `Value::Nil` sentinel (ADR-0010/U1) must NEVER reach user code (Invariant 4).** It has
  no surface syntax, no literal, and cannot be produced by user code. Every place today that pushes
  `Bytecode::Nil` from a *surface* path must be rerouted to produce `None` or be rejected — see §4.
  The sentinel may only ever *back* an uninitialized slot internally and is surfaced as `None`.
- **`None` is never a `Some`.** The surfacing helper converts sentinel→`None`; it must be impossible
  for `Value::Nil` to end up inside a `Some(_value)`.
- **Do NOT alter dispatch, the metaclass tower, or blocks** — U3/U2/U4 own those. Consume them.
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). **Do not self-approve** — a reviewer gates this unit.

## 2. Preconditions (verify first; do not assume)
- **U1 merged + green** (Heap + `Copy` handles + tagged `Value` with a **private** `Nil`; `PhRef`
  retired). U6 builds the *surface* `Option` on top of that private sentinel — confirm `Value::Nil`
  is already private and non-surfaceable before starting.
- **U4 (blocks/closures) merged + green.** `??`/`?.` desugar to sends that take **blocks**
  (`a.orElse { b }`, `opt.map { x => x.foo }`); the golden tests exercise block-taking combinators.
  If U4 is not landed, U6 cannot be verified green — **STOP**.
- **U5 (control-flow-as-message + inliner) merged + green** — it owns the branch-lowering / typed
  condition path that the "no truthiness" rule (§4, BD-U6-1) hooks into. Confirm where a condition is
  compiled to a branch before designing the `if(opt)` diagnostic.
- Confirm `./scripts/verify.sh` is green on the base before the first edit (baseline).
- Re-run `graphify affected "nil"`, `graphify affected "Bytecode::Nil"`, and
  `graphify explain "LetBinding"` on the actual HEAD to confirm nothing new sits outside §3.

## 3. Confirmed write-set (from `graphify affected` on binding/nil symbols + source read)
| File | Why it's in scope |
|---|---|
| `phalcom-ast/src/token.rs` | Add `Token::Var`; add `Token::CoalesceQuestion` (`??`) + `Token::QuestionDot` (`?.`); retire the surface `Token::Nil`. |
| `phalcom-ast/src/lexer.rs` | Lex `var`; lex multi-char `??` and `?.` (currently only single `?` → `Token::Question`, `lexer.rs:269`); drop the `"nil" => Token::Nil` keyword mapping (`lexer.rs:177`). |
| `phalcom-ast/src/ast.rs` | Add mutability to `LetBinding` (a `mutable: bool` or a `BindingKind::{Let,Var}`); remove `Expr::Nil` (`ast.rs:87`); add `??`/`?.` expression nodes (or desugar in the parser). |
| `phalcom-ast/src/parser.rs` | Parse `var`; parse `??`/`?.` at the precedence in [lexical-structure §9](../spec/lexical-structure.md); remove the `nil` literal parse (`parser.rs:887`). |
| `phalcom-core/src/compiler/lib.rs` | `let`/`var` mutability tracking (`Local` gains mutability; compile-time immutable-global set); `var x` no-init → `None`; **reject `let x` no-init**; **reject assignment to a `let`**; remove `Expr::Nil` lowering (`lib.rs:400`); reroute the `Bytecode::Nil` surface emits (`lib.rs:237,257,473`); desugar `??`/`?.` to Option sends; emit the `if(opt)` compile-error diagnostic (BD-U6-1). |
| `phalcom-core/src/value.rs` | Surface guards: no public `Nil` constructor; a `sentinel_to_option`/surfacing helper (`Value::Nil` → `None` at read boundaries). |
| `phalcom-core/src/universe.rs` | Bootstrap the `Option`/`Some`/`None` kernel classes (mirror `Bool`/`True`/`False`); bind the shared `None` singleton global; register `Some` construction + `match` eliminator primitives. |
| `phalcom-core/src/nil.rs` + `phalcom-core/src/primitive/nil.rs` | Strip any *surface* `nil` class/primitive exposure; keep only the private-sentinel support U1 defined. |
| `phalcom-core/src/vm.rs` | Reroute the `Return`-default sentinel push (`vm.rs` bare-return path); surface uninitialized reads as `None`; wire `Some`/`match` primitives. |
| `phalcom-core/src/bytecode.rs` | If any surface use of `Bytecode::Nil` is removed, keep the opcode only for internal sentinel use (or retire it) — document which. |
| `phalcom-core/src/diagnostics.rs` **or** the `CompilerError` enum in `compiler/lib.rs` | New diagnostics: `if(opt)` truthiness error, `let x` no-init, assign-to-`let`, read-of-surface-`nil`. |
| `core/core.ph` | Declare the `Option`/`Some`/`None` class **skeletons** + the `None` global (combinator bodies → U-STD). Shared file → **sequence U6 before U-STD**, never parallel. |

## 4. Design decisions (ADR-0007 / ADR-0014 / ADR-0010 — realize, don't re-litigate)
- **Binding forms (ADR-0014).** `let` = immutable (reassignment is a **compile error**); `var` = mutable;
  `var x` with no initializer **reads `None`** (private `Value::Nil` backs the slot, surfaced `None`);
  **`let x` with no initializer is rejected** at compile time. `Local` gains a mutability flag; the
  assignment path checks it. Module-level (global) immutability needs a compile-time set of immutable
  global names in the `Compiler` (globals span statements — `resolve_local` won't see them).
- **Absence = Option (ADR-0007).** `Option` is abstract; `Some` (one field `_value`) and `None`
  (a single shared singleton, identity-comparable, zero-allocation) are its concrete subclasses —
  **exactly mirroring `Bool`/`True`/`False`** (ADR-0004). Dispatch replaces branching: there is no
  variant tag. `None` is a global bound to the singleton; `Some(v)` is an ordinary construction send.
  **Bootstrap `Some`/`None` via Rust primitives here** (do *not* depend on U7's user-facing
  `construct`) so U6 is independent of U7.
- **The eliminator.** `match(some:, none:)` is the one primitive that leaves Option-world with a value
  ([values-and-absence §3.2](../spec/values-and-absence.md)); every other extractor (U-STD) is defined
  over it. Provide `match` as substrate.
- **Surfacing boundary.** One helper (`value.rs`) converts the private `Value::Nil` sentinel → `None`
  at exactly these read boundaries: uninitialized `var` read, unassigned field read (shared with U7),
  bare-`return` default, and any method that falls off its end. `Some(_value)` construction asserts its
  argument is never the sentinel.
- **`??`/`?.` desugar (values-and-absence §3.4).** `a ?? b ≡ a.orElse { b }`; `opt?.foo ≡
  opt.map { x => x.foo }`; `opt?.bar(baz) ≡ opt.map { x => x.bar(baz) }`. Both short-circuit; chained
  `?.` stays inside `Option` and the first `None` short-circuits the rest. Lower to message sends over
  blocks (needs U4). Precedence per [lexical-structure §9](../spec/lexical-structure.md).
- **No truthiness (values-and-absence §3.5).** `Option` is not `Bool`. Reach through `.isSome`/`.isNone`
  or use `ifSome`/`ifNone`. See **BD-U6-1** for the enforcement mechanism.

### BLOCKED-ON-DECISION — BD-U6-1: how is `if (opt)` a *compile* error?
Spec §3.5 / ADR-0007 state `if (opt)` is a **compile error**. Phalcom is **dynamically typed** with
**no static type/flow analysis**, and U5 has already lowered control flow to message sends / an inlined
branch opcode — so the compiler generally cannot know a condition's runtime class. The literal spec
word "compile error" is not fully realizable for the general case. Options:
- **(A) Runtime no-coercion floor + literal-only compile check (RECOMMENDED).** `Option`/`Some`/`None`
  simply never implement the boolean-branch protocol, and the branch opcode requires a `Bool` — any
  non-`Bool` condition is a hard **runtime** type error (no silent coercion, ever). Additionally, the
  compiler rejects the *syntactically detectable* cases at compile time (`if (None)`, `if (Some(...))`,
  a condition that is literally an Option construction). This guarantees "no truthiness" everywhere and
  honors "compile error" for the detectable class.
- **(B) Full static compile error.** Requires type/flow inference Phalcom does not have. Out of scope.

**Recommendation: (A).** It narrows the spec's "compile error" to "compile error where statically
detectable + hard runtime type error otherwise." This needs ratification because it refines §3.5, and
likely a **short ADR** ("no-truthiness enforcement = typed branch + literal-only compile check") or an
amendment to ADR-0007, plus coordination with U5's branch-opcode typing. **Do not pick unilaterally.**
The rest of U6 proceeds regardless of BD-U6-1's resolution; only the `if(opt)` diagnostic waits.

### Minor decision (recommend, don't block): bare `return` default
`return;` with no operand currently pushes the sentinel. Spec pins implicit return = last expression
but is silent on a bare `return`. **Recommend: bare `return` yields `None`** (consistent with the
absence model). Fold into the surfacing boundary; note in the return contract.

## 5. Build order (keeps the change reviewable; land as one coherent diff)
1. **`phalcom-ast`** — `token.rs` (`Var`, `??`, `?.`; retire `Nil`), `lexer.rs` (keywords + multi-char
   ops), `ast.rs` (`LetBinding` mutability; drop `Expr::Nil`; `??`/`?.` nodes), `parser.rs` (parse
   `var`, `??`/`?.` precedence, drop `nil`). Full rustdoc on new tokens/nodes.
2. **Kernel bootstrap** — `universe.rs` + `core.ph` skeletons: `Option`/`Some`/`None` classes, `None`
   singleton global, `Some` construction + `match` primitives (`primitive/*`). Cite ADR-0007/0004.
3. **Surfacing boundary** — `value.rs` helper + `nil.rs`/`primitive/nil.rs` de-surfacing; `vm.rs`
   reroute of the `Return`-default and uninitialized reads to `None`.
4. **Compiler bindings** — `compiler/lib.rs`: `let`/`var` mutability (locals + immutable-global set),
   `var x`→`None`, reject `let x` no-init, reject assign-to-`let`, remove `Expr::Nil` lowering, reroute
   surface `Bytecode::Nil` emits.
5. **Desugar** — `??`/`?.` → Option sends over blocks.
6. **Truthiness diagnostic** — `if(opt)` per BD-U6-1 (gated on that decision; implement the (A) floor
   unconditionally, add the literal compile check once ratified).
7. **Diagnostics + tests** — wire the new `CompilerError` variants; add goldens (§7-tests below).

## 6. Fold-in cleanup (only if fully inside this write-set)
U6 owns `phalcom-ast/src/parser.rs`, so it may fold **DEFERRED #2** (carry the real span through
`LexicalError` for `InvalidInteger`/`InvalidFloat`) and **DEFERRED #3** (reject malformed assignment
targets earlier — directly relevant, since U6 reworks the `let`/`var` assignment path). Both are
`parser.rs`-local and low-rank; fold only if they don't expand the diff materially. `graphify affected`
first; otherwise leave them in `DEFERRED.md`. Do **not** touch DEFERRED #1 (U1 owns it).

## 7. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `//!` on
  every touched module; `///` on every new public item (tokens, AST nodes, `Option`/`Some`/`None`
  bootstrap, the surfacing helper, every new `CompilerError` variant) with ADR-0007/0014/0010 citations
  and intra-doc links. `cargo doc --workspace --no-deps` adds **no new warnings**.
- **Green gate:** `./scripts/verify.sh` exits 0 (build + test + clippy + golden + invariants). Golden
  output byte-identical where unchanged. Don't add clippy warnings; fix pre-existing ones in files you rewrite.
- **Tests the harness must assert:**
  - Positive goldens: `var x` (no init) prints/behaves as `None`; `Some(42).map { … }` and
    `None.unwrapOr(0)` (thin skeleton combinators or the `match` eliminator); `a ?? b`; an `opt?.foo`
    chain that short-circuits on the first `None`.
  - Negative goldens (must fail to **compile**): a `.ph` using surface `nil`; `let x` with no
    initializer; reassignment of a `let`; and — once BD-U6-1 is ratified — `if (None) { … }`.
  - Invariant: the private sentinel is unreachable from user code (no program can print or compare it);
    `Value::Nil` never appears inside a `Some`.

## 8. Return contract (to the reviewer, not self-approval)
Report: binding-form semantics (let/var mutability enforcement, `var x`→`None`) · the surfacing-boundary
helper + every reroute of a surface `Bytecode::Nil` emit · Option/Some/None bootstrap + `None` singleton
+ `Some`/`match` primitives · the `core.ph` skeleton vs U-STD combinator boundary · `??`/`?.` desugar ·
**BD-U6-1 status** (did the user ratify the `if(opt)` mechanism? what shipped?) · bare-`return`
decision · goldens/negatives added with `verify.sh` tail · `cargo doc` tail · any new `DEFERRED.md`
entries. A `phalcom-reviewer` independently verifies the sentinel never leaks (Invariant 4), `None` is
never a `Some`, and the green gate.
