# Handoff — U5: control-flow-as-message + inliner, then U6: absence → `Option` + `let`/`var`

## Context

`main @ 286e107` has U-FE/U0/U3/U1/U2/**U4** landed. U4 (blocks/closures, Lua-style
open/closed upvalues, frame-token infrastructure — ADR-0013/0006) landed in two
passes: a first cut had the front end right but the runtime stubbed out (caught by
an independent `phalcom-reviewer`), closed in the same session (see `STATE.md`'s
U4 section for the full story). Per the serial spine
(`U1 → U2 → U4 → U5 → U6 → U7`, `PHASE2-INDEX.md` §2), **U5 is next, then U6**.

Both units have complete, pre-written work orders:
- [`U5-plan.md`](U5-plan.md) — control-flow-as-message + sacred-selector inliner
  (control-flow.md §1–3, ADR-0012, new ADR-0017). **Reviewer OFF** (STATE.md policy
  — U5 is not in the load-bearing set U1/U2/U4/U6).
- [`U6-plan.md`](U6-plan.md) — absence → `Option`, `let`/`var` bindings
  (ADR-0007/0014/0010, values-and-absence.md). **Reviewer ON** — load-bearing,
  can corrupt the value model and leak the private `Value::Nil` sentinel.

This handoff re-verifies each plan's "Preconditions" section against the actual
post-U4 `main` HEAD and flags where the plan's assumptions have drifted, the way
the U4-handoff did for U4. **Read both plans in full before starting** — this
handoff does not repeat their design decisions, write-sets, or build orders
verbatim, only what changed or needs resolving.

---

## Part 1 — U5: control-flow-as-message + sacred-selector inliner

### Precondition reconciliation (§2 of U5-plan.md, re-verified on `286e107`)

- **U4 is landed and green** — confirmed. U5's hard dependency is satisfied.
- **Block-literal AST node**: `Expr::Block(Box<BlockExpr>)` in
  `phalcom-ast/src/ast.rs:98` — confirmed, matches the plan's expectation.
- **Closure/block primitive module**: `phalcom-core/src/primitive/block.rs` —
  confirmed, matches the plan's guess.
- **⚠️ Block-invocation selector — the plan is WRONG on this, correct it before
  writing any inliner code.** U5-plan.md §2 says "the block-invocation selectors
  (`value`, `value(_:)`)" — that was the pre-U4 brief's Smalltalk-flavored guess.
  **U4 actually shipped `call`/`call(_:)`/`call(_:_:)`… (registered per-arity up to
  4) and `callWith(_:)`**, per functions.md §1–2 (ADR-0006). Every place U5-plan.md
  says "the deopt fallback is a real block send" or references block invocation,
  read `call`/`call(_:…)`, not `value`/`value:`. `whileTrue(_:)`/`repeat(_:)`
  (§3 write-set row) are new selectors *you* register on `Block` in this unit —
  they don't already exist, so this doesn't block you, it's just a naming
  correction so you don't cargo-cult the stale name into the fallback-send emitter.
- **Operator opcodes still hardwired** — confirmed unchanged:
  `Bytecode::{Add,Subtract,Multiply,Divide,Modulo,Equal,NotEqual,Less,LessEqual,
  Greater,GreaterEqual,And,Or,Negate,Not}` all still exist in `bytecode.rs`, with
  live `vm.rs` arms (`binary_op!` macro, `handle_primitive_op`, lines ~403–450 and
  ~745–798 as of this HEAD). U5 Layer 0 removes all of this — write-set §3 stands
  as written.
- **No `If`/`While`/`For` AST nodes** — confirmed. `Token::If`/`Token::While`/
  `Token::For` are lexed (`phalcom-ast/src/token.rs`) but the parser does not
  consume them and `Expr` has no corresponding variant. **BD-U5-1 (who owns
  `if`/`while` surface parsing) is still live and unresolved** — see below.
- `./scripts/verify.sh` equivalent (build+test+doc+clippy) is green on this HEAD
  (verified when U4 landed; re-confirm as your literal first step per the plan).

### Open decisions needing your input before/during U5

The plan flags two `BLOCKED-ON-DECISION` items (U5-plan.md §"BLOCKED-ON-DECISION",
end of file). Per this repo's working model (self-verify, no architect
reconciliation pass for this stretch — same precedent as U2/U4), the recommended
defaults below are **pre-authorized to proceed on** unless you tell the
implementer otherwise:

- **BD-U5-1 — who parses `if`/`while`/`for`?** Plan recommends **Option A: U5 owns
  it** (tightly-scoped parse-time desugar to `MethodCall` sends over U4 block
  literals — `ifTrue:`/`ifTrue:ifFalse:`/`whileTrue:`). This adds
  `phalcom-ast/{parser,ast}.rs` to U5's write-set (a sequencing concern with
  later U-LEX, not a conflict — U5 runs alone on the spine). **Recommend
  confirming Option A explicitly before the implementer starts**, since it
  changes U5's write-set boundary.
- **BD-U5-2 — `repeat(_:)` scope + unary selector names.** Plan recommends
  implementing the unambiguous sacred selectors first (`ifTrue:`, `ifFalse:`,
  `ifTrue:ifFalse:`, `and:`, `or:`, `whileTrue:`) and treating `repeat(_:)` as
  inlinable only once its receiver/semantics are pinned — defer it to
  `DEFERRED.md` otherwise. Not a hard blocker; flagged so `repeat` semantics
  aren't invented silently.

### Working model (same precedent as U2/U4 unless told otherwise)
1. Work **directly on `main`** — no worktree/branch, matching U2/U4 (the plan's
   header mentions "its own worktree off main" as the generic template; the
   repo's actual working precedent for this stretch has been in-tree).
2. **No `phalcom-architect` pass** — this handoff is the reconciliation.
3. **Reviewer OFF per STATE.md policy** — self-verify on the green gate + the
   §7 soundness tests (semantic equivalence, override-honored, non-local-return
   transparency, zero-alloc hot path, guard-on-wrong-type). This is explicit in
   the plan (§9, §10) — don't skip the soundness tests just because there's no
   external gate.
4. **Land Layer 0 fully green before starting Layer 1** (plan §5) — this is a
   hard sequencing rule, not a suggestion: if Layer 1 can't be verified green on
   its own diff, split the sacred selectors into batches (§1 guardrail).
5. Draft **ADR-0017** (sacred-selector inliner + override-epoch deopt guard) —
   required to land with the unit (plan §4.3); use the `documentation-and-adrs`
   skill.
6. Commits: conventional format, e.g. `feat(u5): control-flow-as-message +
   sacred-selector inliner`, ending `Co-Authored-By: Claude Sonnet 5
   <noreply@anthropic.com>`.
7. On landing: update `docs/forge/STATE.md` (U4 ✅ → U5 ✅, note reviewer OFF
   per policy) and `docs/forge/PHASE2-INDEX.md`'s U5 roster row, same pattern as
   U2/U4.
8. Do **not** push to `origin`.
9. graphify-first: `graphify affected "Bytecode"`, `graphify affected
   "binary_op"`, `graphify explain "Compiler"` before editing; `graphify update .
   --no-cluster` after.

### Return contract
Per U5-plan.md §10: sacred-selector set actually inlined vs. left as plain sends
+ why · final opcode set added/removed · exact deopt-guard mechanism + pristine-
flag hook location · non-local-return-through-inlined-block proof (test name +
output equivalence) · inliner bytecode snapshot (zero closure alloc) · operator
opcodes fully removed, no dead arms · BD-U5-1/BD-U5-2 resolutions · `verify.sh`
tail at both Layer-0 and Layer-1 gates · `cargo doc` tail · ADR-0017 landed · any
`DEFERRED.md` entries (`for`/collections, per-call-site IC).

---

## Part 2 — U6: absence → `Option` + `let`/`var` bindings

**Do not start U6 until U5 is landed and green** — U6-plan.md §2 makes this a
hard precondition (the `if(opt)` no-truthiness diagnostic hooks into U5's typed
branch-lowering path).

### Precondition reconciliation (§2 of U6-plan.md, re-verified on `286e107`)

- **U1 merged + green, `Value::Nil` already private** — confirmed.
  `phalcom-core/src/value.rs:7` and `phalcom-core/src/nil.rs` both document it
  explicitly as "a private uninitialized-slot sentinel with no surface class...
  can never be produced or observed by user code (Invariant 4)". U6 builds the
  surface `Option` on top of this unchanged.
- **U4 merged + green** — confirmed (see Part 1 context).
- **U5 merged + green** — will be true once Part 1 lands; re-confirm at U6's
  actual start, don't assume time has not passed.
- **No `Token::Var`** — confirmed. `phalcom-ast/src/token.rs` has no `Var`
  variant; the lexer (`lexer.rs:171`) maps `"let"` but not `"var"` as a keyword
  (currently lexes as a plain identifier). U6 must add both.
- **No `??`/`?.` tokens** — confirmed. Only single-char `Token::Question` exists
  (`lexer.rs:269`, `b'?' => (1, Token::Question)`), no multi-char lookahead for
  `??`/`?.`. U6 adds both per lexical-structure.md §9 precedence.
- **`Token::Nil` / surface `nil` keyword still live** — confirmed.
  `phalcom-ast/src/token.rs` has `Nil`; `lexer.rs:177` maps `"nil" =>
  Token::Nil`. U6 retires both (§3 write-set).
- **`LetBinding` has no mutability field** — confirmed.
  `phalcom-ast/src/ast.rs:69-73`: `pub struct LetBinding { name, value:
  Option<Expr>, range }` — no `mutable`/`BindingKind` distinction yet, and the
  optional initializer is *currently* accepted with no init (no compile-time
  rejection). U6 adds the mutability flag and the reject-no-init-for-`let` rule.
- No dedicated `Option`/`Some`/`None` node found in the current graphify graph
  (spot-checked via `graphify query`) — confirms the kernel classes don't exist
  yet; U6 bootstraps them from scratch per §4.

No drift found beyond the above — U6-plan.md's write-set (§3) and design
decisions (§4) can be followed as written.

### Open decision needing your input before/during U6

- **BD-U6-1 — how is `if(opt)` a *compile* error** given Phalcom has no static
  type/flow analysis? Plan recommends **(A): runtime no-coercion floor +
  literal-only compile check** — `Option`/`Some`/`None` never implement the
  boolean-branch protocol (so any non-`Bool` condition is a hard runtime type
  error, no silent coercion), plus a compile-time rejection of the syntactically
  *detectable* cases (`if (None)`, `if (Some(...))`). This needs ratification
  (it refines values-and-absence.md §3.5) and coordinates with U5's branch-opcode
  typing — **do not let the implementer pick this unilaterally**; confirm Option
  (A) explicitly, or override, before U6's step 6 (truthiness diagnostic).
- **Minor, non-blocking:** bare `return;` with no operand — plan recommends it
  yields `None` (consistent with the absence model). Proceed on this unless told
  otherwise; it's noted in the return contract either way.

### Working model
1. Work **directly on `main`** — no worktree, same precedent as U2/U4/U5.
2. **No `phalcom-architect` pass** — this handoff covers reconciliation.
3. **Reviewer ON — this is a real gate, not self-verify.** U6-plan.md's own
   framing is explicit: "Load-bearing unit → independent `phalcom-reviewer` gate
   afterward (it can corrupt the value model and leak the private VM sentinel to
   user code)." Do not skip this the way U2/U4's first pass tried to skip
   verification — U4's near-miss (runtime stubbed, caught only because a
   reviewer ran) is exactly the failure mode this gate exists to prevent, and
   U6 touches something *more* dangerous (a private sentinel leaking to surface
   code, Invariant 4). Spawn a `phalcom-reviewer` pass before considering U6 done.
4. **U6 owns `phalcom-ast/src/parser.rs` for this stretch** — it may fold in
   DEFERRED #2 (real span through `LexicalError` for `InvalidInteger`/
   `InvalidFloat`) and DEFERRED #3 (reject malformed assignment targets earlier)
   if they don't materially expand the diff (plan §6). Check `DEFERRED.md`
   first for current entries before assuming these are still open.
5. **Sequence before U-STD** — `core.ph` gets Option/Some/None *skeletons* only
   in U6; the combinator bodies (`map`/`flatMap`/`filter`/`orElse`/…) are
   U-STD's job. Never co-schedule a `core.ph` editor with U6.
6. Commits: conventional format, e.g. `feat(u6): absence → Option + let/var
   bindings`, ending `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
7. On landing (post-reviewer-approval): update `docs/forge/STATE.md` (U5 ✅ →
   U6 ✅, reviewer gate ON and passed) and `docs/forge/PHASE2-INDEX.md`'s U6
   roster row.
8. Do **not** push to `origin`.
9. graphify-first: `graphify affected "nil"`, `graphify affected
   "Bytecode::Nil"`, `graphify explain "LetBinding"` before editing; `graphify
   update . --no-cluster` after.

### Return contract
Per U6-plan.md §8, **to the reviewer, not self-approval**: binding-form
semantics (`let`/`var` mutability enforcement, `var x`→`None`) · the
surfacing-boundary helper + every reroute of a surface `Bytecode::Nil` emit ·
Option/Some/None bootstrap + `None` singleton + `Some`/`match` primitives ·
the `core.ph` skeleton vs. U-STD combinator boundary · `??`/`?.` desugar ·
**BD-U6-1 status** (was it ratified? what shipped?) · bare-`return` decision ·
goldens/negatives added with `verify.sh` tail · `cargo doc` tail · any
`DEFERRED.md` entries. The reviewer independently verifies the sentinel never
leaks (Invariant 4), `None` is never a `Some`, and the green gate.

---

## Summary of what needs your explicit sign-off before implementation starts

| Decision | Unit | Recommendation | Blocks implementation? |
|---|---|---|---|
| BD-U5-1: who parses `if`/`while`/`for` | U5 | Option A — U5 owns it | No (pre-authorized default), but confirm to lock the write-set |
| BD-U5-2: `repeat(_:)` scope | U5 | Defer until semantics pinned | No |
| BD-U6-1: `if(opt)` compile-error mechanism | U6 | Option A — runtime floor + literal-only compile check | No (pre-authorized default), but this refines a spec doc — confirm before U6 step 6 |
| Bare `return;` default | U6 | Yields `None` | No |

If you don't respond, both implementers proceed on the recommended defaults and
report the decision explicitly in their return contract, same as the pattern
already used for U2/U4.
