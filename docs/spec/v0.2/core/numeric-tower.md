# Specification — The Numeric Tower (`Int` / `Float`)

> **Status:** **Normative + implementation-ready.** This is one document: the design
> contract *and* the implementation detail. There is no companion unit plan.
>
> **Realizes:** [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
> (Accepted 2026-07-12; **zero implementation** — verified, §1). ADR-0024 rules the tower,
> representation, `==`/`hash` canonicalization, `/`, `~/`, and promotion. **This spec does not
> redesign any of them**; it specifies how they land.
>
> **Supersedes in part:** [ADR-0005](../../../adr/retired/0005-number-as-flat-f64.md) — its
> `f64` survives as `Float`'s representation only.
>
> **Rulings:** [PDR-0012](../../decisions/0012-numeric-tower-implementation-and-floor-amendment.md)
> — **Accepted** (ratified 2026-07-20). It carries this spec's 24 implementation rulings and the
> ADR-0019 floor amendment. [PDR-0025](../../decisions/0025-numeric-tower-residue-rulings.md)
> resolves its former primitive blockers; this spec is implementation-ready.
>
> **Floor impact:** **REQUIRES an [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)
> amendment**, carried and ratified by PDR-0012 ruling 20 (`docs/adr/` is frozen). The per-class
> split is `137 → 153` installed bindings (§6). It composes with PDR-0011's `Bytes` amendment;
> recompute against the live census at implementation rather than trusting prose totals.
>
> **Ordering constraint:** must land **before** any arithmetic fast path is burned into
> bytecode (ADR-0024 §Context). That window is still open — verified, §11.
>
> **New runtime dependency:** `num-bigint`, pinned in the root `[workspace.dependencies]` (§3.3).
>
> **Depends on:** nothing unlanded. **Conflicts with:** in-flight U-CLASSNS work re-keying
> `vm.classes` to `ClassKey` (§12.0).
>
> **Owner:** unassigned. **Baseline:** HEAD `8b4465c`, 2026-07-20.

---

## 0. Scope

**In scope.** Representation, the class tower, literal typing, arithmetic and comparison,
division (`/` and `~/`), promotion, `==`/`hash` canonicalization, and every site in the tree
that constructs or destructures a numeric `Value`.

**Out of scope — explicitly deferred to a follow-on unit.** ADR-0024's Consequences promise
that "`list.at(i)`, `size`, arity, and loop counters take `Int`; a `Float` index is a type
error at the boundary". **Call-site tightening is NOT in this spec.** It touches every
collection primitive and a large fixture corpus, and it is separable: the tower is correct
without it.

The line between them is sharp and worth stating, because it is easy to blur:

| | In scope | Out of scope |
|---|---|---|
| `size` **returns** `Int` rather than `Float` | ✅ — that is choosing the right constructor (§9.3) | |
| `list.at(2.0)` **is rejected** | | ❌ — that is tightening an accept-predicate |

`expect_index` (`phalcom-core/src/primitive/list.rs:29`) therefore keeps accepting an integral
`Float` after this spec lands. It gets *narrower*, not stricter: it must accept `Value::Int`
too, and the follow-on unit is what removes the `Float` arm. **Q-NUM-5** asks whether to leave
a marker.

---

## 1. Verified tree baseline

Every row below was checked against the tree at `8b4465c` during authoring. Rows marked
*assumed* were **not** checked — treat them as claims to re-verify, not facts.

| Claim | Evidence | Status |
|---|---|---|
| `Value::Number(f64)` is the single numeric arm | [`value/mod.rs:45`](../../../../phalcom-core/src/value/mod.rs) | ✅ checked |
| `core.ph` still has a flat, empty `class Number {}` | [`core.ph:82`](../../../../phalcom-core/core/core.ph) — note ADR STATUS row 0005 cites `core.ph:75`, which is stale | ✅ checked |
| `~/` does not exist — no token, no lexer entry, no parser arm | `phalcom-ast/src/token.rs` has `Slash`/`Percent`/`SlashEqual` and **no** `Tilde` of any kind | ✅ checked |
| `//` is unavailable — it is the line comment | ADR-0024 §5 states it; not re-checked in the lexer | ⚠️ assumed |
| Number literals are decimal only: digits, `_` separators between digits, optional `.` fraction. **No hex, no exponent.** | [`lexer.rs:208-218`](../../../../phalcom-ast/src/lexer.rs) `scan_number` — the entire grammar is `scan_digits [ '.' scan_digits ]` | ✅ checked |
| **The lexer destroys the int/float discriminant.** `Token::Number(f64)` — `1` and `1.0` both lex to `Token::Number(1.0)` | [`lexer.rs:217`](../../../../phalcom-ast/src/lexer.rs); `Token::Number(1.0)` appears in the lexer's own tests at `lexer.rs:810` | ✅ checked — **see §4.1, this is the spec's most consequential finding** |
| `Number` carries **12 instance + 2 static** floor bindings (not "14 instance + 2 statics") | [`universe/primitives.rs:102-123`](../../../../phalcom-core/src/universe/primitives.rs); census rows [`invariants.rs:759-773`](../../../../phalcom-core/tests/invariants.rs) | ✅ checked |
| `primitive/number.rs` anticipates the split in a doc comment that constrains dispatch identity | [`primitive/number.rs:83-85`](../../../../phalcom-core/src/primitive/number.rs) — `toString` is bound "on `Number` … not a concrete f64 path, so a future `Integer`/`Float` split … can refine this per-subclass without breaking dispatch identity" | ✅ checked |
| Installed floor is **137**, not 125 | Sum of the per-amendment constants at [`invariants.rs:642-723`](../../../../phalcom-core/tests/invariants.rs); `floor_census_matches_installed_bindings` **runs green** in a clean detached worktree at `8b4465c` | ✅ checked — **the handoff brief's "125" is stale by 12 (`NEW_FIBER`)** |
| `floor-census.md` itself says **136** | [`floor-census.md:36,843`](./floor-census.md) — stale by 1 (`Fiber#isRoot`, `NEW_FIBER` was raised 11→12) | ✅ checked — cite the test, never the doc |
| No arithmetic opcodes; arithmetic is an ordinary send | [`inliner.rs:5-8`](../../../../phalcom-core/src/compiler/inliner.rs) — sacred set is `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_:ifFalse:)`, `and(_)`, `or(_)`, `whileTrue(_)`. Control flow only. | ✅ checked |
| `f64` appears in **15** files under `phalcom-core/src` + `phalcom-ast/src` | §9.1 | ✅ checked |
| `num-bigint` would be phalcom-core's *first* runtime dependency | **FALSE.** `phalcom-core/Cargo.toml` already carries `indexmap`, `slotmap`, `rand`, `lazy_static`, `clap`, `anyhow`, `color-print`, `thiserror`, `tracing` | ✅ checked — §3.3 |
| Adding a `Value` arm is a compile-time-exhaustive change | Rust semantics; every `match` on `Value` without a `_` arm breaks | ⚠️ assumed (not enumerated) |

---

## 2. The tower

### 2.1 Shape

```
        Object
          │
       Number            (abstract — no instances, no floor bindings)
        ╱   ╲
      Int   Float
```

`Number` remains a real class row so `isA(Number)` stays meaningful and so
`core.ph`-derived numeric protocol has somewhere to live. It gains **zero** floor bindings
(§6) and **zero** instances.

### 2.2 Interaction with PDR-0001 (classes are closed)

[PDR-0001](../../decisions/0001-classes-are-closed.md) is Accepted and **unimplemented**
(verified: `docs/decisions/STATUS.md` row 0065, `❌ ruled 2026-07-19, unimplemented`). Two of
its rulings touch this spec:

- **0065 ruling 3 — kernel class names are reserved**, and the reserved set is "the name set
  … enumerated in `add_class!`" ([`vm/bootstrap.rs:183-194+`](../../../../phalcom-core/src/vm/bootstrap.rs)).
  `Int` and `Float` get `add_class!` rows (§12.2), so they become reserved **automatically**
  once 0065 lands. **No conflict.** The spec's only obligation is to add the rows.
- **0065 ruling 4 — stub completion is gated on the core module.** `Int`/`Float` are
  Rust-installed kernel classes whose `.ph` protocol is layered on in `core.ph`, i.e. exactly
  the stub-completion path (`classes` hit + `field_layouts` miss,
  [`class_decl.rs:332`](../../../../phalcom-core/src/compiler/lib/class_decl.rs) per 0065's own
  table). This is the *sanctioned* branch. **No conflict.**

**Ordering is free in both directions.** If 0065 lands first, this spec adds two `add_class!`
rows and rides the existing gate. If this spec lands first, 0065 inherits two more reserved
names. Neither blocks the other.

### 2.3 `Number` is abstract — but nothing enforces abstractness

There is no `@abstract` mechanism in the tree that this spec verified. `Number` is abstract by
**construction**: no literal produces one, no primitive returns one, and its statics are
removed (§10). A determined user could still reach it. **Q-NUM-6** asks whether that is
acceptable or whether the allocator must raise.

---

## 3. Representation

### 3.1 `Value` arms

`Value::Number(f64)` is **replaced**, not supplemented:

```rust
pub enum Value {
    Nil,
    Bool(bool),
    /// An exact integer on the small path — the common case, no heap.
    /// Large values live in [`Object::LargeInt`]; both surface as `Int`.
    Int(i64),
    /// An IEEE-754 double ([ADR-0005]'s representation, retained for `Float`).
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}
```

Removing the `Number` arm (rather than keeping it as an alias) is deliberate: it makes the
compiler enumerate every site for you. Every non-wildcard `match` on `Value` in the workspace
fails to build until it is handled. **Do not add a `_ =>` arm to silence this** — the semantic
arms (`class`, `value_eq`, `Hash`, `type_name`, `to_string`, `as_obj`, `to_context`) must each
be updated deliberately, and a wildcard hides exactly the misrouting U12's old plan warned
about (`U12/plan.md:95-98`).

### 3.2 The large path

`LargeInt` is a heap object under [ADR-0009](../../../adr/accepted/0009-handle-arena-heap.md):

```rust
/// An arbitrary-precision integer — the large tier of `Int`
/// ([ADR-0024](…/0024-…) §2). Invisible at the surface: a `LargeInt`'s
/// class is `Int`, exactly as a `Value::Int`'s is.
LargeInt(BigInt),
```

Added to `Object` ([`heap/object.rs:24`](../../../../phalcom-core/src/heap/object.rs)).
Boxing: `BigInt` is 32 bytes on 64-bit (`Vec<u64>` + sign) — under `StringObject`'s footprint
and far under `ClassObject`'s boxed 280 B, so it goes **inline**, not `Box`ed. The `Object`
doc comments at `heap/object.rs:28-45` state the slot-size rule; follow it and say so.

`Value::class` must resolve `Object::LargeInt` to `int_class`, so
`(2 ** 200).class == Int` holds without a surface tier leak (ADR-0024 §2).

`trace_object` needs a `LargeInt` arm that traces **nothing** — a `BigInt` holds no `ObjRef`.
It still needs the arm (the match is exhaustive) and it must be an explicit no-op with a
comment, not a fallthrough.

### 3.3 `num-bigint` — where it is pinned

**Bind, do not hand-roll** (user ruling). Add to the **root** `Cargo.toml`
`[workspace.dependencies]`:

```toml
num-bigint = "0.4"
num-traits = "0.2"     # only if the conversion traits are actually used — see below
```

and in `phalcom-core/Cargo.toml`:

```toml
num-bigint = { workspace = true }
```

Two corrections to the brief this spec was written from:

1. **This is not phalcom-core's first runtime dependency.** It already depends on `indexmap`,
   `slotmap`, `rand`, `lazy_static`, `clap`, `anyhow`, `color-print`, `thiserror`, and
   `tracing` (verified, `phalcom-core/Cargo.toml`). The claim to make instead is narrower and
   still true: it is the first dependency that participates in **user-visible value
   semantics**, so it is the first whose behaviour is part of the language contract rather
   than an implementation convenience.
2. **The workspace-pinning convention is not currently uniform.** `indexmap`, `tracing`,
   `tracing-subscriber` use `{ workspace = true }`; `thiserror`, `anyhow`, `lazy_static`,
   `rand`, `clap`, `color-print`, `slotmap` are pinned literally in the crate (and `thiserror`
   is pinned in *both* places at the same version). The user's ruling stands regardless — pin
   `num-bigint` in the workspace — but do **not** describe that as "matching the existing
   convention", because there isn't one. **Q-NUM-7** asks whether to normalize.

**Do not take `num-traits` on speculation.** Take it only if `ToPrimitive`/`FromPrimitive` are
genuinely used; `num-bigint` re-exports enough for `TryFrom<&BigInt> for i64` in most flows.
An unused dependency in the first value-semantics dependency is a bad precedent.

### 3.4 Promotion and demotion

ADR-0024 §2: `checked_*` on the `i64` path, box to `LargeInt` on overflow, demote when a
result fits back into `i64`. Two rules the ADR leaves implicit and this spec fixes:

1. **Demotion is mandatory, not optional.** A `LargeInt` whose value fits in `i64` must be
   demoted to `Value::Int` *before it is returned to the caller*. Otherwise two equal `Int`s
   have different representations, and every `==`/`hash`/`match` path needs a
   cross-representation case for a situation that should be unreachable.
2. **`Value::Int` is the canonical form.** State this as an invariant:
   `∀ v : Value. v = Obj(LargeInt(b)) ⇒ b ∉ [i64::MIN, i64::MAX]`. Assert it in a debug
   assertion inside the single normalization helper, and test it (§13, T-REP-3).

Both live in **one** function — `normalize(BigInt) -> Value` — which is the only constructor
of a `LargeInt` value anywhere in the tree.

---

## 4. Literals

### 4.1 The lexer discards the discriminant — this is the pivot of the whole unit

U12's old plan (`U12/plan.md:38-44`) posed exactly the right precondition: *"does the
lexer/`Token` preserve enough of a numeric literal's text for the compiler to classify `42`
vs `42.0` without a lexer change?"* It could not answer it and scheduled around both branches.

**The answer is no.** [`lexer.rs:217`](../../../../phalcom-ast/src/lexer.rs):

```rust
Ok(Token::Number(cleaned.parse::<f64>()?))
```

`1` and `1.0` are the same token. The information is destroyed before the parser runs, let
alone the compiler. **A `phalcom-ast` change is mandatory**, which pulls the front-end crate
into the write set — the branch U12's plan hoped to avoid ("Prefer to avoid", `U12/plan.md:56`).

### 4.2 Token shape

Split the token, do not tag it:

```rust
/// An integer literal — no fraction part in the source text.
Int(i64),
/// A floating-point literal — the source text carried a fraction part.
Float(f64),
```

rather than `Number(f64, bool)`. Reasons: the payload types genuinely differ (an `i64` literal
must not round-trip through `f64` — that is the exactness the ADR exists to deliver); and the
parser's `match` arms then state which literal kind they accept.

`scan_number` becomes:

```rust
fn scan_number(&mut self) -> Result<Token, LexicalError> {
    let start = self.pos;
    self.scan_digits()?;
    let is_float = self.peek_at(0) == Some(b'.')
        && matches!(self.peek_at(1), Some(b'0'..=b'9'));
    if is_float { self.pos += 1; self.scan_digits()?; }
    let cleaned = self.input[start..self.pos].replace('_', "");
    if is_float {
        Ok(Token::Float(cleaned.parse::<f64>()?))
    } else {
        // An integer literal too large for i64 is still exact — it becomes a
        // LargeInt constant, never a lossy f64.
        Ok(cleaned.parse::<i64>().map_or_else(
            |_| Token::BigInt(cleaned.clone()),
            Token::Int,
        ))
    }
}
```

**The oversized-integer-literal case is real and is not in ADR-0024.** `Int` is unbounded, so
`99999999999999999999` is a legal literal. It cannot be an `i64` and it must not become an
`f64`. Options:

- **(a)** a third token `Token::BigInt(String)` carrying the digits, parsed to `BigInt` in the
  compiler (shown above);
- **(b)** `Token::Int(BigInt)` uniformly, which makes `phalcom-ast` depend on `num-bigint`;
- **(c)** reject oversized literals as a lex error.

**Recommend (a).** It keeps `phalcom-ast` dependency-free, keeps the common path an `i64`, and
puts the one `BigInt` parse where the heap already is. [PDR-0026](../../decisions/0026-numeric-literals.md)
ratifies that choice and extends the payload to `{ digits, radix }` so binary, octal, and
hexadecimal literals remain exact.

### 4.3 Parser and AST

`ast.rs` currently carries `f64` for numeric literals (verified: `f64` appears in
`phalcom-ast/src/ast.rs`). The literal node splits to match the token. The `.` disambiguation
against `DotDot` (`lexer.rs:827` test `number_dot_dot_is_not_a_decimal`) is untouched — the
`is_float` predicate is the same predicate that already governed the branch.

### 4.4 Compiler constants

Numeric literals become `Value::Int` / `Value::Float` constants. A `Token::BigInt` mints a
heap `LargeInt` at **compile time**, which means the constant pool holds an `ObjRef`. Check
that the constant pool's GC rooting covers compile-time-minted objects — the spec did **not**
verify this. **Q-NUM-3.**

### 4.5 Hex and exponent: leave them out of this unit

The grammar has neither (§1). ADR-0024 does not ask for them. Adding `0x…` and `1e5` here
would mean deciding whether `1e5` is an `Int` or a `Float` — a real question (Python says
`float`, Ruby says `Float`, Dart says `double`) that has nothing to do with the tower.

**Ruling: this spec adds neither.** The tower does not need them, and bundling them puts a
new syntax decision inside a representation change. They are now specified separately by
[PDR-0026](../../decisions/0026-numeric-literals.md); implement them as the follow-on literal unit.

---

## 5. `~/` — the full pipeline

Nothing exists (§1). Five layers, in order:

1. **Token.** `Token::TildeSlash`, `phalcom-ast/src/token.rs`, alongside `Slash` (`token.rs:231`).
2. **Lexer.** On `~`, peek for `/`. A bare `~` is not currently a token at all, so a lone `~`
   must produce `LexicalError::InvalidToken` — verify no other construct claims `~` first.
3. **Parser — precedence.** `binary_op_precedence` (`parser.rs:2824+`) gives `Slash` and
   `Percent` precedence **6** (`parser.rs:2841-2842`). `~/` is the same *kind* of operation and
   takes **precedence 6**, left-associative, so `a ~/ b * c` groups as `(a ~/ b) * c` like
   `a / b * c` does. Any other choice makes `~/` read differently from the `/` it replaces.
4. **AST.** `BinaryOp::IntegerDivide` in the enum at `ast.rs:792-806`.
5. **Selector encoding.** `~/` must be spellable as a method name so it is overridable and
   `super`-callable. Two sites, and they must stay in lockstep — the parser's own doc comment
   at `parser.rs:2171-2175` records that they diverged once already (U-ERR-FIX SUPER-OP-SYNTAX):
   - `Parser::parse_method_name` (`parser.rs:1441`, next to `Token::Slash => "/"`),
   - the `super.<operator>` arm (`parser.rs:2210`).

   The selector string is `"~/"`, giving the encoded selector `~/(_)` — one positional
   argument, matching `/(_)`. `make_signature` is label-free and handles it (floor-census §8
   notes the only two hand-rolled exceptions are `ifTrue(_:ifFalse:)` and `match(some:none:)`,
   both label-carrying; `~/` is neither).

**`~/=` compound assignment: no.** `SlashEqual`/`PercentEqual` exist (`token.rs:170-172`).
`~/=` is not requested by ADR-0024 and adds a token, a lexer path, and a desugar for no stated
need. Out of scope; say so rather than leaving it ambiguous.

---

## 6. The floor: per-class split and the ADR-0019 amendment

### 6.1 Current state — cite the test, never the doc

The installed floor is **137**. This is the sum of the per-amendment constants at
[`invariants.rs:642-723`](../../../../phalcom-core/tests/invariants.rs), and
`floor_census_matches_installed_bindings` was run green in a clean detached worktree at
`8b4465c` during authoring.

Two records disagree and both are wrong: the handoff brief this spec was written from says
**125** (stale by the whole `NEW_FIBER` amendment), and
[`floor-census.md:36,843`](./floor-census.md) says **136** (stale by one — `Fiber#isRoot`
raised `NEW_FIBER` from 11 to 12). `floor-census.md:843` states the rule that resolves this:
*"The test is the source of record for the count; do not restate the number here."* Correcting
`floor-census.md`'s two stale figures is part of this unit's write set (§12).

### 6.2 `Number`'s current 14 bindings

From [`universe/primitives.rs:102-123`](../../../../phalcom-core/src/universe/primitives.rs),
census rows [`invariants.rs:759-773`](../../../../phalcom-core/tests/invariants.rs):

**Instance (12):** `+(_)` `-(_)` `*(_)` `/(_)` `%(_)` `<(_)` `<=(_)` `>(_)` `>=(_)`
`negated()` `hash` `toString`
**Static (2):** `new()` `new(_)`

### 6.3 The post-split enumeration

Per the user's ruling, **every numeric primitive is split per-class**; neither `Int` nor
`Float` inherits an arithmetic implementation from `Number`.

| Selector | `Number` | `Int` | `Float` | Note |
|---|---|---|---|---|
| `+(_)` | — | ✅ | ✅ | |
| `-(_)` | — | ✅ | ✅ | |
| `*(_)` | — | ✅ | ✅ | |
| `/(_)` | — | ✅ | ✅ | always returns `Float` (ADR-0024 §4) |
| `%(_)` | — | ✅ | ✅ | `Int%Int` exact; `Float` keeps `fmod` |
| `~/(_)` | — | ✅ | ✅ | PDR-0025: total over the tower; always returns `Int` |
| `<(_)` | — | ✅ | ✅ | |
| `<=(_)` | — | ✅ | ✅ | |
| `>(_)` | — | ✅ | ✅ | |
| `>=(_)` | — | ✅ | ✅ | |
| `negated()` | — | ✅ | ✅ | `Int` must handle `i64::MIN` → `LargeInt` |
| `hash` | — | ✅ | ✅ | canonicalizing (§8) |
| `toString` | — | ✅ | ✅ | `primitive/number.rs:83-85` pre-authorized exactly this refinement |
| `new()` static | ✗ removed | ✅ | ✅ | §10 |
| `new(_)` static | ✗ removed | ✅ | ✅ | §10 |

**`Number` keeps zero floor bindings.** It stays in the census and in `core_class_rows`
precisely so that an accidental future primitive on `Number` shows up as a **red test** rather
than a silent re-flattening of the tower.

### 6.4 Census arithmetic

```
  137   current installed (invariants.rs:642-723, verified green at 8b4465c)
 − 14   Number's bindings, all removed
 + 26   Int (13 instance) + Float (13 instance)
 +  4   Int.new()/new(_) + Float.new()/new(_)
 ────
  153   post-split
```

The numeric floor goes **14 → 30**: it more than doubles, which is precisely why this needs
ratification rather than a census bump.

**Implementation note — the assertion cannot express a removal.** Every constant in
`floor_census_matches_installed_bindings` is a `usize` *addition*
(`assert_eq!(expected.len(), BASELINE + NEW + …)`, `invariants.rs:952+`). This unit is the
first to **remove** bindings. Do not fold the removal into a smaller positive constant — that
erases the fact that `Number` was emptied. Instead:

```rust
/// U-NUMTOWER (ADR-0024, ADR-0019 amendment): `Number` is emptied — all 14 of
/// its bindings move to the concrete subclasses (12 instance + 2 static).
const NUMERIC_SPLIT_REMOVED: usize = 14;
/// U-NUMTOWER: `Int` and `Float` each take the full arithmetic/comparison
/// surface plus `~/`, `hash`, `toString`, and both `new` arities (15 each).
const NUMERIC_SPLIT_ADDED: usize = 30;
```

and extend the assertion with `+ NUMERIC_SPLIT_ADDED - NUMERIC_SPLIT_REMOVED`, keeping the
addition ahead of the subtraction so the `usize` expression never goes negative mid-evaluation.

### 6.5 The ratified amendment

ADR-0019 is a deliberate **one-way ratchet**
([`0019…md:26`](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)), and
`floor-census.md` §7.1 fixes the protocol: *"open an ADR amending 0019, justify why the
capability fails the §1 derivability test, then update this file in the same change."*

**Deliverable for the user: ratify or refuse.** The justification, stated plainly so it can be
argued with:

- The split adds **no new capability**. Every one of the 30 bindings is an existing blessed
  capability, re-homed. `Int#+` does what `Number#+` did, on one of the two representations.
- The count doubles for a **representation** reason, not a derivability reason. Under the
  user's per-class ruling there is no shared implementation to inherit, so one binding becomes
  two.
- The honest alternative — a shared `Number#+` that dispatches internally on the arm — is
  **the thing the user ruled against**, and §16 records why that ruling is defensible.

So the amendment is not "we found 16 new things the VM must do". It is "the ruled class
structure costs 16 bindings to express". A reviewer could reasonably respond that this is the
ratchet working as designed and the count is the price of the ruling.

**Also required in the same change** (`floor-census.md` §7.2's coverage caveat, which was
written *because* `Fiber` slipped this exact gap): `Int` and `Float` must gain rows in
`core_class_rows` ([`invariants.rs:48`](../../../../phalcom-core/tests/invariants.rs), `29`
rows → `31`). **A kernel class absent from that list is unfrozen in fact, whatever any ADR
says.** The array's length is in its type signature, so this is a compile error if forgotten —
which is the one piece of luck in this design.

---

## 7. Arithmetic and promotion

### 7.1 One coercion helper

U12's plan set this guardrail and it is the right one (`U12/plan.md:30-33`): **one** promotion
table, in one function, not `match (Int, Float)` scattered across primitives.

```rust
/// The numeric promotion lattice (ADR-0024 §6): `Int ⊕ Int → Int` (exact,
/// auto-promoting); any `Float` operand contaminates to `Float`.
enum Promoted {
    /// Both operands are exact integers — the result is exact.
    Ints(BigInt, BigInt),
    /// At least one operand is a `Float`; the `Int` side has been converted,
    /// which may lose precision (ADR-0024 §6 — the user opted in).
    Floats(f64, f64),
}
```

**The small-int fast path must not route through `BigInt`.** `Promoted` above is the general
lattice; the actual binop is:

```rust
match (lhs, rhs) {
    (Int(a), Int(b)) => /* checked_* on i64; overflow → BigInt → normalize */,
    _                => /* the general lattice above */,
}
```

`i64 + i64` allocating a `BigInt` per addition would make the common case slower than the
`f64` it replaces, which would be a self-inflicted regression on a codebase that measures its
arithmetic (`docs/forge/perf-log/SCOREBOARD.md`).

### 7.2 Per-operator

| Op | `Int ⊕ Int` | mixed | `Float ⊕ Float` |
|---|---|---|---|
| `+` `-` `*` | `checked_*`, overflow → `LargeInt`, then demote | → `Float` | `f64` |
| `/` | **always `Float`** — convert both, then `f64` divide | → `Float` | `f64` |
| `~/` | floor division, exact `Int` | exact `Int` (PDR-0025) | exact `Int` (PDR-0025) |
| `%` | exact, **sign follows the divisor** so it agrees with `~/`'s floor | → `Float` | `fmod`, sign follows dividend (unchanged) |
| `< <= > >=` | exact | compare by mathematical value | `f64` |
| `negated` | `i64::MIN` overflows `checked_neg` → `LargeInt` | — | `f64` |

**`%` changes meaning on the `Int` path, and ADR-0024 forces it.** §5 says `~/` is floor
"(rounds toward −∞, so its sign agrees with `%`)" and gives `-7 ~/ 2 == -4`. For the identity
`a == (a ~/ b) * b + (a % b)` to hold, `-7 % 2` must be `1`, not `-1`. Rust's `i64 %` truncates
and yields `-1`. **The `Int#%` primitive must use `rem_euclid`-style floored modulo, not Rust's
`%`.** Today's `number_mod` (`primitive/number.rs:148-152`) is Rust `%` on `f64` and its doc
comment says "sign follows the dividend" — correct for `Float`, wrong for `Int` after this
lands. `Float#%` keeps `fmod`. This divergence is a direct consequence of ADR-0024 §5's floor
ruling; it is not a new decision, but it **is** a behaviour change for negative operands and
needs a golden test (§13, T-DIV-4).

### 7.3 `/` by zero

`Int / Int` promotes to `Float` first, so `1 / 0` stays `inf` and `0 / 0` stays `NaN` —
IEEE-754, unchanged, and the existing arithmetic goldens keep passing
(`primitive/number.rs:106-109` records that they pin this).

`~/` has **no such escape**: it returns an exact `Int` and there is no integer infinity.
`7 ~/ 0` must **raise**. `RuntimeError::ZeroDivision` reportedly already exists and is unused
(episodic memory 7467, 2026-07-19) — **not verified by this spec**; if it does exist, use it,
and note that `raise`-arm coverage is a known-weak area
(memory `gc-ensure-temp-root-uaf`). `Int#%` by zero raises identically.

---

## 8. `==` and `hash`

### 8.1 `==` compares by mathematical value

ADR-0024 §3. `Value::value_eq` (`value/mod.rs:227`) gains cross-arm cases:

```rust
(Value::Int(a),   Value::Int(b))   => a == b,
(Value::Float(a), Value::Float(b)) => a == b,
(Value::Int(a),   Value::Float(b)) |
(Value::Float(b), Value::Int(a))   => int_eq_float(*a, *b),
```

plus `LargeInt`-vs-everything through the same predicate. `int_eq_float` must **not** be
`a as f64 == b`: for `|a| > 2^53` that cast rounds, and two distinct `Int`s would compare
equal to one `Float`. Compare exactly — reject non-finite and non-integral `b` up front, then
compare in the integer domain.

**This is not a wildcard-able change.** `value_eq`'s final `_ => false` arm (`value/mod.rs:258`)
will happily swallow a missing `Int`/`Float` case and make `1 == 1.0` silently `false`. The
new arms must be written explicitly and tested from both directions (§13, T-EQ-2).

### 8.2 `hash` canonicalizes — and today's canonicalization is about to become wrong

ADR-0024 §3: an integral `Float` hashes as the equal `Int`, so `2.hash == 2.0.hash`.

The existing `number_hash` (`primitive/number.rs:60-73`) already does a version of this, but
its guard is:

```rust
} else if n.is_finite() && n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
    (n as i64) as u64          // integral and in safe-integer range → hash as that integer
} else {
    n.to_bits()                // otherwise → canonical bits
}
```

**Under the split that upper bound becomes a live defect.** `2.0f64.powi(100)` is finite,
integral, and exactly equal to the `Int` `2^100` — so after §8.1 they are `==`. But it exceeds
`2^53`, falls to the `to_bits()` branch, and hashes differently from the `Int`. That breaks
`a == b ⇒ a.hash == b.hash` (R-INV-1.3, ADR-0023) and silently desyncs `Map`/`Set` keys.

Today the bound is harmless — there is nothing for a large integral float to *be* equal to.
The split is what makes it wrong. **`Float#hash` must, for any finite integral `f64` of any
magnitude, hash as the exact integer it equals** (via `BigInt` above `2^53`). This is a
correction the ADR does not spell out, derived from its own §3 rule.

### 8.3 `send_hash` — a hard-fail site

[`primitive/mod.rs:338-348`](../../../../phalcom-core/src/primitive/mod.rs):

```rust
match vm.send_dynamic(value, sym, &[])? {
    Value::Number(n) => Ok(n as i64),
    other => Err(RuntimeError::Type { expected: "Number", found: other.type_name() }.into()),
}
```

Map and Set hash their keys by **sending the Phalcom `hash` selector** through this function
(`primitive/map.rs:55`, and the module doc at `map.rs:12-15` calls it "the re-entrant key-hash
crux"). After the split, `hash` returns a `Value::Int` — which this `match` **rejects**. Every
`Map` and `Set` insertion would fail at runtime. It must accept `Int` (and decide whether a
`Float` return from a user-defined `hash` is an error or is truncated — recommend: accept
integral `Float` for compatibility, reject non-integral).

This is the single most likely site to be missed, because it is a `match` on a `Value` *return*
rather than a `match` on `Value` in a signature — the compiler's exhaustiveness check catches
it only because `Value::Number` ceases to exist (§3.1). **This is the concrete payoff for
removing the arm instead of aliasing it.**

### 8.4 `hash_code`'s 53-bit mask can be widened — but need not be

[`primitive/mod.rs:150-155`](../../../../phalcom-core/src/primitive/mod.rs) masks every digest
to 53 bits with the stated reason: *"so the `as f64` cast is lossless and round-trips"*. Once
`hash` returns `Value::Int(i64)` that constraint is gone.

**Recommend leaving the mask in place for this unit.** Widening changes every hash value in
the system for zero correctness gain (digest stability is only required within a run —
R-INV-1.4), and it would land inside a change that is already touching `Map`/`Set` key paths.
Note it as available; do not spend it here. The doc comment's *reason* must be updated in
either case, or it becomes a lie about why the mask exists.

### 8.5 `impl Hash for Value` (the Rust-side hash)

`value/mod.rs:296-306` hashes `Value::Number` via `hash_f64` (bits). It must gain `Int`/`Float`
arms that agree with each other for equal values, for the same reason as §8.2.

**Under-claim, deliberately:** a grep for `HashMap<Value`, `HashSet<Value`, and `ConstKey`
across `phalcom-core/src` found **no consumer**. Either this impl is currently dead or it is
reached by a route this spec did not find. **Do not conclude it is dead and skip it** — that
is precisely the "audit the predicate, not the conclusion" failure that produced this repo's
temp-root use-after-free. Keep it coherent; if the implementer establishes it really is
unreachable, that is a separate finding worth writing down.

---

## 9. Collateral sites

### 9.1 The 15 `f64` files

`phalcom-ast/`: `ast.rs`, `lexer.rs`, `token.rs` — literal representation (§4).

`phalcom-core/src/`:

| File | What it does with `f64` | Action |
|---|---|---|
| `value/mod.rs` | the arm, `class`, `value_eq`, `Hash`, `hash_f64`, `type_name`, `to_context`, `as_obj` | §3, §8 — the core of the change |
| `value/render.rs` | `n.to_string()` at `:23`, `:136`; `Display`/`Debug` at `:170`, `:182`; **a `Value::Number` fast-path guard at `:110`** | §9.2 |
| `primitive/number.rs` | every primitive | splits into `primitive/int.rs` + `primitive/float.rs` (§12.3) |
| `primitive/mod.rs` | `hash_code` mask (`:150-155`), `send_hash` (`:338`) | §8.3, §8.4 |
| `compiler/lib/patterns.rs` | mints `Value::Number(i as f64)` at `:125`, `:141`, `:206`, `:229` | §9.4 |
| `primitive/list.rs` | `expect_index` (`:29-38`) | §9.3 |
| `primitive/string.rs` | `byteCount_` `:78`, `byteAt_` `:107` | §9.3 |
| `primitive/block.rs` | `arity` `:62`, `:65`, `:67`, `:68` | §9.3 |
| `primitive/map.rs` | `size_` `:40` | §9.3 |
| `primitive/set.rs` | `size_` `:37` | §9.3 |
| `primitive/tuple.rs` | `size_` `:66` | §9.3 |
| `heap/mod.rs` | GC growth factors `:102`, `:112`, `:115`, `:329`, `:335` | **untouched** — internal `f64` arithmetic, never a `Value` |
| `opcode_stats.rs` | instrumentation | **untouched** |

### 9.2 The `toString` pristine flags

`Universe::note_method_installed` ([`universe/mod.rs:196-198`](../../../../phalcom-core/src/universe/mod.rs))
keys an override-epoch flag on `number_class`:

```rust
if class_id == self.classes.number_class && Self::LEAF_TOSTRING_SELECTORS.contains(&name) {
    self.number_tostring_pristine = false;
}
```

paired with a fast path in `value/render.rs:110` (`if let Value::Number(_) = self`). The flag
lets `System.print` on a number skip a dispatch while no user override exists.

`number_tostring_pristine` becomes **two** flags — `int_tostring_pristine` and
`float_tostring_pristine` — because `Int` and `Float` are independently overridable. Collapsing
them into one flag keyed on `Number` would mean overriding `Float#toString` silently deopts
`Int`'s fast path (merely slow, acceptable) — but worse, a flag keyed on `Number` would **never
be flipped** by an override on `Int`, since `note_method_installed` compares `ClassId`s for
equality and would never match. **That direction is a correctness bug, not a slowdown**: a user
override of `Int#toString` would be ignored by `System.print`. Split the flag.

Also snapshot both in the post-`core.ph` reset (`universe/mod.rs:207+`).

### 9.3 Count/size/arity primitives mint `Int`

Every site listed in §9.1 that constructs `Value::Number(x as f64)` from a `usize`/`u8` count
switches to `Value::Int(x as i64)`. These are counts; they are exactly what ADR-0024's
Consequences mean by "`size` … takes `Int`".

`expect_index` (`list.rs:29-38`) is the *reading* side and, per §0, **stays permissive**: it
accepts `Value::Int` (the new normal path) **and** an integral, non-negative, finite
`Value::Float` (today's behaviour). Its doc comment must say the `Float` arm is transitional
and name the follow-on unit, or the next reader will assume the permissiveness is intended.

### 9.4 Compiler-minted pattern constants

`compiler/lib/patterns.rs` mints numeric constants for destructuring — element indices
(`:125`), an expected element count (`:141`, `:206`), and the literal `1` (`:229`). All four are
**integers by nature** and become `Value::Int`. If they stayed `Float`, destructuring would
compare a `Float` index against an `Int` size and — under §8.1's value equality — still work,
which is exactly why this could be missed. It would then quietly block the follow-on
tightening unit.

### 9.5 `expect_value!`

[`error.rs:185-196`](../../../../phalcom-core/src/error.rs) has a `Number` arm matching
`Value::Number(n) => *n`. It needs `Int` and `Float` arms.

Incidental observation, **not this unit's to fix**: the macro's `String` arm (`error.rs:173-175`)
matches `Value::String(s)`, and `Value` has no `String` variant (`value/mod.rs:37-51`). That
arm is unreachable. Flag it; do not fix it here — it is unrelated debt and belongs in its own
change.

### 9.6 Adopted debt from U12's plan

`U12/plan.md:61-66` adopted one incidental bug: the string-parse-failure arm of the numeric
coercion error hardcodes `found: "value"` instead of `arg.type_name()`. **Still present** —
`primitive/number.rs:34`, with the `TODO` intact. It sits in `number_class_new`, which §10
re-homes, so fix it in passing and pin a negative test asserting the message names the real
type.

---

## 10. Disposition of `Number.new`

`Number` becomes abstract with no instances, so `Number.class::new()` and `new(_)`
(`primitive/number.rs:22-45`, bound at `universe/primitives.rs:122-123`) cannot survive as-is.

`number_class_new` is a **coercion constructor**: number → identity, `Bool` → `1`/`0`, string →
parsed. String→number parsing is genuinely not `.ph`-derivable, so the capability is
floor-worthy; only its home is in question.

**Ruling: re-home, do not delete.** `Int.new(_)` and `Float.new(_)`, each with its own
implementation and its own `new()` zero-arity default (`Int.new()` → `0`, `Float.new()` →
`0.0`). Rationale: deleting the capability would push string parsing onto `String#toInt` /
`String#toFloat`, which is *also* +2 floor, lands in U-STRING's territory, and changes a public
spelling for no gain. Re-homing keeps the count honest at +4 and keeps the surface familiar.

Semantics per class:

| Argument | `Int.new(_)` | `Float.new(_)` |
|---|---|---|
| `Int` | identity | widen to `f64` |
| `Float` | raises — no implicit narrowing (PDR-0025) | identity |
| `Bool` | raises — coercion removed (PDR-0025) | raises — coercion removed (PDR-0025) |
| numeric string | exact parse, `LargeInt` if oversized | `parse::<f64>()` |
| non-numeric string | `TypeConversion` (with `arg.type_name()` — §9.6) | same |

`Float.new(2.7)` is identity. **`Int.new(2.7)` raises**: PDR-0025 rejects every `Float`
argument, including integral values, so construction is never a value-dependent narrowing door.
The explicit narrowing selectors belong to the Float-protocol follow-on.

---

## 11. Ordering constraint — the arithmetic fast path

ADR-0024 §Context gates this decision on landing **before the arithmetic inliner hardens**:
*"both the two-representation arithmetic and the division result-type rule must be in place
before fast paths are burned into bytecode."*

**Verified: the window is open, and it is open for a reason different from the one the brief
gave.** The chain:

- The sacred-selector inliner is **control-flow only** — `ifTrue(_)`, `ifFalse(_)`,
  `ifTrue(_:ifFalse:)`, `and(_)`, `or(_)`, `whileTrue(_)`
  ([`inliner.rs:5-8`](../../../../phalcom-core/src/compiler/inliner.rs)). No arithmetic.
- **DEC-PRIM-B** (the guarded arithmetic fast path) was raised in `U-PRIM-ABI/plan.md:90` and
  resolved as *deferred*: the on-stack arg buffer alone won ~41% on `arith_send`, so the
  superinstruction "and the full ~70-primitive window-status ABI migration were deliberately
  not pursued — deferred to `U-IC`" (`docs/forge/UNITS-TRACKER.md:129`).
- **But U-IC then dropped it.** `U-IC/implementation-spec.md:16` records Change 3
  (superinstructions) as **DROPPED**, on the ground that the Wren technique targets a `u8`
  bytestream and Phalcom's `Bytecode` is a `Copy` enum with inline operands — "it is a no-op
  here".

So the arithmetic fast path is not merely unbuilt; **it currently has no owning unit at all**.
`SCOREBOARD.md:438` still lists `vm::send::call_method` (4.8%) as `open — DEC-PRIM-B`. The
window is open because the work fell between two units, not because anyone is holding it.

**Two things follow, and both are this spec's obligation to state:**

1. **This spec must land before any arithmetic fast path is built.** After the split, such a
   path must guard on `Int(i64)` with the overflow check *as* its deopt edge (ADR-0024
   §Consequences), and `/` needs no result-type guard because it is unconditionally `Float`.
   Building the fast path against a flat `f64` first would have to be torn out.
2. **Whoever eventually owns DEC-PRIM-B must record the constraint in their plan.** Since
   U-IC dropped it, there is no plan currently carrying it, which is the failure mode: an
   unowned constraint is invisible. **Action: add a line to `docs/forge/units/U-IC/plan.md`
   noting that Change 3 was dropped and that any successor arithmetic fast path is gated on
   this spec.** That edit is in this unit's write set (§12) precisely so the constraint has a
   home before it is needed.

---

## 12. Implementation order

### 12.0 Concurrency hazard — read first

`main` has **live concurrent sessions** (memory: `phalcom-concurrent-session-hazards`). At
authoring time the working tree did **not compile** — a session mid-flight on U-CLASSNS was
re-keying `vm.classes`/`sealed_classes`/`field_layouts` from `Symbol` to `ClassKey`, producing
14 type errors across `vm/dispatch.rs`, `vm/bootstrap.rs`, and `compiler/lib/class_decl.rs`.
That work directly touches `add_class!` (§12.2) and `sealed_classes`, both of which this spec
edits.

**Do not start until `vm/bootstrap.rs` and `compiler/lib/class_decl.rs` are green on `main`.**
Stage narrow explicit paths; never `git add -a`/`-A`; never `git checkout -b`; verify with
`git diff --cached --stat` before every commit.

### 12.1 Phases

Each phase ends at a compiling, testable tree. Commit per green checkpoint, not as one
end-of-unit batch (memory: `commit-frequently`).

| # | Phase | Contents | Gate |
|---|---|---|---|
| 1 | **Representation** | `Value::Int`/`Float`, `Object::LargeInt`, `normalize`, `num-bigint` pin, every exhaustive `match` | builds; existing tests green with `Float` behaving as today's `Number` |
| 2 | **Tower** | `int_class`/`float_class` in `CoreClasses`, `add_class!` rows, `core.ph` `class Int < Number` / `class Float < Number`, `core_class_rows` 29→31 | `verify_invariants` green; `1.class == Int` |
| 3 | **Literals** | `Token::Int`/`Float`/`BigInt`, `scan_number`, AST, parser, compiler constants, `patterns.rs` | `1.class != 1.0.class` |
| 4 | **Primitives** | `primitive/int.rs` + `primitive/float.rs`, the coercion helper, `Number` emptied, `Number.new` re-homed | census test green at the new count |
| 5 | **`~/`** | token, lexer, precedence 6, `BinaryOp::IntegerDivide`, both selector-name sites | `-7 ~/ 2 == -4` |
| 6 | **Equality & hash** | `value_eq` cross-arms, `Int#hash`/`Float#hash`, the `2^53` fix, `send_hash`, `impl Hash for Value` | `1 == 1.0`; `2.hash == 2.0.hash`; Map/Set coherence |
| 7 | **Docs & status** | ADR-0019 amendment, `floor-census.md` (incl. the two stale counts, §6.1), `core-classes.md`, `README.md` baseline pin, `docs/adr/STATUS.md` row 0024 `❌ → ✅`, U-IC plan note (§11) | — |

**Phase 7 is not optional and is not deferrable.** Flipping ADR-0024's shipped status means
editing `docs/adr/STATUS.md` in the **same commit** (memory: `adr-status-two-way-sync`).
STATUS.md's own rule 4 says the same thing, and names ADR-0024 as the cautionary example.

### 12.2 Write set

| Path | Why | Contention |
|---|---|---|
| `phalcom-ast/src/{token,lexer,ast,parser}.rs` | literals, `~/` | front-end crate |
| `phalcom-core/src/value/{mod,render}.rs` | arms, class, eq, hash, render | **high** |
| `phalcom-core/src/heap/object.rs` + tracer | `LargeInt` | |
| `phalcom-core/src/primitive/{int,float}.rs` | **new** | |
| `phalcom-core/src/primitive/number.rs` | **deleted** | |
| `phalcom-core/src/primitive/{mod,list,string,block,map,set,tuple}.rs` | `send_hash`, `hash_code`, count minting, `expect_index` | |
| `phalcom-core/src/universe/{mod,core_classes,primitives}.rs` | classes, pristine flags, bindings | **high** |
| `phalcom-core/src/vm/bootstrap.rs` | `add_class!` rows | **in-flight (§12.0)** |
| `phalcom-core/src/compiler/lib/patterns.rs` | pattern constants | |
| `phalcom-core/src/error.rs` | `expect_value!` arms | |
| `phalcom-core/core/core.ph` | `Number` abstract, `Int`/`Float` stubs | **additive only, never co-schedule** |
| `phalcom-core/tests/invariants.rs` | census constants, `core_class_rows` | **in-flight (§12.0)** |
| `Cargo.toml`, `phalcom-core/Cargo.toml` | `num-bigint` | |
| `docs/adr/…/00XX-amend-0019-numeric-split.md` | **new** | |
| `docs/adr/STATUS.md` | rows 0005, 0019, 0024 | |
| `docs/spec/v0.2/core/{floor-census,core-classes,README}.md` | census, class rows, baseline | |
| `docs/forge/units/U-IC/plan.md` | the §11 constraint note | |

### 12.3 `.ph` uses current keywords

U-BINDINGS has landed: `let` is mutable, `const` is immutable, **`var` is deleted**. Any `.ph`
in the implementation must use current keywords. **Verify against the lexer, not memory** —
this spec did not re-check the keyword table, and `value/mod.rs:270` still says "an
uninitialized `var` read" in a doc comment, which is either stale prose or evidence the
deletion was not swept. Either way, check before writing `.ph`.

---

## 13. Test strategy

The green gate must assert all of these. Positive fixtures are stdout-exact; error fixtures go
in the negative lane or the suite reddens (memory: `phalcom-golden-test-lanes`).

**Representation**
- T-REP-1 `1.class == Int`; `1.0.class == Float`; `Number` has no instances.
- T-REP-2 `(2 ** 200).class == Int` — no `LargeInt` surface leak.
- T-REP-3 **canonical form**: a computation whose result crosses into `LargeInt` and back
  yields a `Value::Int`, not a demotable `LargeInt` (§3.4 invariant). Rust-level test.
- T-REP-4 oversized *literal* `99999999999999999999` is exact, not an `f64` (§4.2).

**Arithmetic**
- T-ARI-1 `1 + 2 == 3` and is `Int`; `1 + 2.0` is `Float`.
- T-ARI-2 overflow: `i64::MAX + 1` is exact and `Int` — no trap, no wrap.
- T-ARI-3 `100.factorial` is exact (ADR-0024 §2's own example).
- T-ARI-4 `negated` on `i64::MIN` promotes rather than panicking.

**Division**
- T-DIV-1 `7 / 2 == 3.5`; `(6 / 2).class == Float` (ADR-0024 §4's "even division surprise").
- T-DIV-2 `7 ~/ 2 == 3`; `(7 ~/ 2).class == Int`.
- T-DIV-3 `-7 ~/ 2 == -4` — floor, not truncation.
- T-DIV-4 `-7 % 2 == 1` — floored modulo on `Int`, and the identity
  `a == (a ~/ b) * b + (a % b)` over a negative-operand table (§7.2). **`Float#%` unchanged**
  — pin that too, or the divergence looks like a regression.
- T-DIV-5 `7 ~/ 0` raises; `7 / 0` is `inf` (unchanged).
- T-DIV-6 `100.factorial ~/ 2` is exact (ADR-0024 §5).

**Equality & hash**
- T-EQ-1 `1 == 1.0`, and `1.0 == 1` — both directions (§8.1).
- T-EQ-2 `2.hash == 2.0.hash`.
- T-EQ-3 **the `2^53` case**: a large integral `Float` and its equal `Int` hash equal (§8.2).
  This is the test that would have caught the defect; without it the fix is unpinned.
- T-EQ-4 a `Map`/`Set` treats `1` and `1.0` as **one** key, inserted in both orders.
- T-EQ-5 `Map` insertion works at all after the split — the `send_hash` regression (§8.3).

**Floor & tower**
- T-FLR-1 `floor_census_matches_installed_bindings` green at the new count.
- T-FLR-2 `Int` and `Float` present in `core_class_rows`.
- T-FLR-3 `Number` has **zero** installed primitives — a positive assertion, so a future
  accidental binding reddens.
- T-FLR-4 `verify_invariants` green; `Int.superclass == Number`, `Number.superclass == Object`,
  `Int.class.superclass == Number.class` (the parallel rule).

**Non-regression**
- T-REG-1 the whole existing golden corpus, unchanged, **except** negative-operand `%` on
  integers (T-DIV-4) and any fixture printing an even division result. **Enumerate the
  changed goldens in the return report** — a silently-updated golden is how a semantic
  regression ships.

---

## 14. Rust documentation

Non-negotiable ([`docs/rust-documentation-guidelines.md`](../../../rust-documentation-guidelines.md)):
`//!` on every new/touched module (`primitive/int.rs`, `primitive/float.rs`), `///` on every
public item **including enum variants and struct fields** — `Value::Int`, `Value::Float`,
`Object::LargeInt`, `Token::Int`/`Float`/`BigInt`, `BinaryOp::IntegerDivide`,
`CoreClasses::int_class`/`float_class`, both pristine flags, `normalize`, the coercion helper,
and all 30 primitives. `cargo doc --workspace --no-deps` adds no warnings. Undocumented public
API is an incomplete change.

Docs that must be **corrected**, not merely added:
- `value/mod.rs`'s module `//!` names `Value::Number` as an immediate arm (`:5`).
- `primitive/mod.rs:150-155`'s `hash_code` explains the 53-bit mask by the `as f64` round-trip
  (§8.4) — that reason expires here.
- `primitive/number.rs:106-109`'s `/` doc cites ADR-0005's flat `Number`.
- `primitive/list.rs:24-28`'s `expect_index` doc (§9.3).

---

## 15. Consequences

- **Integers are never silently wrong** — the entire point (ADR-0024 §Consequences).
- **`6 / 2` is `3.0`, not `3`.** Deliberate; `~/` is the tool when an `Int` is wanted.
- **Negative-operand `%` changes on the `Int` path** (§7.2) — the least-obvious user-visible
  consequence, forced by ADR-0024 §5's floor ruling, not chosen here.
- **The numeric floor doubles**, 14 → 30 (§6). Requires ratification.
- **`phalcom-ast` enters the write set.** The lexer change is mandatory (§4.1), so this is no
  longer a compiler-side unit as U12's plan hoped.
- **Arithmetic gets slower before it gets faster.** Two arms and a promotion lattice replace
  one `f64` op, on a hot path with no fast path yet (§11). Expect a measurable arithmetic
  regression on `SCOREBOARD`'s `arith_send` and **measure it deliberately** — an unexplained
  regression discovered later costs more than a predicted one recorded now.
- **`Number` becomes an empty class.** Its census row stays as a tripwire (§6.3).
- **The follow-on tightening unit is now cheap.** `size` already returns `Int`, indices are
  already `Int` — that unit only has to remove `expect_index`'s `Float` arm and fix fixtures.

---

## 16. Alternatives rejected

ADR-0024's own Alternatives (f64-backed tag, trap-on-overflow `i64`, wraparound, surface
`SmallInteger`/`LargeInteger`, flooring `/`, truncating `~/`) are **settled and not
relitigated here**. New to this spec:

- **Shared `Number` implementation with per-arm dispatch inside.** Would keep the floor at
  ~15 and avoid the ADR-0019 amendment entirely. **Rejected by user ruling.** The ruling is
  defensible on merit: a shared `Number#+` is a lie about the tower — `Int` and `Float` have
  genuinely different arithmetic (exact/promoting vs IEEE-754), and hiding that behind one
  binding means the class a user overrides is not the class that implements the behaviour.
  Recorded here so the amendment request (§6.5) can be argued on the real trade.
- **Keep `Value::Number` as a third arm during migration.** Rejected: the exhaustiveness break
  is the migration's best tool, and §8.3 shows a real hard-fail site that only surfaces
  because the arm disappears.
- **`Token::Number(f64, is_float: bool)`.** Rejected: the `i64` payload must not round-trip
  through `f64` (§4.2).
- **Adding hex and exponent literals in this unit.** Rejected: an independent syntax decision
  (§4.5); cheaper after this lands, not before.
- **`~/=` compound assignment.** Rejected: unrequested, adds a token and a desugar (§5).
- **Deleting `Number.new` outright.** Rejected: pushes string parsing to `String#toInt`, which
  is also +2 floor and lands in another unit's territory (§10).
- **Widening `hash_code`'s 53-bit mask in this unit.** Rejected as scope: available, no
  correctness gain, changes every digest inside a change already touching key paths (§8.4).

---

## 17. Open questions — for the user

Numbered rather than guessed, per house rule.

| # | Question | Recommendation |
|---|---|---|
| **Q-NUM-1** | ~~Is `~/` defined on `Float`?~~ | **Resolved — PDR-0025:** it is total over the tower, returns exact `Int`, and raises when no exact result exists. |
| **Q-NUM-2** | ~~What payload carries oversized integer literals?~~ | **Resolved — PDR-0026:** a dependency-free `{ digits, radix }` token payload; compiler parses to `BigInt`. |
| **Q-NUM-3** | Does the constant pool's GC rooting cover a compile-time-minted `LargeInt` `ObjRef`? **Not verified.** | Verify before Phase 4; if not, root it. |
| **Q-NUM-4** | ~~Ratify the ADR-0019 amendment?~~ | **Resolved — PDR-0012 accepted:** 137 → 153 was ratified; recompute on the live implementation baseline. |
| **Q-NUM-5** | Should `expect_index`'s transitional `Float` arm carry a machine-checkable marker (a `#[deprecated]`-style tripwire) rather than only a doc comment, so the follow-on unit cannot be forgotten? | A doc comment naming the follow-on unit is probably enough; a tripwire is cheap insurance. |
| **Q-NUM-6** | `Number` is abstract by construction only — no mechanism enforces it (§2.3). Should its allocator raise? | Low priority; the surface is unreachable through literals or statics once §10 lands. |
| **Q-NUM-7** | `phalcom-core`'s dependency pinning is split between workspace and crate-literal, with `thiserror` pinned in both (§3.3). Normalize in this unit, separately, or not at all? | Separately — unrelated to the tower. |
| **Q-NUM-8** | ~~What does `Int.new(2.7)` do?~~ | **Resolved — PDR-0025:** raises for every `Float`; explicit narrowing is a Float-protocol selector. |

---

## 18. Verified vs assumed

**Checked against the tree at `8b4465c`** (with `file:line`, §1): the `Value` arm; `core.ph`'s
flat `Number`; the absence of `~/` in the token set; the number-literal grammar and the
**loss of the int/float discriminant at lex time**; `Number`'s 12+2 bindings; the
`primitive/number.rs:83` dispatch-identity note; the floor count **137** (test run green in a
clean detached worktree — not read off a doc); `floor-census.md`'s stale 136 and the brief's
stale 125; the inliner's control-flow-only sacred set; the 15 `f64` files and each site listed
in §9.1; `send_hash`'s `Value::Number`-only match; `hash_code`'s 53-bit mask and its stated
reason; `Map`/`Set` hashing keys via the `hash` **send**, not the Rust `Hash` impl; the
`number_tostring_pristine` flag and its `ClassId` equality test; `patterns.rs`'s four minted
constants; `expect_index`; `expect_value!`'s `Number` arm (and its unreachable `String` arm);
U12's adopted-debt `TODO` still present at `number.rs:34`; PDR-0001's rulings 3 and 4 and
its unimplemented status; the `add_class!` name set; `core_class_rows`'s 29 rows;
`phalcom-core`'s existing dependency list; DEC-PRIM-B's deferral to U-IC **and U-IC's
subsequent drop of it**; that the working tree did not compile at authoring time and why.

**Assumed — not verified:**
- That `//` is genuinely unavailable as the line-comment token (taken from ADR-0024 §5; not
  re-checked in the lexer).
- That `RuntimeError::ZeroDivision` exists and is unused (episodic memory only, §7.3).
- That no construct other than a would-be `~/` claims `~` in the lexer (§5 step 2).
- The exact set of `match`es on `Value` that break — asserted to be compile-time-exhaustive
  (Rust semantics), not enumerated.
- The current keyword table after U-BINDINGS (§12.3) — a stale-looking `var` reference survives
  in a doc comment at `value/mod.rs:270`.
- Whether `impl Hash for Value` has any live consumer. A grep for `HashMap<Value`,
  `HashSet<Value`, and `ConstKey` in `phalcom-core/src` found **none**. **This is reported as a
  failed search, not as a conclusion** (§8.5).
- `BigInt`'s in-memory size (stated as 32 B from general knowledge, not measured) — the inline
  vs `Box` decision in §3.2 should be confirmed against `heap/object.rs`'s slot-size rule
  before it is relied on.
- Every performance claim in §15 is a prediction. Numbers come from
  `docs/forge/perf-log/SCOREBOARD.md` only, never from memory (memory: `perf-baseline-measured`).
