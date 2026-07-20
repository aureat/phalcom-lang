# Specification — Process & environment (`Command`, `Child`, `Output`, `Stdio`, `System` env rows)

> **Status:** **Proposed — normative upon ratification of
> [PDR-0019](../../../decisions/0019-process-and-environment-surface.md)** (rule 5).
> Promotes [`../system.md`](../system.md) §2's specified-unbuilt process rows and the
> [`../drafts/stdlib-catalog.md`](../drafts/stdlib-catalog.md) §3.5/§3.6 drafts —
> ruling S-5 (no `Env` namespace) and S-13 (classes, not modules) on the way; the
> catalog's no-shell security note is now a ruling, not a note. Already-Accepted
> inputs: [PDR-0004](../../../decisions/0004-io-is-future-shaped-reactor-owned.md)
> §1/§3, [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md)
> §3-§5, [PDR-0013](../../../decisions/0013-path-is-bytes-backed-filesystem-surface.md)
> ruling 4 (lossy/exact split), ADR-0012/ADR-0043.
> **Floor delta: nonzero — the native family is ruled in PDR-0019 ruling 11**, exact
> census at impl time under PDR-0012 ruling 21.
> **Build order:** phase A (everything except `Stdio.piped`) needs U-RESOURCE +
> U-REACTOR; phase B (piped child streams) additionally needs U-NET's poller.
>
> **Owner:** unassigned.

## 1. Scope and the shape of the surface

| Kind | Blocks? | Returns |
|---|---|---|
| `System` env/args/pid accessors (§2) | never — process-local reads | plain values |
| `System.cwd` | never — `getcwd` | `Result` of `Path` |
| `System.exit(_)` | n/a — does not return | — |
| `Command` builder sends (§4) | never — local state | the receiver |
| `Command#run` / `#spawn`, `Child#wait` (§4-§6) | yes — worker pool | `Future` |
| `Child#tryWait` / `#kill` (§6) | never — `WNOHANG` / `kill(2)` | plain / `Result` |
| `Child#close` | never | `Result` (from `Resource`) |

The environment is **read-only** (PDR-0019 ruling 3 — `setenv` is UB against pool
threads; the child's env is built on `Command`, §4). No selector anywhere accepts a
shell string (ruling 5).

## 2. `System` — the environment rows

Promoted from system.md §2 with the PDR-0013 ruling-4 lossy/exact split (ruling 4):

| Selector | Returns | Meaning |
|---|---|---|
| `System.env(_)` | `Option` of `String` | value **lossily** decoded (U+FFFD) — for humans; `None` if unset |
| `System.envBytes(_)` | `Option` of `Bytes` | the exact octets — the pipeline form; an env value that is a path goes `envBytes` → `Path.ofBytes`, never through `String` |
| `System.args` | `List` of `String` | argv, lossily decoded |
| `System.argsBytes` | `List` of `Bytes` | argv, exact |
| `System.pid` | `Number` | this process |
| `System.cwd` | `Result` of `Path` | `getcwd` — synchronous (non-blocking) and fallible (a deleted cwd settles nothing: it *returns* `Err`) |
| `System.exit(_)` | — | integral `0..255`, raise otherwise; runs reactor shutdown ([`reactor.md`](reactor.md) §8), resource-table drain, and the leak report **before** `exit(2)` (ruling 9) |

Variable names are `String` (Q-P5 holds the exact-name case until a consumer exists).

## 3. `Stdio`

Three singletons, the `OpenMode` pattern (PDR-0013 ruling 5): `Stdio.inherit`,
`Stdio.piped`, `Stdio.null`. A plain `.ph` class with three statics and `toString`.

## 4. `Command`

A builder (ADR-0043's sanctioned shape — every setting its own selector, no options
bag). Builder sends return the receiver and touch nothing outside it.

| Selector | Meaning |
|---|---|
| `Command.new(_)` | program by `String` name, resolved against `PATH` at spawn |
| `Command.newAt(_)` | program by `Path`, executed exactly, no lookup (two selectors, not a union — PDR-0015 ruling 4's precedent) |
| `arg(_)` / `args(_)` | append one `String` argv entry / a `List` of them |
| `argBytes(_)` | append exact octets (the rare exact-argv case) |
| `env(_, to:)` | set a variable **in the child's environment** — never the parent's (ruling 3 is not violated by construction) |
| `clearEnv` | child starts from an empty environment (default: inherit) |
| `cwd(_)` | child working directory, a `Path` |
| `stdin(_)` / `stdout(_)` / `stderr(_)` | a `Stdio` singleton; default `inherit` |
| `run` | `Future` → `Ok(Output)`: spawn, capture, reap — §5 |
| `spawn` | `Future` → `Ok(Child)` — §6; **phase A rejects `Stdio.piped` with a raise** (contract: the phase seam is loud, not silently blocking) |

`run` forces `stdout`/`stderr` to captured regardless of the builder's `Stdio` (that
is what `run` *is*); it feeds the child no input. Capture of both pipes is concurrent
worker-side — a child writing >64KiB to both streams completes (the harness proves the
no-deadlock claim).

## 5. `Output`

An immutable snapshot (the `Metadata` posture — no accessor blocks, none returns a
`Future`):

| Selector | Returns | Meaning |
|---|---|---|
| `exitCode` | `Number` \| `None` | `None` iff signal-terminated |
| `signal` | `Number` \| `None` | the terminating signal, if any |
| `succeeded` | `Bool` | `exitCode == 0` |
| `stdout` / `stderr` | `Bytes` | captured octets (empty when the stream was not captured). Bytes, never `String` — child output is arbitrary octets; decode is the caller's decision |

No magic status integer (PDR-0019 ruling 8's last clause).

## 6. `Child`

`Child < Resource` — stream-protocol §3 laws apply (synchronous idempotent fallible
`close`, use-after-close raises `#useAfterClose`). The pid and pipe handles live in
U-RESOURCE's table row (`ResourceKind::Child`).

| Selector | Returns | Meaning |
|---|---|---|
| `pid` | `Number` | cached at spawn |
| `wait` | `Future` | blocking `waitpid` on a pool worker; settles `Ok(Output)` (empty captures unless piped) and reaps. **Occupies a worker until exit** — the PDR-0019 ruling-8 cost, restated wherever `wait` is taught |
| `tryWait` | `Option` of `Output` | `WNOHANG` poll; `None` while running; reaps on `Some`. Synchronous |
| `kill` | `Result` | `SIGKILL` via `kill(2)`; synchronous; never implicit |
| `stdin` | `Option` — a Writer (phase B) | `Some` iff spawned with `Stdio.piped`; a conformant Writer over the pipe (poller-backed, U-NET machinery unchanged) |
| `stdout` / `stderr` | `Option` — a Reader (phase B) | same, Reader side |
| `close` | `Result` | releases pipe handles, **detaches**: does not kill, does not reap |

**Laws:**

1. **Close is not kill and not wait.** A closed `Child` keeps running; killing and
   reaping are explicit sends (Rust/Go detach precedent — PDR-0019 ruling 8).
2. **An abandoned `Child` is reported distinctly.** Exited-unreaped is a zombie;
   `System.leakReport` names the pid and the spawn site as a condition distinct from
   an fd leak (PDR-0005 §5 / stream-protocol §7 posture).
3. **One pending `wait` per child** — a second concurrent `wait` raises
   `#concurrentOperation` (the PDR-0015 ruling-7 rule, same reason: two waiters, one
   reap, no defined answer for the second).
4. **Piped streams obey the stream protocol** — EOF on child exit is `read` settling
   `0`; a full pipe reports short writes; `#closed` on close-with-pending — all
   inherited, none restated.

## 7. Phasing

| Phase | Needs | Delivers |
|---|---|---|
| A | U-RESOURCE, U-REACTOR | §2 `System` rows, `Command` with `inherit`/`null`, `run` (captured via worker), `Child` minus the three stream accessors, `wait`/`tryWait`/`kill`, leak-report rows |
| B | + U-NET (poller) | `Stdio.piped` on `spawn`; `Child#stdin`/`stdout`/`stderr` as live streams |

Phase A's `spawn` **raises** on a `piped` `Stdio` (a loud seam); nothing in phase B
changes a phase-A signature.

## 8. Laws, consolidated

1. **Blocking is visible in the type, both ways** (PDR-0004 §1; PDR-0015 ruling 2's
   honesty): spawn/run/wait are `Future`s; env reads, `tryWait`, `kill`, `cwd` are
   synchronous.
2. **Display may be lossy; the pipeline never is** (ruling 4): every `String` form has
   a `Bytes` sibling, and syscalls see octets.
3. **The parent environment is immutable; the child's is data** (ruling 3/6).
4. **No shell, anywhere** (ruling 5).
5. **`Err` for the world, raise for the caller** (filesystem law 3): spawn failure
   (`#notFound`, `#permissionDenied`) settles `Err`; bad status range, `piped` in
   phase A, double `wait`, use-after-close raise.
6. **Nothing is implicit at the end of life**: no kill-on-close, no reap-on-collect
   (PDR-0005 §1 — no finalizers), and `System.exit` drains and reports before leaving.

## 9. Conformance harness

Phase-A rows run with `/bin/echo`-class fixtures; loopback-free, network-free.

| Check | Asserts |
|---|---|
| run round-trip | `Command.new("echo").arg("x").run` settles `Ok`; `stdout` is `x\n` bytes; `succeeded` |
| no pipe deadlock | a child writing >64KiB to stdout **and** stderr completes under `run` (§4's concurrency claim) |
| exit code / signal split | a nonzero-exit child: `exitCode` `Some`, `signal` `None`; a `kill`ed child: `exitCode` `None`, `signal` `Some(9)` |
| spawn + wait | `spawn` then `wait` settles the same `Output` shape; fiber parks and resumes via the queue |
| tryWait | `None` while running, `Some` after exit; reaps (no zombie in the leak report after) |
| double wait | second concurrent `wait` raises `#concurrentOperation` |
| close detaches | `close` then the child keeps running (observable via a file the child writes); `kill` after `close` raises `#useAfterClose` |
| zombie report | exited-unreaped child at exit: leak report names the pid as the distinct condition |
| env building | `clearEnv.env("K", to: "v")` child sees exactly `K`; parent env unread (`envBytes` round-trip on a non-UTF-8 value) |
| lossy/exact split | a non-UTF-8 env value: `env(_)` contains U+FFFD, `envBytes(_)` is exact |
| newAt vs new | `newAt(Path.of("/bin/echo"))` runs without `PATH`; `new("definitely-not-on-path")` settles `Err(#notFound)` |
| piped is loud (phase A) | `stdin(Stdio.piped).spawn` raises — the phase seam |
| exit obligation | `System.exit(3)` after opening a resource: leak report prints **before** exit; status is 3 |
| phase B: stream conformance | piped `stdout` passes the stream-protocol §8 Reader rows; child-exit EOF settles `read` to `0` |

## 10. Open questions

| # | Question | Notes |
|---|---|---|
| Q-P1 | Signals (`Signal.on`) | needs a signal-safe reactor completion source (signalfd / `EVFILT_SIGNAL`); the catalog's ready-queue-push idea is unsound (PDR-0019 ruling 10). Own record, after U-NET |
| Q-P2 | Non-pool `wait` (pidfd / `EVFILT_PROC`) | removes the worker-occupancy cost; platform-split and `mio` does not expose `EVFILT_PROC` — deferred with the cost named, not forgotten |
| Q-P3 | `Os` namespace (catalog §3.7) | no consumer, no ruling |
| Q-P4 | `executablePath`, `Command#stdin` feeding under `run` | wait for consumers |
| Q-P5 | Exact-bytes env *names* | the fourth selector nobody has needed yet |

## 11. What this document does not cover

- **Signals** (Q-P1), **process groups, sessions, detached daemons** — no design.
- **The worker pool and completion pipeline** — [`reactor.md`](reactor.md) /
  [`../impl/reactor.md`](../impl/reactor.md).
- **Pipe stream mechanics** — U-NET's poller machinery ([`net.md`](net.md) /
  [PDR-0016](../../../decisions/0016-poller-backend-is-mio.md)), reused unchanged in
  phase B.
- **A `Time` type, `Os` facts, resource limits** — elsewhere or nowhere yet.
