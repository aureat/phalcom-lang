# U-CLASSNS LSP half: `ClassMap` collapsed to `(Url, name)`, plus a regression the fixtures caught

- Date: 2026-07-20
- Scope: `phalcom-lsp` only — `index.rs`, `completion.rs`, one line of `backend.rs`
- Realizes: [U-CLASSNS implementation-spec.md](../forge/units/U-CLASSNS/implementation-spec.md)
  §8 ("LSP — collapse, do not just re-key"), the last unbuilt piece of
  [PDR-0001](../pdr/0001-classes-are-closed.md)
- Closes: [`docs/deferred/class-sealing-followups.md`](../deferred/class-sealing-followups.md)
  item 7, filed earlier the same day
- Related: [2026-07-20-u-classclose-two-issues-and-five-restored-tests.md](2026-07-20-u-classclose-two-issues-and-five-restored-tests.md)
  (the VM half this completes)

## 0. Why it was safe to do now

Worth recording, because the tree was *not* quiet at the time.

A concurrent session had an uncommitted in-flight refactor in the working tree —
`phalcom-core/src/diagnostics.rs` → `diagnostics/mod.rs`, two new untracked modules, and a
workspace `Cargo.toml` edit adding `unicode-width` and removing `miette`. Building the workspace
would have meant building someone else's half-finished state and misattributing any red.

It was safe anyway, for one reason that is worth stating precisely rather than assuming:
**`phalcom-lsp` does not depend on `phalcom-core`.** Its `Cargo.toml` says so in a comment —
"Front end only — no `phalcom-core`. See ADR-0056 §2" — and its only workspace deps are
`phalcom-ast` and `phalcom-common`, neither of which the other session had touched. `miette`'s
removal turned out to reach nothing: zero references in any of the four crates' manifests.

Checks actually run before the first edit:

1. `phalcom-lsp/Cargo.toml` — confirmed no `phalcom-core` dependency.
2. `git log -5 -- phalcom-lsp/` — last touch was `42aafce` (U-BINDINGS), several commits back. No
   concurrent work in this crate.
3. `cargo build -p phalcom-lsp` on their dirty tree — **green**, establishing a baseline that
   was not mine to break.

So verification stayed scoped to `-p phalcom-lsp` throughout and never compiled `phalcom-core`.
Staging was `git add phalcom-lsp/ docs/` — never `-a`.

## 1. The change

```
DashMap<String, Vec<ClassEntry>>   →   DashMap<(Url, String), ClassEntry>
```

`ClassEntry` loses its `uri` field: the file is now half the key rather than a payload. The three
public accessors — `class_members`, `class_parent`, `has_class` — each grow a `uri` parameter, as
do the four inherent `ClassMap` methods behind them. `collect_class_members` threads it through
the inheritance walk, and `completions` takes it from `Backend::completion`, which already had the
request `uri` in scope one line above the call.

The `Vec` is deleted rather than re-keyed, which is the part of §8 worth not glossing: it existed
**solely** to model one class reopened across several files. U-CLASSCLOSE removed class
reopening, so it no longer modelled anything real — it only merged genuinely distinct classes
that happened to share a name.

`Url` is the correct module proxy here and the only one available: a file is a module (ADR-0045),
this crate never resolves `import` (`Statement::Import` is a no-op in every walker), and
`ClassEntry` already carried the `uri`.

## 2. The two live bugs this fixes

Both were reachable in an ordinary two-file workspace and neither had a test.

**Members unioned across files.** `members()` walked every entry under a name and de-duplicated
first-seen-wins, so `p.<cursor>` in `a.ph` offered `b.ph`'s same-named class's members.

**Parent read from the wrong file.** `parent()` was
`entries.iter().find_map(|e| e.parent.clone())` — the *first entry that named any superclass*
answered for every file. A parentless `Point` in `a.ph` inherited `b.ph`'s `Point extends Shape`.

Add a third, found while writing the fixtures: `contains()` took only a name, so `has_class`
returned `true` for a class declared in any open file, not the one being edited.

## 3. A regression the fixtures caught, and the distinction it forced

This is the part worth reading.

`collect_class_members` walks `extends` up to the first builtin ancestor. File-scoping every hop
means a superclass declared in *another* file is no longer resolvable — correct, since it is a
different class and unnameable here without import resolution. But the function's own doc comment
carries a stated invariant:

> so `Object`'s selectors (`==`, `hash`, `isNil`, …) are always eventually walked, closing the
> "implicit `Object` never offered" gap

Scoping broke it. `class Dog is Animal` with `Animal` in another file now walked `Dog`, found
`Animal` unresolvable, and **terminated** — dropping the entire builtin surface. `inheritance_walk_does_not_cross_files`
failed on exactly that assertion. Restoring the invariant is defending documented behavior, not
scope creep, so the walk now falls back to `IMPLICIT_ROOT_CLASS` on an unresolvable parent.

**That first fix was wrong, and a pre-existing test caught it.**
`completions_unknown_receiver_falls_back_to_full_builtin_surface` went red: expected 181 items,
got 20. Falling back to `Object` unconditionally also caught the *unknown receiver* case —
`completions()` treats a non-empty collection as "we resolved something", so a completely
unresolvable receiver started returning `Object`'s handful instead of the full builtin surface.
Strictly worse than before the fix, and precisely the graceful degradation that test exists to pin.

The two cases needed separating, which the final shape does with one flag:

| Situation | Wanted | Mechanism |
|---|---|---|
| Starting class unresolvable (`Nonexistent.<cursor>`) | return empty → `completions` serves the **full** builtin surface | `walked_user_class == false` ⇒ terminate |
| Mid-chain parent unresolvable (`Dog extends Animal`, elsewhere) | keep `Dog`'s members, still walk `Object` | `walked_user_class == true` ⇒ fall back to root |

Both behaviors now have a test. The lesson is the ordinary one: the invariant was written down in
a doc comment, and only a fixture noticed it had been broken — twice, in opposite directions.

## 4. Deliberate behavior change

Cross-file inheritance completion stops working, and that is the fix rather than a regression.
Previously `class Dog is Animal` in `a.ph` would pick up an `Animal` from `b.ph` — an
unrelated class — and offer its members. Under `(module, name)` identity those are two different
classes, and the LSP cannot resolve the `import` that would make a real cross-module superclass
nameable. So the walk stops at the file boundary and falls through to the builtin surface.

If a user genuinely imports a superclass, completion under-offers rather than wrongly-offers.
That trade is the right way round and is what §8 asked for, but it *is* a visible change and
should not surprise anyone reading a bug report about it later.

## 5. Verification

`cargo test -p phalcom-lsp`: **113 green across all targets, 0 failed** — 101 lib (95 before, +6)
plus 12 in the integration targets.

`cargo clippy -p phalcom-lsp`: **no warning in `phalcom-lsp`.** One warning is emitted in the
build, "useless conversion to the same type: `std::ops::Range<usize>`", and it is attributed to
**`phalcom-ast`** — a crate this change does not touch, so it is pre-existing and inherited
through the dependency. Stated this way deliberately: "clippy clean" would have been true of the
crate and misleading about the build.

`phalcom-core` was never built — see §0.

**Negative control, run rather than asserted.** Six new fixtures claim to fail on the pre-collapse
index. Claiming that is worthless, so it was checked: `ClassMap::{members,parent,contains}` were
temporarily rewritten to ignore the `uri` and scan by name across all files — the old semantics,
in the new shape — and the suite re-run. All six failed:

```
completion::tests::completions_do_not_leak_a_same_named_class_from_another_file ... FAILED
completion::tests::inheritance_walk_does_not_cross_files ... FAILED
index::tests::has_class_is_scoped_to_the_declaring_file ... FAILED
index::tests::same_class_name_in_two_files_does_not_merge_members ... FAILED
index::tests::removing_one_files_class_leaves_the_same_name_in_another_file ... FAILED
index::tests::same_class_name_in_two_files_does_not_share_a_parent ... FAILED
```

The simulation was then reverted and the suite re-run green. Six-for-six in both directions, so
the fixtures discriminate the actual behavior change and none of them is vacuous.

New fixtures:

| Test | Pins |
|---|---|
| `same_class_name_in_two_files_does_not_merge_members` | bug 1 |
| `same_class_name_in_two_files_does_not_share_a_parent` | bug 2 |
| `has_class_is_scoped_to_the_declaring_file` | bug 3 |
| `removing_one_files_class_leaves_the_same_name_in_another_file` | invalidation is keyed too, not just reads |
| `completions_do_not_leak_a_same_named_class_from_another_file` | bug 1 end-to-end through completion |
| `inheritance_walk_does_not_cross_files` | §4's behavior change **and** the §3 `Object` invariant |

## 6. Not done

- `core_table.rs`'s `classes: HashMap<String, Vec<CoreMember>>` is untouched, per §8's explicit
  "out of scope". Kernel classes are process-global and the VM has no per-module identity for them
  either, so it is legitimately name-keyed.
- `hover.rs` was not audited for the same name-keyed assumption. It does not call the three
  accessors changed here, but it has its own `DefinitionInfo.class` field carrying a bare class
  name, and `definition_meta` is still keyed by selector alone. **Whether hover can show the wrong
  file's class for a duplicated selector is an open question this change did not answer** — it is
  a different map with a different key, and it was out of scope.
