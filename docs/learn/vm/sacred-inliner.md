# The Sacred-Selector Inliner — two programs, one `if`

*Pays [Doc 5](caches-and-fusion.md)'s and [Doc 6](frame-identity.md)'s handoff. Doc 5 named
`GuardBool`/`GuardBlock` and the five `pristine` flags and set them down; Doc 6 leaned on the inliner
for its "the two `ifTrue` blocks are not blocks at runtime" trace. Both deferred the machinery to
here. Neither is re-derived.*

---

## The grip

> **Every `if` in Phalcom is compiled twice, and a guard picks which copy runs.**
>
> Phalcom has no `if` statement. A conditional is a message sent to a boolean, with blocks as
> arguments — and the language means it: `ifTrue` is an ordinary, overridable method. Making that
> cost what a native `if` costs, without giving up the overridability, is done by emitting **both
> programs into the same chunk, side by side** — the inlined fast path and the real-send fallback —
> and jumping between them on a one-instruction check.
>
> This makes deoptimization *free*, because there is no deoptimization. No OSR, no frame
> reconstruction, no recompilation, no invalidation sweep. The slow path is nine instructions away
> and was always there.
>
> The bill is paid somewhere nobody was looking. Each block body is compiled twice, and for one
> commit range each nesting level doubled it again — a 26-deep conditional took **70.9 seconds** to
> compile and bootstrap regressed **35×** while every gate stayed green. And the two copies, which
> the module doc promises are "observationally identical in every case a Phalcom program can detect,"
> **are not**: send the same block through the slow copy and a `return` inside it comes back wrapped
> in `Some`.

Two programs is the whole doc. Everything good and everything broken follows from it.

---

## The fork this doc is about

Most languages answer this question by not asking it. `if` is grammar, booleans are a primitive tag,
the compiler emits a branch, and nothing is overridable because there is nothing to override.

[ADR-0018](../../adr/accepted/0018-sacred-selector-inliner-and-override-guard.md) considered exactly
that and rejected it. From its `## Alternatives considered`:

> **Grammar-level control flow** (compile `if`/`while` straight to jumps, no selector). Fastest, but
> it makes control flow non-overridable and splits "looks like a send, isn't a send," breaking the
> uniform object model the rest of the spec depends on. Rejected.

Two other branches are recorded and rejected too: **no guard** (inline unconditionally, forbid
overriding sacred selectors) — rejected because it "would make Bool/Block second-class"; and
**per-selector invalidation** — deferred as bookkeeping for a case that never happens.

The Consequences section states the goal plainly: control flow must be "cheap on the common path …
while remaining fully overridable — the two goals the spec insisted could not be traded off against
each other." This document is about what it costs to refuse that trade.

---

## Predict before you read

```phalcom
const c = true
if (c) { System.print("A") } else { System.print("B") }
```

Two lines. `if`/`else` desugars to the single selector `ifTrue(_, ifFalse:)` — one base name, one
positional argument, one `ifFalse:`-labelled argument (`inliner.rs:28-47` explains why this shape and
not Smalltalk's `ifTrue:ifFalse:`).

**How many instructions?** The natural guess is four or five: push the condition, branch, one arm, one
arm. That is what an inlined conditional *should* cost.

It is seventeen.

```
0000: True
0001: DefineGlobal(0)
0002: GetGlobal(1)
0003: GuardBool(9)        ; not a pristine Bool? -> jump to 0013
0004: JumpIfFalse(4)
0005: GetGlobal(2)        ; ┐
0006: InvokeConst(3, 1, 4); │ then arm, spliced inline
0007: Invoke(1, 4)        ; ┘
0008: Jump(7)             ; -> end
0009: GetGlobal(5)        ; ┐
0010: InvokeConst(6, 1, 7); │ else arm, spliced inline
0011: Invoke(1, 7)        ; ┘
0012: Jump(3)             ; -> end
0013: Closure(8)          ; ┐ FALLBACK: materialize the then block
0014: Closure(9)          ; │           materialize the else block
0015: Invoke(2, 10)       ; ┘           real ifTrue(_, ifFalse:) send
0016: Return
```

Both arms appear **twice**. Once spliced as straight-line code at `0005` and `0009` — no
`ClosureObject`, no call frame, no dispatch. And once as closure constants `[8]` and `[9]`, waiting
at `0013` for a guard failure that will materialize them and perform the send the program literally
wrote.

`GuardBool(9)` is not a check-then-do-something-clever. It is a forward jump whose target is the
other program.

This listing is also a small mercy from the tooling: `disasm` only walks the top-level chunk, which
normally hides block bodies — but an inlined body *is* top-level chunk. The optimization makes itself
visible to a tool that cannot see blocks, because it stopped making blocks.

---

## What gets recognized, and how little it takes to lose it

`recognize` (`inliner.rs:138`) is a pure syntax match — no types, no profiling, no runtime feedback.
Selector name, argument count, and whether each block argument is literally an `Expr::Block` node at
the call site:

| Selector | Recognized shape |
|---|---|
| `ifTrue(_)`, `ifFalse(_)` | one literal block |
| `ifTrue(_, ifFalse: _)` | two literal blocks, second labelled `ifFalse` |
| `and(_)`, `or(_)` | one literal block — **including** the compiler-synthesized blocks from infix `a and b` |
| `whileTrue(_)` | argument *and receiver* both literal blocks |

Everything else falls through to an ordinary send. The doc comment is cheerful about this: a variable
holding a block "falls through to an ordinary `Invoke` send, which is correct, just not fast."

Verify it. Same conditional, one `let` apart:

```phalcom
c.ifTrue { 1 }              ->  0003: GuardBool(6)   ... inlined
let bb = { 1 }; c.ifTrue(bb) ->  no guard at all, plain Invoke
```

So the fastest and slowest forms of the same conditional are one refactor apart, with nothing in the
source marking the difference. Hoisting a block into a named variable — the most ordinary tidying
edit there is — silently deletes the optimization.

"Correct, just not fast" is the claim. It is wrong twice, and the rest of this document is those two
times.

---

## The two guards are not symmetric

```rust
// vm/dispatch.rs:1190-1201
Bytecode::GuardBool(offset) => {
    let top = *self.stack.last().ok_or(…)?;
    let takes_fast_path = matches!(top, Value::Bool(_)) && self.universe.bool_sacred_pristine;
    if !takes_fast_path { self.apply_jump_offset(offset); }
}
Bytecode::GuardBlock(offset) => {
    if !self.universe.block_sacred_pristine { self.apply_jump_offset(offset); }
}
```

`GuardBool` asks **two** questions on one branch: is the receiver actually a `Bool`, and has anyone
redefined a sacred `Bool` method. Two entirely different reasons to fall back — a type error and a
global override — indistinguishable at runtime, sharing one deopt target. They can share it precisely
because the fallback is *the program as written*: whatever went wrong, sending the real message is
always the right answer.

`GuardBlock` asks only the second. It never peeks a receiver, because the receiver of an inlined
`whileTrue` is a block literal the compiler materialized itself — its type is static.

ADR-0018's Decision says otherwise:

> **Receiver guard.** `GuardBool`/`GuardBlock` verify the receiver's type before the inline body runs.

That is true of one of them. The ADR describes a symmetry the implementation does not have, and the
asymmetry is the interesting part: it is evidence of how much the design leans on *static* knowledge.
`whileTrue`'s guard also runs **once, before the loop**, not per iteration (`inliner.rs:418-426`) —
there is nothing per-iteration that can change. The condition is still type-checked every time
around, but by `JumpIfFalse` itself, which raises on a non-`Bool`. That is where `while`'s
no-truthiness floor actually comes from: not a guard, a branch opcode that refuses to guess.

---

## Bill #1 — the copies multiply

Two copies per conditional is a constant factor. Nesting is not, if the second copy is also run
through the inliner. It was.

A conditional inside a conditional: the outer emits its body twice, and *each* copy contains both
copies of the inner. Depth *d* costs 2^*d*. From
[`SCOREBOARD.md`](../../forge/perf-log/SCOREBOARD.md) §3d, with source length held linear:

| nest depth | 16 | 18 | 20 | 26 |
|---|---|---|---|---|
| compile time before `0274f10` | 0.17 s | 0.70 s | **2.8 s** | **70.9 s** |
| after | — | — | — | **0.022 s** |

Its note: *"A 20-line source method could hang the compiler for minutes."* The lang suite ran
**122 s → 2.8 s**. Bootstrap — which recompiles `core.ph` on *every* process, every golden test,
every benchmark iteration — had regressed from ~5 ms to 180 ms, a **35× regression** across
`3b2dd97`…`0274f10`.

The SCOREBOARD row for it is four words long and worth the whole section:

> **35× regression, passed every gate** — nothing measured bootstrap.

Not a weak test. *No* test, because compile time was not a thing anyone had thought to assert on. The
suite measured whether the compiler was right, never whether it terminated in reasonable time, and a
35× bootstrap regression is invisible to every green checkmark in the project.

The fix (`0274f10`) is one branch: inside a fallback copy, skip recognition and emit an ordinary send.

```rust
// compiler/lib/expr.rs:56
let recognized = if self.in_deopt_fallback() { Err(*method_call) } else { inliner::recognize(*method_call) };
```

Measured at HEAD, depth 1→26 — linear, exactly 9 instructions per level, which is precisely the
guard-plus-fallback sequence:

| depth | 1 | 2 | 4 | 8 | 12 | 16 | 20 | 26 |
|---|---|---|---|---|---|---|---|---|
| instructions | 15 | 24 | 42 | 78 | 114 | 150 | 186 | 240 |
| compile s | 0.023 | 0.023 | 0.022 | 0.023 | 0.023 | 0.024 | 0.025 | 0.026 |

**A correction to my own first reading**, because it is the kind of mistake this course is about. I
assumed the suppression was a *soundness* requirement — a fallback runs because a sacred selector was
overridden, so surely inlining inside it re-assumes the thing that just proved false. That is wrong,
and the adversarial check caught it. A nested guard inside a fallback copy reads the same
globally-authoritative flag and gets the same correct answer; nesting would be sound, just enormous.
`0274f10`'s own message says so: *"the inliner is a guarded optimization over the `bool_if_true`/
`bool_and` primitives, not a semantic."* The guard's information is global, so it does not care where
in the code it sits. Size-only.

---

## Bill #2 — the two copies do not agree

`inliner.rs:22-26` states the invariant the whole scheme rests on:

> the two paths are built to be **observationally identical** in every case a Phalcom program can
> detect

and `:224-229` makes it specific, calling this the highest-value correctness property in the unit:

> A `return` inside therefore compiles to the enclosing method's ordinary `Bytecode::Return` and
> unwinds to the home method exactly as the non-inlined send form's frame-token non-local return
> would.

It does not. Three programs, one difference:

```phalcom
class A { test() { (true).ifTrue { return "A" }; return "B" } }              // -> A
class A { test() { let b = { return "A" }; (true).ifTrue(b); return "B" } }  // -> Some(A)
class A { test() { let b = { return "A" }; b.call(); return "B" } }          // -> A
```

The first is inlined: `return` is the enclosing method's own `Return`, and `test()` returns `"A"`.
The third calls the block directly: also `"A"`. The middle one — the same block, sent through the
same `ifTrue` the fallback would reach — returns **`Some("A")`**.

The cause is four lines (`primitive/boolean.rs:127-134`):

```rust
pub fn bool_if_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        let result = block_call(vm, &args[0], &[])?;
        Ok(wrap_some(vm, result))
    } else { … }
}
```

`block_call`'s `?` does not distinguish "the block finished and produced a value" from "the block
performed a non-local return that is still in flight." Both arrive as `Ok(value)`. So `wrap_some`
runs on a payload that was never supposed to stop here, and the enclosing method returns `Some("A")`
where the source said `return "A"`. The third program proves `block_call` is not the culprit — a
direct call unwinds correctly. Only a primitive that *post-processes* the result corrupts it, which
is why this hits `bool_if_true`/`bool_if_false` (which `Some`-lift) and not `bool_and`/`bool_or`
(which tail-return `block_call` untouched).

Note what this is not. It is not "the deopt path is broken" — it is reachable from ordinary code with
no override anywhere, because the *non-recognized* send lands in the same primitive. The `let` that
merely cost you speed two sections ago also changes your program's answer.

Filed as [E005](../../errors/E005-nonlocal-return-some-wrapped.md).

**This document's method note.** The blind theory pass — an agent given the design problem and no
access to this repository, asked to name the bug it would expect a competent implementation to have —
ranked *exactly this* first:

> the fast path is not a faithful speed-optimization of one fixed semantics — it's silently narrower
> semantics wearing the identical syntax … Code that does `x ifTrue: [ ^nil ]` has been running the
> *restricted* semantics the entire time it was fast.

It reached that from the mechanism alone, and predicted the symptom would surface through an
innocuous instrumentation shim. It was wrong about one thing, in the safe direction: it assumed an
override was needed to expose the divergence. A `let` is enough.

---

## Bill #3 — `break` is not a slowdown, it is a compile error

The `whileTrue` emitter is the only caller of `push_loop_context` (`inliner.rs:444`). The loop context
is what makes `break`/`continue` bind. So losing recognition on a `whileTrue` does not deoptimize the
loop — it removes the concept of the loop:

```phalcom
let c = { true }; let b = { break }; c.whileTrue(b)
```
```
Error: `break` outside of a loop: `break` may only appear inside a `for` loop body.
```

The same hoist that costs speed on `ifTrue`, and changes the answer on `ifTrue { return }`, refuses
to compile on `whileTrue { break }`. Three different failure modes from one syntactic cliff, and the
source gives no sign which one you are standing on.

---

## The guard now outlives its threat

The entire mechanism exists so that a user who redefines `Bool>>ifTrue` gets their redefinition
honored. Try it at HEAD:

```phalcom
class Bool { and(x) => false }
```
```
Error: class.reserved_name: 'Bool' is a kernel class name, reserved to the core module;
declare a differently-named class instead.
```

Since classes were closed (decision 0065, `c346200`/`7c2cfab`), reopening a kernel class is not
expressible in surface Phalcom. The five golden fixtures that once proved override-and-deopt
end-to-end could no longer be written; they now live in-crate, hand-installing methods through
`install_kernel_method` and bypassing the parser entirely
(`universe/mod.rs:228-238`, and the tests at `:294`, `:336`, `:368`).

So `GuardBool`'s second question — *has anyone redefined a sacred method* — is asked on every
conditional a Phalcom program ever executes, and at HEAD no Phalcom program can make the answer be
"yes." The guard is not dead: the flags are real, `note_method_installed` really flips them, and the
Rust-level tests really exercise the deopt. But the surface-language threat model it was built for
was closed off by a *different* decision, four months later, and nothing reconciled the two.

This is not an argument for deleting it. `Bool` is one `install_kernel_method` away from mattering
again, the check is one predictable branch, and a language that intends to reopen this door later
should keep the lock. It is an argument for knowing which of your safety mechanisms are currently
load-bearing and which are currently ceremonial, because the answer changes underneath you when an
unrelated decision lands.

One more coarse edge, from the same family: **all six** sacred `Bool` selectors share a single flag
(`universe/mod.rs:156`). Installing `and` deopts every `ifTrue` in the program — asserted directly by
`kernel_bool_sacred_override_deopts_nested_iftrue`. ADR-0018 chose that deliberately and says so.

And a latent asymmetry worth recording: `bool_sacred_pristine`/`block_sacred_pristine` are seeded
`true` in `Universe::new` with **no** post-bootstrap re-snapshot, unlike the leaf `toString` flags,
which seed `false` and are explicitly re-marked after `core.ph` runs (`vm/bootstrap.rs:134-140`)
*because* `core.ph` legitimately installs `String>>toString` during bootstrap. If `core.ph` ever
reopened a sacred `Bool`/`Block` method, it would permanently dirty the flag for every user program
process-wide. It does not today — `core.ph:421-431` reopens `Bool` only for `toString` and says in a
comment that this is deliberately not a sacred selector. The landmine is armed and nothing is
standing on it.

---

## What nobody measured

There is no measurement, anywhere in `docs/forge/perf-log/`, of what the inliner is worth **at
runtime**. Every number attached to it is compile time. `0274f10` states throughput was unchanged and
did not re-measure. No cut isolates an inlined branch against the equivalent real send.

The mechanism is almost certainly a large win — it removes two heap allocations, a dispatch, and a
call frame from every conditional, and Doc 5's own numbers show what a send costs. But "almost
certainly" is the point. The project's standing rule is that performance claims come from
`SCOREBOARD.md` and nowhere else, and by that rule the runtime value of Phalcom's single largest
compile-time optimization is **unquantified**. The one thing that *is* measured about it is how much
it cost.

---

## The design space

The branch Phalcom refused — grammar-level control flow — is the one nearly everything else takes,
and it is worth being precise about what refusing it actually bought. Not expressiveness in the
abstract: the ability for `Bool` to be an ordinary class, for `ifTrue` to appear in a method
dictionary, for reflection to find it, and for the object model to have no second tier of
things-that-are-not-objects. Phalcom pays for that with a guard on every conditional, a doubled
chunk, and the three cliffs above.

**Smalltalk** is the direct ancestor and took the identical approach — special-selector recognition
on the literal syntactic shape, with the same well-known consequence: send `ifTrue:` via `perform:`
instead of writing it literally and you leave the fast path. Phalcom's `let`-hoist cliff is that same
seam. This is the strongest evidence that the cliff is structural to the technique rather than a
local slip: a mature implementation has had it for decades.

**Lua** faces the operator-shaped version of this — a metamethod-presence check guarding the fast
path for `+`/`__index` — and lands on the same fast-path-plus-guard structure for a different feature.

**Ruby** invalidates its per-call-site caches with a global serial bumped on any method redefinition:
structurally the same coarse-flag trade Phalcom made, and evidence that "coarse but always correct"
is a normal place to stop.

**CPython 3.11+** quickens opcodes in the instruction stream with embedded caches and deopts on guard
failure — the closest modern relative of the dual-emission shape, though it rewrites code in place
where Phalcom emits both copies up front and never mutates.

Deliberately cut: **Java/C#** (booleans are primitive, `if` is grammar — comparing to them smuggles
in the rejected branch as if it were free), **V8** (hidden classes solve polymorphic *property
access*; `if` was never a send there), and **Self** (the fine-grained dependency-list end of the
space, which is JIT-tier machinery and a different doc).

---

## What you can now re-derive

1. Why a two-line `if` is seventeen instructions, and which nine of them never execute.
2. Why deopt costs one predicted branch and no machinery — and why that is a compile-time decision,
   not a runtime one.
3. Why nesting was 2^depth and why one `if` in the compiler fixed it, and why sound-vs-size was the
   wrong first guess.
4. Why hoisting a block into a `let` can make code slower, change its answer, or refuse to compile,
   depending only on which sacred selector it was.
5. Why `GuardBlock` needs no receiver but `GuardBool` does, and why `while`'s no-truthiness rule
   comes from a branch opcode rather than a guard.

---

## Anchors

| Claim | Where | Verified by |
|---|---|---|
| Both paths emitted into one chunk; guard is a forward jump | `inliner.rs:295-318`, `:298` | `disasm`, 17 instructions |
| Recognition is purely syntactic | `inliner.rs:126-173` | two disassemblies, one `let` apart |
| `GuardBool` = type ∧ pristine; `GuardBlock` = pristine only | `dispatch.rs:1190-1201` | quoted |
| ADR overstates the guard symmetry | ADR-0018 §Decision vs `bytecode.rs:250` | quoted both |
| `whileTrue` guard runs once, not per iteration | `inliner.rs:418-426` | quoted |
| Compile time was 2^depth; 70.9 s at depth 26; bootstrap 35×; suite 122 s → 2.8 s | SCOREBOARD §3c/§3d | quoted |
| Linear at HEAD, 9 instrs/level, 0.026 s at depth 26 | — | measured, depths 1–26 |
| Suppression is size-only, not soundness | `0274f10` message; `expr.rs:56` | REFUTE ask; my hypothesis refuted |
| Non-local return through the non-inlined path returns `Some(A)` | `primitive/boolean.rs:127-134` | three programs run |
| `break` in a non-recognized `whileTrue` is a compile error | `inliner.rs:444` | program run |
| Kernel-class reopening unreachable at HEAD | decision 0065, `c346200`/`7c2cfab` | program run: `class.reserved_name` |
| Six `Bool` selectors share one flag | `universe/mod.rs:156` | test `…deopts_nested_iftrue` |
| Sacred flags have no post-bootstrap re-snapshot | `universe/mod.rs:138-139` vs `bootstrap.rs:134-140` | quoted both |
| No runtime measurement of the inliner exists | `docs/forge/perf-log/` | searched; none |

Defect record: [E005](../../errors/E005-nonlocal-return-some-wrapped.md).

---

## Forward

- **E005 is open.** The fix is not obvious — `block_call` would have to distinguish an in-flight
  non-local return from a completed value, which is a signalling change in the primitive ABI, not a
  local patch to `bool_if_true`. Every primitive that post-processes a `block_call` result shares the
  hazard; two are known, the rest are unaudited.
- **The runtime value of the inliner is unmeasured** — the one cut nobody has run.
- **The reserved-name/guard reconciliation** is unowned: decision 0065 closed the threat the guard
  was built for and no record connects them.
- `SuperSend` is the remaining owed doc, and it is the last of the three ranked gaps.
