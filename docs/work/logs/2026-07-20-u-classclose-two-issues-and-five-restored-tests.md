# U-CLASSCLOSE: two spec bugs caught mid-implementation, five deleted goldens restored in-crate

- Date: 2026-07-20
- Commits: `14cdfb9` (U-CLASSNS follow-up fix), `c346200` (parser nested-class ban),
  `7c2cfab` (U-CLASSCLOSE landed), this session's follow-up (in-crate kernel-reopen tests)
- Realizes: [PDR-0001](../pdr/0001-classes-are-closed.md), as amended by
  [PDR-0002](../pdr/0002-class-declarations-join-the-binding-namespace.md), per
  `docs/forge/units/U-CLASSCLOSE/implementation-spec.md`
- Related: [U-CLASSNS implementation-spec.md](../forge/units/U-CLASSNS/implementation-spec.md) —
  the module-scoped `ClassKey` re-key this unit builds on

## 1. Two things the spec got wrong, found by actually running the tree

Both surfaced as real test failures, not review — the spec read as internally consistent until
the tree disagreed with it.

### 1.1 The redefinition predicate: `field_layouts` alone, not `classes && field_layouts`

§2.1's table says a `classes[(module,name)]` hit **and** a `field_layouts[(module,name)]` hit
together mean "already defined." Implemented literally, it let a same-file duplicate through:

```phalcom
class Point { x => 1 }
class Point { y => 2 }
```

compiled with no error. `field_layouts` is written at **compile time**, synchronously, the moment
each class body is lowered. `classes` is written at **runtime**, only when
`Bytecode::Class`/`Bytecode::Constant` actually executes — and a whole compile unit lowers to one
closure before any of its bytecode runs. So at the second `class Point`'s compile time,
`field_layouts_hit` is already `true` but `classes_hit` is still `false`. Requiring both silently
admitted every same-unit duplicate — the table's own row 3 was unreachable for the case that
matters most.

Fix: `field_layouts_hit` alone (still exempted for a REPL cell — ruling 6, cells shadow). This is
exactly what the pre-U-CLASSCLOSE reopen guard already checked; the spec's re-derivation of the
predicate from "first principles" dropped the reason the old guard used that one field.

### 1.2 Reserved names: `add_class!`'s set, not "the whole core `classes` map"

§4 explicitly rejects `add_class!`'s set as the source ("invisible to the compiler") and instead
prescribes "the core-module keys present once `core.ph` has finished running" — i.e. every class
`core.ph` itself goes on to declare, not just the Rust-installed primitives.

That breaks real code the moment you run it:

```phalcom
class ArgumentError is Error {
  @constructor
  new(msg) { super.new(msg) }
}
```

is an ordinary, idiomatic pattern (`errors_throw_try_catch_finally.ph`, a real golden fixture,
not a contrived case) — `ArgumentError` is declared in `core.ph` (`extends Error {}`, a plain
`.ph` library class), not by `add_class!`. Under §4's literal instruction it becomes
`class.reserved_name`, which is wrong: `ArgumentError` carries no literal-bound `ClassId` the way
`List`/`Object`/`Number` do (nothing indexes it by name at a hot path), so redeclaring or
shadowing it from a non-core module is not the trap [PDR-0001 ruling
3](../pdr/0001-classes-are-closed.md) exists to close.

Fix: a new `VM::kernel_class_names: HashSet<Symbol>`, populated by `add_class!` itself as each
primitive installs (plus one explicit `insert` for `None`, which bypasses `add_class!` entirely
since its global binds the singleton value, not the class). This is PDR-0001 ruling 3's own
literal wording ("the name set is the one enumerated in `add_class!`") — the spec's §4 elaboration
overreached past the ruling it was supposed to implement.

Both fixes are in `phalcom-core/src/compiler/lib/class_decl.rs` and
`phalcom-core/src/vm/bootstrap.rs` (`7c2cfab`).

## 2. REPL shadowing needed its own carve-out, twice

`tests/repl_immutability.rs::class_shadows_across_repl_cells` — `class Foo {}` in one REPL cell,
`class Foo {}` again in a later cell — must succeed (ruling 6). The redefinition check
(`field_layouts_hit`, per §1.1's fix) and the stub-completion emit fork (`classes_hit` deciding
`Constant` vs `Class`) both had to gate on `self.unit_kind != UnitKind::Repl` independently; the
stub-completion fork already had this guard from before the unit, the redefinition check did not
and needed adding.

## 3. Two `invariants.rs` tests were exploiting the exact mechanism being removed

`subclass_field_offset_stability` / `subclass_static_field_offset_stability` called
`vm.create_class(module, "Subclass", Some(base_cls))` directly to pre-register a superclass link,
then compiled a `.ph` body for `Subclass` with **no `extends` clause at all** — relying on the old
"reopen keeps the Rust stub's established superclass" path to smuggle in the inheritance the
source never declared. `create_class` also already wires field layout from `vm.field_layouts` once
it exists (`vm/api.rs:70-76`), so the tests' own manual `field_slots`/`field_count` copy-in was
redundant scaffolding for the same removed mechanism. Rewritten to declare `extends Base` in the
`.ph` source directly and let the ordinary compile→run path (`Bytecode::Class` → `create_class`)
wire everything; assertions unchanged, same numbers.

## 4. Five goldens tested kernel-class reopening itself — deleted, then restored in-crate

Five `.ph` fixtures reopened a **kernel** class from user source specifically to flip an
override-epoch flag or bust an inline cache on it — a technique the whole unit removes by
definition:

| Fixture | Kernel class/selector | What it proved |
|---|---|---|
| `strings/print_number_reopen_agrees.ph` | `Number#toString` | leaf `toString` fast path falls back the instant the flag flips |
| `absence/absence_iftrue_nested_deopt_path.ph` | `Bool#and` (sacred) | nested-`ifTrue` deopt fallback computes the same answers as the fast path |
| `absence/absence_iftrue_some_lift_deopt_path.ph` | `Bool#and` (sacred) | the untaken-arm Some-lift survives through the deopt path |
| `control-flow/control_flow_inline_override_honored.ph` | `Block#whileTrue` (sacred) | an inlined `whileTrue` site honors a real override over its fast path |
| `absence/absence_combinators_route_through_match.ph` | `Option#match` | every `Option` combinator truly derives from `match`, not a variant tag |

First cut (`7c2cfab`) deleted all five and flagged the gap rather than claim done. This session
restored coverage in-crate (`phalcom-core/src/universe/mod.rs`'s new `#[cfg(test)] mod tests`,
five tests), following the same pattern the spec itself prescribes for
`ic_add_method_invalidates`/`ic_override_after_caching` (§11.2): drive the install path directly —
`ClassObject::add_method` + `VM::world_version` bump + `Universe::note_method_installed`, in that
exact order, mirroring `Bytecode::Method`'s own handler — against the real kernel `ClassId`s in
`vm.universe.classes`. This needs `world_version` (`pub(crate)`), so in-crate is the only place it
can live, same reasoning as the two `chunk.rs` tests.

One simplification the rewrite gets for free: the original fixtures had to reopen `Bool`/`Block`
and run in the **same program**, because the pristine flag has to flip *before* the guarded site
first executes. Driving the install from Rust means the flag is already flipped before the `.ph`
snippet is even compiled, so `GuardBool`/`GuardBlock` take the deopt branch on the very first
execution — no need to run the same closure twice or thread state across two program halves.

### 4.1 Known gap this does *not* close

The in-crate tests assert the mechanism (flag flips, deopt branch taken, override's value comes
through) using values fabricated in Rust (`Value::Bool(false)`, a sentinel `Value::Number(-999.0)`,
a native `toString` returning `"N"`). They do **not** exercise the compiler's own
`.ph`-syntax-to-override path — that half is now structurally untestable from surface Phalcom, by
design (PDR-0001). If the *install call site itself* (`Bytecode::Method`'s handler) ever stops
calling `add_method`/bumping `world_version`/calling `note_method_installed` in this exact order,
these tests would not catch it via a `.ph` regression — only via the two `chunk.rs` tests (which
exercise a **user** class through the real `.ph` compile path) or a future test that compiles a
class body directly into the core module. Recorded here rather than left implicit, per the house
"no silent caps" habit.

## 5. Verification

- `bash scripts/verify.sh` (build + full `cargo test --workspace` + clippy) green at each step.
- `cargo doc --workspace --no-deps` — 12 pre-existing warnings (verified by name against origin
  files before and after this session's changes: `opcode_stats`, `fiber_is_root`, `force_gc`,
  `semantic_tokens`, `Program`, `attach_attribute` `Result<_, ()>`, `AttributeRegistry` — none in
  any file this unit touched), zero new.
- The five in-crate replacements: `cargo test -p phalcom-core --lib universe::tests` — 5 passed.
- Negative control on §1.1 (the `field_layouts`-alone fix): reverting to `classes_hit &&
  field_layouts_hit` and re-running `classes` reproduces the silent same-unit-duplicate pass —
  confirms the test is non-vacuous.
