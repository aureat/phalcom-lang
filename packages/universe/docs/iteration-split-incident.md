# S4 Iteration Split Dependency Incident

## Decision

Keep the S4 shadow iteration declarations together in `src/collections/iteration/package.ph`. Record the six-child layout as deferred until source imports can express and validate interface-only dependencies, or another design preserves the same declaration identities and member behavior without eager runtime cycles. Do not weaken cycle validation or change protocol methods to bypass the dependency.

This is a source-layout decision for the unregistered shadow package. It does not change the active production source owner or authorize production cutover.

## Required invariant

The iteration protocols and adapters must keep one canonical authored declaration each, preserve all protocol members and generic constraints, and retain the current runtime import and initialization rules. The split must not create duplicate protocols, weaken associated-type relationships, or conceal an eager cycle.

## Evidence

- At the S4 starting revision `a1621a7cec577952612685508a4c87c90aea86cd`, `iterator_laws.ph` contains the two protocols, blanket `Iterable` implementation, and fourteen adapters. The source body moved as a whole to `iteration/package.ph` with an identical SHA-256 digest.
- `Iterator` default methods construct adapters such as `MappedIterator`, `FilterIterator`, `ChainIterator`, and `StrictZipIterator` (`iteration/package.ph`, lines 4–31). `Iterable.indexed` constructs `IndexedIterator` (line 46).
- The adapters conform to `Iterator` (for example `MappedIterator` at lines 379–383, `ChainIterator` at 584–588, and `StrictZipIterator` at 867–871). If moved to child modules, their conformance requires imports of the protocol owner, while protocol defaults require imports of adapter owners.
- `ModuleLinker::collect_imports_and_graphs` creates runtime dependency edges for selective imports and re-exports as well as whole-module imports (`phalcom-modules/src/linker.rs`, lines 701–785). The linker’s runtime graph represents eager initialization dependencies (`phalcom-modules/src/graph.rs`, lines 100–121, 357–358). The source import model has no authored interface-only import form.
- Therefore, the proposed child-module split would introduce a runtime dependency SCC between `protocols` and adapter modules. This is a predicted SCC from the source references and linker rules; it is not reported as a measured SCC from a linked shadow graph, because no split source graph has been built.

## Alternatives considered

1. Keep all declarations in one accurately named `iteration` package module. This is the conservative S4 fallback and preserves the moved body exactly.
2. Split protocols and adapters with ordinary imports. Rejected because it introduces the predicted runtime SCC.
3. Add a new type-only import syntax, suppress cycle checks, or define duplicate protocols. Rejected as unsupported or identity/semantics changing; any interface-only source mechanism belongs to a separately scoped compiler integration plan.

## Verification and boundary

The S4 source validator parses all shadow `.ph` files and builds unlinked source interfaces. It checks source paths, import targets and selected exports, package exposures, and the migrated iteration path’s hierarchical visibility. It does not typecheck method bodies, build a linked runtime graph for the proposed split, establish `ProjectIdentity::Universe` semantic identities, or validate runtime initialization.

The legacy `phalcom-core/core/universe/` source and production provider remain unchanged. The authored source import inventory still contains 216 distinct edges; S4 retargets 23 existing iteration edges without increasing that count.

## Next owner and gate

The next compiler/module-system integration plan should first implement or identify a tested interface-only source dependency mechanism under the established import and runtime rules. Revisit the six-child split only after a focused linked-interface and initialization-graph test demonstrates an acyclic runtime graph and preserved canonical declaration identity. Until then, mark the fine-grained split `BLOCKED/PARTIAL`.
