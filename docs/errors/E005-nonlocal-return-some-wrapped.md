# E005 · A non-local `return` through `bool_if_true`/`bool_if_false` comes back `Some`-wrapped

- **Status:** OPEN — confirmed 2026-07-20 (reproduced under `target/debug/phalcom`, isolated by two controls)
- **Severity:** major — silent wrong answer, no error, reachable from ordinary code with no override anywhere
- **Subsystem:** primitive ABI / non-local return × the sacred-selector inliner's fallback path
- **Related:** contradicts `phalcom-core/src/compiler/inliner.rs:22-26` and `:224-229`, which assert the inlined and non-inlined paths are observationally identical. Narrative: [`docs/learn/vm/sacred-inliner.md`](../learn/vm/sacred-inliner.md).

## Defect

`bool_if_true` (`phalcom-core/src/primitive/boolean.rs:127-134`):

```rust
pub fn bool_if_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        let result = block_call(vm, &args[0], &[])?;
        Ok(wrap_some(vm, result))
    } else {
        Ok(vm.none_value())
    }
}
```

`block_call`'s `?` does not distinguish **"the block ran to completion and produced a value"** from
**"the block performed a non-local `return` that is still unwinding."** Both surface as `Ok(value)`.
So `wrap_some` runs on a payload that was supposed to pass straight through to the home method, and
the enclosing method returns `Some(v)` where the source said `return v`.

The same shape applies to `bool_if_false` (`boolean.rs:145-152`). `bool_and`/`bool_or`
(`boolean.rs:79-84`, `:94-99`) tail-return `block_call(...)` with no post-processing and are not
affected — **by inspection, not yet by repro.**

This is the *non-inlined* path, so it is reached whenever the sacred-selector inliner does not
recognize the call site — most commonly because the block was hoisted into a variable
(`compiler/inliner.rs:126-132`). **No override and no deopt is required.** It is also, by
construction, the path a genuine `GuardBool` deopt would take, so an override would expose it too.

## Reproduction

Under `target/debug/phalcom`:

```phalcom
// A — inlined (literal block): correct
class A { test() { (true).ifTrue { return "A" }; return "B" } }
System.print(A.new().test())          // -> A
```

```phalcom
// B — NOT inlined (block in a variable): WRONG
class A { test() { let b = { return "A" }; (true).ifTrue(b); return "B" } }
System.print(A.new().test())          // -> Some(A)     <-- expected A
```

**Controls:**

```phalcom
// C — direct block call, same non-local return: correct.
//     Isolates block_call itself as NOT the cause.
class A { test() { let b = { return "A" }; b.call(); return "B" } }
System.print(A.new().test())          // -> A
```

Control C is the load-bearing one: it proves the non-local-return machinery works, and that the
corruption comes specifically from a primitive that *post-processes* `block_call`'s result.

Also verified: with no `return` in the block, both paths agree exactly — `isSome`, the wrapped value,
and behaviour in statement vs value position are byte-identical. The `want_value`/`WrapSome` elision
(`inliner.rs:305`) has no observable effect here. **The divergence requires a non-local return.**

## Blast radius (unaudited)

The predicate is **"a native primitive that inspects, wraps, or otherwise post-processes the result of
a `block_call` it made."** Two instances are known and confirmed by inspection
(`bool_if_true`, `bool_if_false`). The remaining primitives that call `block_call` have **not** been
audited against this predicate. That audit is the first thing any fix should do — E001's post-mortem
records the same failure mode, where a hazard class was declared empty because the search predicate
was wrong.

## Why the suite is green

The inlined form is what every test and every realistic program writes, and the inlined form is
correct. Reaching the bug requires a non-local `return` inside a block that was *also* hoisted out of
the call site — a combination with no reason to appear in a test suite, because the hoist looks like
a pure refactor and the module doc promises the paths are identical.

The five golden fixtures that once exercised the override/deopt path end-to-end were rewritten
in-crate when classes were closed (`universe/mod.rs:228-238`), and hand-install methods rather than
running Phalcom source — so none of them runs a `return` through the fallback either.

## Fix direction (NOT implemented / NOT verified)

Recorded as a sketch of the space, not a prescription — in this codebase a reproduced diagnosis is
not a verified fix (see [README](README.md)), and E004 is the standing example of a correct diagnosis
with an unimplementable fix. Re-derive from code before acting.

The defect is **not** local to `bool_if_true`. The primitive cannot currently tell the two cases
apart, so patching it in place would require inventing a way to ask — which is the actual change:
`block_call` (or the `PhResult` it returns) needs to signal "this value is an in-flight non-local
return, do not touch it, propagate immediately." That is a change to the primitive ABI's error/return
channel, affecting every caller, not a four-line edit.

Whatever lands: audit every `block_call` caller against the predicate above, and add a fixture in
**both** shapes — literal block and hoisted block — since testing only the shape people write is
precisely what hid this.
