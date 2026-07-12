# Pending-Test Retirement Map (U-CORE-0)

> **Status:** Normative. Maps every **currently-pending** language-corpus fixture
> (`phalcom-core/tests/lang/<label>/pending/`) to the *real* blocker that keeps
> it red and the unit that will flip it green. It is the executable side of the
> [`catalog-delta.md`](./catalog-delta.md): the catalog says what protocol is
> missing; this says which **test** proves it and who retires it.

> **Baseline:** HEAD `0f84232`. Folds in **U10** (non-local return, fully
> landed — `blocks_non_local_return`/`blocks_non_local_return_bare` retired),
> **U-LEX** (comments, numeric separators, newline suppression, string
> interpolation shipped as **`\(expr)`** per [ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md)
> — not the `"{expr}"` form this doc previously cited — plus a coverage
> fixture for `?.`/`??`, already built in U6; U-LEX explicitly did **not**
> add selector literals `#…`, collection literals `[…]`/`{…}`/`(…)`, `::`,
> or spread-call syntax — DEFERRED.md #6/#21/#28), **U-STD** (`Option`
> combinators `map`/`flatMap`/`filter`/`ifSome`/`unwrapOr` and `List`
> combinators `map`/`filter`/`reduce`/`includes`/`isEmpty`, all pure `.ph`),
> and **U11** (`True`/`False` singleton subclasses of `Bool`, ADR-0004 — no
> effect on this map, no new pending fixtures). The pending set is still a
> **live moving target** — re-run the audit in §1.2 before citing a specific
> fixture's status.

## 1. What "retirement" means

### 1.1 The mechanism

A pending fixture is a `<name>.ph` + `<name>.expected` pair under a label's
`pending/` subdirectory. `check_pending(label)`
([`tests/support/mod.rs`](../../../../phalcom-core/tests/support/mod.rs) L176) runs
each one and asserts **exact stdout** against `.expected` — but the wrapping test
fn in [`tests/lang.rs`](../../../../phalcom-core/tests/lang.rs) is `#[ignore]`, so it
stays out of the green run. `.expected` pins the **intended spec output**, not
today's behavior ([`tests/lang/MANIFEST.md`](../../../../phalcom-core/tests/lang/MANIFEST.md)).

**Retiring** a fixture is a `git mv <label>/pending/<name>.* <label>/` — it moves
into the active `check_pass` lane and starts guarding the now-shipped feature.
Historically every unit that closed a spec gap did this: U3 (`arithmetic/`), U4
(`blocks_literal_call`), U5 (`booleans/`, `control_flow_if_else`), U6
(`absence/` +7), U7 (`classes/` construct set), U10 (`blocks_non_local_return`,
in flight). **Retirement is a manual move, not automatic** — so a fixture can sit
in `pending/` already passing (see category A).

### 1.2 How this map was built (re-runnable audit)

Every row below is empirical, not inferred from the catalog. The audit runs each
pending fixture through the built binary exactly as the harness does (trailing
newline trimmed) and records pass/fail:

```sh
cargo build -p phalcom-core --bin phalcom
for ph in phalcom-core/tests/lang/*/pending/*.ph; do
  exp="${ph%.ph}.expected"
  a=$(./target/debug/phalcom "$ph" 2>&1); a="${a%$'\n'}"; e=$(cat "$exp"); e="${e%$'\n'}"
  [ "$a" = "$e" ] && echo "GREEN  ${ph#*pending/}" || echo "red    ${ph#*pending/}"
done
```

The `2>&1` folds stderr in so an error diagnostic never masquerades as a pass; a
real GREEN means the fixture is promotable **today**.

> **Why the empirical run matters — the drift it caught.** The pre-U8 assumption
> was "U8/U9 retired the `dispatch`/`messages`/`variadics` pending set." The audit
> shows that is **false**: U8 shipped the `perform`/`respondsTo` *primitives*, but
> `dispatch_perform`/`dispatch_responds_to` invoke them through `#+(_)`
> selector-literal and `[4]` list-literal **syntax the lexer still rejects** — so
> they fail at "Invalid token", not at dispatch. Only `dispatch_does_not_understand`
> (plain call syntax) actually went green. This is the central subtlety of §3:
> **the unit that adds a capability is often not the unit that flips its test.**

## 2. The retirement categories

| Cat | Meaning | Owner shape |
|---|---|---|
| **A** | **Already green** — capability shipped; the fixture is passing but was never `git mv`'d out of `pending/`. Retirement is pure housekeeping. | a *landed* unit |
| **B** | Blocked on a **U-CORE-N** protocol (this roadmap). | U-CORE-1…6 |
| **C** | Blocked on **U-LEX surface syntax** (selector literals `#…`, collection literals `[…]`/`{…}`/`(…)`, spread `*args`, string interpolation, `::`, `@attr`, `:` inheritance, `for`). The underlying capability may already exist. | U-LEX (+ maybe a co-unit) |
| **D** | Blocked on **U-STD** (`.ph` base-surface growth: `List#reduce`, `Option#unwrapOr`, …). | U-STD |
| **E** | **Out of core scope** — concurrency, `System` services, standalone collection classes. | deferred units |

### 2.1 Category A — promote now (housekeeping, no code)

These pass against `.expected` **today**. The capability that flips them already
landed; they simply need the `git mv`. A U-CORE unit does **not** need to claim
these — but whoever next touches the relevant label should promote them so the
green lane actually guards the feature.

| Fixture | Passing because of | Move to |
|---|---|---|
| `absence/pending/absence_comparison_le_ge` | `<=`/`>=` as sends (U5) | `absence/` |
| `absence/pending/absence_comparison_lt_gt` | `<`/`>` as sends (U5) | `absence/` |
| `bindings/pending/binding_var_reassignment` | `var` reassignment (U6, ADR-0014) | `bindings/` |
| `control-flow/pending/control_flow_while` | `while` lowering (U4/U5) | `control-flow/` |
| `control-flow/pending/control_flow_sacred_selector_inliner` | `{…}.whileTrue{…}` sacred inliner (U5, ADR-0018) | `control-flow/` |
| `dispatch/pending/dispatch_does_not_understand` | overridable dNU hook (U8) | `dispatch/` |
| `control-flow/pending/control_flow_iftrue_iffalse` | `Option#unwrapOr` (U-STD, `core.ph:105`) | `control-flow/` |
| `dispatch/pending/dispatch_rest_param` | `List#reduce` (U-STD, `core.ph:169`) | `dispatch/` |

> `blocks/pending/blocks_non_local_return` (U10) and
> `blocks/pending/blocks_argument_to_method`, `lexical/pending/lexical_numeric_separator`,
> `lexical/pending/lexical_string_interpolation` (U-LEX/U-STD) have already
> been `git mv`'d out of `pending/` — confirmed absent from the current tree,
> removed from this table and from §3 below.
>
> These eight are **not** a U-CORE deliverable; they are a standing cleanup the
> forge spine left behind. Listed here so the count in §3 is honest about what is
> genuinely *unbuilt* versus merely *un-moved*.

## 3. The full per-fixture map

Blocker = the *first* thing that makes the fixture red today (from the §1.2 run).
Owner = the unit(s) that must land for it to go green; **bold** = the load-bearing
one, others are co-requisites. "syntax" always means **U-LEX**.

| Fixture (`…/pending/`) | Cat | Real blocker (observed) | Flips when |
|---|:--:|---|---|
| `metaclass/metaclass_is_a` | B | ~~`3.isA(Number)` → dNU `isA(_:)`~~ **retired** — `isA(_)` landed in U-CORE-1; fixture uses plain syntax (no U-LEX gate) and has been `git mv`'d to the active lane (`status: PASS`) | **U-CORE-1 — landed/green** |
| `absence/absence_option_none` | B | `print(None)` → `<None instance>` (needs `None#toString`) | **U-CORE-4** |
| `absence/absence_var_defaults_to_none` | B | `var x` → `<None instance>` (needs `None#toString`) | **U-CORE-4** |
| `bindings/binding_var_uninitialized` | B | same as above (`None#toString`) | **U-CORE-4** |
| `absence/absence_option_some` | B+C | `Some(42)` → `<class Some> dnu 'call(_:)'` (needs `Some(_)` sugar **and** `Some#toString`) | **U-CORE-4** + U-LEX |
| `dispatch/dispatch_perform` | C | `#+(_)` selector literal + `[4]` list literal (perform primitive exists) | **U-LEX** |
| `dispatch/dispatch_responds_to` | C | `#+(_)` selector literal (respondsTo primitive exists) | **U-LEX** |
| `dispatch/dispatch_spread_call` | C | `[1,2,3]` list literal + `f(*args)` spread-call syntax | **U-LEX** (STATE.md: spread-call future) |
| `functions/functions_method_bind` | C+B | `#greet(_)` literal + `methodFor(_)`/`Method#bind(_)` | U-LEX + **U-CORE-1/3** |
| `functions/functions_method_for_invoke_on` | C+B | `#+(_)` literal + `[4]` literal + `methodFor(_)`/`Method#invokeOn(_,_)` | U-LEX + **U-CORE-3** |
| `messages/messages_family_reference` | C+B | `p::move` family reference (`::`) | U-LEX + **U-CORE-3** (Family) |
| `messages/messages_selector_symbol_literal` | C | `#move(_,to,duration)` selector literal + its `toString` | **U-LEX** (+ U-CORE-4 Symbol `toString`) |
| `lexical/lexical_list_literal` | C | `[…]` literal lowering | **U-LEX** |
| `lexical/lexical_tuple_literal` | C+E | `(a,b)` literal + `Tuple` class | U-LEX + collections |
| `lexical/lexical_map_literal` | C+E | `{k:v}` literal + `Map` class | U-LEX + collections |
| `lexical/lexical_set_literal` | C+E | `Set(…)`/`#{…}` + `Set` class | U-LEX + collections |
| `lexical/literals_escape` | C | string escape sequences (empirically still red at HEAD `0f84232`: `\n` prints literally) | **U-LEX** (escape sequences not yet built, despite interpolation D4 landing) |
| `control-flow/control_flow_for` | C | `for (x in …)` + list literal | **U-LEX** |
| `classes/class_inheritance_super` | C | `class Dog : Animal` + `super.speak()` | **inheritance unit** (not U-CORE) |
| `classes/class_attribute_construct_get_set` | C | `@construct` / `@get` attribute annotations | **attribute unit** (not U-CORE) |
| `errors/errors_throw_try_catch_finally` | B+C | `throw`/`try`/`catch`/`finally` (ADR-0008) | **U-CORE-6** + error-syntax |
| `errors/errors_result_bridge` | B | `Result`/`Ok`/`Err` + `.attempt()`/`.unwrap()` bridge (ADR-0008) | **U-CORE-6** (Result unit) |
| `concurrency/concurrency_fiber_yield_resume` | E | `Fiber` class (concurrency.md) | concurrency unit |
| `concurrency/concurrency_future_async_await` | E | `Future` class (concurrency.md) | concurrency unit |
| `system/system_args` | E | `System.args` | system unit |
| `system/system_clock` | E | `System.clock` | system unit |

## 4. Per-U-CORE-unit "flips" rollup

This is the slice each U-CORE implementation spec must cite under **"`_pending`
tests this unit flips."** The recurring lesson: **most reflection tests are
gated behind U-LEX**, so a U-CORE unit that adds a reflective primitive should
name the fixture as *"unblocks, flips once U-LEX lands `#…`"* rather than claim
an immediate green.

| Unit | Flips **directly** (plain syntax, goes green on its own) | **Unblocks** (capability lands here; fixture waits on U-LEX/U-STD co-land) |
|---|---|---|
| **U-CORE-1** kernel reflection | `metaclass/metaclass_is_a` (via `isA(_)`) | `functions/functions_method_bind` (via `methodFor`/`Method`), partial |
| **U-CORE-2** absence + Boolean | — (its combinators already landed `0da64d6`; no pending fixture is gated on the residue) | — |
| **U-CORE-3** callables/Block | — | `functions/functions_method_for_invoke_on`, `functions/functions_method_bind`, `messages/messages_family_reference` (all need U-LEX `#…`/`::`) |
| **U-CORE-4** value classes | `absence/absence_option_none`, `absence/absence_var_defaults_to_none`, `bindings/binding_var_uninitialized` (all via `None#toString`) | `absence/absence_option_some` (needs U-LEX `Some(_)` sugar too); `messages/messages_selector_symbol_literal` (Symbol `toString`, needs U-LEX literal) |
| **U-CORE-5** collection contract | — (contract, not classes — ADR-0020; flips nothing directly) | enables `Map`/`Set` fixtures once those classes land |
| **U-CORE-6** errors | — | `errors/errors_result_bridge`, `errors/errors_throw_try_catch_finally` (both need error-syntax sugar) |

> **U-STD update:** `List#reduce` and `Option#unwrapOr` have since landed
> (`core.ph:169`, `core.ph:105`), directly flipping `dispatch/dispatch_rest_param`
> and `control-flow/control_flow_iftrue_iffalse` — both moved to §2.1
> (category A, housekeeping-only) and out of this table.

**Reading:** U-CORE-1 and U-CORE-4 are the only units with a *direct* flip; U-CORE-4
has the most (three `None#toString` fixtures). Everything reflective is
double-gated on U-LEX. A U-CORE unit's spec should therefore treat "flips a
pending test" as a *joint* claim with U-LEX where the fixture uses `#…`/`[…]`/`::`
syntax, and set its own acceptance bar on a **new** unit-local fixture written in
already-supported syntax (e.g. `3.isA(Number)`, `None.toString`) rather than
waiting on the lexer.

## 5. Traceability

| Claim | Source |
|---|---|
| Harness semantics (exact stdout, `#[ignore]`) | `tests/support/mod.rs` L137–178; `tests/lang.rs`; `tests/lang/MANIFEST.md` |
| Per-fixture pass/fail | the §1.2 audit against `./target/debug/phalcom` at HEAD `0f84232` |
| U10 retiring `blocks_non_local_return` | working-tree `git status` (in-flight); STATE.md "U10 — LANDED" |
| `reduce` blocked on U-STD (not U10) | STATE.md DEFERRED #25 |
| spread-call syntax is future | STATE.md (U9 section); PHASE2-INDEX.md soft-flag |
| `None#toString` / per-type `toString` owner | [`catalog-delta.md`](./catalog-delta.md) §4.4 → U-CORE-4 |
| `Option#unwrapOr` re-scoped to U-STD | [`catalog-delta.md`](./catalog-delta.md) §2.2 |
| `isA`/`hash` on `Object` | [`object-model.md`](../object-model.md) §8 → U-CORE-1 |
| error mechanism | [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md) → U-CORE-6 |
