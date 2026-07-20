# The Proving Ground

A question bank for someone being interviewed as a **language and runtime implementer** —
not a user of languages, a builder of them. Every question here is one where a fluent
user of five languages still answers wrong, and only someone who has had to *make the
decision* answers right.

This is deliberately not a Phalcom document. Phalcom is one data point; the interview is
about the design space. Where a Phalcom scar illuminates a general question it appears as
a worked example, but the questions stand on real-world precedent — Smalltalk-80, SELF,
Lua, V8, HotSpot, CPython, PyPy, BEAM, CLR, Rust, GHC, LuaJIT, Wren, JSC.

## How to use it

Each file is **questions first, answers second**. Do not scroll. Answer out loud, in
full sentences, as if to an interviewer — then read the answer and diff it against what
you said. The gap is the lesson, and the gap is usually not "I didn't know the fact,"
it is "I knew the fact and could not say what it *buys* or what it *costs*."

Three grades to give yourself:

- **Recalled** — you produced the fact. Worth little; facts are cheap and interviewers know it.
- **Derived** — you reconstructed the mechanism from first principles without recall.
- **Traded** — you named what the design forecloses, and what you would pick instead under
  a different constraint. This is the only grade that reads as senior.

Most answers close with a **Trap** — the plausible, confident, wrong thing a strong
candidate says here. If your answer matched the trap, that question is worth revisiting
cold in a week.

## Files

| # | File | The through-line |
|---|------|------------------|
| 01 | [dispatch-and-object-model.md](01-dispatch-and-object-model.md) | What a method call *is* once objects can be created at runtime |
| 02 | [closures-and-control-flow.md](02-closures-and-control-flow.md) | Variables that outlive frames; jumps that leave frames |
| 03 | [bytecode-and-interpreter-design.md](03-bytecode-and-interpreter-design.md) | The shape of the loop and what the instruction set costs |
| 04 | [inline-caches-and-speculation.md](04-inline-caches-and-speculation.md) | Being fast by being wrong rarely, and surviving being wrong |
| 05 | [values-and-representation.md](05-values-and-representation.md) | Fitting a universe of values into 64 bits |
| 06 | [memory-and-gc.md](06-memory-and-gc.md) | Who may point at what, and when it is safe to look |
| 07 | [concurrency-and-coroutines.md](07-concurrency-and-coroutines.md) | Suspending a computation and resuming it elsewhere |
| 08 | [errors-and-unwinding.md](08-errors-and-unwinding.md) | Leaving a computation early without leaving it broken |
| 09 | [lexing-and-parsing.md](09-lexing-and-parsing.md) | Turning bytes into a tree, and being useful when you cannot |
| 10 | [types-and-checking.md](10-types-and-checking.md) | Rejecting programs before they run, and paying for the ones you cannot |
| 11 | [compilation-ir-and-bootstrapping.md](11-compilation-ir-and-bootstrapping.md) | Lowering, and building the thing that builds the thing |
| 12 | [metaprogramming-and-open-world.md](12-metaprogramming-and-open-world.md) | Programs that reshape themselves while running |
| 13 | [modules-namespaces-and-linking.md](13-modules-namespaces-and-linking.md) | Names, scopes, and what identity means across compilation units |
| 14 | [core-library-and-protocol-design.md](14-core-library-and-protocol-design.md) | The library decisions that are really language decisions |
| 15 | [performance-methodology.md](15-performance-methodology.md) | Knowing whether you actually made it faster |

## Format contract

Every question follows the same shape, so the bank stays uniform as it grows:

````markdown
### Q7 — Short title naming the tension

Concrete setup: code, a scenario, or two designs side by side.

The ask, in one to three numbered parts. Each part demands a mechanism or a
trade-off, never a definition.
````

and, in the Answers half:

````markdown
### A7 — Same title

The mechanism, then the trade-off, then what it forecloses.

**Trap.** The confident wrong answer.
````
