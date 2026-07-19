# Deferred: class-sealing follow-ups (unowned)

Split out of the [decision 0065](../decisions/0065-classes-are-closed.md) design work
(2026-07-19). These are items that work surfaced which are **not** in scope for either of
0065's two units and currently have no owning unit.

Items 1–2 are **performance** items *unlocked by* sealing rather than required by it. Neither
is measured. Do not fold either into a correctness unit — a dispatch-path change inside a
correctness gate makes a red tree ambiguous, and this repo's standing rule is that
`docs/forge/perf-log/SCOREBOARD.md` is the only source for a perf claim.

Item 3 is the **`None` stub-completion / sealing-unification** thread: the part of
`docs/forge/DEFERRED.md` #17 and #35 that 0065 deliberately stopped short of.

All file:line references verified 2026-07-19 against `main`.

---

## 1. `SuperSend` does a runtime name lookup whose justification is now dead

**Unlocked by:** 0065 (and ADR-0026 Axis 2, which already sealed reparenting).

Every super send probes a `HashMap` by class *name* to find the defining class, then reads its
superclass:

- `phalcom-core/src/vm/dispatch.rs:885`

```rust
let parent = self.classes.get(&defining_sym).and_then(|&c| self.heap.class(c).superclass);
```

`defining_sym` is a bare class-name symbol baked into the chunk constant by the compiler.

The site's own comment (`dispatch.rs:879-884`) justifies this:

> DEC-INH-B: the defining class is resolved by name and its superclass read at dispatch, so a
> future `superclass=` mutation stays correct.

**That justification is void.** ADR-0026 Axis 2 already sealed reparenting and 0065 strengthens
it; there is no `superclass=` and there will not be one without a migrating `reshape` primitive
(ADR-0026's own forward note). So the lookup buys a guarantee nothing needs, and pays a hash
probe per super send for it.

**Why it is not trivial.** The compiler cannot simply bake the `ClassId`: a whole compile unit
lowers to one closure *before* any `Bytecode::Class` executes, so a same-unit user class has no
`ClassId` at compile time. That is precisely why the site is name-baked today. Three routes:

1. **Stamp at finalize.** Resolve name → `ClassId` once at `FinalizeClass`
   (`phalcom-core/src/bytecode.rs:325`) and patch the chunk constant. Removes the probe
   entirely.
2. **One-entry site cache.** The defining class is fixed per call site, so the cache never
   misses after the first execution. Smallest change; keeps a branch.
3. **Leave it, re-keyed.** 0065's unit A must in any case make this probe `(module, name)` —
   under module-scoped identity a bare name symbol is ambiguous across modules. This is the
   *minimum* the correctness work forces, and it makes the probe slightly more expensive, not
   less.

Route 3 ships regardless. 1 and 2 are the optional follow-on.

**Do not promise a win before measuring.** This is a real dispatch path, but super sends are not
obviously hot in the current benchmark set, and nothing here has been A/B'd.

---

## 2. Post-bootstrap freeze — the actual H17 unlock

**Unlocked by:** 0065 rulings 3 and 4 (reserved kernel names, core-gated stub completion).

Perf item H17's core.ph cursor probe was gated on "kernel-collection sealing." 0065 delivers the
precondition, but **not** in the shape the perf notes assume.

What 0065 does **not** buy: deletion of the override-epoch guard machinery. `world_version`
bumping (`phalcom-core/src/vm/dispatch.rs:927,930`) and the sacred-pristine flags
(`Universe::note_method_installed`, `phalcom-core/src/universe/mod.rs:191-194`) are artifacts of
*any* runtime method install, not of reopening. Bootstrap still installs methods, and 0065's
ruling 7 explicitly keeps a future reflection layer that will too. The guards stay.

What it **does** buy: once core.ph has finished loading, no method can be installed on a kernel
class from any source. Kernel class shape becomes statically known after a fixed point in
startup. That — not guard removal — is the property the cursor probe needed.

**Sequence:** land both 0065 units → define and enforce the freeze point → *then* measure H17.
Nothing before the measurement should be stated as a win. See
`docs/forge/perf-log/SCOREBOARD.md`; the standing rule against quoting perf from memory exists
because it already produced one wrong answer in this repo.

---

## 3. Give `None` a `core.ph` body, and unify the sealing representation

**Status:** deliberately deferred by 0065 (user ruling, 2026-07-19). Unit B fixes the
*blocker* (`DEFERRED` #17) and stops there; this item is everything downstream of that.

### What unit B does ship

`Statement::Class` unconditionally emits `DefineGlobal` at the end of every class body. For
every kernel class whose global already points at the class object that is a harmless no-op.
`None`'s global is deliberately bound to the shared singleton **instance**, not the class
(`phalcom-core/src/vm/bootstrap.rs:210-212`), so a `class None { ... }` body would rebind it to
the class and break every `x == None` downstream — `DEFERRED` #17.

Unit B lands the guard: skip `DefineGlobal` when the current binding is not that same class
object. Reproduced live 2026-07-19, so #17's "unverified since 2026-07-11, re-ground before
acting" caveat is discharged.

    class None { isNothing => true }
    let y = None
    y.isNothing   // <class None> does not understand 'isNothing'

Note the naive probe misses this — `None == None` still reads `true` because both sides rebind.
It takes a method call to surface.

### What is deferred (this item)

**(a) The body itself.** Bootstrap already expects it. The `None` class row is inserted into
`self.classes` (`bootstrap.rs:262-265`) for one stated purpose: so a core.ph skeleton "reopens
this bootstrapped row instead of forging a fresh one that would clobber the `None` global."
U-LIST dropped that skeleton and it was never restored. Under 0065 ruling 4 this is no longer
"reopening" — it is ordinary **stub completion**, the same core-gated mechanism the other 22
kernel classes use.

**(b) `DEFERRED` #35 — sealing has two representations that can disagree.** `Option`, `Some`,
and `None` are sealed by a **native write** at `bootstrap.rs:214-229,266-269` rather than by
`@sealed`, for exactly one reason the comment states outright:

> because `None` has no `.ph` class reopen to carry the annotation

`Option` (`core.ph:474`) and `Some` (`core.ph:544`) already have bodies and could carry
`@sealed` today. `None` is the sole holdout keeping the native-write path alive. Give it a body
and #35's option B — attribute list as the single source, `VM::sealed_classes` derived from it,
the union-read in `compiler/attributes.rs` collapsed back to one source — becomes reachable.

### Why 0065 stopped short

Adding the body drags ADR-0044's bootstrap ordering into a unit that otherwise does not touch
it: `Nil`→`None` surfacing runs *during* bootstrap, **before** `.ph` decorators. #35 flags that
as an open unknown, not as known-safe. Two questions must be answered before any code:

1. Can `None` take a `.ph` body (or a native attach) without disturbing ADR-0044's `Option`
   bootstrap?
2. ~~**Ownership.**~~ **RESOLVED 2026-07-19 — #35 overstates this; there is no hazard.**
   #35 predicted that bootstrap writes `sealed_classes[Option] = <bootstrap module>` while an
   `@sealed` on core.ph's body would write core.ph's module — "same key, different value,
   last-writer-wins," changing *who may subclass* via the
   `sealed_in_module != self.module` check (`compiler/lib/class_decl.rs:366`).

   They are the **same handle**. `install_core` creates the core module as `m`
   (`vm/bootstrap.rs:175`); `run_core_module` fetches it back with
   `get_module_from_str(CORE_MODULE_NAME)` (`vm/bootstrap.rs:167`), which is a plain
   `self.modules.get(&sym).copied()` (`vm/api.rs:117`). So the compiler's `self.module` while
   compiling core.ph *is* `m`. Both writers write the identical value; last-writer-wins is a
   no-op here.

   Unit A still re-keys `sealed_classes` on `(module, name)` — that is required for user
   classes regardless — but it inherits no `Option`/`Some`/`None` ownership conflict to
   untangle. Only question 1 above remains genuinely open.

Not to be confused with the B-wide form (`@sealed class Option { @variant Some(v); @variant
None }`), which needs its own decision: `@variant` generates `@data` classes with mutable
fields, but `None` must stay a zero-allocation singleton (ADR-0044), and
`Option#match(some:none:)` is already hand-rolled native
(`phalcom-core/src/universe/primitives.rs:191-198`) precisely because it is the eliminator
`@variant` would generate.
