# LANG004.C1.P2 — Selector shapes and family activation

## Repository state

- HEAD: `347b4d7bf2505176a861feea84828256abe2a129`
- Branch: `main`
- Implementation began with the supplied P2 plan untracked in the working
  tree and unrelated user edits in `docs/.obsidian/workspace.json` and
  `docs/spec/reflection/Untitled.md`; those files remain preserved.
- P2 implementation changes are currently uncommitted. No commit or push was
  requested.

## Established invariants

- **I-01:** named and subscript setters encode canonically as `=(_)`; the
  setter value is not a selector slot.
- **I-02:** setter declarations accept exactly one positional value binder.
- **I-03:** subscript setter slots contain only index/key positions and labels.
- **I-04:** ordinary positional-before-label validation remains unchanged.
- **I-05:** indexed assignment evaluates receiver, index values/labels, then
  the RHS, each once.
- **I-06:** indexed assignment produces the original RHS.
- **I-07:** named references distinguish exact Getter, Setter, Method, named
  accessor pattern, Method pattern, and whole named-family pattern.
- **I-08:** subscript references distinguish SubscriptGet, SubscriptSet, and
  the corresponding subscript kind patterns.
- **I-09:** operators remain Method selectors, including operator bases that
  contain `=`.
- **I-10:** prefix `&` preserves the written selector shape and evaluates its
  receiver once.
- **I-11:** exact Getter capabilities use `get()`/`value`; `()` remains Method.
- **I-12:** `value`/`value=(_)` are named accessor aliases for `get()`/`set(_)`.
- **I-13:** subscript Family APIs accept a real Tuple/product, with setter RHS
  outside the tuple shape.
- **I-14:** exact singleton references are one-entry Getter-shaped associated
  Family capabilities; direct singleton lookup remains a value.
- **I-15:** Family construction does not invoke or probe the captured target.

## Decisions

1. `CallableReferenceTarget` uses one shared bound/associated target shape with
   named, operator, and subscript member syntax rather than parallel target
   enums.
2. `SelectorKindPattern::NamedAccessors` and `AnySubscript` are the only new
   kind sets; selector identity remains owned by `phalcom-common`.
3. Dynamic subscript assignment uses a dedicated setter-pack bytecode. The
   RHS is evaluated separately and is never inserted under a synthetic label.
4. `Family` exposes `get()`, `set(_)`, `value`, `value=(_)`, `get(Tuple)`, and
   `set(Tuple, value)` through native shape-aware gateways.
5. Associated-family activation remains descriptor-restricted and does not
   route through live bound-receiver lookup.

## Evidence ledger

| Area | Evidence | Result |
|---|---|---|
| Common selectors | `cargo test -p phalcom-common` | Green: 29 unit, 7 family-pattern, 8 selector, and 2 doctest cases |
| AST/parser | `cargo test -p phalcom-ast --test integration` | Green: 203 tests, 1 ignored |
| Semantic family/reference | `cargo test -p phalcom-semantic --test semantic families:: -- --nocapture` | Green: 16 tests |
| Bound subscript reference | focused semantic test | Green |
| Bound Family subscript lanes | focused core runtime test | Green |
| Named Family aliases | focused core runtime test | Green |
| Tuple subscript Family APIs | focused core runtime test | Green |
| Singleton associated reference | focused core runtime test | Green |
| Bound/Family runtime slice | `cargo test -p phalcom-core --test core execution_family_runtime -- --nocapture` | Green: 17 tests |
| Workspace check | `cargo check --workspace --all-targets` | Green |
| Core full suite | `cargo test --workspace --all-targets --no-fail-fast` | Core target green: 465 passed, 24 ignored; the earlier standalone typing failures did not reproduce in the aggregate run |
| Workspace Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Green |
| Format and whitespace | `cargo fmt --all -- --check`; `git diff --check` | Green |
| Native surface drift | `cargo run -p phalcom-native-surface-gen -- --root . --check` | Green: 332 primitive declarations |
| Reflection census | `cargo test -p phalcom-core --test core reflection_census -- --nocapture` | Green: 6 tests |
| Family floor census | `cargo test -p phalcom-core --test core object_model_invariants::floor_census_matches_installed_bindings -- --nocapture` | Green |
| LSP integration | `cargo test -p phalcom-lsp --test integration -- --nocapture` | Green: 58 tests |
| Language corpus family/index lanes | `cargo test -p phalcom-core --test language-corpus family -- --nocapture`; `indexing_negative` | Green after migrating stale whole-family and setter diagnostics |
| Semantic full suite | `cargo test --workspace --all-targets --no-fail-fast` | Semantic target green: 1140 passed, 42 ignored |

## Deferred gates

- The workspace aggregate test gate is not fully green because the unrelated
  REPL import targets fail: `repl_imports` has 12 failures and `repl_phase_b`
  has 2 failures around stale reflection exports and the removed `std` builtin
  package. The P2-related core and semantic targets are green in that run.
- Historical/as-built documents retaining the old spelling must remain clearly
  classified; current normative selector/family/governance text has been
  migrated.

## Active incident

None. The initial runtime failure was an invalid test path that used the
standalone interpreter instead of the semantic-to-lowering program pipeline;
the test now uses the compiled program path and passes.

## Next resume action

Review the scoped diff and classify the remaining historical old-spelling
documents. Preserve unrelated worktree files and do not commit unless
requested.
