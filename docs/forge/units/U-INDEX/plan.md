# U-INDEX — postfix `[]` index read/write, dispatching to a dedicated `[](...)`/`[](...,put:)` operator selector

Status: **PLANNED**. Not a performance tier — a syntax unit. Independent of
[U-IC](../U-IC/plan.md) (Tier 3, dispatch cost); riding on it, not blocked by
it (see Perf below). Single-writer on `phalcom-ast/src/parser.rs` +
`phalcom-ast/src/ast.rs` + `phalcom-core/src/compiler/lib.rs` → worktree-isolate
if run alongside another parser/compiler unit
([[phalcom-concurrent-session-hazards]]).

## Role

Closes a real, already-documented gap: `benchmarks/wren-suite/README.md`'s
porting notes record `list[i]` → `.at(i)` and `map[k] = v` → `.at(k, put: v)`
as manual workarounds for every ported Wren benchmark, and
`benchmarks/math/vectors.ph`/`stats.ph` fail today (`Error at 39:19` /
`Error at 61:14`, "Expected one of ';', newline") because their own header
comments assumed a `[]` operator that was never built.

**Revision (this session, user direction, supersedes the original
"pure sugar, zero new selectors" draft):** `[]` is no longer a silent
parser-level rewrite into `at(_)`/`at(_,put:)`. It compiles to its own
selector, spelled `[](...)`/`[](...,put:)` — a literal bracket-shaped
operator name, defined the same way `==(other)` is already a definable
operator-method name on HEAD (`class Op { ==(other) { return true } }`
dispatches correctly today). This still adds **zero** new primitives and
**zero** new `Value` representation — floor stays `+0`
([ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)
unaffected) — but it **does** add one new selector name (`"[]"`,
`parse_method_name`-recognized) and, as a direct consequence, `List`/`Map`/
`Tuple` need an explicit `[](...)` method defined in `core.ph` (thin
delegation to the existing `at(_)`/`at(_,put:)` they already implement).
That is the real cost of this direction versus the original pure-sugar
design: indexability is no longer "free" for any future `at`-implementer —
a collection author must opt in to `[]` explicitly, same as they'd opt in
to `==`. See Design → "Method-definition selector" below for why this
buys more than it costs.

## Spec anchor

- [collection-protocol.md](../../../spec/v0.2/core/collection-protocol.md) §2–3 —
  `at(_)` is a **total** operation (`Option`-shaped: raw value or `None`,
  never a raise) per law 1; `at(_,put:)` is `List`'s ordered/mutable
  refinement. `[](...)`/`[](...,put:)` are new `core.ph` methods that
  delegate to these — the underlying contract (total read, raising OOB
  write) is unchanged and still governs.
- [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) —
  comma-canonical selector spelling; a selector's identity is name + arity +
  labels. `"[]"` is just another name under this same rule — `[](_)`,
  `[](_,_)`, `[](_,put:)` are distinct selectors exactly like `at(_)`,
  `at(_,_)`, `at(_,put:)` are today. No change to ADR-0012 itself, just
  another name populating it.
- **Confirmed on HEAD, grounding the selector-naming decision:**
  `parse_method_name` (`phalcom-ast/src/parser.rs:1015`) already recognizes
  a fixed set of operator tokens (`+`, `-`, `==`, `<`, `and`, `is`, …) as
  literal, user-definable method names — `class Op { ==(other) {...} }`
  dispatches correctly today. Params are parsed *separately*, in the
  ordinary `(params)` slot every method uses (`parse_class_member`,
  `parser.rs:965`) — `==(_)`'s params are not part of the `==` name token.
  Adding `[`/`]` to this same recognized-token set, spelled as
  `[](params)` (name token `[`+`]`, then ordinary parens), is the
  **same-shape** extension — one new arm in `parse_method_name`, no new
  grammar fork. (A literal `[_, _, param]`-with-params-inside-the-brackets
  spelling was considered and rejected — see Design → "Method-definition
  selector" for why.)
- **Produces a new ADR** (next free slot after 0051 — re-check for a
  concurrent claim, same caveat 0051 itself carries) documenting both: (a)
  postfix `[...]` call-site sugar as `Expr::Index`/`Expr::SetIndex`
  desugaring to a `[](...)`/`[](...,put:)` send, and (b) `[]` joining the
  operator-selector name set alongside `==`/`+`/etc. This is new
  user-facing grammar on both the call side and the definition side, not an
  internal refactor.

## Preconditions (verify on HEAD)

Confirmed empirically this session (`./target/release/phalcom`, ad-hoc
scripts — not yet in the corpus):
- `xs.at(1)` → raw value (`Number`, not `Some(Number)` — today's `List#at`
  returns the element directly or the bare `None` singleton on
  out-of-range, not an explicit `Some`-wrapped Option; matches law 1's
  "total, never a raise" without literal ADT wrapping).
- `xs.at(1, put: 99)` and `m.at("k", put: 5)` both work today — `at(_,put:)`
  is live on `List` and `Map`.
- `xs.at(99, put: 1)` raises `Expected an in-range index, got an
  out-of-range Number` (a catchable `RuntimeError`, not a panic) — matches
  the existing `runtime_list_at_put_out_of_range` NEGATIVE fixture. **Read
  is total via `None`; write is not (raises OOB).** This asymmetry is
  pre-existing and inherited automatically once `[]=` desugars to
  `at(_,put:)` — not something this unit decides or should paper over.
- Postfix chaining (`.`, `(`, `::`, `{`) already **stops at a newline** —
  confirmed: `System.print(xs\n  .size)` is a hard parse error ("Expected
  ')'"), not a silent cross-line chain. `parse_call`'s postfix `while
  matches!(self.peek(), Token::Dot | ...)` loop (`parser.rs:1465`) sees a
  `Newline` token first when a continuation starts a fresh source line, so
  the loop exits — the classic JS-style ASI hazard ("statement starts with
  `[` on the next line, silently becomes an index into the previous line's
  expression") **does not apply**: adding `Token::LBracket` to that same
  `matches!` inherits the identical newline-termination protection the
  other four postfix forms already have. Verify this holds for `[`
  specifically (not just re-assert it by analogy) before relying on it —
  one golden fixture with a list literal immediately following an
  expression-statement on the next line, asserting it still parses as two
  statements.
- `parse_primary()` (`parser.rs:1854`) already fully consumes a *leading*
  `[...]` as a list literal (`parse_list_literal`, `parser.rs:2028`) before
  `parse_call`'s postfix loop ever runs — so adding `LBracket` to the
  postfix `matches!` only ever fires on `[` **following a completed
  primary/postfix chain** (i.e. `expr[idx]`), never reinterprets a
  standalone `[1,2,3]` literal. No grammar ambiguity between the two uses.
- **Known pre-existing gap, out of scope:** the compound-assignment path in
  `parse_assignment` (`parser.rs:1200-1214`) wraps *any* `left` — including
  `Expr::GetProperty` — as a plain `Expr::Assignment`, unconditionally,
  never as `SetProperty`. Whether `obj.prop += 1` already works today is
  unverified; if it doesn't, `xs[i] += 1` inherits the identical gap once
  `Expr::Index` exists, and fixing it is **not** this unit's job (fix once,
  for both `.prop` and `[]` targets, as its own follow-on if the gap is
  confirmed). Verify and record which case (property vs index) it is before
  implementation, so the implementer doesn't accidentally special-case only
  one.

## Design

### Method-definition selector — `[]` joins the operator-name set (user direction, supersedes reuse-`at` draft)

**Decision:** the *definition-side* selector for user-overridable subscript
behavior is a literal `[]` operator name — `[](_, param) { ... }` for read,
`[](_, put: param) { ... }` for write — not a reuse of `at(_)`/`at(_,put:)`
under the hood. This directly revisits and **supersedes** the original
DEC-INDEX-A recommendation ("reuse `at`, no dedicated selector").

**Two candidate spellings were considered:**

1. **`[]` as a name token, params in the ordinary `(...)` slot** —
   `[](_, _, param) { ... }`. Mirrors exactly how `==(other)` already works:
   `parse_method_name` recognizes a fixed operator-token set and yields a
   name string; `parse_class_member` then separately parses `(params)` the
   same way for every method, operator or not. Adding `Token::LBracket` to
   `parse_method_name`'s match arms (consume `[`, expect `]`, name = `"[]"`)
   is the only new parser code — zero grammar fork, reuses the existing
   name-then-parens shape every selector in the language already has.
2. **Literal `[_, _, param]`, params living inside the brackets themselves**
   — no separate `(...)` at all. This does not fit `parse_method_name` +
   `parse_class_member`'s shared shape; it would need a second,
   bracket-specific params-list branch forked into `parse_class_member`
   just for this one selector, breaking the "every method decl is `name`
   then `(params)`" invariant every other construct (unary, keyword,
   operator) currently shares.

**Adopted: option 1 — `[](_, _, param)`.** Reuses the existing
name-then-parens invariant instead of forking it; the bracket *shape* the
user wants is still visible (the name itself is literally `[]`), it just
doesn't also swallow the parameter list. Read form: `[](_)` / `[](_,_)` /
etc. (arity matches however many index args the call site passes). Write
form: `[](_, put: value)` — mirrors `at(_,put:)`'s existing
positional-index/`put:`-value shape exactly, appended not reordered.

**Consequence — `core.ph` now needs explicit `[]` methods.** Because `[]`
is its own selector rather than a silent rewrite to `at`, `List`, `Map`,
and `Tuple` must each define `[](...)` in `core.ph`, delegating to the
`at(_)`/`at(_,put:)` they already implement:
```
[](i) { return self.at(i) }              // List, Map, Tuple (read)
[](i, put: v) { return self.at(i, put: v) }  // List, Map (write only)
```
This is the direct cost of choosing a dedicated selector over pure sugar —
indexability is no longer automatic for every `at`-implementer, a
collection author opts in the same way they'd opt in to `==`. Traded for:
consistency with Phalcom's own precedent (operators are ordinary
dispatchable methods, not metamethods — see Design-space reconciliation
below) and headroom for a collection to make `[]` diverge from `at` later
(e.g. auto-vivifying `Map#[]=` while `at(_,put:)` stays strict) without
needing a new selector or an ADR to introduce one — `[]` is already its
own thing.

### AST — two new nodes, parallel to `GetProperty`/`SetProperty`, arg-list-shaped like a call

**Decision (supersedes the original single-expression draft, user direction
this session): `[...]` takes a full argument list — positional + labeled,
comma-separated — identical grammar to call args `(...)`, not a single
expression.** `xs[a, b, named: c]` parses exactly like `xs.foo(a, b, named:
c)` would. Concretely, reuse `parse_arg_list()` (`parser.rs:2249`) verbatim
— it already produces `Vec<Argument>` (`{label: Option<String>, expr,
range}`) from comma-separated, optionally-`label:`-prefixed expressions,
zero-arg list included (`Token::RParen` short-circuit → mirror with
`Token::RBracket`). No new grammar to invent; point the bracket case at the
same parsing function the paren case already uses.

```rust
// phalcom-ast/src/ast.rs
Expr::Index(Box<IndexExpr>)       // expr[args...]           — read form
Expr::SetIndex(Box<SetIndexExpr>) // expr[args...] = value    — write form
// IndexExpr { object: Expr, args: Vec<Argument>, range }
// SetIndexExpr { object: Expr, args: Vec<Argument>, value: Expr, range }
```

**Why arg-list, not single-expr:** this is what makes `[]` extend to
multi-key/slice/defaulted-lookup forms *without ever touching this unit's
parser or compiler code again* — `grid[i, j]` sends the distinct selector
`[](_,_)`, `cache[key, default: fallback]` sends `[](_,default:)`, both
coexisting with plain `[](_)` under ADR-0012's name+labels+arity identity
(no fight with the identity-dispatch-⊗-optional-arity hazard — these are
genuinely different user-defined overloads a collection author opts into
later, not an omitted argument on one selector). Today neither `List` nor
`Map` will define `[](_,_)` or any labeled-default form (only the plain
`[](_)`/`[](_,put:)` wrappers this unit adds — see Method-definition
selector above), so `xs[i,j]` correctly `doesNotUnderstand` until someone
adds that overload — this unit does not need to build slicing, it needs to
not foreclose it, and forwarding the raw arg list does exactly that for
free.

**Do not** desugar `[` directly into `MethodCall{method:"[]",...}` inside
`parse_call`. That would make the read and write forms indistinguishable to
`parse_assignment` by the time it sees `left` — exactly the reason
`GetProperty`/`SetProperty` are kept as distinct nodes instead of an
immediate method-call desugar (`parser.rs:1519` vs `1222`). Mirror that
precedent exactly:

1. `parse_call` (`parser.rs:1465`): add `Token::LBracket` to the postfix
   `matches!`. On `[`, call `self.parse_arg_list()` (short-circuiting on
   `Token::RBracket` the same way the existing call does on `Token::RParen`),
   `self.expect(&Token::RBracket, ...)`, produce `Expr::Index{object, args}`.
2. `parse_assignment` (`parser.rs:1217-1233`): add an arm — `Expr::Index(ix)
   => Expr::SetIndex(SetIndexExpr{object: ix.object, args: ix.args, value,
   range})` — parallel to the existing `Expr::GetProperty(get) =>
   Expr::SetProperty(...)` arm.
3. Compiler (`compiler/lib.rs`): compile `Expr::Index{object,args}` as an
   ordinary `MethodCall{object, method:"[]", args}` send — whatever
   selector `args`' arity+labels encode (`[](_)`, `[](_,_)`, `[](_,label:)`,
   ...). Compile `Expr::SetIndex{object,args,value}` as `MethodCall{object,
   method:"[]", args: [...args, Argument{label:Some("put"), expr:value}]}` —
   append, don't replace, so `xs[i,j] = v` sends `[](_,_,put:)` consistently
   with the read form's `[](_,_)`. **No new opcode, no new `Invoke`
   variant** — this rides the existing generic-send bytecode path
   unchanged, for any arity the arg list happens to encode; `"[]"` is just
   another interned selector name, dispatched the same as any other.

### Perf — deliberately no bespoke fast path, but the dedicated selector is NOT free (reconciled against ADR-0051)

**Revised finding (dedicated-selector direction changes this from the
original pure-sugar draft):** `xs[i]` now costs **two** generic `Invoke`s,
not one — the outer `[](_)` send, whose `core.ph` body is `return
self.at(i)`, which is itself a second `Invoke`. Under the original
"desugar straight to `at(_)`" design this unit's `[]` was dispatch-neutral
by construction; under the now-adopted dedicated-selector design it is not
— every `[]`/`[]=` pays one extra full `IndexMap<Symbol, ObjRef>` hash-probe
send relative to calling `.at(i)` directly
([ADR-0051](../../../adr/0051-performance-strategy-measure-first-tiered-optimization.md)
context: "every `Invoke` resolves through an `IndexMap<Symbol, ObjRef>` hash
probe… no IC populated"). This is a real, measurable cost of the user's
directed change, not a hypothetical — flag it plainly rather than repeat
the old "flat timing expected" claim, which no longer holds.
- The `for.wren` port's 144× outlier was flagged as possibly
  `List#add`/`List#at`-attributable (`benchmarks/wren-suite/README.md`) —
  this unit still does not fix that number, and the extra `[]`→`at`
  indirection makes any `[]`-heavy hot loop *slightly worse* until U-IC's
  inline cache lands (both sends are equally cacheable, so U-IC absorbs
  the doubled call count once populated — see below).
  `phalcom-perf --bench-only` after landing should show a **small,
  expected increase** (not "flat") on any benchmark that exercises `[]` in
  a hot loop — record the delta rather than asserting no-change.
- **Not this unit's job to close:** whether the `[](_)`/`[](_,put:)`
  wrapper bodies should be marked for tail-call/inline treatment to erase
  the double-send is a U-IC/U-HOTPATH-scoped question (a monomorphic
  inline cache on both the `[]` send and the inner `at` send absorbs most
  of the cost anyway). Record the measured overhead in Return shape below
  so a future tier has a concrete number to react to, but do not add a
  compiler-level special case for `[]` in this unit — that would be
  exactly the "ship the optimization speculatively, skip the harness"
  move ADR-0051 rejects.

The actual dispatch-cost fix is [U-IC](../U-IC/plan.md) (Tier 3: selector-only
interner + monomorphic inline cache), already planned and unblocked by this
unit's design specifically *because* both `[]` and its inner `at`
delegation are ordinary sends on the same generic-send path U-IC
instruments — no separate cache, no separate fast-path opcode to maintain
in parallel. **Recommendation: do not build a bytecode-level "sacred
index" fast path** (the pattern used for `ifTrue`/`whileTrue`,
[ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md))
for `[]`/`[](_,put:)` (or `at`/`at(_,put:)`) in this unit or as a
follow-on to it. Reasons:
- ADR-0051 is explicit that optimization work is measure-first and
  sequenced through the named tiers — a bespoke index fast path outside
  that sequence is exactly the "ship the optimization speculatively, skip
  the harness" alternative ADR-0051 rejects.
- A sacred-selector fast path needs a deopt-type-guard because `[]`/`at`
  are *ordinary, overridable* `.ph` methods (any user class implementing
  the collection protocol can override either) — this is the
  **speculative inlining ⊗ late binding** hazard. `ifTrue`/`whileTrue` are
  safe to inline because they're privileged `Bool`/`Block` primitives with
  a narrow, closed set of implementers; `[]`/`at` are not — ordinary
  collection-protocol methods, open to override by any future `Iterable`
  subclass or by a collection wanting `[]` to diverge from `at` (see
  Method-definition selector above). Fast-pathing them needs the **same**
  class-identity guard + deopt machinery U-IC is already building
  generically. Building a second, bespoke guard mechanism just for
  indexing duplicates U-IC's purpose instead of riding it.
- Once U-IC lands, both the `[]` send and its inner `at` delegation get
  the inline-cache speedup **for free** — no re-work, because they're
  ordinary sends at the `Invoke` seam U-IC instruments. This absorbs most
  of the doubled-send cost flagged above without this unit doing anything
  special. This is the concrete instance of ADR-0051 §5's "nothing in the
  locked contract is reopened… populates the deferred-sanctioned
  optimizations rather than altering the surfaces they sit behind."

### Design-space reconciliation (language-design skill, step 4/5)

- **Axis:** dispatch.md's message-syntax-sugar axis + syntax.md's operator
  design axis — **revised this session**: `[]` is no longer positioned as
  sugar over an existing message; it is its own operator-selector name,
  joining the set `==`/`+`/`-`/`<`/etc. already occupy.
- **Precedent with consequence — recommendation flipped from the original
  draft:** Lua's `t[i]`/`t[i]=v` **is** its `rawget`/`rawset`/`__index`/
  `__newindex` metamethod protocol — one accessor family, `[]` pure sugar
  over it, no parallel `[]`-specific method; the original draft picked
  this route on the reasoning "Phalcom already locked `at(_)`/`at(_,put:)`
  before `[]` existed, so reuse it like Lua reuses `rawget`." **That
  reasoning under-weighted a stronger, Phalcom-specific precedent:**
  `class Op { ==(other) { return true } }` already dispatches correctly on
  HEAD — Phalcom, unlike Lua, already treats every operator (`==`, `+`,
  `<`, …) as an **ordinary, directly dispatchable method**, not a
  metamethod behind a separate protocol layer. Python/Ruby/Swift's route —
  `[]` as its **own** dedicated protocol member (`__getitem__`/`[]`/
  `subscript`), distinct from any named accessor — is the one consistent
  with how Phalcom already treats every other operator. **Recommendation
  (adopted, supersedes the original "reuse `at`" call): `[]` is a
  dedicated selector, following the `==`/`+`/`<` precedent, not the Lua
  metamethod-reuse route.** The "fragments the collection protocol into
  two ways to be indexable" concern the original draft raised is real but
  bounded: `core.ph`'s `[](...)` wrappers are a thin, one-time delegation
  to `at`, not a parallel semantics — see Method-definition selector above
  for the concrete cost/benefit.
- **Hazard-catalog scan (mandatory, answered explicitly even though clean):**
  - *Identity-dispatch ⊗ optional arity* — does not fire. `[](_)`/
    `[](_,put:)` (and the `at(_)`/`at(_,put:)` they delegate to) are all
    fixed-arity, already-interned-shape selectors; this unit adds no
    default/optional args, so no new selector-identity miss is possible.
  - *Keyword-message selectors ⊗ evaluation order* — `[](_,put:)`'s
    positional/label shape mirrors `at(_,put:)` exactly (index positional,
    value under `put:`); `xs[i] = v` calls it in that exact order — no
    reordering, so no currying/evaluation-order hazard.
  - *Speculative inlining ⊗ late binding* / *inline cache ⊗ mutable
    hierarchy* — would fire **only if** a bespoke fast path were added (see
    Perf above); explicitly not doing that, so the hazard doesn't apply to
    this unit's actual scope. Note the hazard is now slightly *more* live
    in principle than under the pure-sugar draft, because `[]` is a
    genuinely separate, independently-overridable method from `at` — a
    class could override `[]` without overriding `at`, or vice versa. This
    is accepted as the intended flexibility of the dedicated-selector
    direction, not an oversight; it is exactly the same shape as any two
    independent user-defined methods and needs no special dispatch
    handling.
  - *Preclusion (mandatory step 5):* locks `[]` in as its own selector,
    joining the operator-name set — forecloses ever again treating `[]` as
    pure sugar with zero independent identity (reversing this would be a
    breaking, separately-ADR'd change). Does **not** foreclose U-CORE-5's
    future Option-wrapped `at(_)` — the `core.ph` `[](...)` wrapper bodies
    are one-line delegations (`return self.at(i)`), so they inherit
    whatever `at(_)` returns automatically, no rework needed when that
    lands.

## Write-set (STOP-and-report if outside)

- `phalcom-ast/src/ast.rs` — `Expr::Index`, `Expr::SetIndex` node
  definitions.
- `phalcom-ast/src/parser.rs` — `parse_method_name` (`parser.rs:1015`, new
  `Token::LBracket` arm consuming `[`+`]` → name `"[]"`, the
  **method-definition** side), `parse_call` (postfix `[` recognition —
  the **call-site** side, `parser.rs:1465`), `parse_assignment` (the new
  `Expr::Index => Expr::SetIndex` arm, `parser.rs:1217`).
- `phalcom-core/src/compiler/lib.rs` — compile `Expr::Index`/`Expr::SetIndex`
  to `[](...)`/`[](...,put:)` sends (same shape `Expr::GetProperty`/
  `Expr::MethodCall` already compile through).
- `phalcom-core/core/core.ph` — **new**: `[](_)`/`[](_,put:)` on `List` and
  `Map`, `[](_)` (read-only) on `Tuple`, each a one-line delegation to the
  existing `at(_)`/`at(_,put:)`. This is a required deliverable of the
  dedicated-selector direction, not optional — without it `xs[i]` on
  today's collections cleanly `doesNotUnderstand` (see Method-definition
  selector above).
- `docs/adr/00XX-...md` — the new ADR (see Spec anchor).
- `docs/spec/v0.2/lexical-structure.md` (or wherever postfix operators are
  documented) — one new section for `[]` call-site syntax **and** `[]`
  joining the operator-selector name set, cross-referencing
  collection-protocol.md rather than restating its laws.
- **Floor: +0.** No `primitive/*.rs` changes, no new `Value` arm, no new
  opcode. (`core.ph` is a library-surface addition, not a floor change —
  ADR-0019 governs VM-blessed primitives, not `.ph`-defined methods.)

## Build order

1. `parse_method_name` recognizes `[]` as a definable operator name — prove
   `class C { [](i) { ... } }` parses and dispatches (no call-site sugar
   needed yet to test this in isolation; a plain `.["[]"](i)`-shaped direct
   send, or just calling it via the eventual call-site syntax once step 2
   lands, confirms it).
2. AST nodes + parser recognition for call-site `[...]` (read form only,
   `Expr::Index` → `[](_)` send) — prove golden-clean, add the
   newline-boundary fixture from Preconditions.
3. Assignment-form parsing (`Expr::SetIndex` → `[](_,put:)` send) — prove
   golden-clean, including the OOB-write-raises fixture.
4. `core.ph`: add `[](...)` wrapper methods to `List`/`Map`/`Tuple`,
   delegating to `at`/`at(_,put:)` — without this step, none of the
   previously-broken math fixtures or any other `[]` use site actually
   dispatches to anything.
5. Fix the 5 previously-broken benchmark math fixtures that assumed `[]`
   (`vectors.ph`, `stats.ph`, and any others discovered) — delete their
   `.at(i)`-workaround-free header comments' caveat once real.
6. New ADR + spec section. Commit per green step
   ([[commit-frequently]]).

## Tests / verification

- **Primary gate = zero golden diff** on the existing corpus (I1) — this
  unit adds new syntax, so also add new PASS fixtures under a new
  `tests/lang/indexing/` label:
  - read/write on `List`, write-through updates a later read, `Map`
    read/write via `[]`, chained `xs[i][j]` if lists can nest.
  - **`[]` is independently overridable from `at`** — a fixture defining a
    class with only `[](i)` (no `at(_)`) confirms `xs[i]` still works and
    `xs.at(i)` correctly `doesNotUnderstand`, and vice versa. This is new
    behavior versus the original pure-sugar draft and needs explicit
    coverage, not an assumption.
  - `Tuple` write (`tup[i] = v`) raises a plain `doesNotUnderstand` (no
    `[](_,put:)` defined on `Tuple`) — assert the exact shape, per the Gaps
    addendum below.
  - NEGATIVE: OOB write raises; `[]` on a non-indexable `Iterable` (e.g.
    bare `Range`, which gets neither `at` nor `[]`) does-not-understand
    cleanly, not a panic; zero-arg `xs[]` does-not-understand (`[]()` is
    undefined on every collection).
- **The newline-boundary fixture** from Preconditions is load-bearing —
  without it, a regression that started consuming `[` across a line break
  would pass every existing test silently.
- `cargo build && cargo test && cargo clippy --workspace` green; `cargo doc`
  clean. Re-run `phalcom-perf --bench-only` after landing — expect a
  **small, measured increase** on any benchmark exercising `[]` in a hot
  loop (see Perf: the dedicated-selector direction costs one extra
  `Invoke` per `[]`/`[]=` versus calling `.at`/`.at(_,put:)` directly — do
  not assert "unchanged," record the actual delta).

## Gaps found on second-pass review (addendum)

Re-scanned against the hazard catalog + Phalcom's own precedent (not just
generic-language precedent) after the initial draft. Five items the first
pass under-specified:

- **Operator-overload precedent, missed the first time — since adopted.**
  Confirmed live on HEAD: `class Op { ==(other) { return true } }` — user
  classes **already** define operators as literal symbol-selector methods
  (`==(_)`, and by the same mechanism presumably `+(_)`/`<(_)`/etc.). This
  was originally recorded as a stronger-than-credited counter-argument
  while still recommending sugar-over-`at`; **the user has since directed
  the stronger reading be adopted**: `[]` is its own symbol-selector method
  (`[](_)`/`[](_,put:)`), not a rewrite to `at`. See Design →
  "Method-definition selector" and Design-space reconciliation above for
  the adopted design and its cost (every collection needs an explicit
  `[](...)` in `core.ph`, delegating to `at`, rather than getting `[]` for
  free by implementing `at`).
- **Multi-arg/slice syntax inside `[...]` — resolved (superseding the first
  draft's "exactly one expression" restriction, user direction this
  session): `[...]` takes a full call-shaped argument list** — positional +
  `label:` — via `parse_arg_list()` verbatim, not `parse_expr()`. `xs[i,
  j]`, `xs[i, j, k]`, `cache[key, default: fallback]` all parse; each sends
  whatever selector its arity+labels encode (`[](_,_)`, `[](_,_,_)`,
  `[](_,default:)`). No range-literal slice sugar (`xs[1..3]`) — still out,
  because no range-literal parser production exists at all yet
  (wren-suite/README.md), unrelated to this unit. `xs[]` (empty arg list,
  `parse_arg_list` already short-circuits on an immediately-closing
  delimiter) sends `[]()` — zero-arity, undefined on every collection this
  unit's `core.ph` step defines (only `[](_)`/`[](_,put:)` land), so it
  cleanly `doesNotUnderstand`; add as a NEGATIVE fixture rather than
  leaving it an unstated assumption. This is *why* arg-list (not
  single-expr) is the better design: it makes `[]` extensible to future
  multi-key/slice/defaulted-lookup forms with **zero** further changes to
  this unit's parser or compiler code — a collection author adds whichever
  `[](...)` overload they want directly (no detour through `at` required
  for new arities/labels — only the base `[](_)`/`[](_,put:)` delegate to
  `at` at all).
- **Negative-index semantics, unverified.** Checked: `xs.at(0 - 1)` raises
  `Expected a non-negative integer index, got number` (`RuntimeError`, not
  `None`, not Python-style wrap-to-end). `[]` inherits this as-is —
  `xs[-1]` raises, it does not return the last element. Worth a fixture
  specifically because Python/Ruby users will reflexively try `xs[-1]` and
  should get a clean, expected diagnostic, not a surprise.
- **Whitespace-before-bracket — checked, not actually a new hazard.**
  Confirmed `f (5)` (space before paren) already dispatches as a call on
  HEAD — postfix `(`/`.`/`::` are already whitespace-insensitive, only
  newline-sensitive (see Preconditions). `xs [i]` will therefore also mean
  "index `xs`," consistent with existing convention, not a new ambiguity
  `[` introduces.
- **Cross-collection support matrix, incomplete.** Confirmed `Tuple#at(_)`
  works today (`(1,2,3).at(1)` → `2`) — under the dedicated-selector
  direction this means `Tuple` needs its own `[](_)` wrapper in `core.ph`
  (build-order step 4) to make `tup[1]` work; it is **not** free the way
  it would have been under pure sugar. `Tuple` has no `at(_,put:)`
  (immutable, collection-protocol.md law 4) and gets no `[](_,put:)`
  wrapper either — `tup[i] = v` is expected to raise a plain
  `doesNotUnderstand`, correct-by-construction, but **should be a
  fixture**, not an assumption. `Set`/`Range`/lazy `Iterable` views
  (`MapView` etc.) get **no** `[]` wrapper at all — collection-protocol.md
  §2 states `at(_)` is `List`'s refinement, not part of the minimal
  `Iterable` surface, and this unit only adds `[]` wrappers where `at`
  already exists — so `[]` on a bare `Range` should dNU on both `at` and
  `[]`. Precondition for the implementer: enumerate every current
  `Iterable` subclass and record which do/don't implement `at(_)`/
  `at(_,put:)` today, since that census now directly determines which
  classes' `core.ph` entries need a new `[](...)` wrapper — this doc's
  Tuple/Range findings are a sample, not a full census.

## Decisions to flag (DEC-INDEX)

- **DEC-INDEX-A — selector reuse vs new dedicated selector. RESOLVED,
  adopted this session (supersedes the original "reuse `at`"
  recommendation).** `[]` is a dedicated selector — `[](_)`/`[](_,put:)` —
  following the `==`/`+`/`<` operator-method precedent, not a silent
  rewrite to `at`. `core.ph` gets explicit `[](...)` wrapper methods on
  `List`/`Map`/`Tuple` delegating to `at`/`at(_,put:)` (build-order step
  4). This buys a maintainer the ability to make a future `[]` diverge
  semantically from `at` (e.g. a `Map` wanting `[]` to auto-insert on
  write like Ruby's `Hash#[]=` while `at(_,put:)` stays strict) **without**
  a new ADR — `[]` is already its own selector, so changing its `core.ph`
  body is an ordinary library change, not a protocol-level one. Costs:
  every collection needs an explicit opt-in `[](...)` definition (no
  longer automatic for any `at`-implementer), and every `[]`/`[]=` call
  pays one extra generic `Invoke` versus calling `.at` directly (see
  Perf).
- **DEC-INDEX-B — compound assignment (`xs[i] += 1`).** Recommend
  descoping from v1 pending the Preconditions finding on whether
  `obj.prop += 1` already works; ship `[]`/`[]=` only, add compound support
  as a follow-on once the property case's status is known (fix both
  targets together, not `[]` alone).
- **DEC-INDEX-C — chained/nested index on the LHS (`grid[i][j] = v`).** The
  `SetIndex` desugar rewrites only the outermost `Index` node
  (`parse_assignment` sees `left` after the full postfix chain has already
  built nested `Expr::Index(Expr::Index(...))`) — the outer `[j]` becomes
  `SetIndex` (sends `[](j, put: v)`), the inner `[i]` stays an ordinary
  `Index` read (sends `[](i)`), which is correct by construction (same
  pattern as nested property sets already handle via `GetProperty`/
  `SetProperty` composition) — record as confirmed-by-design, not a new
  case to handle.
- **DEC-INDEX-D — should `at`/`at(_,put:)` eventually be removed, collapsing
  onto `[]` as the sole accessor. DEFERRED to v0.3 (user direction this
  session).** Raised and considered: `at` is not just a public accessor —
  `core.ph` itself calls `self.at(i)` internally (`iteratorValue`, `==`,
  `hash`), and ADR-0020 names `at` as the protocol-layer accessor sitting
  directly above the native-array floor. Removing it is a breaking
  protocol change (rewrite every internal `core.ph` call site + the
  existing corpus/benchmarks that call `.at(i)` directly today), not a
  syntax-unit-scoped edit, and it would undo the reason DEC-INDEX-A adopted
  a *dedicated* `[]` selector in the first place — a maintainer needs both
  to exist for `[]` to ever diverge from `at` without a new ADR. **This
  unit ships with both `at` and `[]` live, `[]` delegating to `at`.** Not
  revisited until v0.3; do not fold this into U-INDEX's scope or block
  U-INDEX on resolving it.

## What must this not preclude (P4)

- **U-CORE-5's Option-wrapped `at(_)`.** The `core.ph` `[](...)` wrapper
  bodies are one-line delegations (`return self.at(i)`), so zero rework is
  needed when `at(_)` starts returning explicit `Some(x)` instead of a raw
  value — `[]` inherits it automatically through the delegation. Do not
  hand-roll any None-vs-raw-value special case inside the `[]` wrapper or
  the compiler's desugar.
- **U-IC's inline cache.** `[]`/`[]=` (and the `at`/`at(_,put:)` they
  delegate to) must stay ordinary `Invoke` sends — no parallel dispatch
  path, no bespoke cache, so U-IC's monomorphic IC covers both sends
  automatically once populated. Confirmed by explicitly rejecting the
  bespoke-fast-path alternative above.
- **A future `[]` divergence from `at`** (the point of adopting DEC-INDEX-A)
  — because `[]` is already its own selector, a maintainer can later change
  a specific collection's `[](...)` body to do something `at` doesn't
  (e.g. auto-vivify on write) as an ordinary library edit, no new ADR or
  dispatch mechanism required. This is the flexibility the dedicated
  selector was adopted for — nothing further needs to stay open for it.
- **DEC-INDEX-D — whether `at` itself survives.** Deferred to v0.3, not
  this unit (see Decisions to flag). This unit's design keeps that door
  open either way: `[]` delegates to `at` today, but nothing about
  `Expr::Index`/`Expr::SetIndex` or the compiler's `method:"[]"` desugar
  hard-codes `at`'s existence into the call-site machinery — if a future
  v0.3 decision collapses onto `[]` alone, only `core.ph`'s wrapper bodies
  and the internal `core.ph` call sites (`iteratorValue`/`==`/`hash`) need
  to change, not the parser or compiler this unit lands.

## Return shape (implementer)

commit SHA(s) · `parse_method_name`'s new `[]`-as-operator-name arm landed
(confirm `class C { [](i) {...} }` parses) · `Expr::Index`/`Expr::SetIndex`
landed, compiling to `[](...)`/`[](...,put:)` sends · newline-boundary
fixture result (confirms no ASI-style hazard) · compound-assignment
Precondition finding (does `.prop += 1` work today — DEC-INDEX-B basis) ·
`core.ph` `[](...)` wrapper methods added to `List`/`Map`/`Tuple`
(delegating to `at`/`at(_,put:)`) · new `indexing` label fixture counts
(PASS/NEGATIVE), including the `[]`-without-`at` / `at`-without-`[]`
independence fixture · the 2 (or more) previously-broken
`benchmarks/math/*.ph` fixtures now green via `phalcom-perf --bench-only
--label math` · new ADR number + link · confirmation of zero golden diff ·
`phalcom-perf` before/after showing the **measured delta** for `[]`-heavy
benchmarks (expected: small increase from the extra `[]`→`at` `Invoke`,
not "unchanged" — see Perf) · floor delta (exp 0, `core.ph`-only library
addition) · verify + `cargo doc` tails · write-set confirm.
