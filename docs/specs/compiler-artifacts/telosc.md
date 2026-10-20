# Telosc compiler artifact

Status: accepted container and architecture contract; payload implementation is in progress.

This document owns the `.telosc` container and portable compiler-image boundary.
Language semantics remain governed by their owning language specifications.
Implementation progress is recorded in PERF003.C2, not inferred from this specification.

## Purpose and authority

`universe.telosc` is a compiled dependency artifact for the builtin Universe.
An artifact build may parse, link, analyze, lower, compile, and execute Universe
in an isolated build VM. An artifact consumer loads completed canonical
products; it must not reconstruct them from source. Initializer execution is
fresh work in each receiving VM and is distinct from source compilation.

The artifact crate owns wire encoding and bounded decoding; modules owns
module-world projection; semantic owns formal type/static/query authority;
core owns executable projection, native binding, and VM materialization.
Raw snapshots, semantic databases, heaps, and live runtime objects are not wire payloads.

## Container schema v2

All integers are fixed-width little endian. No host padding, pointer widths,
native enum discriminants, or native-endian memory appear on disk.
Strings are UTF-8 with a u32 byte-length prefix. Booleans use 0/1;
optional values and variants use explicit stable tags. Every decoded count,
length, graph depth, and total allocation is bounded by validation limits.

Magic: `54 45 4c 4f 53 43 01 00` (eight bytes; stable across container schema
revisions). Container schema and minimum reader schema are u32. This contract
uses schema 2; readers reject schema 1 because its header does not carry the
native catalog compatibility fingerprint. The header encodes package identity,
semantic-model version, stable-identity schema, module semantic ABI,
semantic artifact ABI, query fingerprint ABI, native-surface schema, the
trusted native catalog fingerprint `[u8; 16]`, type-metadata schema, capability
bits, six 32-byte digests, and section count.
Package tags are 0 (builtin namespace/version) and 1 (package name/version).
Digests are source, interface, semantic, query, metadata, and artifact, in that order.

Each directory entry is 68 bytes: kind u32, section schema u32, flags u32,
offset u64, encoded length u64, decoded length u64, and digest [u8;32].
Section order is ascending numeric kind. Payloads follow the directory and
must be nonoverlapping. Schema 2 writes uncompressed payloads; compression is
rejected.

| Tag | Section | Capability |
| --- | --- | --- |
| 1 | MODULE_WORLD | Required compiler authority |
| 2 | SEMANTIC_WORLD | Required compiler authority |
| 3 | QUERY_IMAGE | Required by CANONICAL_COMPILER_IMAGE_V1 |
| 4 | METADATA | Required by RUNTIME_PUBLIC_METADATA |
| 5 | PRESENTATION | Optional tooling |
| 6 | SOURCE_MAPS | Optional logical source mapping |
| 7 | EXECUTABLE | Required by PORTABLE_EXECUTABLE |

Capability bits 0 through 4 are compiler image, RuntimePublic metadata,
presentation, source maps, and portable executable respectively.
Section flag bit 0 means REQUIRED. Bit 1 reserves COMPRESSED and is unsupported
in schema 2. Unknown flag bits are rejected. Unknown required section kinds are
rejected; unknown optional kinds may be skipped. Known section schemas must
be supported before decoding. Declaring a capability never substitutes for
its required payload. A complete Universe artifact declares compiler-image,
RuntimePublic metadata, and portable-executable capabilities.

## Integrity and compatibility

Artifact integrity uses BLAKE3-256. Domain framing is u32 UTF-8 domain length,
domain bytes, `ff`, then payload. Domains use `telos.telosc.v1.` followed by
source, module-world, semantic-world, query-image, metadata, artifact, or section.
The whole-artifact digest hashes the header with its artifact-digest field
zeroed, directory bytes, and payload bytes in directory order.
Integrity digests are separate from internal semantic fingerprints.

Compatibility uses stable package identity, explicit schemas/ABIs, and the
trusted native catalog fingerprint. The fingerprint covers the canonical
native row contracts and runtime surface facts that may change while an
individual borrowed row's transported effects, raises, flow, and lifecycle
remain equal. Readers compare it with the current trusted catalog immediately
after container parsing and before section materialization. They also validate
each borrowed row's exact stable key and transported symbolic payload before
borrowing catalog-owned static values. The fingerprint is a compatibility
discriminator, not an authenticity mechanism; artifact digests provide
integrity, not producer authentication.
Artifact loading relocates graph indices into fresh type/static stores and
stable owners into receiving-process semantic identities. It does not restore
WorkspaceId, project graph numbers, TypeStoreId, StaticValueStoreId, ClassId,
ObjRef, or MetadataPoolId. Source digest records provenance; raw source text
is not required to validate and consume completed authority.

## Compiler image

MODULE_WORLD transports completed interfaces, linked reads, module identities,
package exposure, topology, and dependency fingerprints. Its module-layer
materialization creates CanonicalExternalModuleWorld without a source provider.
External canonical imports are terminal; they cannot trigger parsed discovery,
BuiltinInterfaceBuilder, or canonical linking.

SEMANTIC_WORLD projects the exhaustive P5 persistence inventory and relocatable
type/static graphs. Secondary P6 indexes are rebuilt from validated primary
products without semantic reanalysis. QUERY_IMAGE uses the existing canonical
query eligibility authority and contains only closed, immutable, relocatable
products. METADATA transports the existing canonical RuntimePublic bundle;
loading does not invoke its exporter.

### Semantic-world payload schema 3

`SEMANTIC_WORLD` starts with a u32 payload schema. Schema 2 carries the complete
compiler-required callable-analysis owner graph. Schema 3 preserves that graph
and adds exact accepted callable-definition owners, including their complete
semantic signatures, origin, source member ordinal, body provenance, and
inherent body context. Schema-3 readers reject schema 2 semantic-world
payloads; the outer container remains schema 2. Every body-local expression,
binding, explanation, flow node/edge, diagnostic cause, and related handle uses
a deterministic artifact-local ordinal qualified by its stable callable owner.
Loaders validate owner/reference closure and relocate every handle into fresh
process-local identities before publishing the snapshot. Source ranges are
attachments, never identities; overlapping ranges remain distinct expressions.

The payload preserves exact expression judgments, binding state, selected
semantic products, control-flow and exit facts, diagnostics, causal sharing,
explanation graphs, and dependencies. The body result fingerprint is recomputed
after relocation, and query products/fingerprints are rebuilt from relocated
owners. Canonical images reject nonempty internal incidents, nonempty
call-resolution handles without a transported owner, and predicate references
without their owner. Empty optionals remain empty. The compiler-required
binding/site/target reader is reconstructed from the same callable owner facts;
the broad source index and raw syntax remain optional/omitted.

Each callable binding handle also carries an explicit storage origin. A
`BodyLocal` binding is qualified by its callable owner. A `ModuleValue` binding
retains the canonical module symbol and, for a selective import, the exact
module-local import slot validated against `MODULE_WORLD`. Its body-local
`BindingId` is only an analysis handle; it does not create a second global or
callable storage identity. Ambient copies of a module value never become local
declarations in that callable. Source ranges locate expression and declaration
sites but do not establish identity. Lowering and product planning compare
`BodyLocal { callable, binding }` or `ModuleValue { symbol }`, so equal numeric
binding handles in different callables remain distinct while copies of one
module symbol agree. Missing origins, mismatched import slots, and conflicting
qualified identities fail closed.

Readers reject semantic-world schemas 1 and 2. Schema 1 lacks the complete
callable-analysis owner graph; schema 2 lacks accepted callable-definition
authority required by source-independent lowering.

Optional presentation/source provenance is a separate tooling capability.
Source-built images may retain it without requiring it for formal authority.

## Executable image

EXECUTABLE has an independently versioned header: executable schema,
portable-bytecode ABI, emitter ABI, native ABI digest, and compilation profile.
The native digest covers canonical primitive identities/signatures and the
explicit native runtime ABI version. The default Universe profile is Debug.
Machine-native instructions are outside this contract.

The build adapter reuses the existing emitter in an isolated VM and captures
code before initializer execution. Portable prototypes describe all importable
Universe modules, module initializers, nested callables, explicit opcode tags,
parameter/local/upvalue layouts, literal constants, symbolic references,
linked reads, executable semantic recipes, native anchors, and bootstrap order.
Constants referring to callables use prototype indices; class/type/native
references use stable identities or construction recipes. Floating-point bits
and all observable ordering are preserved.

The loader validates opcode operands, branches, frame limits, prototype and
constant references, upvalues, semantic-pool entries, native bindings, and
cross-section identities before initialization. Each VM receives fresh symbols,
objects, closures, chunks, metadata pools, and empty caches. Relocation maps
preserve required sharing inside that VM. Mutable globals, captured live values,
heap state, GC state, resources, caches, and fibers are never persisted.

Native/primordial installation and Universe initializers retain existing order
and failure behavior. Later imports consume stored prototypes without fallback
to source compilation. Logical URIs and source mappings identify runtime frames
without making raw source a compiler dependency.

## Explicit compilation and loading

The implementation target is `phalcom artifact build-universe --output PATH`
and `phalcom --universe-artifact PATH USER.ph`, backed by explicit build/write/load
APIs and a fallible artifact-backed VM constructor. These entry points are not
claimed implemented by this document. Explicit artifact errors must not silently
fall back to source. Automatic discovery, caches, and distribution belong to P9.

Writers publish through an atomic temporary-file replacement. Loaders publish
only after complete validation; partial or failed loads do not poison a global
cache. Unsupported projections and incompatible images are typed failures,
never requests to repair missing authority by analyzing Universe source.

## Module-world payload v1

The section starts with a u32 payload schema, initially 1. ArtifactModuleId is
a u32 index into the sorted Universe-path module table, separate from ModuleId.
The payload contains entry, initializer order, module rows, semantic graph rows,
and the module-ABI topology fingerprint. Module rows retain unlinked and linked
interfaces, inert metadata, import surfaces, binding indices, linked reads,
runtime/reference edges, parent/children, and interface/private-link fingerprints.
Source spans use checked u64 byte offsets; no physical source path is required.
Module kind tags are 0 (module) and 1 (package). Linked target tags are 0
(module) and 1 (binding). Binding semantics tags are 0 (runtime plus mutability),
1 (static constant), and 2 (declaration). The explicit codec field/tag tables
are maintained in phalcom-compiler-artifact/src/module_world.rs.

Loading validates closed references, symbol ownership, unique layouts, matching
public surfaces, package exposure, topology fingerprints, and runtime DAG/order.
It materializes completed LinkedProgram/ModuleTopology directly without linking
or discovering source. Decoder allocation, item, string, and depth budgets are
cumulative across nested module metadata.

## Type-graph payload v1

The graph starts with a u32 payload schema, initially 1. Distinct artifact-local
u32 IDs address types, kinds, parameter versions, scoped nodes, lambdas, record
rows, families, variants, static domains, and static constructors. Stable owners
use explicit project/module/declaration/callable codecs; session project identity
is nonpersistent. Node tags and fields are maintained explicitly in
phalcom-compiler-artifact/src/semantic/type_store.rs.

All native TypeData, KindData, and ScopedTypeData cases have explicit portable
forms. Parameter arena versions and current owner-slot selection are different
products: historical versions retain distinct references, while current slot
selection is transported separately. Variance overrides are likewise explicit.
Parameter/lambda source presentation is omitted from this compiler payload.

Loading uses a fresh TypeStore and checked relocation for every graph edge.
Normal interning restores immutable type/kind/row/family/scoped/lambda products.
Parameter versions are allocated at the store ownership boundary and current
slots are restored after relocation. Structural cycles, invalid references,
noncanonical duplicate arena nodes, missing primitive nodes, checked structural
kind mismatches,
and escaped de Bruijn binder references are rejected before returning the store.
Global unions normalize relocated member order; scoped union body order and
repeated contributions remain as represented by the existing compiler.

These payload implementations have compiler-check evidence only. Cross-process
determinism, semantic parity, complete image loading, and execution acceptance
remain unverified; their implementation records are authoritative for progress.

Declaration/signature-dependent type formation and full cross-section consistency
remain responsibilities of completed semantic-world import before image publication.
