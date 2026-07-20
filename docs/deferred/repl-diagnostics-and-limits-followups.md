# Deferred: diagnostics, depth-limit and REPL follow-ups (unowned)

Surfaced while verifying U-REPL and implementing
[PDR-0006](../decisions/0006-repl-completeness-is-a-parser-signal.md),
[PDR-0007](../decisions/0007-bounded-call-depth-and-native-reentrancy.md),
[PDR-0008](../decisions/0008-cell-boundary-diagnostics-and-state-hygiene.md) and
[PDR-0009](../decisions/0009-defer-lsp-backed-repl-surface.md). These are the items those
records either explicitly scoped out, or that turned up during implementation and have no
owning unit.

Ranked by severity. All `file:line` references verified 2026-07-20 against `main` at `d7426ee`.

---

## 1. REVIEW OWED — `Block#on` now unwinds before probing, changing core unwind order

`phalcom-core/src/primitive/block.rs:282`

The `isA` probe that decides whether a handler matches is an ordinary dynamic send, so it
needs a call frame. It used to run *before* `unwind_to(stack_len, frames_len)`, i.e. on top of
the failed protected block's abandoned frames. That was merely untidy until the frame budget
itself became exhaustible: at `MAX_CALL_DEPTH` the probe could not get a frame, so
`RuntimeError::DepthExceeded` propagated straight through every `try`/`catch` and was
**uncatchable** — the recovery path needed exactly the resource that had run out. PDR-0007 §2
requires it be catchable, so the unwind moved first.

**Why this needs a second pair of eyes:** it is a semantics change to the error-handling core,
made in service of a different unit. Two consequences worth confirming deliberately:

- The **non-matching branch** (`Err(err)` re-raise) previously returned *without* unwinding, so
  an outer `on` saw the inner block's frames still on the stack. It no longer does. Outer
  handlers record their own `frames_len` at entry, so this should be safe — but "should be" is
  the whole reason this entry exists.
- An uncaught error that passes through a non-matching `on` now has its frames unwound
  earlier, so any traceback rendered *above* that point is shorter than it was. For the REPL
  this is moot (PDR-0008 §2 reports at the raise site, before any unwind); for `cmd_run` the
  report happens after `run_in_module` returns and may now show less.

Covered by `phalcom-core/tests/depth_limits.rs::depth_error_is_an_ordinary_catchable_raise`,
which fails without the reorder. No test pins the outer-handler case.

## 2. Depth is bounded; CPU and allocation are not

`phalcom-core/src/vm/mod.rs:41,62`

PDR-0007 bounds recursion on both axes it identified and **explicitly scopes out** the other
two exhaustion paths. Both remain fully open:

- **Loops.** ADR-0018's sacred-selector inliner lowers `whileTrue` to `Jump`, pushing no
  frame, so `while (true) {}` spins forever and no depth counter can see it. Bounding this
  needs an instruction/time budget, which taxes the dispatch loop ADR-0051's performance
  program is trying to make cheaper — a real trade, not an oversight.
- **Allocation.** `List` growth in a loop exhausts the heap with no cap.

A one-line script can still take the host down; PDR-0007 only removed the *recursive* route.
Wants its own record if ever taken.

## 3. `MAX_NATIVE_REENTRY` is a proxy, and the resource it proxies varies by thread

`phalcom-core/src/vm/mod.rs:62`

32 was chosen by measurement against a **2 MiB** thread stack (Rust's default for test and
spawned threads; the main thread gets 8 MiB). 128 aborts there, 64 and 32 survive. An embedder
that spawns the VM on a thread with a smaller stack, or a future interpreter change that grows
`run_until`'s frame, can still abort the process before the counter fires — the counter cannot
observe the resource it stands for.

The principled fix is native stack probing (`stacker`, or Go-style growable stacks), rejected
in PDR-0007's alternatives as disproportionate. Revisit if native re-entrancy depth ever
becomes load-bearing, or if an embedding host reports an abort.

## 4. Compile-error diagnostics carry no span

`phalcom-core/src/diagnostics.rs:102` (`print_compile`),
`phalcom-core/src/vm/dispatch.rs:180` (`compiler_error`)

`compiler_error` was an empty function body until PDR-0008 §1; it now renders, but message-only.
Two reasons, both real:

- Most `CompilerError` variants carry no `SourceRange` at all.
- The few that do (e.g. `DestructuringWithoutInitializer`) are not threaded together with the
  source text an excerpt needs — `compiler_error` receives a `PhError` and nothing else.

So a compile error prints a sentence where a parse error prints a caret and a code frame.
Closing this means plumbing source (or a `source_id`) to the reporting site and giving the
variants spans; it is a compiler-diagnostics project, not a patch.

## 5. Two independent run paths must be kept in reporting-sync

`phalcom-core/bin/phalcom/cli.rs:171,178` vs `phalcom-core/src/interpret.rs`

`cmd_run` does **not** go through `interpret_source`, which is why it inherited neither
reporter and file-mode runtime errors carried no traceback despite `print_rt` being fully
built. Both now report — but they report *separately*, with duplicated logic. Whoever changes
one must change the other, and nothing enforces that.

Worth deciding whether `interpret_source` is dead for the CLI and should be deleted, or whether
`cmd_run` should be rewritten to call it. Leaving two paths that must agree is the setup for
the same bug a third time.

## 6. `ReplSession` writes to stdout/stderr and cannot be embedded silently

`phalcom-repl/src/repl.rs:106`

PDR-0008 §3 names this cost explicitly and accepts it: `eval` prints the diagnostics it owns,
matching the file-run path. The consequence is that an LSP, a test harness, or any embedding
host has no way to evaluate a cell without the diagnostic hitting the process's stderr.

The fix is an injected reporter/sink (rustc's `DiagCtxt` is the shape) and is purely additive —
nothing here forecloses it. Left until there is a first embedder to design against, so the
sink's shape is driven by a real caller rather than guessed.

## 7. PDR-0006's per-mode obligation has no enforcement

`phalcom-ast/src/parser.rs:175` (`push_lex_error`)

PDR-0006 §3 binds every future lexer mode: *"Every lexer mode that can be left open at end of
input must co-emit `UnrecognizedEof`."* Today that is block comments and strings. A future
heredoc, raw string, or nested-interpolation mode that forgets silently stops continuing in the
REPL — the buffer submits mid-construct and the author sees a syntax error for text they were
still typing.

There is no compiler check for this; it is a rule in a document, and its failure mode is quiet.
The only defence is the per-mode test the PDR asks for, in the same commit that adds the mode.
A cheap structural improvement would be making the co-emission a property of the mode's
definition rather than a `match` arm in `push_lex_error` that a new variant can simply miss.
