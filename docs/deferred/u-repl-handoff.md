You are continuing work on: U-REPL — finish the spec, tie the loose ends, and land it on `main`.

First: adopt /forge senior. Everything below is verified against the tree. Do NOT re-survey, and do NOT re-derive the Preconditions section of `docs/forge/units/U-REPL/plan.md` — it cost real time and several of its entries CORRECT earlier wrong assumptions (listed under DO-NOT-REPEAT below).

This is a spec-completion + merge task. **Write no implementation code.** The unit stays plan-only when you are done; the last step is landing the documents, not the feature.

## State

- `main` was at `77b7030` when this was written. It moves constantly — live concurrent sessions.
- Branch **`worktree-repl-cell-model`** (worktree at `.claude/worktrees/repl-cell-model`, tip `cabe8dd`), UNMERGED, 4 commits, 553 insertions, rebased onto `main`, builds green:
  - `bdb96bd` plan.md (evaluation substrate, §D1–§D10)
  - `46a6ca8` probe_continuation.rs (continuation evidence)
  - `1dd2df8` UNITS-TRACKER row
  - `cabe8dd` surface.md (§S1–§S6) + D4 rewrite + DEC-REPL-A closure
- Branch `worktree-core-table-regen` — already MERGED to `main` as `43df84f`. Worktree still on disk; remove it.

## Your tasks, in order

### 1. Close the three open items — take these recommendations, do not re-litigate

All three live in `surface.md` under "## Open". Rewrite each from an open question into a ruling, with its rationale, and delete it from the Open list.

- **DEC-REPL-B — snippet tab-stops.** RULE: degrade to inserting `name(` and placing the cursor inside. reedline has no tab-stop engine and the LSP emits snippet-format items; a real tab-stop implementation is its own unit, not a polish pass here.
- **DEC-REPL-C — dead editor stack.** RULE: delete `phalcom-repl/src/rustyline/` and drop the `rustyline` dependency from `phalcom-repl/Cargo.toml`, as the first step of implementation (not now — you write no code). Both `rustyline` and `reedline` are currently dependencies; `src/*.rs` is the live reedline path.
- **REPL command namespace.** RULE: spec `:reload`, `:reset`, `:help`; implement only `:reload`. `:reload` re-runs accumulated cells — the replay model rejected as the *persistence* mechanism in §D1, but sound as an explicit escape hatch.

### 2. Tie the loose ends

- **Remove the merged worktree**: `git worktree remove .claude/worktrees/core-table-regen`. Verify `43df84f` is an ancestor of `main` first.
- **Prune the stale worktree** at `/private/tmp/claude-501/.../wt/prev`, marked prunable in `git worktree list`. Not ours; `git worktree prune` is sufficient.
- **Raise the `completer.rs` collision.** Another session had `phalcom-repl/src/completer.rs` modified in the working tree. `surface.md` §S5 deletes that file's regex/`guess_type_from_name` logic outright. Check whether their change landed; if it did, note in `surface.md` that §S5 supersedes it, and say so to the user. Do NOT revert or edit their work.
- **`examples/simple.ph`** is broken and the user said they would handle it separately — a task chip exists. Do not fix it. Cause is settled if asked: `@construct` is `Target::Class` only (`compiler/attributes.rs:497`), so it is a class-level field derive, never a method annotation; the example attaches it to a method.

### 3. Land it

Merge `worktree-repl-cell-model` into `main`. Convention is a real merge commit (`--no-ff`) — 8 exist in history including prior `worktree-*` branches.

Before merging: rebase onto current `main` and expect a conflict in `docs/forge/UNITS-TRACKER.md`. It conflicted once already because `main` rewrote that file wholesale (re-audit at `de49d3a`). **Resolution rule: take `main`'s version of everything and re-apply only the single U-REPL row into §10.** `main`'s §10 header (`## 10. Tooling (\`vsphalcom\`, \`phalcom-lsp\`)`) is better than the branch's — keep `main`'s.

After merging, verify: `cargo build --workspace && cargo test --workspace`. Both were green at `cabe8dd`.

## Decisions already locked — do NOT reopen

From `plan.md`: §D1 persistent session module · §D2 cell-indexed source map · §D3 `CompileMode::Repl` echo + `_` · §D4 two-set immutability · §D7 EOF-routed continuation + trailing `\` + blank-line submit + Ctrl-C discard + `...` prompt · §D8 live oracle from the start · §D10 unwinding cell boundaries via `unwind_to(0,0)` · no new ADRs (rationale lives in rustdoc + named tests).

From `surface.md`: §S1 snapshot oracle (never a live borrow, never reflective sends on a keystroke) · §S2 static layer sees the current line only · §S3 own-before-inherited ranking · §S4 ghost text + value echo, signature hints deferred · §S5 three refine-only highlighting layers · §S6 two-tier latency.

**DEC-REPL-A is CLOSED** by `docs/decisions/0065-classes-are-closed.md` ruling 6 (REPL cells shadow; nothing migrates). That ruling explicitly instructs whoever merges this branch to mark it resolved rather than decide it again. Already marked in `plan.md`. Do not reopen.

**§S6 latency is a user ruling, quoted in the doc**: *"not laggy; it's fast, but it's okay if the REPL takes some time intentionally before giving suggestions."* Debounce is intended behavior, not a fallback. Do not "optimize" it into eager recomputation.

## DO-NOT-REPEAT — corrections already paid for

Each of these was asserted confidently and then falsified. The Preconditions section holds the corrected versions.

1. Hoisting the immutability set onto `ModuleObject` does NOT enable REPL rebinding — it makes the REPL **strict**, reproducing Wren's regret (`wrenDefineVariable` returns `-1` on redefinition, `resources/wren/src/vm/wren_vm.c:1575`). The fix is hoist **plus** a `CompileMode::Repl` exemption.
2. Frame-clearing between cells is NOT a fiber-lifetime risk. `FiberObject` owns private `stack`/`frames`/`open_upvalues` (`heap/fiber.rs:62`); suspended fibers survive and stay resumable. The real hazard is `open_upvalues` aliasing — `run_in_module` clears frames and stack but NOT that map (`interpret.rs:167`), and it is keyed by absolute stack index.
3. `gen-core-table` is NOT Rust-only and does NOT lack bracket support. It parses `core/core.ph` (`main.rs:57-62`) and has an explicit `ClassMember::Index` arm (`main.rs:129`). The checked-in artifact was simply stale; that is fixed and merged.
4. Constructors do NOT install an instance-side `init name(...)`. One method, class-side, under the ordinary selector (`new(_)`); `SignatureKind::Initializer` no longer picks the selector and survives only as a `SuperSend` marker (`compiler/lib/class_decl.rs:602`). Verified: `B.methods → [#x]`, `B.class.methods → [#new(_)]`.
5. Variadics ARE fully implemented — `static sum(*nums)` called `sum(1,2,3)` yields `[1, 2, 3]`. Zero variadic definitions exist in `core.ph`, which is why it looked unbuilt.

## Hazards

- **`Compiler`'s per-cell lifetime now carries three independent rulings**: const immutability, U-BINDINGS' same-scope redeclaration ban (`scope.rs:181`), and 0065 ruling 6's class shadowing (via 0066 registering class declarations in the same per-`Compiler` `global_bindings`). Nothing documents or tests this. The cross-cell regression test specced in `plan.md` is the only guard. Do not weaken that test's description.
- **U-BINDINGS already landed** (`b843fe2`, `42aafce`). `BindingKind` is `Let`/`Const`; `immutable_globals` no longer exists. §D4 was rewritten for this — if you find any doc still saying `immutable_globals`, it is stale.
- **main is 419 commits ahead of `origin/main`.** Every new worktree branches from the stale `origin/main` and lands ~400 commits behind; reset onto local `main` immediately after creating one.
- Live concurrent sessions on `main`. Verify before trusting any landed-state claim. Commit narrow paths. NEVER `git add -a`. NEVER `git checkout -b`.

## Out of scope — do not bundle

- Implementing any part of U-REPL. Stage order when someone does: §D2 source map, then §D7's parser half (EOF routing) — both standalone defect fixes that land value independently.
- `docs/deferred/core-table-inheritance.md` — the table encodes no superclass chains, so `ArgumentError` completes to nothing. Real, already filed, separate unit. The REPL is NOT blocked by it (§S1's snapshot walks live chains from the VM).
- U-TRACE. §D2 ownership vs U-TRACE is already ruled **split**: U-REPL owns where source lives (`source_id` on the artifact), U-TRACE owns when it resolves. Recorded in `plan.md` §D2.

Verify green with: `cargo build --workspace && cargo test --workspace && cargo clippy --workspace`
