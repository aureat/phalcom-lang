# U-COLL — Work order: collection literals (`[…]` list, `(a,b)` tuple, `{k:v}` map disambiguation)

_Self-contained implementation plan for **one** implementer. Post-U-LIST / post-U-STD surface unit.
**Reviewer OFF** — self-verify on the green gate (`./scripts/verify.sh` exits 0) + `cargo doc` clean.
Grounded in **lexical-structure.md §4 (literals), §6 (brace disambiguation), §7 (why tuples survive),
§8 (`*` grammar)**, resolving the Tier-B gap [`implementation-status.md` line 59]
("Tuple/list/map/set literals + brace disambiguation: no AST nodes"). Depends on **U-LIST** (the
landed kernel `List` is the lowering target) and **U4** (block literals own `{…}`, which the map
literal must disambiguate against). Governing ADRs: **[ADR-0029](../../../adr/0029-list-literal-syntax.md)**
(list literals, Accepted) + **[ADR-0032](../../../adr/0032-collections-representation-and-literals.md)**
(collections umbrella — map/tuple literals ratified, set/range sigils reserved, Accepted)._

> **Unit-name note.** This is a Tier-B **surface-syntax** unit, so it follows the named-unit
> convention (U-LEX / U-LIST / U-STD) rather than a number. The numeric slots U13–U17 were being
> claimed concurrently by the design-questions cluster (class-hierarchy, **destructuring=Q7**,
> modules, `::`-references, `Option`-bootstrap) during planning; a name sidesteps that race. The
> collision consequences of that cluster are load-bearing — see §4.1.

---

## 0. Claim, gates, and decisions (all resolved)

### 0.1 Unit claim
- **Unit ID `U-COLL`** — verified free at plan time (numeric U13–U17 taken by the concurrent cluster).
- This plan does **not** touch `docs/forge/PHASE2-INDEX.md` (shared file, concurrent editors).

### 0.2 Governing ADRs (ratified — no draft needed)
Collection-literal lowering is now ratified — no stub required:
**[ADR-0029](../../../adr/0029-list-literal-syntax.md)** (list literals, Accepted) and the collections
umbrella **[ADR-0032](../../../adr/0032-collections-representation-and-literals.md)** (Accepted), which
ratifies the **map `{k:v}`** and **tuple `(a,b)`** literals (§3.1/§3.2: bare-ident keys are symbols,
`{}` stays a block, `(a)` stays grouping, one-element `(a,)`) and reserves the **set `#{…}`** and
**range `..`/`...`** sigils (inactive, committed meaning). Its decision, in one paragraph:

> Surface collection literals are **desugared in the parser to ordinary message sends on existing
> kernel classes** — the same layer as `if`→`ifTrue`, `while`→`whileTrue`, `\(e)`→`toString`,
> `??`→`orElse`, `?.`→`map`. **No new `Expr` variants, no new bytecode, no new floor primitives.**
> `[a,b,c]` → `List.new().add(a).add(b).add(c)`; `(a,b)` → a distinct minimal `Tuple` (see DEC-COLL-A);
> `{k:v}` disambiguation is solved per spec §6 but its runtime target is deferred (see DEC-COLL-B).
> This keeps the write-set inside `phalcom-ast` (+ optionally `core.ph`) and honours the frozen
> primitive floor (ADR-0019): **not one of the 73 floor bindings is added.**

### 0.3 Decisions resolved (no blocker remains)
DEC-COLL-A and DEC-COLL-B (§8) are both **resolved** by ADR-0032: `Tuple`/`Map` are native arms built
by the collection-runtime unit (U-COLLTYPES); U-COLL ships the literals + `{k:v}` disambiguation against
a construction/pending target. The whole unit is now dispatch-ready.

---

## 1. Mission (one sentence)
Give the surface syntax its collection literals — **list `[…]`** and **tuple `(a,b)`** desugaring in
the parser to sends on the landed kernel `List` (and a minimal `Tuple`), plus the **spec §6 one-token
brace disambiguation** so `{k: v}` is recognised as a map and never mis-parsed as a block — all with
**zero new primitives** and the map runtime target explicitly deferred to the U-CORE `Map` unit.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U-LIST landed** — `class List` exists in `core.ph` with `new` (native class-side allocate),
  `add(_)` **returning `self`** (core.ph:133–136, verified — this is what makes `.add(a).add(b)`
  chaining a valid single-expression desugaring), `size`, `at(_)`, `each(_)`, `toString`.
- **U-STD landed** — `List` additionally has `map`/`filter`/`reduce`/`includes`/`isEmpty`/`at(_:put:)`
  (pure `.ph`). Not required, but confirms `List.new()` + `add` is the blessed construction idiom.
- **U4 landed** — block literals own `{…}` in expression position; `parse_primary`'s `Token::LBrace`
  arm (parser.rs:1439–1499) already does a param-scan (`{ x, y => … }` vs `{ … }`). **The map slice
  extends this exact arm** — re-read it on HEAD before editing.
- **U9 landed** — rest params `*name` collect into a `List`; `Asterisk` is the spread token
  (token.rs:190). Only relevant to the *must-not-preclude* spread check (§9); U-COLL ships no spread.
- **Tokens present** (token.rs): `LBracket`/`RBracket` (111/113), `LBrace`/`RBrace` (107/109),
  `Colon` (147), `Comma` (151), `LParen`. **No lexer change is needed.**
- **`==` is value equality** — `object_eq` → `Value::value_eq` (primitive/object.rs:62). Relevant only
  to DEC-COLL-B's map-key semantics; not used by list/tuple.
- Baseline `./scripts/verify.sh` green before the first edit.
- Re-run `graphify explain "parse_primary"`, `graphify affected "core.ph"` **and check the concurrent
  U13/U14/U15/U16/U17 landing state** (§4.1) on real HEAD before dispatching any slice.

## 3. Design (realise; the syntax is fully specified in lexical-structure.md — do not re-litigate it)

### 3.1 List literal `[e1, …, en]` — UNBLOCKED, no fork, no `core.ph`
Add a `Token::LBracket` arm to `parse_primary` (parser.rs, alongside the `LParen`/`LBrace` arms):
advance past `[`, parse a comma-separated expression list (reuse the element-scanner from §3.4) until
`]`, then **desugar to a `List` construction chain**:
- `[]`            → `List.new()`
- `[a]`           → `List.new().add(a)`
- `[a, b, c]`     → `List.new().add(a).add(b).add(c)`

Built as nested `Expr::MethodCall` nodes: receiver `Expr::Var{"List"}`, selector `new` (0-arg), then
fold each element into `MethodCall{ object: acc, method: "add", args: [elem] }`. `add` returns `self`
(verified), so the chain is one well-typed expression. The synthetic range spans `[ … ]` so
diagnostics point at the literal. **Zero new primitives, zero `core.ph` edits** — this is the entire
list slice, and it is green on its own.

### 3.2 Tuple literal `(e1, …, en)`, n ≥ 2 — recommended slice (DEC-COLL-A)
Extend the existing `Token::LParen` arm (parser.rs:1433–1438). Today it is grouping: `(` → parse one
expr → expect `)`. Change to: parse the first expr, **then if the next token is `,`**, keep parsing
comma-separated exprs to `)` and build a tuple; **otherwise behave exactly as today** (a single
parenthesised expr stays grouping — `(x)` is `x`, never a 1-tuple). Unambiguous per **spec §7**:
`(a, b) => …` is *not* in the language (unbraced arrows are single-parameter), so `(` never begins a
parameter list and no cover grammar is needed.

**Lowering target (DEC-COLL-A, recommendation A):** a **distinct minimal pure-`.ph` `Tuple`** authored
in `core.ph`, backed by a `List` field, so `Tuple` ≠ `List` (the typing surface,
`docs/spec/experimental/typing-stdlib-surface.md`, specifies `(a,b) : Tuple<A,B>` as a fixed-arity
heterogeneous product — *not* a `List`). To dodge any dependency on variadic `construct`, desugar via a
**non-variadic factory that takes an already-built `List`**:
- `(a, b)`   → `Tuple.fromList(List.new().add(a).add(b))`

```phalcom
class Tuple {
  construct(elems) { _elems = elems }          // elems : an already-built List
  static fromList(xs) => Tuple.new(xs)          // the parser's desugar target
  at(i)   => _elems.at(i)
  size    => _elems.size
  each(f) { _elems.each(f) }
  toString => /* "(e1, e2)" rendering; mirror List.toString but with parens */
}
```
This is **pure `.ph`, zero new primitives**, and reuses the list slice's construction chain as its
argument. See DEC-COLL-A for the rejected alternative (tuple → `List`: zero-`core.ph`, but collapses the
tuple/list distinction and mildly boxes out the typing surface).

### 3.3 Map literal `{k: v, …}` — brace disambiguation is solved here; the *target* is deferred
**This is the hard part of the surface (spec §6) and it is fully designed here so the follow-on is
dispatch-ready.** Extend `parse_primary`'s `Token::LBrace` arm with the spec §6 LR(1) table, decided by
**one token of lookahead after `{`**:

| `{` followed by | Construct | Action |
|-----------------|-----------|--------|
| `IDENT :`       | **Map literal** | new branch (below) |
| `IDENT ,`       | Block, with parameters | existing param-scan |
| `IDENT =>`      | Block, with parameters | existing param-scan |
| `}`             | Empty block | existing |
| anything else   | Block, zero parameters | existing |

The map branch peeks `IDENT` then `Colon` (the *only* new discriminator — everything else the U4 arm
already handles), then parses comma-separated `IDENT : expr` pairs to `}`. Keys are **symbols**
(`{a: 1}` ≡ key `#a`), matching the spec's `Map<Symbol, ?>` default and mirroring labeled-argument
parsing (parser.rs:1511). Empty map is **`Map()`**, *not* `{}` (spec §6: `{}` is the empty block) —
there is no empty-map literal, by design. Per
[ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §3.1 a **string/number literal
or parenthesized key** is taken as an *expression* (`{"a": v}` → string key, `{(k): v}` → computed);
the implementer handles those or explicitly defers them to a follow-on.

**Runtime target — DEC-COLL-B (resolved, §8).** `Object#hash`-as-floor is **ratified**
([ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md), landed U-CORE-1), the
equality/mutability model is ruled ([decisions.md](../../../spec/v0.2/core/decisions.md) Q5), and
native-arm `Map` is ratified ([ADR-0032](../../../adr/0032-collections-representation-and-literals.md)
§1). U-COLL **must not author a competing `Map`**: it ships §6 disambiguation + a precise diagnostic,
and the one-line `{k:v}` → real-native-`Map` wiring lands in the **collection-runtime unit
(U-COLLTYPES)** after it. The genuinely hard work (the LR(1) disambiguation) ships in U-COLL either way.

### 3.4 Shared element-scanner (reused by list, tuple, and — LIVE — the concurrent U14 destructuring)
Factor the "comma-separated expression list with a terminator" loop into one private parser helper
(`parse_comma_exprs(terminator)`), used by both the `[…]` and `(…)` arms. **Its element grammar
(bare name, nested literal, reserved `*`-prefix slot) is deliberately identical to what the concurrent
U14 destructuring-pattern parser scans** — see §4.1 and §9. Landing U-COLL first lets U14 build on this
scanner instead of forking a parallel one.

### 3.5 Set literal — NON-GOAL (spec §4)
Spec §4/§6 are explicit: **there is no set literal.** `{1, 2, 3}` is ambiguous with a block and "not
resolvable by lookahead"; `Set(…)` is a plain send and costs nothing. U-COLL ships **no** set syntax.
Open-Q6's `#{1, 2, 3}` alternative is preserved as a future option, not precluded (§9).

### 3.6 Native-vs-`.ph` split (task step 5) & the frozen-floor rule
- **List literal:** pure parser desugaring to existing sends. **0 new primitives, 0 `core.ph`.**
- **Tuple literal:** pure parser desugaring + a **pure-`.ph`** `Tuple` class. **0 new primitives.**
- **Map literal:** disambiguation only in U-COLL; runtime deferred. **0 new primitives.**
- **Net floor delta: 0.** No ADR-0019 amendment required — exactly the "prefer pure lowering to
  existing List/construct calls over new primitives" outcome the frozen-floor non-negotiable demands.

### Rubric — hazards & preclusion (mandatory)
- **Parser precedent (soundness of the desugaring layer):** every existing surface-sugar in Phalcom
  desugars **in the parser to `MethodCall` nodes** (if/while/interp/`??`/`?.`). U-COLL follows that
  layer. Dedicated `Expr::ListLit`/`TupleLit`/`MapLit` variants were **considered and rejected**: they
  pull `phalcom-core/src/compiler/lib.rs` (a spine file) into the write-set for no semantic gain, and do
  not help U14 — destructuring patterns live on the *LHS*, a different parse path, so an expression-side
  AST node buys nothing there.
- **Brace-disambiguation ⊗ block literals (the U4 interaction):** the map branch must not regress any
  block form. Pin goldens for **all five §6 rows** (`{a:1}` map, `{x, y => …}` param-block,
  `{x => …}` param-block, `{}` empty block, `{ expr }` zero-param block) so a future arm edit can't
  silently reclassify one. This is the single highest-risk line of the unit.
- **`,` in `(…)` ⊗ grouping (the tuple interaction):** the `(` arm must fall through to *exactly*
  today's grouping when there is no top-level comma — regression-test `(x)` ≡ `x` and `(a + b) * c`
  precedence to prove the tuple path didn't perturb grouping.
- **Representation/dispatch impact:** none. No `Value` tag change, no selector-encoding change, no new
  opcode. `List`/`Tuple`/`Map` dispatch through the ordinary `Invoke` path.
- **Preclusion (mandatory step-5):** desugaring collection *expressions* forecloses nothing for U14/Q7
  destructuring *patterns* (LHS) or Q6's `#{…}` set literal (a distinct `#{` lexer token) — see §9. It
  **does** commit to "list literal == a `List`", which is intended and additive.
- **Precedent:** Smalltalk brace arrays `{a. b. c}` desugar to `Array with:with:` sends (parser-level,
  no new bytecode) — the direct model. Ruby/Python literals compile to dedicated opcodes (rejected: they
  freeze the construction path against the dogfooding goal and cost bytecode surface).

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why it's in scope | Slice |
|---|---|---|
| `phalcom-ast/src/parser.rs` | new `Token::LBracket` arm (list); extend `Token::LParen` arm (tuple); extend `Token::LBrace` arm with the §6 map discriminator; add the `parse_comma_exprs` helper. Full rustdoc on new fns. | list + tuple + map |
| `phalcom-ast/src/error.rs` | only if the map slice adds a precise "map literal pending" diagnostic (DEC-COLL-B=B) — a new `SyntaxErrorKind` variant. Prefer reusing an existing variant. | map (B) |
| `phalcom-core/core/core.ph` | **tuple slice only** — the pure-`.ph` `class Tuple` reopen. **Omitted entirely if DEC-COLL-A resolves to B (tuple → List).** | tuple (A) |
| `phalcom-core/tests/lang/collections/` (**new label**) + `tests/lang/MANIFEST.md` | goldens + negatives + PENDING (§7). Create the `collections` label per MANIFEST's "adding a case" note. | all |

**Deliberately NOT in the write-set:** `phalcom-core/src/compiler/*`, `vm.rs`, `bytecode.rs`,
`primitive/*`, `universe.rs`, `heap.rs`, `value.rs` (zero new primitives / opcodes / VM-blessed classes;
`Tuple` is an *ordinary* `.ph` class defined after `List` in `core.ph`, unlike `List`/`Option`).

### 4.1 Write-set collision risk (task step 4 — flagged, not resolved) — SHARPENED by the live cluster
The concurrent design-questions cluster claimed **U13** (class-hierarchy), **U14** (destructuring = Q7),
**U15** (modules/imports), **U16** (`::` method references), **U17** (`Option` bootstrap). Their
write-sets vs U-COLL:

- **`phalcom-ast/src/parser.rs` is now contended by U14, U15, U16, and U-COLL simultaneously** (plus
  U13's *permissive* traits branch `class C with T` — U13's own plan says its conservative branch avoids
  `phalcom-ast` and "can run in parallel with a `phalcom-ast`/compiler-bound unit"). **These units
  cannot share a parallel wave** — same file. The orchestrator must **serialize the `phalcom-ast`
  editors** (U14 ‖ U15 ‖ U16 ‖ U-COLL → each its own slot), exactly as the spine already serialized
  U4/U5/U6/U7/U-LEX (PHASE2-INDEX §3). I flag this; I cannot coordinate live.
- **U14 (destructuring) ⊗ U-COLL is the tightest coupling, not just a file clash.** U14 parses
  destructuring **patterns** `(a, b)` / `[first, *rest]` on the **LHS** of `let`/`var`; U-COLL parses the
  same `(…)`/`[…]` comma+spread grammar as **expressions**. They share the §3.4 element grammar.
  **Recommendation: land U-COLL first** so U14 reuses `parse_comma_exprs` (and its reserved `*`-slot)
  rather than forking a divergent pattern-scanner — this is the boundary the task said to "note, don't
  absorb." U-COLL owns expression position; U14 owns pattern position; the shared helper is the seam.
- **`phalcom-core/core/core.ph` — the never-two-editors file.** If U-COLL's tuple slice authors `class
  Tuple` (DEC-COLL-A=A), it edits `core.ph`, which the **U-CORE track is actively editing** (live churn
  in git status: `docs/spec/core/*`; U-CORE authors `Map`/`Set` there). Per "never co-schedule two
  `core.ph` editors," **U-COLL's tuple slice must be serialized against every U-CORE `core.ph` unit.**
  The **list slice touches no `core.ph`** and is free of this — the reason to keep list as the
  collision-free MVP.
- **`tests/lang/` corpus** — append-only shared file; low risk; coordinate the MANIFEST count bump.

## 5. Build order (small, independently-green diffs)
1. **`parse_comma_exprs` helper** + **list `[…]` arm** + goldens → verify green. *(A complete shippable
   MVP: pure parser, no `core.ph`, no fork, collides only in `parser.rs`.)*
2. **Tuple `(…)` arm** + `class Tuple` reopen in `core.ph` + goldens → verify green. *(Gated on
   DEC-COLL-A; serialize against U-CORE `core.ph`.)*
3. **Map `{…}` §6 disambiguation** + all-five-rows goldens + (per DEC-COLL-B) either the minimal `Map`
   or the "pending" diagnostic + PENDING goldens → verify green. *(Gated on DEC-COLL-B.)*

Each step is a self-verifiable commit; if any can't go green alone, it is already split correctly.

## 6. Mandatory rules
- **Docs:** `///` on every new parser fn (`parse_comma_exprs`, doc additions to the extended arms)
  citing lexical-structure §4/§6/§7 and ADR-0029/0032. `cargo doc --workspace --no-deps` adds no new warnings.
- **Green gate:** `./scripts/verify.sh` exits 0. No new clippy warnings. No `unsafe`.
- Follow `rust-best-practices`.

## 7. Test strategy (the green gate must assert) — new `collections` label
- **List (PASS):** `[]` → empty (`.size` 0); `[1, 2, 3]` → `.size` 3, `.at(0)`/`.at(2)` round-trip;
  `[1, 2, 3].map { x => x + 1 }` proves the literal yields a real `List` (combinator interop).
- **List `toString` (PASS):** `[1, 2, 3].toString` renders exactly as the landed `List.toString`
  (pin the format; do not invent a second one).
- **Tuple (PASS, DEC-COLL-A=A):** `(3, 4).at(0)` → 3, `.size` → 2; `(3, 4).class` is **`Tuple`, not
  `List`** (proves the distinction the typing surface requires).
- **Grouping unchanged (PASS, regression):** `(x)` where `x = 5` → 5; `(1 + 2) * 3` → 9 (tuple path
  didn't perturb precedence/grouping).
- **Brace disambiguation (PASS + NEGATIVE) — all five §6 rows:** `{a: 1}` (map or the pending
  diagnostic per DEC-COLL-B), `{x, y => x + y}` (2-param block, invoked), `{x => x}` (1-param block),
  `{}` (empty block), `{ 1 + 1 }` (zero-param block → 2). These five are the anti-regression harness
  for the U4 interaction.
- **Map (PENDING, under `collections/pending/`):** `{a: 1, b: 2}` with `.expected` pinned to the
  *intended* spec output (e.g. `at(#a)` → 1). Wired `#[ignore]` `check_pending`; graduates to PASS when
  the map follow-on / U-CORE `Map` lands (or immediately, if DEC-COLL-B resolves to A).
- **NEGATIVE:** `{}` used as a **statement** is a parse error (spec §6); empty-map-as-`{}` errors with a
  message pointing to `Map()`.

## 8. Decisions flagged (the "flag, don't pick" register)
| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-COLL-A** ✅ **RESOLVED** | **Tuple lowering target.** | Superseded by **[ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1**: `Tuple` is a **native heap arm** (`Object::Tuple`, immutable, value-hashable), **not** `.ph`-over-`List`. | `(a,b)` desugars to a **`Tuple` construction send**; the native `Tuple` class is built by the **collection-runtime unit (U-COLLTYPES)**, not here — same defer-the-runtime pattern as the map (DEC-COLL-B). U-COLL ships the literal + disambiguation against a pending/construction target. |
| **DEC-COLL-B** ✅ **RESOLVED (B)** | **Map runtime target.** | Blocker cleared: `Object#hash`-as-floor is **ratified** ([ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md), landed U-CORE-1), the equality/mutability model is ruled ([decisions.md](../../../spec/v0.2/core/decisions.md) Q5), and native-arm `Map` is ratified ([ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1). | **(B)** — U-COLL ships §6 disambiguation + a precise "pending" diagnostic; the **collection-runtime unit (U-COLLTYPES)** wires `{k:v}` → the real native `Map` in one line after it lands. Do **not** build a throwaway `.ph` `Map` or open a competing `Map` in `core.ph`. |

## 9. Must-not-preclude check (task step 6)
- **Q7 destructuring — now the LIVE concurrent U14** (`let (a, b) = point`, `let [first, *rest] = list`):
  **not precluded; actively coordinated.** U-COLL desugars collection *expressions* (RHS, via
  `parse_primary`); U14 binds *patterns* on the **LHS** — a separate parse path U-COLL never enters. The
  §3.4 `parse_comma_exprs` helper keeps its element grammar (bare name, nested literal, reserved
  `*`-slot) pattern-compatible so U14 **shares** the scanner. **Boundary (per the task): U-COLL = value
  position, U14 = pattern position; do not absorb U14.** Recommend U-COLL lands first to seed the shared
  scanner.
- **Q6 set literal `#{1,2,3}`** (open, alternative to `Set(…)`): **not precluded.** U-COLL adds no set
  syntax and does not claim `#{`. A future `#{…}` is a **distinct lexer token** (`#`-symbol branch +
  `{`), resolved before `parse_primary` sees it, so it cannot collide with U-COLL's `{`/map work. Spec
  §4's `Set(…)`-as-a-send remains the shipping answer.
- **Spread `[*xs, y]` / `(*a, b)`** (spec §8, `Asterisk`): **deferred, not precluded.**
  `parse_comma_exprs` reserves a `*`-prefix slot per element; wiring it to a spread-send is additive
  when spread-at-call-site (`f(*args)`) is finalized. U-COLL ships no spread but leaves the §8 grammar
  hole open.
- **Range literal `1..5` / `1...5`**: **reserved-inactive with committed meaning**
  ([ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §3.3: `..` inclusive,
  `...` exclusive). U-COLL adds no `..` token; unrelated to `[`/`(`/`{`. Activation is a later slice.

## 10. Return contract (self-report; no reviewer)
Report: the parser arms added/extended + confirmation grouping (`(x)`) and all five §6 brace rows are
regression-pinned · the list desugaring chain (`List.new().add()…`) and confirmation `add` returns
`self` on HEAD · how DEC-COLL-A resolved and (if A) the `Tuple` `.ph` shape + that `.class` distinguishes
it from `List` · how DEC-COLL-B resolved and, if B, the exact "pending" diagnostic + the PENDING golden
that graduates when the map follow-on lands · confirmation **net floor delta = 0** (no ADR-0019
amendment) · the `collections` corpus label + MANIFEST count bump · whether U-COLL landed before/after
the concurrent U14 and whether `parse_comma_exprs` was shared · `verify.sh` + `cargo doc` tails · any new
`DEFERRED.md` entries (map lowering, spread-in-literal, range literal, `#{}` set).
