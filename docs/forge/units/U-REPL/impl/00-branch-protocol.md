# §00 — Branch protocol: building U-REPL beside the class work

`main` is carrying U-CLASSNS → U-CLASSCLOSE. U-REPL is built on a side branch and
consolidated after. This section is the mechanism.

## The problem, stated exactly

Three units want the same five files.

| File | U-REPL wants | U-CLASSNS wants | U-CLASSCLOSE wants |
|---|---|---|---|
| `vm/dispatch.rs` | `unwind_to` public (§D10) | re-key `Bytecode::Class` arm at `:768,786` **in place** | **delete** `:768-788` |
| `compiler/lib/mod.rs` | unit-kind field (§D3), binding lookup (§D4) | `current_class: Option<ClassKey>`, `class_key()` | `global_bindings` doc `:57-61` |
| `compiler/lib/expr.rs` | const check at `:303` (§D4) | `field_layouts` reads at `:258`, `:316` | — |
| `chunk.rs` | *(done — §D2)* | — | two IC tests in `#[cfg(test)]` |
| `diagnostics.rs` | *(done — §D2)* | — | `line_col` extraction |

`dispatch.rs` is the sharp one: CLASSNS edits lines that CLASSCLOSE then deletes. Those
two are already strictly ordered by decision 0065 and must stay that way. U-REPL merging
into the middle of that sequence would rebase onto code in the act of being deleted.

There is also a **premise** collision, not just a textual one. U-CLASSCLOSE removes the
class-reopen seam that DEC-REPL-A reasons about; its own spec says "whichever lands
second rebases onto a changed premise. Flag at integration." Ruling 6 (cells shadow,
never reopen) already answers it, so U-REPL is written against the post-CLASSCLOSE
world from the start — see [§02 §1.4](02-session-and-cells.md).

## The strategy: shrink the branch's core footprint to zero

Do **not** branch the whole unit. Split it:

**Phase A — land the core-side surface on `main`, before or between the class units.**
Stages 0–2 ([§01](01-wiring.md), [§02](02-session-and-cells.md), [§03](03-immutability.md))
are the only parts touching `phalcom-core`. They are small, they are independently
useful, and none of them conflicts with the class units *semantically* — the overlap is
textual (same files, different regions) except for `dispatch.rs`, where U-REPL's need is
a **visibility change only** and touches no line CLASSNS re-keys or CLASSCLOSE deletes.

**Phase B — branch the rest.** Stages 3–6 ([§04](04-continuation.md),
[§05](05-oracle-and-selectors.md), [§06](06-surface.md), [§07](07-commands.md)) touch
`phalcom-repl/**` and nothing else. No class unit touches `phalcom-repl`. The branch is
then conflict-free by construction, not by careful merging.

This inverts plan.md's stage order, which put the surface last. The reason is purely
mechanical: plan.md ordered by *dependency*, this orders by *conflict*. The dependency
order still holds within each phase.

> If Phase A cannot land on `main` first — because the class work is mid-flight and the
> tree is not accepting core changes — then Phase A goes on the branch too, and
> [§08](08-consolidation.md)'s conflict-resolution table applies. That is the degraded
> path, not the plan.

## Branch mechanics

**Use a worktree. Never `git checkout -b`.** Every session in this repo shares one
working directory. Switching that directory's branch redirects other sessions' commits
onto your branch. This has bitten this repo before.

```sh
git worktree add ../phalcom-urepl -b u-repl
cd ../phalcom-urepl
git reset --hard main        # MANDATORY — see below
```

**The reset is not optional.** `git worktree add` branches from `origin/main`, which is
**450 commits behind** local `main` (verified 2026-07-19). A worktree that skips the
reset is building against a tree from weeks ago. Two subagents hit exactly this on
2026-07-19; both had it in their brief and it still needed catching. Verify before
writing a line:

```sh
git log --oneline -1     # must show dcd0567 or later
```

**Rebase cadence.** Rebase onto `main` whenever a class unit lands, not on a timer:

```sh
git fetch . main:main 2>/dev/null || true
git rebase main
```

For Phase B this is nearly always a no-op — `phalcom-repl/**` is untouched by the class
work. If a Phase B rebase produces a conflict, something has gone wrong with the
phase split; stop and re-read the write-set table above rather than resolving it.

## Commit discipline on the branch

- Commit **per green checkpoint**, not one batch at the end. Never commit a
  non-compiling tree.
- Stage **narrow explicit paths**. Never `git add -a` / `git add .` — the shared working
  directory routinely holds other sessions' uncommitted work.
- Each stage's gate (`cargo build && cargo test && cargo clippy`, all `--workspace`)
  must be green before the next stage starts. The stages are ordered so this is always
  possible; if a stage cannot be made green alone, that is a spec defect — report it.

## Test-count baseline

At `dcd0567` the workspace is **28 suites, 0 failures**, plus one pre-existing
`dead_code` warning on `VM::init_selector_cache` (`vm/mod.rs:243`) that is **not yours**.

A shared working directory may report **29** — another session's uncommitted `testing/`
crate is a workspace member in their `Cargo.toml`. In an isolated worktree the honest
number is 28. Do not chase the difference.
