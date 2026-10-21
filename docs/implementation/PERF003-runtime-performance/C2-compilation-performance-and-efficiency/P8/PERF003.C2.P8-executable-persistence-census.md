# PERF003.C2.P8 executable persistence census

This census records the current boundary between canonical emitter products and
the portable executable image. It describes the live implementation in
`phalcom-core/src/bytecode.rs`, `callable.rs`, `chunk.rs`, `value/`,
`heap/object.rs`, `heap/class.rs`, `heap/module.rs`, and `method/object.rs`.
Portable owners are under `phalcom-compiler-artifact/src/executable/`; capture
and projection are under `phalcom-core/src/artifact/`.

`UNSUPPORTED` means capture returns a typed error if the value is reached. It
does not mean that the value is silently omitted. The image remains structural
data until P8.T3 supplies executable validation.

## Coverage totals

- `Bytecode`: 128/128 source variants have an exhaustive portable projection;
  all portable opcode rows have explicit unique tags and operand classifications.
- `Object`: 55/55 heap variants are classified below. Twelve have portable
  recipe support; the other 43 fail closed when reached as emitted constants.
- `ConstantRecipeV1`: 22/22 explicit wire tags are exercised by a recipe
  round-trip test, including exact integer/float encodings and graph references.
- `Callable`: 8/8 source fields have a portable or reconstruct disposition.
  `Chunk`: 7/7 source fields are persisted, reconstructed, or runtime-only.
- `ExecutableSemanticPool`: 17/17 vector families have explicit projection
  paths. The two runtime index/cache families are rebuilt or start empty.
- Canonical modules, initializers, prototypes, constants, semantic-pool rows,
  classes, native anchors, primordial roles, and detached methods are counted
  by the canonical capture integration test. Structural decode validates all
  represented references before it reports those totals.

## Instruction and callable graph

| Runtime family / field | Source owner | Portable representation | Relocation requirement | Validation domain | Disposition |
|---|---|---|---|---|---|
| Every `Bytecode` variant and operand, including fused instructions, conditional dispatch, ADT/data operations, literal builders, and branches | `Bytecode` | Explicit `PortableInstructionV1` tag and fixed-width operands; exhaustive projection in `artifact/instruction.rs` | Per-operand rows in `OPCODE_MANIFEST_V1`; constant, linked-read, or semantic-pool relocation where listed | Per-operand `ValidatorRequirementV1` in the same manifest | PERSIST |
| Callable code, `max_slots`, arity, upvalue descriptors, parameter shape, selector/name, local names | `Callable` | `PortablePrototypeV1` | Prototype-local constant, source-map, and semantic-pool IDs | Frame slots, arity, local/upvalue slots, operand stack shape | PERSIST |
| `Callable.num_upvalues` | `Callable` | Rebuilt from the portable upvalue descriptor count | None | Must agree with upvalue layout | RECONSTRUCT |
| Chunk `code`, `constants`, `spans` | `Chunk` | Prototype code, constant-reference vector, and per-instruction `SpanV1` rows | Constant recipes and source map | Instruction operands, constant references, span count/order | PERSIST |
| Chunk `source_id` | `Chunk` | Prototype `SourceMapId`, mapped through the captured source table | Source-map table | Source-map reference and module ownership | RECONSTRUCT |
| `enum_specs`, `variant_targets`, `data_specs`, `data_targets`, `data_constructions`, `anonymous_product_specs`, `call_environment_recipes`, `associated_targets`, `family_descriptors`, `conditional_family_descriptors`, `family_operations`, `family_candidate_sets`, `trait_invocations`, `trait_requirement_invocations`, `type_parameter_class_receivers` | `ExecutableSemanticPool` | Corresponding vectors in `PortableSemanticPoolV1` | Stable declaration/callable/variant/implementation identities, portable type recipes, constant IDs, and pool-local IDs | Semantic recipe tags, reference bounds, shapes, and operand-specific pool domain | PERSIST |
| `variant_target_index`, `data_target_index` | `ExecutableSemanticPool` | Rebuilt from persisted target vectors | Stable variant/declaration identity | Index/vector consistency on reconstruction | DERIVE |
| `data_construction_caches`, `associated_target_caches` | `ExecutableSemanticPool` | None | None | None | RUNTIME_ONLY |
| Chunk `caches`, `gcaches` | `Chunk` | None; new chunks start with empty caches | None | None | RUNTIME_ONLY |

`OPCODE_MANIFEST_V1` contains one row for each of the 128 current portable
opcodes. Its exhaustive construction maps every operand to a fixed-width
encoding, domain, relocation requirement, and future-validator obligation.
The core projection is exhaustive over `Bytecode`; adding a source opcode
without a portable projection fails compilation. The const check rejects
duplicate wire tags.

## Values and heap constants

| Runtime family / field | Source owner | Portable representation | Relocation requirement | Validation domain | Disposition |
|---|---|---|---|---|---|
| `ValueTag::Nil`, `Unit`, `Bool`, `Int`, `Float`, `Symbol`, `None`, `EmptyRecord`, `EmptyPack`, `Char` | `ValueTag` / `Value` | Corresponding exact `ConstantRecipeV1`; float and integer payloads preserve bits; symbols persist by text | Symbol text becomes a fresh interner entry | Tag, Unicode scalar, and constant-reference checks | PERSIST |
| `ValueTag::Obj` for supported immutable compiler products | `Value` / `Object` | Object-specific constant recipe below | Artifact-local recipe identity; nested values use `ConstantId` | Constant tag, graph edges, stable owner/module/class references | PERSIST |
| `ValueTag::AdtSingleton`, `DataSingleton`, `UninitializedBinding` when found in a constant pool | `ValueTag` / `Value` | No value recipe; emitted singleton/sentinel forms use dedicated bytecodes and semantic-pool rows | Semantic-pool identity belongs to the dedicated opcode | Constant projection rejects these immediate values | UNSUPPORTED |
| String, arbitrary-precision integer, vector, frozen array, tuple, record, pack | `Object::{Str, LargeInt, Vector, Array, Tuple, Record, Pack}` | `String`, `BigInteger`, `Vector`, `FrozenArray`, `Tuple`, `Record`, `Pack` recipes | Nested immutable members share `ConstantId` references | Recipe shape and closed constant graph | PERSIST |
| Closure template | `Object::Closure` / `ClosureObject` | `ClosureTemplate` with prototype, stable module, lexical class, and super anchor | Stable prototype/module/class refs | Empty captured upvalue environment; prototype and owner refs | PERSIST |
| Class and module values | `Object::{Class, Module}` | `ConstantRecipeV1::{Class, Module}` with class construction details in class recipes | Stable declaration/module identities | Class/module reference closure | PERSIST |
| Method value and detached method registry entry | `Object::Method`, `MethodObject`, `VM::detached_method_objects` | `MethodRecipeV1` plus `PortableDetachedMethodV1` | Closure prototype or native anchor; stable holder/access/origin identities; constant refs for contracts and attributes | Method signature, visibility, owner, and body-kind checks | PERSIST |
| Selector pattern | `Object::SelectorPattern` | Structural `SelectorPatternV1` | None beyond portable strings/tags | Pattern tag and shape checks | PERSIST |
| Mutable array; `Instance`, `Block`, `BoundMethod`, `Upvalue`, `List`, `Buffer`, `Slice`, `Fiber`, `Map`, `Set`, `MutableBag`, `IndexedDict`, `OrderedSet`, `PersistentHashNode`, `Bag`, `Bijection`, `Bytes`, `Range`, `Family`, `Selector`, `MethodFamily`, `BoundMethodFamily` | Corresponding `Object` variants in `heap/object.rs` | No current constant recipe | None | Rejected with contextual `UnsupportedConstant` if reached as an emitted constant | UNSUPPORTED |
| `ArrayLiteralBuilder`, `PackBuilder`, `RecordLiteralBuilder`, `Project`, `ProjectManifest`, `PackageInfo`, `PackageAuthor`, `PackageRequirement`, `ResolvedProjectDependency`, `ModuleDependency`, `ExportTable`, `Export`, `ChildModuleTable`, `ModuleIdentity`, `PackageIdentity`, `ProjectIdentity`, `Uri`, `Typing`, `AdtCase`, `Data`, `AssociatedFamily` | Corresponding `Object` variants in `heap/object.rs` | No current constant recipe | None | Rejected with contextual `UnsupportedConstant` if reached as an emitted constant | UNSUPPORTED |
| Build-VM `ObjRef`, `ClassId`, `Symbol`, method cache entries, object addresses | Heap, interner, and method side tables | None; projection maps them to recipes or stable identities before emission | No runtime coordinate is a wire identity | Full validator later checks portable references | RUNTIME_ONLY |

The `ConstantRecipeV1::Nil` tag remains a structural recipe for the private
sentinel. It does not turn that sentinel into a surface value; executable use
must be rejected by the later validator unless a future accepted contract
defines a safe construction role.

## Class, module, and native products

| Runtime family / field | Source owner | Portable representation | Relocation requirement | Validation domain | Disposition |
|---|---|---|---|---|---|
| Class identity, superclass/metaclass, ordered instance/class field layouts, slot counts, immutable slots, native representation, sealed and abstract flags | `ClassObject`, `VM::field_layouts` | `PortableClassRecipeV1` | Stable class references | Closed class graph, exact layout slots/counts, primordial role resolution | PERSIST |
| Class method maps, rest-method maps, base-name index | `ClassObject` | Rebuilt by native installation and initializer bytecode/method recipes | Class and method recipe references | Method attachment and class finalization belong to future validation/materialization | RECONSTRUCT |
| Class static slot values and attached attribute instances | `ClassObject` | No initialized slot snapshot; executable initializer/method recipes describe installation | Constant recipes only where explicitly emitted | No build-VM mutable state | RUNTIME_ONLY |
| Module identity, initializer, linked reads, source URI/span, eager/deferred role | `ModuleObject`, canonical module program | `PortableModuleV1`, `PortableSourceMapV1`, bootstrap rows | Stable module identity and prototype/source-map IDs | One module row and one eager/deferred disposition; linked target closure | PERSIST |
| Module globals, static-constant values, name/slot maps, source text, runtime exports, package/project objects, metadata caches | `ModuleObject` | None; initializer and compiler image reconstruct runtime state | Stable module references only where independently represented | No initialized global or source-text payload | RUNTIME_ONLY |
| Native key, owner, selector, dispatch side, signature/arity, contract identity and `PrimitiveAbi` shape | Native registry / method implementation side table | `PortableNativeAnchorV1` and native ABI digest | Stable key and stable owner/selector identities | Exact ABI/contract comparison in future validation | PERSIST |
| Native function pointer and live installed primitive method | Native registry / `MethodKind::Primitive` | None; resolve against compiled-in registry | Native anchor ID points to stable key | Native ABI compatibility and binding resolution | RECONSTRUCT |
| Required primordial roles and native runtime floor | `UNIVERSE_BINDINGS`, runtime bootstrap | `PrimordialRoleV1`, bootstrap role list, class allocation recipe | Stable class identity and role key | Role-to-class closure and ABI digest | PERSIST |
| Role-to-class and native binding identity selection | Canonical native/module authority | Stable binding key, declaration reference, and primordial role | Stable identity mapping | Exact authority resolution before publication | DERIVE_FROM_CANONICAL_AUTHORITY |
| Runtime initializer state and capture scheduling state | VM bootstrap | Eager order is portable; build schedule is not | Stable module identities | Runtime bootstrap order checked separately from capture order | RUNTIME_ONLY |

## Structural staging boundary

`StagedExecutableSectionV1` contains the encoded structurally valid image. A
container may stage it as section tag 7 with executable schema 1 while leaving
`PORTABLE_EXECUTABLE` clear. `TeloscContainerBuilder::stage_executable_section`
rejects the stage if that capability is already set. The later executable
validator must produce the authority required for publication; staging alone
does not do so.
