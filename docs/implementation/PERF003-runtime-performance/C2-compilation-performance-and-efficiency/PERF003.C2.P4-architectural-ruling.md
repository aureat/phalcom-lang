# PERF003.C2.P4 — Architectural Ruling on the Two Incidents

## Decision

**P4 should acquire two bounded prerequisites, while T5 continues. T6 remains blocked until both prerequisites are resolved.** The committed T1 options work should remain exactly as implemented: default `RuntimePublic`, explicit profile selection, semantic publication independent from metadata retention, and the currently blocked end-to-end assertions preserved.

I verified the pushed branch state against `a82dfe32ab744a16a7f24fce1cca85201372df19`: `perf/universe-characterization` is exactly one commit ahead, containing the T1/compiler-session plumbing and lifecycle/incident records, with no hidden source-index, bootstrap, or metadata-schema repair.

| Area | Ruling |
|---|---|
| **T1 options plumbing** | **Accepted as the correct plumbing increment.** Do not change the default profile or weaken the retained-session assertion. |
| **`ProductKind` SourceIndex failure** | **Acquire a bounded P4 prerequisite repair.** This is a source-index classification defect, not a reason to add `ProductKind` to the native class catalog. |
| **Ordinary Record-remainder failure** | **Proceed with T5 first.** Establish whether the ordinary tiny program was failing only because aggregate export traversed irrelevant state. |
| **Record metadata representation** | **A schema extension is nevertheless required before T6.** Canonical `RuntimePublic` already has a required public surface that the v12 schema cannot faithfully represent. |
| **T6 canonical external authority** | **BLOCKED** until the source index builds the canonical program **and** canonical `RuntimePublic` metadata can faithfully represent heterogeneous Record keys/remainders. |
| **T7–T9** | Remain behind T6 as planned. |

This matches the checkpoint's current state: P4 is already recorded as `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`, with T5–T9 still outstanding. CHECKPOINT The checkpoint also already establishes canonical elision, explicit reachability, and the independence of metadata profile from semantic publication, so neither incident justifies backing away from those decisions. CHECKPOINT

---

## 1. `ProductKind`: this is a classification bug, not a missing native class

The decisive code is `phalcom-core/src/native/source.rs::index_program`.

Today it effectively performs:

```text
every Runtime + Intrinsic data declaration
    ↓
index_intrinsic_runtime_data
    ↓
must resolve to UniverseKey::{Tuple, Record, Pack}
```

That premise became false when `LANG007.C4.P5` introduced the compiler-owned structural reflection declarations:

```telos
intrinsic data ProductKind { ... }
intrinsic data ProductShape { ... }
intrinsic data OrderedTail<Rest> { ... }
intrinsic data OrderedShape(...)
intrinsic data Presence { ... }
...
```

Those declarations are semantically intentional. `StructuralReflectionProjector` constructs values of `ProductKind`, `ProductShape`, `UnorderedShape`, and the other structural domains. They are **compiler/semantic intrinsic data**, whereas the `Tuple`, `Record`, and `Pack` declarations are the special source facades for pre-existing runtime-native object classes.

That distinction is also visible in `phalcom-native-meta/src/universe.rs`: `UniverseKey` is explicitly the catalog of canonical built-in **runtime classes**. It includes `Tuple`, `Record`, and `Pack`; it does not include `ProductKind`, `ProductShape`, `OrderedTail`, etc.

Therefore:

> **Do not add `ProductKind` to `UniverseKey`, `UNIVERSE_CLASS_RELATIONS`, or a ProductKind-specific allowlist.**

That would incorrectly reclassify compiler-owned structural ontology as runtime-native class ownership. It would also merely move the failure to the next structural intrinsic declaration.

### Required repair shape

The source index needs to distinguish the declaration category **before** invoking `index_intrinsic_runtime_data`.

Conceptually:

```rust
match source_universe_identity(module, &data_def.name) {
    SourceUniverseIdentity::Canonical(
        key @ (UniverseKey::Tuple | UniverseKey::Record | UniverseKey::Pack)
    ) => {
        // Existing native-runtime structural facade path.
        self.index_intrinsic_runtime_data(module, data_def, key)?;
    }

    SourceUniverseIdentity::Canonical(other_runtime_class) => {
        // Fail closed: a different native runtime class must not silently
        // masquerade as compiler-owned intrinsic data.
        return Err(...);
    }

    SourceUniverseIdentity::WrongModule(key) => {
        // Preserve wrong-module rejection for canonical runtime identities.
        return Err(...);
    }

    SourceUniverseIdentity::NotUniverseName => {
        // Compiler/source-owned intrinsic data. It is part of the parsed
        // Universe semantic program, but does NOT create a native class anchor.
        //
        // Sum-shaped declarations may continue through ordinary enum/census
        // indexing where appropriate.
    }
}
```

The exact factoring can differ, but the semantic categories must remain separate.

This is a **bounded prerequisite**, not a general native bootstrap rewrite. The write-set should remain around `native/source.rs` and its focused tests unless live evidence proves otherwise.

---

## 2. Record remainder: T5 is necessary, but T5 alone cannot solve the architecture

The implementer's proposed ordering is partly correct:

> **For the ordinary retained-session failure, perform T5 root-driven cutover before deciding that its particular offending type needs serialization.**

The current aggregate exporter does not tell us why the non-Symbol Record type was visited. A tiny:

```telos
const answer: Int = 42
```

should not fail because some unrelated declaration somewhere in the semantic world contains a richer Record type. T5 exists specifically to eliminate that behavior.

So after T5, that fixture should either succeed because the offending form is outside the retained closure, or fail with evidence identifying the retained root that actually requires it.

However, there is a second and stronger finding:

### The metadata schema gap is already proven to be real

Canonical Universe source contains:

```telos
type FixedFields = { **[FieldKey]: FieldInfo }
```

and exports it publicly.

`FieldKey` is explicitly heterogeneous:

```text
Symbol | String | Int | Bool
```

Meanwhile P4's `INV-P4-08` requires `RuntimePublic` to preserve **transparent public aliases**.

Therefore canonical `RuntimePublic` metadata must be capable of representing the target of `FixedFields`. That target contains an anonymous Record remainder whose key domain is `FieldKey`, not `Symbol`.

This means the schema question is already settled:

> **Non-Symbol anonymous Record remainder key domains are valid publishable semantic forms. There is no normative `RuntimePublic` restriction making them ineligible. The current restriction is a legacy metadata-format limitation.**

The existing fail-closed behavior in `MetadataExporter` is consequently correct **until the format is upgraded**. It must not be replaced with lossy export.

---

## 3. The schema repair must cover more than the reported remainder

Current semantic representation is sound:

```rust
pub enum AnonymousRecordRemainder<T = TypeId> {
    Unconstrained { key_type: T },
    Typed { key_type: T, value_type: T },
}
```

But `phalcom-type-meta` v12 erases information:

```rust
pub struct RecordFieldRef {
    pub name: Box<str>, // implicitly Symbol-only
    ...
}

pub enum AnonymousRecordRemainderRef {
    Unconstrained,
    Typed(TypeNodeId),  // value type only
}
```

So there are actually **two related representation gaps**:

| Semantic fact | v12 limitation |
|---|---|
| Fixed Record key may be `Symbol`, `String`, `Int`, or `Bool` | `RecordFieldRef` stores only a string `name` |
| Anonymous Record remainder has an explicit `key_type` | `AnonymousRecordRemainderRef` stores no key domain |

A patch that only adds `key_type` to anonymous remainders would therefore leave heterogeneous fixed Record keys unrepresentable.

The durable fix should introduce a stable Record-key representation in `phalcom-type-meta`, conceptually:

```rust
pub enum RecordKeyRef {
    Symbol(Box<str>),
    String(Box<str>),
    Int(StaticInt),
    Bool(bool),
}

pub struct RecordFieldRef {
    pub key: RecordKeyRef,
    pub ty: TypeNodeId,
    pub presence: StructuralMemberPresenceRef,
}

pub enum AnonymousRecordRemainderRef {
    Unconstrained {
        key_type: TypeNodeId,
    },
    Typed {
        key_type: TypeNodeId,
        value_type: TypeNodeId,
    },
}
```

The analogous scoped forms and `RecordRowRef` paths need the same fidelity.

Because `TYPE_METADATA_SCHEMA_VERSION` is currently 12 and the loader advertises support back to schema version 1, this must be a **versioned migration**, probably schema v13. Old metadata should decode its legacy `name` as `RecordKeyRef::Symbol(name)`, and legacy anonymous remainders should semantically acquire the canonical `Symbol` key domain. Merely incrementing the version constant without a legacy decode path would violate the existing compatibility claim.

Structural fingerprints must also include both the fixed key kind/value and anonymous `key_type`; otherwise semantically distinct Records could collide.

---

## 4. Required execution order

The correct continuation is:

1. **Keep the committed T1 increment.** Mark the narrow header/profile plumbing evidence as valid, but leave the retained-session end-to-end gate pending.

2. **Implement P4.T5.** Construct authoritative RuntimeMinimal/RuntimePublic roots and cut production `MetadataExporter` over to `MetadataReachabilityClosure`. There must be no aggregate production fallback. The tiny retained-session fixture should then demonstrate whether its old failure was irrelevant whole-world traversal.

3. **Repair the canonical source-index intrinsic classification as a bounded prerequisite.** This can be done independently of T5. Do not start T6 certification until `NativeSourceIndex::build()` and the exact canonical singleton path pass.

4. **Implement the versioned heterogeneous-Record metadata representation before T6 certification.** T5 can establish which ordinary bundles require it, but canonical `RuntimePublic` already proves that the capability itself is necessary.

5. **Rebuild the canonical RuntimePublic authority and only then proceed through T6 → T9.** At that point ordinary bundles may safely elide canonical declaration/callable/field records because the external authority actually exists and faithfully represents its public surface.

This does **not** require renumbering the P4 task graph. I would record the two pieces as bounded prerequisites/amendments to T6, e.g. `P4.PR-A` for intrinsic source classification and `P4.PR-B` for heterogeneous Record metadata schema, while leaving T5/T6/T7 numbering intact.

---

## 5. Strengthen the export failure before using T5 as evidence

The current compiler converts:

```text
MetadataExportError
```

into:

```text
ProgramCompileError::Io("metadata export error: ...")
```

That is misleading and, more importantly here, throws away the information needed to decide reachability.

T5 should preserve at least the top-level retained entity/root while recursively exporting its type graph. For example:

```rust
MetadataProjectionError {
    root: MetadataRootIdentity,
    entity: MetadataEntityIdentity,
    type_id: Option<TypeId>,
    source: MetadataExportError,
}
```

Then a failure can say, conceptually:

```text
RuntimePublic root:
  alias universe.reflection.structural.FixedFields

export path:
  alias target
  -> Record
  -> anonymous remainder

failure:
  metadata schema cannot represent key domain FieldKey
```

rather than:

```text
Io("metadata export error: legacy metadata cannot represent ...")
```

That provenance is important because the current `MetadataReachabilityClosure` implementation is not itself sufficient evidence that an omitted type is semantically irrelevant. The actual exporter recursively traverses type children, so T5 certification needs positive root-to-export-path evidence, not just “the old exception disappeared.”

---

## 6. Acceptance evidence for the two incidents

| Gate | Required evidence |
|---|---|
| **SourceIndex category boundary** | `ProductKind` and another compiler-owned structural intrinsic are accepted without native runtime class anchors. |
| **Native facade preservation** | `Tuple`, `Record`, and `Pack` still create the existing canonical native anchors and relations. |
| **Fail-closed native identity** | A canonical runtime-class name using an invalid intrinsic form or canonical name at the wrong source path remains rejected. |
| **Bundled source corpus** | `NativeSourceIndex::build()` succeeds for the actual bundled Universe. |
| **Canonical program** | `canonical_universe_program_is_process_singleton` passes through source indexing and projection. |
| **T5 negative control** | Tiny `const answer: Int = 42` no longer encounters unrelated metadata; exported records correspond only to its profile closure. |
| **T5 positive control** | A local public API deliberately containing a non-Symbol Record remainder remains retained under `RuntimePublic`; T5 must not “optimize it away.” |
| **Schema fixed keys** | Round-trip Symbol, String, Int, and Bool fixed Record keys without identity loss. |
| **Schema remainder domains** | Round-trip both `Unconstrained { key_type }` and `Typed { key_type, value_type }` with non-Symbol domains. |
| **Backward compatibility** | Pre-v13 Symbol-key metadata loads with the same semantics it had before the schema extension. |
| **Canonical decisive case** | `FixedFields = { **[FieldKey]: FieldInfo }` appears correctly in canonical RuntimePublic metadata and round-trips with its `FieldKey` domain intact. |
| **P4 profile contract** | Default compile remains `RuntimePublic`; explicit `RuntimeMinimal` remains independent of semantic `runtime_minimal()` publication. |
| **T6 authority gate** | Ordinary local metadata can elide canonical records only after canonical RuntimePublic metadata is installed and resolvable as the immutable external authority. |

The checkpoint already records both failures separately and correctly treats the canonical failure as blocking T6 while retaining the record-export failure as an unresolved end-to-end P4 gate. CHECKPOINT

## Final ruling

**Approve bounded prerequisite repair; do not choose “detached metadata work only” as the final disposition.** T5 should continue because it is architecturally prior to T6 and is the correct cure for whole-world ordinary export. But P4 must own the two prerequisites necessary to make its own canonical-authority premise true.

The important distinction is:

```text
ProductKind incident
    = NativeSourceIndex category error
    ≠ add ProductKind to native runtime catalog

tiny-fixture Record incident
    = possibly aggregate over-export
    → T5 determines local relevance

canonical Record metadata
    = confirmed schema gap
    → versioned schema extension required before T6
```

No profile narrowing, record omission, key-domain erasure, source-index allowlist hack, or weakening of `INV-P4-08` is authorized.