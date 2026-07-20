# Implementation specs

Dispatch-ready implementation plans for the surface specs in [`../core/`](../core/). One
file per unit; **each unit is built by its own agent in its own session** — a spec here
must therefore stand alone: every file it touches named, every tree pattern it copies
anchored `file:line`, hazards and test gates explicit, no reliance on this folder's
authoring context.

An impl spec whose governing record is still **Proposed** is blocked by
[`decisions/README.md`](../../../decisions/README.md) rule 5 until that record is
Accepted; the spec says so in its header.

## Unit board

Dependency-ordered. "Needs" = units that must be **shipped** first, not merely specced.

| Unit | Impl spec | Surface spec | Governing records | Needs | Status |
|---|---|---|---|---|---|
| U-BYTES | [`bytes.md`](bytes.md) | [`core/bytes.md`](../core/bytes.md) | PDR-0011 + PDR-0013 r.4 | — | ✅ **shipped 2026-07-20** (`19c5db9`/`9445d1f`/`732189b` + `5ba6101`); spec is the as-built record |
| U-RESOURCE | [`resource-table.md`](resource-table.md) | [`core/stream-protocol.md`](../core/stream-protocol.md) §3 | PDR-0005 §3-§5 | — | spec ready |
| U-PATH | [`path.md`](path.md) | [`core/filesystem.md`](../core/filesystem.md) §2-§3 | PDR-0013 | U-BYTES | spec ready |
| U-STREAMS | [`streams.md`](streams.md) | [`core/stream-protocol.md`](../core/stream-protocol.md) | PDR-0005 §7a | U-BYTES, U-RESOURCE | spec ready |
| U-REACTOR | [`reactor.md`](reactor.md) | [`core/reactor.md`](../core/reactor.md) | PDR-0004, PDR-0003 §3 | — (first consumer tests need U-BYTES) | spec ready |
| U-FS | [`filesystem.md`](filesystem.md) | [`core/filesystem.md`](../core/filesystem.md) §4-§6 | PDR-0005 §7, PDR-0013 | U-PATH, U-RESOURCE, U-REACTOR, U-STREAMS | spec ready |
| U-NET | [`net.md`](net.md) | [`core/net.md`](../core/net.md) | **PDR-0015 + PDR-0016 (both Proposed — rule-5 blocked)**, PDR-0004 §3/§4, PDR-0005 §3/§4 | U-RESOURCE, U-REACTOR | spec ready, **blocked on ratification** |

## Standing obligations (from as-built §7 of `bytes.md` — read it first)

1. New kernel class ⇒ `install_core`'s `add_class!` list (`vm/bootstrap.rs`), or core.ph
   shadows the natives-bearing row.
2. New heap arm ⇒ sweep **every** `Object::<Existing>` match site and decide each one;
   `matches!`/`if let` sites don't error on omission (`is_mutable_collection_key` was the
   one the file-by-file plan missed).
3. Block-taking selectors stay `.ph`; ordinary `f.call(...)` is flat-entry
   (`vm/send.rs`) and yield-transparent — never route a new feature's block invocation
   through re-entrant `block_call` when a bytecode send can carry it.

## Session rules for implementing agents

- `graphify query "<question>"` before grepping or reading source; rebuild with
  `graphify update . --no-cluster` after edits.
- `main` has live concurrent sessions: verify anchors against the tree, stage exact
  paths, **pathspec the `git commit` itself** (`git commit -- <paths>` — a bare commit
  sweeps whatever another session staged), never `git add -a`, never `git checkout -b`.
- Commit per green checkpoint; clean-worktree verify (`git worktree add … HEAD` +
  `cargo test`) before declaring the unit done.
- Full rustdoc on every added item (`docs/rust-documentation-guidelines.md`).
