# Phalcom Core Takeover — Checkpoint Report

Baseline for this patch: the agent's uploaded in-progress worktree, preserved at
`/mnt/data/phalcom_review_before_takeover`.

Working copy: `/mnt/data/phalcom_takeover_core`.

This is **not** a completion claim. Rust/Cargo are unavailable in the current
sandbox, so these changes have only received source-level/static verification.
The patch intentionally contains only the takeover delta, not the agent's
pre-existing dirty changes.

## Implemented/corrected in this checkpoint

1. **Transitive SemanticDb reuse validation**
   - `SemanticDb::is_reusable` now validates the entire recorded dependency DAG.
   - A dependent is not considered reusable merely because its immediate
     dependency still stores an old matching product fingerprint.
   - Dependency cycles are conservatively rejected.

2. **Required dependency recording no longer silently fails**
   - Query publication now treats unavailable/stale required dependencies as
     explicit query failures instead of `let _ = record_dependency(...)`.

3. **Exact-input invalidation**
   - Added `invalidate_exact` for ordinary direct-input changes.
   - Dependents remain stored and are validated transitively.
   - If a recomputed prerequisite has the same product fingerprint, downstream
     products can become reusable again without destructive reverse-closure
     invalidation.

4. **Semantic fingerprint strengthening**
   - Expanded semantic hashes to cover previously omitted contract/body state.
   - Added focused fingerprint tests, including:
     - body-only edits preserving unlinked-interface meaning;
     - source provenance not altering declaration-surface semantic identity;
     - effect contracts altering callable-signature identity;
     - unknown signature states remaining distinguishable;
     - binding type changes altering callable-body identity.

5. **Callable-signature query product**
   - Added `CallableSignatureProduct` so DB invalidation can preserve full
     `Known` / `Unknown` / `Dynamic` signature knowledge instead of relying only
     on the stricter reflection-oriented semantic signature projection.

6. **Session begins using staged queries**
   - Changed-source handling now invalidates only `ParsedModule`.
   - `query_unlinked_interface` is invoked to bring parsed/interface products
     current.
   - Session now routes linked-interface, hierarchy, declaration-surface, and
     callable-signature publication through query APIs in the modified slices.
   - The full session is **not yet** converted to compiler-owned module lifecycle.

7. **Constructor `Self` semantics — substantial partial implementation**
   - Source constructors now model:
     - public class-side factory under the written selector returning instance
       `Self`;
     - hidden instance-side initializer under `init <name>`.
   - Added canonical constructor-body callable-ID helper.
   - Added recursive `Self` specialization across applied/union/tuple/record/
     callable types.
   - Added regression covering:
     - `Base.new() -> Base`;
     - inherited `Derived.new() -> Derived`;
     - ordinary inherited `-> Base` remains `Base`;
     - constructor body uses hidden initializer identity.
   - This needs Cargo compilation before acceptance.

8. **Overlay SourceProvider safety repair**
   - Replaced two independently ordered locks with one `RwLock<OverlayState>`.
   - Module/source reverse indexes update atomically.
   - Replacing an overlay removes stale old `SourceId` mappings.
   - Added tests for replacement and removal fallback behavior.

9. **TypeStore revision test repaired**
   - The old test interned a type *after* snapshot publication and therefore did
     not prove snapshot TypeId stability.
   - The test now captures a TypeId actually present in revision-1 snapshot and
     checks its denotation after later revisions.

## Still not complete

- Rust compilation/test execution has not been possible in this sandbox.
- Full `SemanticWorkspaceSession` query ownership is incomplete; significant
  declaration/resolver/global materialization code remains.
- Compiler-owned persistent `ProjectUniverse` / resolver / linker lifecycle is
  not yet implemented.
- LSP still needs removal of its legacy formal workspace reconstruction path.
- Module diagnostics/completion/navigation acceptance still remains.
- Cold-vs-incremental equivalence harness remains unfinished.
- Final constructor `Self` implementation must be compiler-tested against
  existing AST/lowering selector behavior.

## Highest-priority next work

1. Compile this checkpoint in a Rust-enabled environment and fix any type/API
   mismatches before adding more architecture.
2. Finish staged query ownership in `SemanticWorkspaceSession`.
3. Move project/module lifecycle into the persistent compiler session.
4. Remove LSP resolver/linker/project reconstruction.
5. Run cold-vs-incremental equivalence as the correctness gate.
