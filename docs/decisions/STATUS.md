# PDR status tracker

One row per Phalcom Design Record in this folder. Binding maintenance rules are in
[`README.md`](README.md) — the short version: status lives here **and** in the PDR's own
header, never anywhere else, and the two are changed in the same edit.

**Status** is Proposed / Accepted / Retired (Retired covers superseded and deferred).
**Shipped** is whether the design is implemented in the tree, independent of paper status —
the two drift apart in both directions in this repo. `✅` = code-verified this pass, with the
evidence named; `❌` = verified absent; `—` = nothing to ship; `?` = not checked, do not
assume either way.

PDR numbering is independent of the ADR sequence and starts at 0001. ADR-0001…0064 are frozen
and remain tracked in [`../adr/STATUS.md`](../adr/STATUS.md); the pairing table for any ADR a
PDR supersedes lives in [`README.md`](README.md#adr--pdr-mapping).

| # | Title | Status | Supersedes | Superseded by | Shipped |
|---|---|---|---|---|---|
| [0001](0001-classes-are-closed.md) | Classes are closed: remove class reopening | Accepted | ADR-0026 (Axis 1) | | ❌ ruled 2026-07-19, unimplemented |
| [0002](0002-class-declarations-join-the-binding-namespace.md) | Class declarations join the binding namespace; duplicate diagnostic carries both spans | Accepted | *amends* 0001 (rulings 2, 8) | | ⚠️ partial — 0001 ruling 8's import half shipped with U-BINDINGS (`b843fe2`), verified live; the rest unimplemented |
| [0003](0003-no-user-visible-threads-fibers-and-isolates.md) | No user-visible shared-memory threads: fibers now, isolates if ever | Accepted | | | — mostly a *constraint*, not code. Its live half is already true (single-threaded VM, `VecDeque` ready-queue); §3's worker-thread boundary binds 0004 and is unimplemented |
| [0004](0004-io-is-future-shaped-reactor-owned.md) | IO is `Future`-shaped and reactor-owned; reactor built before the IO surface | Accepted | | | ❌ ruled 2026-07-20, unimplemented. Precondition met: E004 fixed (`f479189`), fibers genuinely park. Closes system.md's open `System.sleep(_)` question |
| [0005](0005-resources-are-disposable-handles-not-finalized.md) | Native resources are closeable handles with a generation-tagged table; no finalizers | Accepted (**revised three times, same day 2026-07-20** — `close` not `dispose`; **synchronous `Result`**, reversing an interim `Future` spelling; `File` unbuffered; `Resource` root class; `using` withdrawn; **§7a ruled**. See its header) | | | ❌ ruled 2026-07-20, unimplemented. Selector surface ratified in §7; §7a `BufferedWriter#close` **ruled in the third revision** (is a `Resource`; dirty close raises, `finish` flushes-then-closes) — protocol, laws, and conformance harness in [`stream-protocol.md`](../spec/v0.2/core/stream-protocol.md). Closes ffi.md F-3; defers the multi-axis protocol problem to [`docs/deferred/io-protocol-axes-need-stateless-interfaces.md`](../deferred/io-protocol-axes-need-stateless-interfaces.md) |
| [0006](0006-repl-completeness-is-a-parser-signal.md) | REPL completeness is a parser signal; the lexer reports unterminated modes | Accepted | | | ✅ shipped `f62bd72`. `push_lex_error` co-emits `UnrecognizedEof` with the missing closer as its `expected`; `validator.rs` untouched as §1 requires |
| [0007](0007-bounded-call-depth-and-native-reentrancy.md) | Bounded call depth and native re-entrancy: two counters, one error | Accepted (**§1 amended same day** — native ceiling 200 → **32**, measurement refuted the proposed number; see its header) | | | ✅ shipped. Infinite recursion was a measured 5-minute OOM hang, now a catchable error in <1s. Closes the overlay's "resource limits unspecified" gap. Two defects found while implementing: `Block#on` probed `isA` before unwinding (made the depth error uncatchable), and `runtime_error` panicked on a zero-`ip` frame |
| [0008](0008-cell-boundary-diagnostics-and-state-hygiene.md) | Every failed cell reports, and reporting happens before unwinding | Accepted | | | ✅ shipped `dcc4420` + `8466867`. §1 (`compiler_error` implemented, parse path prints), §2 (report moved before `unwind_cell`; traceback verified non-empty), §4/§5 (echo sends `toString`, `catch_unwind` deleted, unwind after failed echo). Also fixed the same defect class on `cmd_run`, which bypassed both reporters |
| [0009](0009-defer-lsp-backed-repl-surface.md) | The LSP-backed REPL surface waits for ADR-0056 to be ratified | Accepted | | | — a deferral; its §4 obligations are ✅ done (DEFERRED entry, `highlighter.rs` doc fix, CLAUDE.md correction, tracker row un-claimed) |
| [0010](0010-errors-carry-structure-and-cheap-origin.md) | Errors carry structure and cheap origin: one cause chain, a `kind` symbol, incremental capture | **Proposed** | | | ❌ not ratified — do not design against it (README rule 5) |
| [0011](0011-admit-bytes-native-octet-buffer.md) | Admit `Bytes`: native octet buffer arm and six floor primitives (amends ADR-0019, floor 137 → 143) | **Proposed** | *amends* ADR-0019 | | ❌ not ratified — do not design against it (README rule 5). Spec drafted at [`bytes.md`](../spec/v0.2/core/bytes.md), normative upon ratification; unblocks [`stream-protocol.md`](../spec/v0.2/core/stream-protocol.md) §9's hard dependency |

## Cross-tracker obligations

A PDR that supersedes an ADR must update the ADR in **both** places — the ADR file's own
status line, and its row in `../adr/STATUS.md` — and add a row to the mapping table in
[`README.md`](README.md#adr--pdr-mapping).

- **PDR-0001 → ADR-0026**: done 2026-07-19. ADR-0026 flipped to Retired in
  `../adr/accepted/0026-class-hierarchy-mutability.md` and in `../adr/STATUS.md`. The file
  stays in `accepted/` — its own status line is authoritative, not its path, which is
  precisely the ADR-layout defect this folder exists to remove.
