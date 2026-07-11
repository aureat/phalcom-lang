# U5 — Work order: control-flow-as-message + sacred-selector inliner

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Grounded in
[`docs/spec/control-flow.md`](../spec/control-flow.md) (§1–3), [`messages-and-selectors.md`](../spec/messages-and-selectors.md)/[`method-lookup.md`](../spec/method-lookup.md) (dispatch shape), and **ADR-0012**
(label-encoded selectors + IC-ready dispatch — the inliner is a coarse IC that special-cases
the "sacred" selectors). Builds on **U4** (blocks/closures, ADR-0013), **U3** (dispatch/selectors,
ADR-0012), **U1** (heap + tagged `Value`, ADR-0009/0010). **Reviewer OFF** for this unit (STATE.md
review policy — U5 is not in the load-bearing set U1/U2/U4/U6); self-verify on the green gate.
STATE.md ADR mapping is authoritative._

---

## 0. Mission (one sentence)
Make **every** control-flow construct an ordinary message send — arithmetic/comparison operators,
`and`/`or`/`not`, `if`/`while`, and the block-form `ifTrue:`/`whileTrue:` families all dispatch
through `Invoke` (Smalltalk semantics) — **then** add a compile-time **inliner** that lowers a fixed
set of *sacred selectors* sent with **literal-block arguments** to guarded branch/jump bytecode, with
a runtime **deopt guard** that falls back to a real send whenever the receiver is not the expected
kernel `Bool`/`Block` **or** the sacred selector has been overridden — so the inlined path is
**observationally identical** to the send it replaces, including non-local return.

## 1. Hard guardrails (read before writing any code)
- **Two layers, each independently green.** Land **Layer 0 (correctness: everything is a send)**
  first and prove the green gate; only then land **Layer 1 (inliner + deopt guard)**. If Layer 1
  can't be verified green on its own diff, it is too big — split the sacred selectors into batches.
- **The inliner is a soundness hazard, not a feature.** An inlined `ifTrue:`/`and`/`whileTrue:`
  **must** behave identically to the send it replaces in *every* observable: result value, side
  effects, error propagation, and **non-local return** (a `return` inside an inlined block must
  unwind to the home method exactly as U4/ADR-0013 specifies for the send form). If you cannot make
  the inlined form transparent for a selector, **do not inline it** — leave it a plain send.
- **Deopt is mandatory and must cover override, not just type.** Overriding `Bool>>and(_:)` (spec
  §2: "`and` and `or` are ordinary methods … and can be overridden") **must** be honored. A guard
  that only checks the receiver *type* is unsound. See §4.
- **Do NOT touch the object model, the metaclass tower, or absence.** No `Option`/`None` work (U6),
  no `True`/`False` split (U11), no `doesNotUnderstand`/`perform` (U8). If the spec's `if/else`
  desugaring appears to need `ifNone` (Option), use the U6-independent form in §4 instead.
- **Stay inside the write-set (§3).** If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). Do not self-approve; the green gate is the gate.
- **`Value::Nil`/`Bytecode::Nil` stay as-is** — surface-`nil` removal is U6's job, not U5's.

## 2. Preconditions (verify first; do not assume)
- **U4 must be landed and green on this branch.** U5 is a *hard* dependent: the deopt fallback for
  every sacred selector is a real block send, and `while`/`repeat` receivers are blocks. Before the
  first edit, confirm via graphify the **actual** names U4 introduced: the block-literal AST node
  (expected in `phalcom-ast/src/ast.rs`), the closure/block **primitive module** (expected new file
  under `phalcom-core/src/primitive/`, e.g. `closure.rs`/`block.rs`), and the block-invocation
  selectors (`value`, `value(_:)`). This plan references them as "U4-provided"; bind them to reality
  first. If U4 is **not** in-tree, **STOP** — U5 cannot start.
- Confirm `./scripts/verify.sh` is **green** before the first edit (baseline). Record the tail.
- **graphify-first:** `graphify affected "Bytecode"`, `graphify affected "binary_op"`, and
  `graphify explain "Compiler"` on the actual HEAD to confirm nothing new references the arithmetic/
  logical opcodes beyond §3 before you delete them.
- Re-derive golden operator usage (`examples/core_new.ph`, `examples/person2.ph` use at least
  `and`, `==`, `-`) so the Number/Bool primitives you register in Layer 0 keep the corpus green.

## 3. Confirmed write-set (derived from `graphify affected` on the control-flow symbols)
| File | Layer | Why it's in scope |
|---|---|---|
| `phalcom-core/src/bytecode.rs` | 0+1 | **Remove** the hardwired operator opcodes `Add,Subtract,Multiply,Divide,Modulo,Equal,NotEqual,Less,LessEqual,Greater,GreaterEqual,And,Or,Negate,Not` (they become sends). **Add** the branch/loop opcodes + deopt-guard opcode(s) (§4). Keep `Constant,Nil,True,False,Pop,Get/Set*,GetSelf,Invoke,Class,Method,Return`. |
| `phalcom-core/src/compiler/lib.rs` (+ `mod.rs`) | 0+1 | Lower `Binary`/`Unary` to `Invoke` sends (Layer 0). Drive the inliner: recognize sacred selector + literal-block args, emit guarded jumps, backpatch offsets (Layer 1). |
| `phalcom-core/src/compiler/inliner.rs` **(NEW)** | 1 | Sacred-selector recognizer + jump/guard emission + fallback-send emission. Isolated from `lib.rs` to keep the hot logic reviewable. Full rustdoc, cite control-flow.md §3 + ADR-0012 + the new ADR-0017 (§4). |
| `phalcom-core/src/vm.rs` | 0+1 | **Delete** the arithmetic/logical opcode arms + `binary_op!` macro + `handle_primitive_op`. **Add** execution for the new jump/loop/guard opcodes. **Add** the sacred-override tracking hook on method installation (§4). |
| `phalcom-core/src/primitive/number.rs` | 0 | Register the arithmetic/comparison selectors as real methods: `+(_:) -(_:) *(_:) /(_:) %(_:) ==(_:) !=(_:) <(_:) <=(_:) >(_:) >=(_:)` and unary negate. These become the *only* path for numeric ops after the opcodes are gone. |
| `phalcom-core/src/primitive/boolean.rs` | 0 | Register the sacred fallbacks on `Bool`: `and(_:) or(_:) not()` (unary), `ifTrue(_:) ifFalse(_:) ifTrue(_:)ifFalse(_:)`. These are what the deopt path calls. |
| `phalcom-core/src/primitive/<closure|block>.rs` (U4-provided) | 0 | Register the loop fallbacks on `Block`: `whileTrue(_:)` and `repeat(_:)` (§4 on `repeat` scope). |
| `phalcom-core/src/primitive/string.rs` | 0 | Register `+(_:)` (and `==(_:)` if a golden concatenates/compares strings) so string operators resolve as sends. Audit golden usage first; add only what the corpus needs. |
| `phalcom-core/src/universe.rs` | 0+1 | Wire the newly-registered primitives into the kernel method dictionaries; initialize the `bool_sacred_pristine`/`block_sacred_pristine` flags at bootstrap. |
| `phalcom-core/core/core.ph` | 0 | Only if a sacred/derived selector is expressed in Phalcom (e.g. `ifFalse:` in terms of `ifTrue:`); keep minimal — the deopt path needs the base cases in Rust so they always exist. |
| `phalcom-core/bin/phalcom/disasm.rs` | 0+1 | Drop the removed opcode arms; disassemble the new jump/loop/guard opcodes (with resolved targets). |
| `phalcom-ast/src/parser.rs` + `phalcom-ast/src/ast.rs` | 0 | **BOUNDARY — see §4/BLOCKED-ON-DECISION-1.** If U5 owns `if`/`while` keyword-sugar parsing, add their productions (recommended: parse-time desugar to `MethodCall` sends over U4 block literals). Tightly scoped to control-flow sugar only. |
| `phalcom-core/tests/lang.rs` + new golden `.ph` + inliner bytecode snapshot | 0+1 | Acceptance corpus rows + goldens + a disasm snapshot proving zero closure alloc on the inlined path (§7). |

**Disjointness note:** U5 runs **alone** on the serial spine (reviewer OFF, its own worktree off
`main`); it is not co-scheduled in a parallel wave, so the `phalcom-ast` and `core.ph` touches are a
**sequencing** concern with later units (U-LEX surface syntax, U-STD `core.ph`, U11 Bool tower), not a
concurrency conflict. U5 lands first; those units extend afterward. Flag the overlap in the return.

## 4. Design decisions (ADR-grounded)

### 4.1 Layer 0 — operators and control words become sends (control-flow.md §1–2, ADR-0012)
- **Arithmetic/comparison** (`+ - * / % == != < <= > >=`) lower directly to `Invoke(selector, 1)`
  using the **single** `encode_selector` helper (ADR-0012 — do not hand-roll a divergent encoder;
  F8 was exactly that). These are **not** sacred and are **never** inlined — every `a + b` is a plain
  send resolved by the Number/String primitive; the U3 monomorphic IC slot absorbs the cost. Unary
  `-`/`not` lower to a 0-arg send (`negated`/`not()` — pick the selector `encode_selector` already
  implies and keep it consistent across compiler + primitive registration).
- **`and`/`or` are lazy** (control-flow.md §2): the argument is a **block**, so the *surface* form
  `a and b` is semantically `a.and { b }` / `a.or { b }` with selectors `and(_:)`/`or(_:)`. In Layer 0
  the compiler lowers `Binary(And/Or)` to a real send whose argument is a U4 block wrapping the RHS
  expression (laziness falls out of the object model — §2). Layer 1 replaces the common case with a
  short-circuit jump (§4.2). `Bool>>and(_:)`/`or(_:)` are ordinary, overridable primitive methods.
- **`if`/`while` keyword sugar** desugars to the block-send forms. **Desugar `if`/`else` to the
  `ifTrue(_:)ifFalse(_:)` sacred selector, NOT to the spec's illustrative `c.ifTrue{}.ifNone{}`
  chain.** Rationale: `ifNone` is an `Option` combinator (U6, which lands *after* U5); the combined
  two-branch sacred selector is semantically equivalent, U6-independent, and directly inlinable.
  Record this reconciliation in the `inliner.rs` `//!` doc. `while (c) { B }` → `{ c }.whileTrue { B }`
  (receiver is a block; sacred). Plain `if (c) { B }` (no else) → `c.ifTrue { B }`.
- **`for (x in xs) { }` is DEFERRED out of U5.** It desugars to `xs.each { x => … }` — a *non-sacred*
  send needing an iterable protocol/`each` that no kernel type defines until collections exist
  (U-STD). Do **not** wire `for` runtime here; if the parser is extended, it may parse+desugar but
  there is nothing to run against, so it is **not** in U5's must-pass set. Note it in the return.

### 4.2 Layer 1 — the sacred-selector inliner (control-flow.md §3, ADR-0012)
- **Sacred set** (spec §3): `ifTrue(_:)`, `ifFalse(_:)`, `ifTrue(_:)ifFalse(_:)`, `and(_:)`, `or(_:)`,
  `whileTrue(_:)`, `repeat(_:)`. The compiler inlines a send **only when** (a) the selector is sacred
  **and** (b) every block argument is a **literal block at the call site** (a U4 block-literal AST
  node, not a variable holding a block). Otherwise it emits the ordinary Layer-0 send.
- **New opcodes** (bytecode.rs): relative `Jump(i16)`, `JumpIfFalse(i16)` (pops/peeks a `Bool`), a
  backward `Loop(u16)`, and a **deopt-guard** opcode `GuardBool(i16)` / `GuardBlock(i16)` (peeks the
  receiver; if it is not the kernel-`Bool`/kernel-`Block` immediate/instance **or** the sacred flag is
  dirty, branch to the fallback offset without consuming the receiver). Emit placeholder offsets and
  backpatch (standard clox-style). Justify the offset width in the opcode `///`.
- **Inlined shape** (example `a.ifTrue { B }`): `⟨eval a⟩; GuardBool(fallback); JumpIfFalse(end);`
  `⟨inline B⟩; Jump(end); fallback: ⟨push literal block⟩; Invoke(ifTrue(_:),1); end:`. The block body
  `B` is compiled **inline** (no `ClosureObject` allocation, no call frame) — this is the whole point
  (spec §3, Invariant 5). `and`/`or` inline to short-circuit: eval LHS, `GuardBool(fallback)`,
  `JumpIfFalse/JumpIfTrue` to the known boolean result, else eval RHS inline. `whileTrue` inlines to a
  guarded backward `Loop`.
- **The deopt guard = type check + override epoch (this is the soundness core).** A type-only guard
  is unsound because `Bool>>and(_:)` is overridable. Because `Value::Bool` is an **immediate** whose
  class is *always* the kernel `Bool` (users cannot forge a `Bool` subclass instance), and U4 blocks
  are always kernel `Block`, the only override risk is replacing a sacred method **on the kernel class
  itself**. So track a coarse per-kernel-class **pristine flag** (`bool_sacred_pristine`,
  `block_sacred_pristine`, on the VM/universe): set true at bootstrap; flip to **false** the moment any
  sacred selector is (re)installed on `Bool`/`Block` (hook the method-installation path — the `Method`
  opcode / class-extension in vm.rs). The guard opcode deopts iff `!matches!(recv, expected)` **or**
  `!pristine`. This is a coarse inline-cache invalidation (ADR-0012's IC-ready dispatch), O(1) per
  guard, and preserves the spec's override genericity. **Reject** the simpler "seal sacred selectors"
  option — the spec explicitly wants `and`/`or` overridable.
- **Non-local return through inlined blocks (ADR-0013).** Inlined block bodies are spliced into the
  home method's chunk, so a `return` inside them is just the method's ordinary `Return` — it unwinds
  to the home method for free, identical to the send form's frame-token non-local return. Verify this
  equivalence with a test (§7); it is the highest-value correctness assertion in this unit.

### 4.3 New ADR required
No ADR pins the **deopt-guard mechanism** (control-flow.md §3 mandates the *behavior* —
"deoptimizes to a real send" — but not the mechanism; ADR-0012 gives IC-readiness but not the sacred
special-case). Per the guardrail (a load-bearing mechanism lacking ADR coverage needs one), draft
**ADR-0017 — sacred-selector inliner + override-epoch deopt guard** (the `documentation-and-adrs`
skill drafts it; cite control-flow.md §3 and ADR-0012). The mechanism above is derivable and
recommended, so this does **not** block implementation — but the ADR must land with the unit.

## 5. Build order (land Layer 0 green before starting Layer 1)
1. **`bytecode.rs`** — add `Jump/JumpIfFalse/Loop/GuardBool/GuardBlock`; **do not remove** the operator
   opcodes yet (keep the tree compiling). Full rustdoc + per-variant `///`, cite control-flow.md/ADR-0017.
2. **Primitives (Layer 0 base cases)** — register the Number/String operator methods and the
   Bool/Block sacred fallbacks (`primitive/number.rs`, `string.rs`, `boolean.rs`, U4's closure module),
   wired in `universe.rs`; initialize the pristine flags. Prove they resolve as sends.
3. **Compiler Layer 0** — lower `Binary`/`Unary` to `Invoke` sends; lower `and`/`or` to lazy block
   sends. **Now delete** the operator opcode arms from `vm.rs` (+ `binary_op!`, `handle_primitive_op`)
   and `bytecode.rs`, and the `disasm.rs` arms. `./scripts/verify.sh` **must be green here** — this is
   the Layer-0 gate (everything is a send; goldens byte-identical).
4. **`if`/`while` surface parsing** (if U5 owns it per BLOCKED-ON-DECISION-1) — parse-time desugar to
   `MethodCall` sends over U4 block literals. Green again.
5. **`inliner.rs` + compiler Layer 1** — sacred-selector recognizer, guarded jump emission, backpatch,
   fallback-send emission; VM execution of the new opcodes; the override-epoch hook. Green again.
6. **Tests + ADR-0017** — acceptance rows, control-flow golden, inliner bytecode snapshot, override/
   deopt test, non-local-return-through-inlined-block test; land ADR-0017.

## 6. Fold-in cleanup (U5 owns the operator opcodes end-to-end)
Removing the hardwired operator opcodes is the cleanup: delete the opcode variants, their `vm.rs`
arms, the `binary_op!` macro, `handle_primitive_op`, and their `disasm.rs`/compiler emitters in one
coherent Layer-0 diff — leave **no** dead arithmetic opcode behind. `graphify affected` on each removed
opcode first to confirm no out-of-write-set references. No other DEFERRED item is assigned to U5.

## 7. Test strategy (what the harness must assert)
- **Semantic equivalence (Layer 0).** For each control construct, a golden `.ph` proving the keyword
  form and the explicit send form produce **identical** output: `if/else` ≡ `ifTrue:ifFalse:`;
  `while` ≡ `{…}.whileTrue{…}`; `a and b` ≡ `a.and{b}`; `a + b` ≡ `a.+(b)` send. Goldens stay
  byte-identical.
- **Override honored (deopt correctness).** A `.ph` that overrides `Bool>>and(_:)` (or `ifTrue:`) and
  asserts the **override runs** — proving the inlined site deopts on the dirty pristine flag rather
  than silently taking the fast path. This is the load-bearing soundness test.
- **Non-local return transparency.** A `.ph` where a `return` inside an inlined `ifTrue:`/`whileTrue:`
  block unwinds the home method, with output identical to the non-inlined (overridden) form. Assert
  both paths agree.
- **Zero-alloc hot path.** A **disasm/bytecode snapshot** of an inlined `if`/`while` showing jump/guard
  opcodes and **no** `Closure`/block-alloc + **no** `Invoke` on the common path (spec §3).
- **Guard-on-wrong-type.** A send of a sacred selector to a non-`Bool`/non-`Block` receiver deopts to a
  real send (and errors/dispatches normally) rather than miscompiling.
- **Regression.** Full `./scripts/verify.sh` green at the Layer-0 gate **and** the Layer-1 gate.

## 8. Forward-looking notes (do not box us in — checked against open-questions.md)
- **U6 (Option).** Keep the `if/else` desugaring on `ifTrue:ifFalse:` so U5 has **no** dependency on
  `Option`/`ifNone`; U6 later adds `if (opt)` as a *compile error* (no truthiness) — U5 must not
  introduce any truthiness coercion that U6 would have to unwind.
- **U11 (Bool → True/False tower).** The deopt guard keys on "kernel `Bool`, pristine" — when U11 splits
  `Bool` into `True`/`False`, the guard must still recognize both as inlinable. Keep the guard's notion
  of "expected kernel boolean class" a single point of truth so U11 can widen it without touching call
  sites (ADR-0004; spec §2 "and/or ordinary methods").
- **open-Q2 (Int/Float tower, ADR-0005 flat-for-now).** Do **not** hardwire two-operand `f64`
  assumptions into the compiler; arithmetic is a plain send resolved by the Number primitive, so a
  future numeric-tower split changes only the primitive, not the call site.
- **IC population (deferred, ADR-0012).** The override-epoch flag is the coarse invalidation seed; keep
  it compatible with a future per-call-site polymorphic IC (don't assume a global-only invalidation
  model forever — a per-class epoch counter generalizes more cleanly than a single bool if cheap).
- **U8 (doesNotUnderstand).** Operators-as-sends means an unresolved `+`/`<` becomes a normal missing-
  method dispatch; leave the miss path exactly where U3 put it so U8 can slot dNU in without a special
  operator case.

## 9. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `//!` on the
  new `inliner` module + every touched module; `///` on every new/changed public item (new opcodes +
  each variant, the guard opcode with a `# Panics`/deopt note, every registered primitive, the
  pristine-flag API) with intra-doc links and control-flow.md/ADR-0012/ADR-0017 citations. `cargo doc
  --workspace --no-deps` adds **no new warnings**.
- **Green gate:** `./scripts/verify.sh` exits 0 (build + test + clippy + golden + `lang.rs` corpus +
  invariants) at **both** the Layer-0 and Layer-1 gates. Golden output byte-identical. Don't add clippy
  warnings; fix pre-existing ones in files you rewrite.
- **Reviewer OFF** (STATE.md policy): no independent `phalcom-reviewer` gate — **self-verify** on the
  green gate + `cargo doc` + the §7 soundness tests. State this explicitly in the return.
- **graphify-first** orientation; `graphify update . --no-cluster` after edits.

## 10. Return contract (self-verified — state reviewer OFF)
Report: the sacred-selector set actually inlined + which were left as plain sends and why · the final
opcode set added/removed · the exact deopt-guard mechanism as built + the pristine-flag hook location ·
how non-local-return-through-inlined-block was proven transparent (test name + output-equivalence) · the
inliner bytecode snapshot proving zero closure alloc · confirmation the operator opcodes are fully
removed with no dead arms · whether `if`/`while` surface parsing was included (BLOCKED-ON-DECISION-1
resolution) and that `for` was deferred · `verify.sh` tail at both gates · `cargo doc` tail · ADR-0017
landed · any new `DEFERRED.md` entries (e.g. `for`/collections, per-call-site IC).

---

## BLOCKED-ON-DECISION

**BD-U5-1 — Ownership of `if`/`while`/`for` surface keyword-sugar parsing (U5 vs U-LEX).**
The lexer already emits `If/Else/While/For` tokens, but the parser does **not** consume them and the
AST has **no** `If`/`While`/`For` nodes — so no `if`/`while` program parses today. control-flow.md §1
is U5's spec and frames these keywords as the primary surface for control flow, which argues U5 should
own their parse-time desugaring. But STATE.md/handoff list **U-LEX (surface syntax)** as a distinct
later Wave-F unit, creating a `phalcom-ast` boundary overlap.
- **Option A (recommended):** U5 owns tightly-scoped `if`/`while` parse-time desugaring now (parser
  rewrites them to `MethodCall` sends over U4 block literals — `ifTrue:`/`ifTrue:ifFalse:`/`whileTrue:`).
  Pros: control-flow-as-message is coherent and demoable; one author owns desugaring + inliner so they
  are provably identical; U-LEX later extends for other syntax. Adds `phalcom-ast/{parser,ast}.rs` to
  the write-set (sequencing-only, since U5 runs alone).
- **Option B:** U5 ships only the send/inliner machinery + `.ifTrue{}`/`.whileTrue{}` block forms;
  `if`/`while` keyword parsing waits for U-LEX. Pros: `phalcom-ast` untouched by U5. Cons: users can't
  write `if`/`while` until U-LEX; the unit named "control-flow" ships without the control keywords.
- **Recommendation: Option A.** Confirm before implementation so the write-set boundary with U-LEX is
  agreed.

**BD-U5-2 — Scope of `repeat(_:)` and unary-operator selector names.** control-flow.md §3 lists
`repeat(_:)` as sacred but does not pin its receiver/semantics (block infinite-loop vs `n`-times), and
the spec does not name the unary-minus/`not` selectors. **Recommendation:** implement the
unambiguous sacred selectors first (`ifTrue:`, `ifFalse:`, `ifTrue:ifFalse:`, `and:`, `or:`,
`whileTrue:`); treat `repeat(_:)` as inlinable only once its receiver/semantics are pinned (else defer
to DEFERRED). For unary ops, use whatever `encode_selector` already yields and keep compiler +
primitive registration in lockstep. Not a hard blocker — flagged so the implementer doesn't invent
`repeat` semantics silently.
