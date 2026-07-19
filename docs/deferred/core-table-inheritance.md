You are continuing work on: core-table.json encodes no superclass chains, so completion on any subclass receiver under-offers — and nothing detects when the generated table goes stale.

First: adopt /forge senior. Start from the entry points below — do NOT re-survey, and do NOT re-audit the 36 removals from the regeneration; they were each confirmed and are listed under VERIFIED CLEAN.

Done so far: `3e2f105` on worktree branch `worktree-core-table-regen` (UNMERGED) regenerated `tools/vsphalcom/src/generated/core-table.json` — 30 → 52 classes, 214 → 305 selector entries. `phalcom-lsp` builds (the JSON is embedded via `include_str!`), its 95 tests pass, full workspace green. No other file touched. Neither item below was built — both are proposals only.

CENTRAL FINDING:
The table is FLAT. Each class maps to a list of entries carrying only `{kind, selector, source}` — no parent link. Stated in the code itself:
  - phalcom-lsp/src/core_table.rs:88 — "builtin superclass chains are not encoded in the [table]"
Consequence, measured on the freshly generated table:
  - `ArgumentError extends Error`   -> 0 entries   (completing on an ArgumentError offers NOTHING, not even `message`)
  - `Ok extends Result`             -> 2 entries
  - `MapView extends Iterable`      -> 3 entries
  - `WhereView extends Iterable`    -> 3 entries
This is PRE-EXISTING and independent of the regeneration. The generator does not flatten inherited members even when `core.ph` declares `extends` explicitly.

WHAT THE REGENERATION CHANGED ABOUT IT:
ADR-0048's Iterable rehome moved `each/iterate/map/filter/reduce/includes/isEmpty` off the concrete collections onto the `Iterable` root. Because the table is flat, `List`/`Map`/`Set`/`Range`/`Tuple` each lost ~7 user-facing selectors from completion. The stale table had them flattened in by accident of predating the rehome. So the regeneration did not create the defect — it moved the five most-used classes into it. This is the one real downside of `3e2f105` and the reason this file exists.

WHY A PARSE-ONLY GENERATOR CANNOT FIX IT:
  - phalcom-core/core/core.ph:779 — `class List {` declares NO `extends`. Same for Map (:866), Set (:975), Tuple (:1043), Range (:1117).
  - phalcom-core/src/universe/core_classes.rs:105 — `Iterable` is constructed in Rust; the real parent wiring lives in the bootstrap, not in `.ph` source.
  - phalcom-core/bin/gen-core-table/main.rs:57-62 — the generator only `parse_source`s `core/core.ph`.
So the generator structurally cannot see List's true superclass today. Emitting a `superclass` field from parsing alone would be correct for `Ok`/`ArgumentError`/the views and WRONG (silently absent) for exactly the five classes that matter most.

THE OPENING:
`gen-core-table` is a bin INSIDE `phalcom-core` (phalcom-core/Cargo.toml:11-12), so it may link the VM freely — boot the universe and walk the real class table for exact parents, including the Rust-wired ones. ADR-0056 §2 constrains `phalcom-lsp` to stay VM-free; it does NOT constrain the generator. The LSP keeps consuming plain JSON and stays VM-free either way. Do not let the ADR talk you out of this — check §2's actual scope before deciding it blocks you.

Next step(s), in order:
  1. Decide representation (see Open decisions) before writing anything.
  2. Teach `gen-core-table` to boot the VM and resolve real superclass chains.
  3. Emit the chain; update `phalcom-lsp/src/core_table.rs` to consume it.
  4. Add the drift gate (proposal A below) in the same pass — it is 5 lines and its absence is why this rotted.

PROPOSAL A — drift gate (5 lines, not built):
Nothing has ever checked this file. `gen-core-table` appears in no script and no CI config; `scripts/verify.sh` exists and is the hook point. Add:

    tmp=$(mktemp)
    cargo run -q -p phalcom-core --bin gen-core-table -- "$tmp"
    diff -u tools/vsphalcom/src/generated/core-table.json "$tmp" || {
      echo "core-table.json is stale — regenerate it"; exit 1; }

Safe to gate on: generator output is DETERMINISTIC — verified two consecutive runs byte-identical, and the committed file byte-matches a fresh run. Cost is one debug-build run.

VERIFIED CLEAN — do not re-audit:
  - All 36 removals in `3e2f105` are legitimate, none a harvesting regression:
      * 21 `raw*` -> renamed to trailing-underscore (`at_(_)`, `length_`, `push_(_)`, `set_(_,_)`, ...).
        The only "raw" left in core.ph is prose inside comments (5 hits, all comments).
      * 14 `each/iterate/map/filter/reduce/includes/isEmpty` -> rehomed to `Iterable` per ADR-0048.
      * 1  `System.print()` -> retired; `print(_)` remains.
  - Absence of `init `-prefixed entries is CORRECT, not drift. A `construct` installs one method,
    class-side, under its ordinary selector (`new(_)`); `SignatureKind::Initializer` no longer picks
    the selector and survives only as a `SuperSend` marker
    (phalcom-core/src/compiler/lib/class_decl.rs:602). Verified: `B.methods -> [#x]`,
    `B.class.methods -> [#new(_)]`.
  - Bracket selectors are real and correctly harvested post-regen (`List[_]`, `List[_,put]`, `Map[_]`,
    `Map[_,put]`, `Tuple[_]`) — ADR-0060 made `[]` a directly-sent selector; core classes opt in by
    forwarding (`[i] { return self.at(i) }`, core.ph:816).

Open decisions:
  - `superclass` field + LSP walks the chain, VS flatten inherited members at generation time.
    RECOMMEND the field: flattening duplicates entries and loses override semantics — a subclass
    overriding a parent selector must appear ONCE, not twice. Flattening also silently doubles file size.
  - Whether the LSP should visually distinguish inherited from own members in the completion list
    (`MemberKind` has no such axis today).
  - Whether `source: "native"` vs `"core.ph"` (already on every entry) should extend to carry the
    DEFINING class once chains exist, so the LSP can render "from Iterable".

Constraints / invariants / gotchas:
  - Rust docs are MANDATORY: `//!` on every module, `///` on every public item incl. fields and enum
    variants. See docs/rust-documentation-guidelines.md.
  - main has LIVE CONCURRENT SESSIONS — it moved twice mid-session while this work was done
    (301044e -> f7f481b). Verify against the tree before trusting any landed-state claim. Commit narrow
    paths. NEVER `git add -a`. NEVER `git checkout -b` (it hijacks other sessions' commits).
  - `3e2f105` is UNMERGED on `worktree-core-table-regen`. Land or rebase it before regenerating again,
    or you will regenerate against a stale base and produce a confusing diff.
  - The table is embedded at build time via `include_str!` — a malformed regeneration breaks the
    `phalcom-lsp` BUILD, not just its tests. Build the LSP after any generator change.
  - Do not re-derive the removal audit. It is above, and it cost real time.

Verify green with: cargo build --workspace && cargo test --workspace && cargo clippy --workspace
