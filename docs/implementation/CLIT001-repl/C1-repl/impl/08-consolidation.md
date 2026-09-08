# §08 — Consolidation: merging the branch back

How the `u-repl` branch rejoins `main` after the class work lands, and what "done" means.

## 1. Exit criteria

The unit is complete when all of the following hold. Not before, and no partial credit
for a working demo.

- [ ] Stages 0–6 landed, each green at its own gate.
- [ ] `cargo build --workspace && cargo test --workspace && cargo clippy --workspace` green
      on the merge result, not just on the branch.
- [ ] The **six load-bearing tests** below all exist and all fail against the
      implementation they guard against.
- [ ] `docs/forge/UNITS-TRACKER.md`'s U-REPL row updated; the unit's checkbox flipped.
- [ ] `../plan.md` §D3's `CompileMode::Repl` wording corrected to `UnitKind`
      ([README delta 2](README.md)), or annotated as superseded by this directory.
- [ ] `../surface.md`'s stale concurrent-edit note removed ([README delta 4](README.md)).

### 1.1 The six tests that must be able to fail

A test that passes against both the correct and the broken implementation is not a guard.
For each of these, **write it, break the implementation, watch it go red, restore**:

| Test | Stage | Breaks against |
|---|---|---|
| `open_upvalue_hygiene_across_cells` | [§02](02-session-and-cells.md) | `unwind_cell()` replaced by a raw `clear()` |
| `const_redeclares_across_repl_cells` | [§03](03-immutability.md) | exemption that relaxes only the const check |
| `const_assignment_errors_in_file_mode` | [§03](03-immutability.md) | exemption leaking into `File` units |
| `validator_matches_probe_classification` | [§04](04-continuation.md) | validator drifting from the parser's rule |
| `value_echo_survives_raising_tostring` | [§06](06-surface.md) | echo turning a good cell into a failed one |
| `reload_survives_declarations` | [§07](07-commands.md) | `:reload` reusing the session `Compiler` |

## 2. Merge order

**If Phase A landed on `main` as planned** ([§00](00-branch-protocol.md)), the branch
contains only `phalcom-repl/**` and the merge is mechanical:

```sh
git checkout main          # in a worktree, never in the shared working directory
git merge --no-ff u-repl
```

No conflicts are expected. If one appears in `phalcom-core`, the phase split was
violated — investigate rather than resolve.

**If Phase A stayed on the branch** (the degraded path), resolve in this order and
re-verify after each:

| File | Conflict | Resolution |
|---|---|---|
| `compiler/lib/mod.rs` | U-CLASSNS's `current_class`/`class_key()`, U-CLASSCLOSE's doc edit, U-REPL's `UnitKind` + binding lookup | Keep all. Separate fields, separate concerns. Mechanical. |
| `compiler/lib/expr.rs` | CLASSNS `field_layouts` at `:258`/`:316`, U-REPL const check at `:303` | **Not mechanical.** Interleaved region. Resolve by hand, then re-run all five §03 tests — a merge can compile and check the wrong set. |
| `vm/dispatch.rs` | CLASSNS re-keys `:768,786`; CLASSCLOSE deletes `:768-788`; U-REPL adds `unwind_cell` in `vm/api.rs` | U-REPL's change is in a **different file**. If it drifted into `dispatch.rs`, move it. |
| `vm/api.rs` | CLASSNS's `create_class` module param at `:76-77`; U-REPL's `unwind_cell` | Keep both; different regions. |
| `phalcom-ast/src/parser.rs` | CLASSCLOSE's nested-class rejection `:1557-1559`; U-REPL's optional shared `classify` | Keep both. If `classify` was not extracted ([§04 §6](04-continuation.md)), no conflict. |

## 3. The premise collision — resolved by construction

U-CLASSCLOSE removes the class-reopen seam DEC-REPL-A reasoned about; its spec says
"whichever lands second rebases onto a changed premise. Flag at integration."

**This is flagged, and it is already handled.** [§02 §1.4](02-session-and-cells.md)
writes the cell loop against PDR-0001 ruling 6 (cells shadow, never reopen) — the
post-CLASSCLOSE world — from the start. Whichever unit lands second, U-REPL's behavior
is identical and nothing rebases.

Confirm at merge with `class_redefinition_shadows` ([§02 §4](02-session-and-cells.md)):
cell 2's `class Foo` is a new class; a cell-1 instance keeps the old one. If that test
passes on the merge result, the collision is closed.

## 4. Verify the merge result, not the branch

An in-tree gate on the branch hides partial-stage commits. After merging, verify at the
merge SHA in a throwaway worktree:

```sh
git worktree add /tmp/verify-urepl <merge-sha>
cd /tmp/verify-urepl && cargo build --workspace && cargo test --workspace
git worktree remove /tmp/verify-urepl
```

Also spot-check that **each commit** on the branch compiles — `git show --stat` per
commit, and build any that touched `phalcom-core`. A branch whose tip is green can still
contain a commit that never compiled, which makes future bisects useless.

## 5. What this unit must not preclude (P4)

Carried forward from plan.md. Check each at consolidation:

- **Fibers.** §D10 leaves suspended fibers resumable across cells (precondition 5).
  Nothing here assumes single-fiber execution.
- **The `const`/`let` rename.** No identifier introduced by this unit encodes a keyword.
  (`UnitKind::File` / `UnitKind::Repl` comply.)
- **A REPL-as-server model.** Nothing in §D8 assumes the oracle is in-process; the merge
  rule survives the REPL becoming a server the editor queries (the Clojure/nREPL shape).
- **`core-table.json` regeneration.** The selector conformance test is live-authoritative
  precisely so the two units never block each other. It must not acquire a dependency on
  that file.
- **A tab-stop engine.** §S7 ships the degraded form deliberately; nothing here forecloses
  a real one as its own unit.
- **U-TRACE.** §D2 settled *where* source lives; U-TRACE owns *when* it resolves. This
  unit must not implement lazy resolution.

## 6. After the merge

Update, in the same pass — not deferred:

- `docs/forge/UNITS-TRACKER.md` — U-REPL row and checkbox.
- `CLAUDE.md`'s `phalcom-repl/src` description, if the module set changed (it will have:
  `snapshot.rs` and `oracle.rs` are new, and `common.rs` may be gone).
- This directory's [README](README.md) status table — mark the remaining stages landed
  with their commits, so the next reader sees the same shape the first three stages have.
