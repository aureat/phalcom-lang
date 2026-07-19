# Deferred: reclaim destructuring scratch slots (unowned)

Split out of the [U-BINDINGS](../forge/units/U-BINDINGS/implementation-spec.md) design work
(2026-07-19), which took the cheap half of this problem and deliberately left the rest.

U-BINDINGS ships **unique scratch names** (ruling **L-9**) so that its same-scope
redeclaration ban (**L-5**) needs no `$`-prefix exemption. That closes the *rule* problem.
It does **not** reclaim the slots. This file is the remainder.

Supersedes the destructuring half of `docs/forge/DEFERRED.md` #494 — that entry framed this
as `perf / DX` only and did not know about the L-5 interaction or the nesting behaviour below.

All file:line references verified 2026-07-19 against `main`.

---

## What leaks

`let (a, b) = point` cannot peel elements straight off the stack — it needs the scrutinee
parked somewhere re-readable, once for the arity check and once per element. So the compiler
reserves a hidden local:

- `phalcom-core/src/compiler/lib/patterns.rs:61-65`

```rust
let scratch_sym = self.vm.interner.intern("$destructure");
self.add_local(scratch_sym, true);
let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
self.emit(Bytecode::SetLocal(slot), *range);
```

`*rest` patterns claim two more, `$destructure_rest` and `$destructure_i`
(`patterns.rs:200,209`).

`compile_for` handles the same problem correctly — it wraps `$for_coll`/`$for_cursor` in
`begin_scope()`/`end_scope()` (`loops.rs:126`, with the comment *"A fresh scope keeps the
synthetic temporaries and the loop variable out of the enclosing scope after the loop"*).

Destructuring cannot copy that. The pattern's **leaf** bindings (`a`, `b`) must survive at the
enclosing depth, and the scratch is claimed *before* the leaves, so it sits below them in the
slot stack. Ending the scope to drop the scratch would drop the leaves too.

## Why this is not just slot waste

**Frame slots are GC roots.** `VM::collect_roots` (`vm/gc.rs:32`) walks the running fiber's
`frames`/`stack`, and `gc.rs:95` traces every value in it. A live slot pins its referent.

So the scrutinee stays reachable until the enclosing scope ends:

```phalcom
let (a, b) = makeHugeTuple()
// ...rest of the block...
// the whole tuple is still rooted; only `a` and `b` are ever read
```

Observable as footprint, and as GC *timing* — [F11](../forge/perf-log/findings.md)'s
yield-adaptive threshold sizes `next_gc` from what a cycle reclaims, so retention moves
collection scheduling. **Not** observable as program values: Phalcom has no finalizers or weak
references, so nothing can witness it except memory.

Bounded, though: loop bodies *are* scoped, so this does not accumulate per iteration. It is
function-scope retention, bounded by the count of destructuring statements in one block.

**The implicit design position, which nobody chose.** Phalcom currently says *a temporary
lives to the end of its block* — the loosest option on the shelf (Rust drops at end of
statement, C++ at end of full-expression, JS/Python at unreachability). It fell out of "we
could not pop the scratch without popping the leaves", not out of a ruling. Closing this item
means picking that position deliberately.

## Nesting: the lifetimes genuinely overlap

The obvious fix — one reusable scratch slot per function, since destructuring statements do
not overlap — **does not work**, and the reason is not obvious from the entry in
`DEFERRED.md`.

Nested patterns claim overlapping scratches *within a single statement*. Reading the
recursion: the `Pattern::Tuple` arm claims `$destructure`, calls
`compile_pattern_bind_from_slot`, which per element calls back into
`compile_pattern_bind_top_of_stack` — and a nested tuple element re-enters the `Tuple` arm,
claiming a **second** scratch while the outer one is still live for the remaining elements.

So `let ((a, b), c) = ((1, 2), 3)` already holds two scratches simultaneously today. There is
a golden for it, `tests/lang/bindings/destructure_nested_tuple.ph`, and it passes.

Any fix must therefore handle a **stack** of live scratches, not a single reusable slot.

## What a fix has to avoid

The scratch names are **write-only** — `patterns.rs` never resolves them by name.
`compile_pattern_bind_from_slot` takes an explicit `value_slot: u16` and emits
`GetLocal(value_slot)` (`patterns.rs:86-97`). `add_local` is called purely to *reserve* the
slot. This is why L-9's rename is semantically inert, and why a reclamation fix is free to
change names at will.

Two invariants that are **not** involved, listed because conflating them makes this look far
more dangerous than it is:

- **ADR-0011's frozen slot offsets** govern *instance field* layout, not frame locals.
- **Upvalues** never apply — a scratch is unnameable, so it can never be captured, and
  `end_scope`'s `CloseUpvalue` emission never covers one.

What *is* involved, and is the real risk: reclaiming a slot means touching allocation order,
and both `resolve_local_in`'s reverse scan (`scope.rs:131`) and `add_upvalue`'s index
arithmetic depend on local slot indices being stable and monotonic. **This is why it was kept
out of U-BINDINGS** — that unit is already rewriting local-slot bookkeeping for L-3/L-5, and
two independent changes to the same machinery in one unit produce bugs that only reproduce
under closure capture.

## Do not sell this as a perf win

`max_slots` sizes the fiber stack frame, so retention does inflate frames. That is **not** an
argument for doing this: [F18](../forge/perf-log/findings.md) measured presizing the fiber
`Vec`s as *negative*. The lever is not buffer size. If this is ever done, it is for the
retention semantics and the code-generation discipline, not for a number.

## When to do it

**With the pattern-matching work, not before.** Full pattern matching (map patterns, match
arms) is deferred to v0.3 under ADR-0046 / open-questions Q7. That is the feature that
justifies the machinery: many arms, each with its own synthetic bindings, each dead at arm
exit. Slot reclamation is exactly what a many-armed `match` wants, and building it there means
the slot-allocation change is reviewed on its own terms rather than inside a correctness gate.

Until then L-9's unique names hold the line: the rule stays exceptionless, and the only cost
is the retention documented above.
