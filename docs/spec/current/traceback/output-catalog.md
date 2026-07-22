# Traceback & diagnostic output catalog

Every user-visible rendering surface U-TRACE is responsible for, specified by example.

**Status:** design target, not implemented. See [`README.md`](README.md) for scope, locked
decisions, and the open decisions that gate several of these renderings.

**These are targets, not fixtures.** Golden tests assert *structure* — frame sequence, event
fields — never byte-exact layout. See README §"Testability without freezing the format".

---

## 1. Runtime traceback — the base case

Source `shop.ph`:

```phalcom
1  class Cart {
2    total { self.sum(_items) }
3    sum(items) { items.fold(0) { acc, it => acc + it.negatd } }
4  }
5
6  let cart = Cart.new()
7  cart.total
```

Rendering:

```
Traceback (most recent call last):
  shop.ph:7   in <main>
      cart.total
  shop.ph:2   in Cart.total
      total { self.sum(_items) }
  shop.ph:3   in Cart.sum(_)
      sum(items) { items.fold(0) { acc, it => acc + it.negatd } }
  [2 core frames elided — pass --trace-core to expand]
  shop.ph:3   in <block in Cart.sum(_)>

  × 1 does not understand 'negatd'
   ╭─[shop.ph:3:48]
 3 │   sum(items) { items.fold(0) { acc, it => acc + it.negatd } }
   ·                                                ─────┬────
   ·                                                     ╰── Number has no method 'negatd'
   ╰────
  help: did you mean 'negated'?
```

Rules this encodes:

- **Ordering is Python's**: most-recent-call-last. Error message sits at the bottom, adjacent
  to the innermost frame, because that is where the terminal leaves the reader's eye.
- **Frame line format**: `  <module>:<line>   in <name>` then the source line indented under it.
  Two lines per frame, no box drawing.
- **Frame names**: `<main>` for module top level, `Cart.total` for a nullary method,
  `Cart.sum(_)` for arity-1 (selector shape, not a bare name — `foo` and `foo(_)` are different
  methods and the trace must distinguish them), `<block in Cart.sum(_)>` for a block, naming its
  enclosing method.
- **Core frames elide by default** with a count and the flag that expands them. A reader
  debugging `shop.ph` does not want `List.fold`'s internals. Ruby and Rust both do this; Python
  does not and its tracebacks are worse for it.
- **Only the innermost frame gets the caret block.** A forty-frame caret render is unreadable.
  Everything above is the cheap two-line form.
- **`help:` is optional** and appears only when a suggestion clears the confidence threshold
  (README §"did-you-mean").

---

## 2. Runtime traceback — across a fiber boundary

Source `job.ph`:

```phalcom
1  let worker = Fiber.new {
2    let rows = load()
3    rows.first.parse()
4  }
5  worker.call()
```

Rendering:

```
Traceback (most recent call last):
  job.ph:5   in <main>
      worker.call()

  ⤷ raised inside fiber #3, spawned at job.ph:1

Traceback (most recent call last):
  job.ph:3   in <block in <main>>
      rows.first.parse()

  × None does not understand 'parse'
   ╭─[job.ph:3:14]
 3 │   rows.first.parse()
   ·              ──┬──
   ·                ╰── receiver is None — `first` on an empty List
   ╰────
```

Rules this encodes:

- **The chain crosses the fiber floor.** It does not stop there. Precedent is Python 3's
  `__context__`/`__cause__` chaining, which prints a second traceback under a linking sentence.
- **The link line carries the fiber id and its spawn site**, so the reader can find where the
  fiber came from without tracing on.
- **Each side is a complete traceback** with its own header. Only the innermost side carries the
  caret block and message.
- **This is the fibers-track payoff**: a fiber switch becomes visible *in the error itself*, with
  no tracing flag set. Today it is invisible at every level.
- **N-deep chains** repeat the link line. See README §"Deep and repetitive stacks" for the
  truncation rule once a chain exceeds the frame budget.

The label on the caret (`receiver is None — \`first\` on an empty List`) is a **second-order
hint**: it explains why the receiver is `None`, not merely that it is. **Aspirational — not v1.**
`None` is a shared singleton, so origin-tracking means boxing absence; v1 renders the first-order
`receiver is None` only. See [`implementation-spec.md`](implementation-spec.md) §10 (hint
provenance classes A/B/C).

---

## 3. Disassembly — recursive

Today `disasm` prints only the top-level chunk; nested closures sit in `constants` as
`Value::Obj` and render as `{:?}` handles, so every method and block body is invisible
([disasm.rs:11](../../../phalcom-core/bin/phalcom/disasm.rs)).

Target, same `shop.ph`:

```
<main>  shop.ph   slots=3 upvalues=0
  constants:
    [0] <class Cart>
    [1] Symbol(total)
  0000  line 6   Class(0)
  0001  line 7   GetLocal(0)
  0002  line 7   Invoke(total, 0)

  └─ Cart.total   slots=2 upvalues=0
       0000  line 2   GetSelf
       0001  line 2   GetField(0)
       0002  line 2   Invoke(sum(_), 1)

  └─ Cart.sum(_)   slots=4 upvalues=0
       0003  line 3   Closure(1)        ← captures: acc
       0004  line 3   Invoke(fold(_,_), 2)

       └─ <block in Cart.sum(_)>   slots=3 upvalues=1
            0000  line 3   GetUpvalue(0)
            0001  line 3   Invoke(negatd, 0)
```

Rules this encodes:

- **Recursion into constants** that hold closures, nested by indentation, so the static call
  structure is legible at a glance.
- **`line N` per instruction** comes from `spans[ip]`, free once byte-offset→line exists for the
  traceback. Both consumers share one resolver.
- **Header per chunk**: name, `slots`, `upvalues`. Constants listed once per chunk, resolved to
  readable forms (`<class Cart>`, `Symbol(total)`) rather than raw `ObjRef` handles.
- **Capture annotations** on `Closure(_)` (`← captures: acc`) — the one piece of information that
  makes upvalue bugs tractable and that no current tool surfaces.
- **Selector shapes in `Invoke`**: `Invoke(sum(_), 1)`, not `Invoke(12, 1)`. Resolve the symbol.

Fused superinstructions (`InvokeLocal`/`InvokeConst`) must render the fusion *and* note the dead
`Invoke` slot they left behind, or a reader diffing disassembly against `spans` will conclude the
line table is corrupt. See [dispatch.rs:538-543](../../../phalcom-core/src/vm/dispatch.rs).

---

## 4. Fiber switch log

`--trace=fibers`, running `benchmarks/concurrency/skynet.ph`:

```
$ phalcom --trace=fibers skynet.ph
[fiber] spawn   #2 <block in Skynet.makeFiber>  parent=#1  at skynet.ph:30
[fiber] switch  #1 <main> @ skynet.ph:51  ──→  #2 @ skynet.ph:30
[fiber] yield   #2 @ skynet.ph:32  value=0
[fiber] switch  #2  ──→  #1 <main> @ skynet.ph:51
[fiber] done    #2  result=0  frames=3
```

Rules this encodes:

- **One choke point.** Every switch goes through `switch_to_fiber_and_deliver`
  ([dispatch.rs:352](../../../phalcom-core/src/vm/dispatch.rs)), which today carries no
  instrumentation of any kind.
- **No `cfg` gate needed.** This is a cold path; perf-log 003's per-opcode argument does not
  apply. The runtime filter alone suffices — unlike `vm-trace`, which is genuinely hot.
- **Frame labels come from the same walk** the traceback uses. One primitive, two consumers.
- Event vocabulary: `spawn`, `switch`, `yield`, `done`, `fail`.

---

## 5. Machine-readable trace stream

`--trace=fibers --trace-format=json`, one event per line:

```json
{"ev":"spawn","fiber":2,"parent":1,"name":"<block in Skynet.makeFiber>","at":{"file":"skynet.ph","line":30}}
{"ev":"switch","from":1,"to":2,"from_at":{"file":"skynet.ph","line":51},"to_at":{"file":"skynet.ph","line":30}}
{"ev":"yield","fiber":2,"at":{"file":"skynet.ph","line":32},"value":"0"}
{"ev":"switch","from":2,"to":1,"to_at":{"file":"skynet.ph","line":51}}
{"ev":"done","fiber":2,"result":"0","frames":3}
```

This exists so golden fixtures can assert on **fields** rather than on human layout. Without it,
the first test that asserts the pretty format freezes that format forever and the trace can never
gain a column. The human renderer is explicitly unstable; this one is the contract.

---

## 6. Syntax error — one register for both commands

Today `phalcom foo.ph` and `phalcom check foo.ph` render the same syntax error two different
ways: the run path propagates through `anyhow` and prints
`SyntaxError`'s bare `Display` (`"{kind} (at bytes 142..148)"`), while the check path converts to
line/column and calls `print_parse` for a caret-underlined snippet.

Both must produce the check path's register:

```
error: expected ')' to close argument list
   ╭─[shop.ph:3:52]
 3 │   sum(items) { items.fold(0) { acc, it => acc + it.negatd }
   ·              ─┬─                                          ▲
   ·               ╰── argument list opened here               ╰── expected ')' here
   ╰────
  help: add ')' before the end of the line
```

Two labels on one span pair (opener and expected-closer) is the shape most parser errors want.
Whether the incumbent renderer can express multi-label spans is an open decision — README
§"Open: renderer".

---

## 7. Compile error — spans exist and are dropped

Four `CompilerError` variants carry a `SourceRange` that is **never rendered**:
`DestructuringWithoutInitializer`, `BreakOutsideLoop`,
`ContinueOutsideLoop`, `ThrowNonError`. `cmd_run` does `vm.compile_closure(...)?` and the `?`
propagates to `main`, which prints `Display` text only. `print_parse` fires for *parse* errors
only, via `compile_closure`'s `map_err`.

Target:

```
error: `break` outside a loop
   ╭─[shop.ph:9:5]
 9 │     break
   ·     ──┬──
   ·       ╰── no enclosing loop to break out of
   ╰────
  help: `break` is only valid inside `while`, `for`, or a loop block
```

**Cost warning for whoever specs this.** Adding a span to a compiler diagnostic costs one struct
field. *Making it appear* costs wiring a renderer into the compile-error path — which changes how
**every** compile error prints and moves the entire negative-fixture corpus. Price the two
separately; they are not one change.

---

## 8. REPL `where` — deferred, recorded

```
phalcom> Cart.new().total
Traceback (most recent call last):
  …
  × 1 does not understand 'negatd'

phalcom> where
  #0  <block in Cart.sum(_)>   shop.ph:3    ← innermost
  #1  Cart.sum(_)              shop.ph:3
  #2  Cart.total               shop.ph:2
  #3  <main>                   <repl>:1
```

**Not in U-TRACE.** Post-mortem inspection means the dead stack outlived the unwind — frames,
their closures, and their modules all stay rooted past the point the unwind normally drops them.
That is structurally the same shape as the confirmed `block_ensure` temp-root UAF: a live
reference the collector cannot see. Gate on `temp_roots` existing (ADR-0050 §7, unbuilt).

The API supports it either way. `StackWalk<'vm>` borrows; a post-mortem variant needs an owned
snapshot type. Leave that type unbuilt rather than design around its absence.
