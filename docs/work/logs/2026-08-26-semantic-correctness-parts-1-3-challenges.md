# Semantic Correctness Parts 1–3: Issues, Challenges, and Incomplete Completion Report

Date: 2026-08-26

Repository: `/Users/altunhasanli/dev/phalcom/phalcom`

Branch: `main`

Latest pushed commit: `7b080b9e` (`stabilize watched source lifecycle`)

## Purpose

This report records why Parts 1, 2, and 3 were not completed despite substantial implementation and debugging work. It distinguishes:

- bugs that were found and fixed;
- issues that were only partially addressed;
- tests that passed as scoped evidence;
- release gates that were not closed; and
- workflow decisions that slowed completion or allowed incomplete work to continue.

The authoritative requirements remain the Part 3 specification and checklist:

- `/Users/altunhasanli/dev/phalcom/phalcom/docs/impl/semantic/semantic-correctness/part-3/phalcom_semantic_correctness_single_world_takeover_part3_persistent_workspace_lsp_cutover_professional_ide_spec.md`
- `/Users/altunhasanli/dev/phalcom/phalcom/docs/impl/semantic/semantic-correctness/part-3/phalcom_semantic_correctness_single_world_takeover_part3_implementation_checklist.md`

## Executive summary

The work was not trapped in one endlessly failing test. Several genuine, subtle defects were found and fixed, and focused tests became green. The failure was at the program-management and completion-validation level: implementation continued as a sequence of narrow verified slices without converting the full Part 1/Part 2/Part 3 gate matrix into a strict burn-down.

The Part 3 checklist explicitly requires all 90 completion gates to have focused evidence. A green workspace-semantic test group was therefore meaningful but insufficient. Part 2 §62 and the Part 3 §88 release gates remained open, so claiming completion would have been incorrect.

The latest pushed slice is real progress. It fixes watched-file rename/delete behavior and preserves compiler ownership. It is not Parts 1–3 completion.

## What was completed or materially advanced

The following areas received implementation work and focused validation during the task:

1. Compiler-first LSP request data was extended across diagnostics, completion, hover, inlay hints, navigation, references, workspace symbols, semantic tokens, and signature-related paths.
2. Same-name declarations in different modules were given distinct canonical identities. The important correction was in compiler-owned import and resolved-module identity construction, not in presentation-layer string matching.
3. Persistent module/session lifecycle work was implemented around `WorkspaceModuleSession` and `SemanticWorkspaceSession`, including overlays, reopen behavior, projectless real-directory identity, and transactional rebuild behavior.
4. Cross-module advisory parameter facts were propagated through fixed-point analysis. Local semantic regressions for parameter transfer passed.
5. Top-level cross-module advisory inference was repaired so expressions such as `Provider.Service` could resolve through compiler-linked exports and compiler surfaces rather than requiring a callable-local attachment.
6. Native contract regressions for `System.print` and `System.gc` were addressed in the compiler/native surface path.
7. Cancellation/latest-wins behavior, old-snapshot immutability, and structural performance counters were covered by focused tests.
8. The final lifecycle slice fixed stale deleted modules being reloaded from the filesystem resolver cache.

Evidence from the final lifecycle slice:

```text
cargo test -p phalcom-lsp --test integration workspace_semantics -- --nocapture
11 passed, 0 failed

cargo test -p phalcom-modules --test workspace_session -- --nocapture
6 passed, 0 failed

cargo check -p phalcom-lsp -p phalcom-modules
passed
```

The lifecycle change was committed and pushed as `7b080b9e`.

## Technical issues encountered

### 1. Two semantic worlds had to be migrated without restoring legacy authority

The LSP already had a mature worker-backed semantic implementation, while the compiler/semantic layer was being made authoritative. This created a difficult boundary:

- formal compiler facts had to remain canonical;
- advisory runtime-shape facts could assist presentation but could not upgrade formal `Unknown` or `Dynamic`;
- incomplete-text recovery still needed to work;
- exact compiler-covered requests could not fall back to legacy LSP semantic indexes;
- presentation metadata such as Phaldoc, source ranges, and member-kind information still had to be preserved.

This was more complicated than replacing one function call. Each request family had separate fallback behavior, identity mapping, and coverage gaps. A locally correct completion or hover path could still leave an authority violation elsewhere.

The Part 3 specification also requires removing the duplicate semantic implementation after compiler coverage exists. That architectural deletion was not finished. The duplicate `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp/src/semantic` implementation remained, and its continued existence kept the single-world release claim open.

### 2. Canonical cross-module identity was easy to get almost right

Same-name classes in separate modules initially risked collapsing into one semantic identity. The visible names were identical, so tests that only checked labels could miss the problem. The correct identity had to carry the resolved module and declaration path through:

- import-map construction;
- linked exports;
- declaration IDs;
- compiler surfaces;
- callable IDs;
- LSP target and presentation mapping.

The fix belonged in compiler-owned resolution. A presentation-layer disambiguation would have hidden the symptom while leaving formal dispatch and navigation incorrect.

The targeted regression eventually passed:

```text
cargo test -p phalcom-lsp --test integration \
  workspace_semantics::same_named_classes_in_different_modules_keep_distinct_identity \
  -- --nocapture
```

### 3. Top-level module-member advisory inference had no callable attachment

The two most informative initial workspace failures were:

- `parameter_facts_from_multiple_consumer_modules_join_instead_of_overwriting`;
- `inferred_parameter_facts_propagate_through_forwarding_calls`.

The compiler's advisory fixed-point logic passed its local semantic regressions, so the problem was not initially in the join operation itself. Debugging showed:

- a formal binding for the top-level result was absent, as expected for that expression shape;
- an advisory binding existed;
- its shape was `ValueShape::Unknown`;
- `Provider.Service` therefore could not become a verified class object;
- `Provider.Service.new()` could not resolve to an instance;
- later forwarding calls lost the return-shape evidence.

The cause was a design assumption in `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/analyzer.rs`: `Expr::GetProperty` analyzed its object and then relied on callable-range resolution. Top-level expressions have no callable formal attachment, so this path could not resolve a module member.

The required correction was compiler-owned module-member resolution using linked exports and compiler surfaces. It could not fabricate a declaration from spelling, query the filesystem, or consult a legacy LSP index. Once this projection was supplied to advisory analysis, the workspace propagation tests passed.

This bug was subtle because:

- local advisory tests were green;
- formal compiler analysis was not necessarily wrong;
- the failure appeared only after composing top-level module access with constructor inference and interprocedural forwarding;
- the final visible symptom was an incomplete completion list, not an obviously failed resolution diagnostic.

### 4. Interprocedural fixed-point propagation had multiple independent loss points

Parameter facts from more than one consumer had to join rather than overwrite. Forwarding calls added another layer: a fact had to travel through a callable whose return type was initially unknown, then become usable by its caller after inference.

The implementation required coordination among:

- formal callable-body analyses;
- advisory return summaries;
- `ValueShape` joins;
- dispatch surfaces with initially unknown return contracts;
- rechecking callers when a return summary became known;
- bounded fixed-point termination;
- provenance and confidence preservation.

The local fixed-point test passing did not prove the workspace case. The workspace added linked modules, top-level module members, multiple consumers, and publication ordering. This distinction consumed time because each layer could look correct in isolation.

### 5. Persistent workspace state had to be coherent across updates, not merely correct per update

The session model retained state across source changes. That introduced requirements beyond ordinary one-shot analysis:

- unchanged modules should remain reusable;
- changed modules should invalidate the correct reverse dependency closure;
- removed modules should not remain in declarations, surfaces, dispatch, or advisory products;
- open overlays should take precedence over disk;
- close/reopen should preserve the latest compiler world;
- failed rebuilds should not replace a coherent publication with an empty or partial candidate;
- source and module identity should remain stable where the semantic object remains the same.

The analysis service also had a separate source catalog, document map, and compiler workspace state. The interaction among those layers made it possible for one source representation to be current while another retained an older module identity or revision.

### 6. macOS `/var` versus `/private/var` broke deletion matching

The watched-file lifecycle regression exposed a platform-specific path issue. On macOS, a test URI could contain `/var/...`, while filesystem canonicalization returned `/private/var/...`.

For an existing file, normal canonicalization corrected this. For a deleted file, canonicalizing the file itself failed because it no longer existed. The deletion path therefore needed to canonicalize the parent directory and append the missing filename.

Without that behavior:

- the source catalog retained the deleted URI under its canonical form;
- the compiler workspace saw the wrong source set;
- deletion appeared to be processed while the old module remained available.

This was not visible in ordinary edit tests because the file existed during those updates.

### 7. Filesystem resolver caches resurrected a deleted source

After canonical URI removal was corrected, the test still failed. The more subtle cause was stale caching in `FilesystemSourceProvider`.

The sequence was:

1. `source_catalog` correctly removed `renamed-provider.ph`.
2. `WorkspaceModuleSession::remove_source` removed the source/module mapping and overlay.
3. The consumer still imported `.renamed_provider`.
4. The next rebuild attempted to resolve that import.
5. The filesystem provider still had cached location/read products for the deleted module.
6. The resolver loaded the deleted module from cache and reintroduced it into `parsed_sources`.

The semantic snapshot then appeared to contain a current source map, but that map had been repopulated with a stale module by the rebuild. This initially looked like a semantic publication or stale-surface bug. Tracing the source set before and after module-session rebuild isolated the resolver cache as the actual source of resurrection.

The fix was to clear the filesystem provider cache during `RemoveSource`, before rebuilding. This is why the final regression tests both URI behavior and compiler module identity rather than checking only a removed LSP index entry.

### 8. Strict linking and interactive workspace recovery needed different policies

A deleted dependency can leave an importer in the workspace with an unresolved import. Strict linker behavior is correct for normal linking: unresolved imports should be errors. However, persistent IDE publication still needs to publish the remaining importer and its diagnostics instead of retaining the removed module or discarding the whole coherent workspace.

The solution introduced a separate permissive workspace path, `link_with_unresolved_imports`, while preserving strict `link` behavior for existing callers. That distinction was necessary to avoid weakening compiler correctness globally just to support deletion recovery.

The challenge was ensuring that permissiveness only skipped missing import targets. It could not silently convert unrelated link failures, missing exports, invalid names, or runtime graph errors into success.

### 9. Watched-file tests had an ordering race

The rename test changed the consumer import and then immediately sent watched-file events. The analysis worker could process the watched event before the consumer-change publication completed. Depending on scheduling, the test could observe a transient world and fail for the wrong reason.

The regression was made deterministic by waiting for the consumer publication before sending the rename/delete watched-file batch. This is a test-harness ordering correction, not a weakening of the assertion. The test still requires the compiler world to reflect the rename and then remove the deleted provider.

### 10. Recovery publication obscured which source set was authoritative

`publish_persistent_compiler_workspace` attempts a full overlay batch and has a recovery path that removes individual entries if a batch rebuild fails. This is useful for keeping an IDE responsive, but it complicates diagnosis:

- a failed batch may produce a smaller active set;
- the module session may retain or reload imported modules;
- the LSP protocol document map and compiler source map can temporarily differ;
- a publication counter can advance for a publication that is not the publication a test intended to observe.

The code therefore needed explicit tracing of source catalogs, module-session updates, compiler snapshots, and publication ordering. Temporary diagnostic logging was added during diagnosis and removed before committing.

### 11. Native metadata and language-level contracts were not perfectly aligned

The compiler's imported native surfaces are generated/registered from native metadata, while the language specification supplies the semantic contract. `System.print` exposed a mismatch where stale metadata could make a trusted call appear to return an `Option`-like value rather than `Unit`.

This mattered because the LSP's formal/advisory boundary depends on the compiler's imported native surface. A presentation fix could hide the wrong type, but it would not correct compiler semantics. The correction had to keep runtime behavior, native metadata, compiler typing, and tests aligned.

### 12. Presentation needed compiler identity without leaking advisory decoration

Hover and completion presentation had to use compiler-owned declaration/callable identity while preserving useful source metadata. Several subtle mappings were involved:

- inherited members should report the defining owner, not merely the receiver class;
- same-named selectors in different modules should not resolve to the first global match;
- constructor class-side dispatch should map to instance construction correctly;
- formal values should not be visually upgraded by advisory evidence;
- stale advisory decorations such as visible approximation markers had to be removed where Part 3 superseded them;
- Phaldoc should attach to the resolved declaration, not a textually matching selector.

These were individually testable but collectively made it difficult to know whether a failure belonged to semantic resolution, source identity, or presentation mapping.

### 13. Incomplete-text recovery competed with the compiler-only cutover

The LSP must remain useful while a document is syntactically incomplete. That requires some syntax/lexer fallback. At the same time, Part 3 prohibits using the old semantic reconstruction as authority for exact requests covered by compiler products.

The practical boundary was:

- lexer or syntax fallback may classify incomplete text;
- compiler products must own exact semantic targets and values when available;
- missing compiler coverage may permit a narrowly bounded fallback;
- fallback must not override a compiler `Unknown`, `Dynamic`, or unresolved result.

This boundary is easy to violate accidentally because fallback makes tests look more robust. It also makes “the request returns something” an insufficient correctness criterion.

### 14. Incrementality and identity requirements pulled in opposite directions

Reuse improves responsiveness but can preserve stale products if dependency fingerprints do not include the right input. Recomputing everything avoids stale state but violates incrementality and can hide incorrect dependency modeling.

The session work therefore had to distinguish:

- changed source products;
- removed source products with no replacement fingerprint;
- reverse dependency closure invalidation;
- reusable unchanged callable analyses;
- inferred return summaries produced after initial surface publication;
- immutable snapshot publication;
- last-known-good recovery.

The deleted-source case demonstrated why removal cannot be treated like an ordinary source edit: there is no replacement product whose fingerprint proves that the old semantic object is gone.

## Validation and test-process challenges

### Focused green tests were not release evidence

The targeted tests gave strong evidence for individual slices. They did not establish Parts 1–3 completion because the specification requires every named gate, including architectural absence checks, cutover checks, native contract checks, presentation checks, and professional IDE acceptance.

The latest work did not complete the full post-change sequence of:

- all Part 1 gates;
- Part 2 §62 release gate;
- every Part 3 §88 gate;
- full LSP cutover absence checks;
- professional IDE golden acceptance;
- final full-suite verification.

An earlier broad LSP test run passed before the final lifecycle edits, but it was not a substitute for rerunning the complete required matrix after those edits.

### Formatting validation was not clean

`cargo fmt --all -- --check` reported formatting differences across the repository, including existing and parallel-owned areas. Running an unrestricted formatter would have modified unrelated work in the shared checkout, so it was not performed. Owned-file `git diff --check` passed, but repository-wide formatting was not a clean gate.

### The test harness itself could create misleading timing evidence

Publication counters are useful, but a test that waits for “some later publication” can observe a publication caused by another queued action. The rename regression required explicit sequencing to avoid interpreting a stale intermediate snapshot as the final result.

### Graphify added an additional maintenance step

The repository requires graph updates after code changes. `graphify update . --no-cluster` was run after the lifecycle edits, and the graph rebuild completed. Graph output is mostly ignored/generated state, but the status check after graphify was still necessary because hooks and incremental rebuilds can affect the worktree.

## Workflow and coordination challenges

### 1. The checkout contained parallel work

The worktree was heavily dirty. The explicit constraint was to preserve unrelated staged, unstaged, deleted, and untracked changes. At the end, these parallel files remained untouched:

- `/Users/altunhasanli/dev/phalcom/phalcom/examples/ide-golden/src/main.ph`
- `/Users/altunhasanli/dev/phalcom/phalcom/examples/ide-golden/src/comprehensive/`
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/semantic/capabilities/callable_publication_capabilities.rs`

Only owned lifecycle files were staged and committed. This was the correct safety decision, but it means the shared worktree could not be made literally clean without violating scope.

### 2. Narrow ownership made broad refactoring unsafe

The compiler/LSP boundary crosses many crates and tests. A safe implementation required touching only the specific authority seams under investigation. Broad cleanup or deletion of the duplicate LSP semantic layer could have collided with parallel work and could have removed still-needed behavior before all compiler coverage was proven.

### 3. The implementation order was not enforced strongly enough

The specification says Parts 1 and 2 are prerequisites for Part 3. In practice, work advanced into Part 3 lifecycle and LSP cutover slices while Part 1/Part 2 release evidence was still incomplete. That produced useful progress but also made the final state harder to reason about: a Part 3 slice could be green while the Part 3 completion gate remained necessarily red because its prerequisites were open.

### 4. Checklist updates were intentionally conservative but not sufficiently operational

The checklist was not marked complete without fresh evidence. That avoided a false completion claim. The weakness was that the checklist was used more as a status record than as the primary execution queue. A stricter workflow would have selected the next unchecked gate, run its exact command, fixed only that gate, and recorded the evidence immediately.

### 5. Debugging crossed multiple publication boundaries

The actual state had to be traced through:

```text
watched-file notification
  -> analysis-service removal/refresh queue
  -> source catalog
  -> persistent module session
  -> filesystem/provider resolver cache
  -> linker
  -> semantic workspace session
  -> immutable compiler snapshot
  -> LSP request context
  -> completion/hover presentation
```

A failure at the final completion list could originate at any earlier boundary. Temporary tracing was necessary to identify whether the stale module was in the source catalog, module session, semantic snapshot, or presentation layer.

## Why completion was not reached

The direct reason is incomplete gate closure, not inability to make any tests pass.

The process reached a state where several focused slices were green, but the following remained unresolved or insufficiently evidenced:

1. Part 1 release and correction/amendment gates were not fully verified from current live source and tests.
2. Part 2 §62 release gate was not closed.
3. Part 3 §88 gates, including prerequisite, cutover, absence, and professional IDE gates, were not all checked.
4. The duplicate LSP semantic implementation remained in production and therefore the requested single-world authority proof was incomplete.
5. The complete focused gate sequence was not rerun after the last lifecycle changes.
6. The IDE golden acceptance area had parallel changes that were outside this slice's ownership.
7. The shared worktree could not be made clean without disturbing unrelated work.

This was not an infinite loop. It was a failure to stop slice development and switch into a disciplined release-gate campaign. The work repeatedly solved the current visible defect, validated that defect, and moved forward, while the global completion condition stayed open.

## What should have happened earlier

The correct control loop should have been:

1. Freeze the authoritative gate matrix from the checklist.
2. Mark every gate as `implemented`, `verified`, `blocked`, or `not run`.
3. Work strictly in dependency order: Part 1, Part 2 §62, then Part 3 prerequisites and cutover gates.
4. For each red gate, make one narrow change, run its exact command, and record the result immediately.
5. Stop adding new slices once a gate has an architectural blocker that requires broader ownership or user direction.
6. Run the entire required matrix after the last code change.
7. Only then update release status, create grouped commits, push, and report completion.

## Current status after the latest push

Pushed commit:

```text
7b080b9e stabilize watched source lifecycle
```

Verified in the final slice:

- watched-file rename/delete compiler identity regression: passed;
- full workspace-semantic target: 11 passed;
- module workspace-session target: 6 passed;
- `cargo check -p phalcom-lsp -p phalcom-modules`: passed;
- graphify update: completed;
- temporary diagnostic logging: removed.

Still present by design because they belong to parallel work:

- `/Users/altunhasanli/dev/phalcom/phalcom/examples/ide-golden/src/main.ph`
- `/Users/altunhasanli/dev/phalcom/phalcom/examples/ide-golden/src/comprehensive/`
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/semantic/capabilities/callable_publication_capabilities.rs`

The repository is therefore pushed at the latest owned commit, but the shared worktree is not literally clean and Parts 1–3 are not release-complete.

## Final assessment

The technical work was real and several difficult bugs were fixed. The completion failure came from treating a large, dependency-ordered release migration as a stream of local implementation tasks. Focused tests passing proved individual repairs, but the project required a complete, current, evidence-backed closure of the Part 1, §62, and §88 gate matrix. That closure did not happen.
