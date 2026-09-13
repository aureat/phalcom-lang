# PDR-0019 — Process and environment: argument-vector spawning, a read-only environment, and `System` stays the one effect namespace

- Status: Proposed
- Date: 2026-07-20
- Related: [`system.md`](../spec/current/system.md) §2 (the specified-but-unbuilt
  `args`/`env(_)`/`exit(_)` rows this promotes — `primitive/system.rs` installs none of
  them at HEAD: `system_class_print`/`system_schedule`/`system_next_scheduled`/
  `system_gc`/`system_raw_write` and the `new` guard are the whole file),
  [`drafts/stdlib-catalog.md`](../spec/current/drafts/stdlib-catalog.md) §3.5/§3.6 (the
  `Env`/`Command` drafts this promotes and prunes; its S-5/S-13 questions are ruled
  here; its no-shell security note is promoted to a ruling),
  [PDR-0004](0004-io-is-future-shaped-reactor-owned.md) §1/§3 (Future shape; worker
  pool), [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §3-§5
  (`Child < Resource`; leak reporting),
  [PDR-0013](0013-path-is-bytes-backed-filesystem-surface.md) ruling 4 (the
  lossy-display / exact-bytes split, reused for env and args),
  [PDR-0015](0015-network-surface-tcp-dns-endpoints.md) ruling 4 (the two-selectors-
  not-a-union precedent), ADR-0043 (no options bags — the builder is the sanctioned
  shape).

## Context

`System.args`/`env(_)`/`exit(_)` have sat specified and unbuilt in system.md since
Draft 0.1, and the stdlib catalog drafted `Env`, `Command`, `Process`, and `Os`
namespaces around them with two structural questions deliberately left open: **S-5**
(do `System.args`/`env` forward to new surfaces or get deprecated?) and **S-13**
(lowercase modules or `System`-style classes?). Meanwhile two decisions changed the
ground under the drafts: U-REACTOR's worker pool makes `setenv` a live undefined-
behavior hazard (POSIX `setenv` racing `getenv`/`getaddrinfo` on pool threads —
glibc documents the crash), and the reactor gives child-process wait a place to block
that is not the VM thread.

## Decision

### 1. S-13 ruled: Tier-3 surfaces are classes with class-side methods

`Fs`, `Dns`, and now `Command` follow `System`'s shape: a class used purely as a
receiver of class messages. Lowercase ADR-0045 modules are rejected for v0.2 surfaces —
a module has no kernel-load-DAG edge where a class does (the catalog's own S-13
analysis), and splitting the convention would make every future surface re-ask the
question. One convention, settled once.

### 2. S-5 ruled: `System.env`/`args`/`exit` are *the* surface; no `Env` namespace

Phalcom has no deprecation mechanism, so shipping `Env.get` beside `System.env(_)`
means both forever (the catalog's own observation). The `Env` draft is pruned, not
promoted: its `get`/`all`/`args` fold into `System`; its `set`/`remove`/`setCwd` are
refused by ruling 3; `executablePath` waits for a consumer (Q-P4).

### 3. The environment is read-only in v0.2

No `setenv`, no `unsetenv`, no `setCwd`. This is not minimalism — it is soundness:
POSIX `setenv` concurrent with `getenv` on another thread is undefined behavior, pool
workers call `getenv` implicitly (`getaddrinfo` reads resolver configuration from the
environment on common libcs), and U-REACTOR makes those threads real. A lock cannot
fix it — libc's own internal `getenv` callers don't take our lock. Rust reached the
same conclusion and made `std::env::set_var` `unsafe` in the 2024 edition; that is
this ruling's precedent with its cost stated by someone else's decade of soundness
bugs. **Building a child's environment is unaffected** — `Command#env(_, to:)`
constructs the *child's* vector and never touches the parent's (ruling 6).

### 4. Env values and args get the lossy/exact split, not a guess

POSIX environment values and argv entries are arbitrary bytes; `String` enforces
UTF-8. PDR-0013 ruling 4's split is reused verbatim rather than inventing a third
posture:

```
System.env(_)      -> Option of String   // lossy decode (U+FFFD) — for humans and the 99% case
System.envBytes(_) -> Option of Bytes    // exact octets — the pipeline form
System.args        -> List of String     // lossy
System.argsBytes   -> List of Bytes      // exact
```

Display may be lossy; the pipeline never is. An env value that is a path goes
`envBytes` → `Path.ofBytes`, never through the `String` form — the spec documents the
idiom. Variable *names* are `String` (non-UTF-8 names exist in theory; a name you can
spell in source is UTF-8 — the exact-name case is not worth a fourth selector until a
consumer shows up, Q-P5).

### 5. Spawning takes an argument vector; there is no shell selector

The catalog's security note is promoted to a ruling: `Command.new(_)` takes a program,
`#arg(_)`/`#args(_)` take argv entries, and **no selector anywhere accepts a shell
string**. Every language that shipped a string-to-`/bin/sh` convenience (`system()`,
Python's `shell=True`, Node's `exec`) made shell injection the default-easy path and
bought a permanent CWE-78 population. If a shell escape is ever wanted it gets its own
record and a name that looks dangerous. Program lookup follows PDR-0015 ruling 4's
two-selectors-not-a-union precedent: `Command.new(_)` takes a `String` name resolved
against `PATH`; `Command.newAt(_)` takes a `Path` and executes exactly it, no lookup.

### 6. `Command` is a builder; `Stdio` is three singletons

Per-setting selectors (`arg`/`args`/`env(_, to:)`/`clearEnv`/`cwd(_)`/`stdin(_)`/
`stdout(_)`/`stderr(_)`), each returning the receiver — the builder is ADR-0043's
sanctioned alternative to an options bag, and `Stdio.inherit`/`.piped`/`.null` is the
`OpenMode` singleton pattern (PDR-0013 ruling 5) for the third time. The child's env
defaults to inheriting the parent's; `clearEnv` starts it empty.

### 7. `run` and `spawn` are pool-backed `Future`s; the unit is phased on `Stdio.piped`

`posix_spawn` resolves and loads an executable — disk IO — so both are `Future`s on
the worker pool (PDR-0004 §1/§3; no poller involvement for spawning itself).

- **`Command#run -> Future`** settles `Ok(Output)`: spawn, feed nothing, read stdout
  and stderr **concurrently on the worker** (interleaved with `poll`/`select` inside
  that worker — a >64KiB write to both pipes must not deadlock, and the harness proves
  it), reap, return the snapshot. Pool-only; **phase A**.
- **`Command#spawn -> Future`** settles `Ok(Child)`. **Phase A supports
  `Stdio.inherit`/`null` only.** `Stdio.piped` hands back live pipe streams — pipes
  are pollable descriptors, so piped `spawn` is **phase B, gated on U-NET's poller**,
  where `Child#stdin`/`stdout`/`stderr` become Reader/Writer-conformant streams on the
  PDR-0016 machinery unchanged. Phasing the unit beats blocking all of process on the
  network unit or hand-rolling a second pipe mechanism.

### 8. `Child < Resource`; close detaches, wait reaps, kill is explicit

- `Child#wait -> Future` — blocking `waitpid` on a pool worker (**the cost, named
  now**: each concurrently-awaited child occupies a worker for its lifetime; see
  Consequences). `Child#tryWait -> Option` — `WNOHANG`, synchronous, non-blocking.
- `Child#kill -> Result` — `SIGKILL`, synchronous (`kill(2)` does not block).
  Killing is never implicit.
- `Child#close` (from `Resource`) releases pipe handles and **detaches**: it does not
  kill (Rust's and Go's shape — kill-on-drop surprises every server that outlives its
  children) and does not reap. An exited-but-unreaped child is a zombie; an abandoned
  `Child` appears in `System.leakReport` as a **distinct condition** naming the pid —
  the PDR-0005 §5 / stream-protocol §7 posture (different bug than an fd leak,
  different fix).
- Exit status is never a magic integer: `Output#exitCode -> Number | None` (`None`
  when signal-terminated), `Output#signal -> Number | None`, `Output#succeeded ->
  Bool` (`exitCode == 0`). `Child#wait` settles the same `Output` shape (empty
  captures when streams were not piped).

### 9. The rest of `System`'s row, and `exit`'s obligation

`System.pid -> Number`; `System.cwd -> Result` of `Path` (`getcwd` is non-blocking —
synchronous by the PDR-0015 ruling-2 honesty — and fallible: a deleted cwd is real);
`System.exit(_)` takes an integral `0..255` (raise otherwise) and runs the full
shutdown sequence — reactor shutdown (reactor.md §8), resource-table drain, leak
report — before `exit(2)`. `exit` skipping the drain would make every leak report
untestable.

### 10. Signals are deferred wholesale

`Signal.on(_, handler:)` (catalog §3.6) needs a signal-safe path into the scheduler;
a signal context cannot push onto `VM::ready_queue` (single-threaded `VecDeque`,
PDR-0003 §3 — the catalog flagged this **unverified**, and it is in fact unsound: the
queue has no synchronization by design). The sound shape is signalfd/kqueue-EVFILT_SIGNAL
as a *reactor completion source* — which is poller territory, so it waits for U-NET's
machinery and its own record (Q-P1). Until then `SIGINT` keeps its default disposition.

### 11. Floor and phasing discipline

The ruled native family: `System.env_`/`envBytes_`/`args_`/`argsBytes_`/`pid_`/`cwd_`/
`exit_` (synchronous), `Command.spawn_`/`run_` and `Child.wait_` (registering — pending
future **last**, the impl/reactor.md §2.3 rule), `Child.tryWait_`/`kill_`
(synchronous), `ResourceKind::Child` in U-RESOURCE's table. Exact census at impl time
under PDR-0012 ruling 21 against the live
`floor_census_matches_installed_bindings`. Phase A needs U-RESOURCE + U-REACTOR only;
phase B additionally needs U-NET.

## Consequences

- system.md §2's process rows stop being aspirational: `args`/`env(_)`/`exit(_)` keep
  their spellings and gain their bytes siblings; system.md is swept to point here.
- **The cost, named plainly:** read-only env means no in-process `PATH`/locale
  mutation — test harnesses must set env via `Command#env(_, to:)` on children, never
  in-process; pool-backed `wait` means N concurrently-awaited children occupy N of
  Q-R2's bounded workers — a supervisor pattern over many children starves file IO,
  and the escape hatch (pidfd / `EVFILT_PROC` as poller sources) is platform-split and
  deliberately deferred (Q-P2) rather than half-built; phase A's `spawn` rejecting
  `Stdio.piped` is a visible seam until U-NET lands.
- **What this precludes:** a future `setenv` cannot be added without either isolating
  env reads from workers (an architecture change) or Rust-style `unsafe` framing that
  Phalcom has no spelling for — the door is closed, not ajar; `Env`/`Process`/`os`
  namespaces are foreclosed for v0.2 (ruling 1/2), and reopening them re-opens S-5's
  both-forever trap.
- The `Os` namespace (catalog §3.7) is untouched by this record — no consumer, no
  ruling (Q-P3).

## Alternatives rejected

- **A shell-string convenience.** CWE-78 by default; see ruling 5's precedent list.
  Rejected permanently absent its own loudly-named record.
- **Mutable env behind a VM lock.** The lock cannot cover libc's internal `getenv`
  callers (resolver, locale, timezone); the UB stands. Rust's 2024-edition `unsafe
  set_var` is the honest end state of trying.
- **`Env`/`Process` namespaces beside `System`.** S-5's both-forever trap with no
  deprecation mechanism; one effect namespace is system.md's founding design rule.
- **Kill-on-close.** Surprises long-lived parents (Go's and Rust's detach precedent);
  killing is a decision, not a side effect of scope.
- **Wait via SIGCHLD self-pipe or pidfd/EVFILT_PROC now.** The sound long-term shape,
  but platform-split (pidfd is Linux-only, `EVFILT_PROC` is not exposed by `mio`) and
  it drags signal handling (ruling 10) in early. Deferred as Q-P2 with the pool cost
  named, not rejected.
- **`Output#status` as one magic integer.** Signal-death encoded as negative-or-128+N
  is the C legacy every runtime regrets; two optional fields cost nothing.
- **Deferring the ruling.** `args`/`env` have been specified-unbuilt for a full spec
  cycle already, and every day of deferral is another chance for a unit to hand-roll
  env access against the mutable-env assumption ruling 3 forecloses.
