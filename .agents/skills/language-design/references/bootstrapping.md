# Bootstrapping & Self-Hosting

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing how the language (or its kernel/stdlib) is brought up — bootstrap ordering, self-hosting, metacircular kernel, image, VM-blessed primitives.

## Contents
- Bootstrap strategy
- Self-hosting stages
- Kernel written in the language itself
- Object-model / metaclass bootstrap
- Absence / primitive bootstrap cycle
- Image-based vs source-based startup
- Trusting trust
- Bootstrap ordering of the stdlib

## Bootstrap strategy
| Option | Langs | Consequence |
|---|---|---|
| Stage0 in another lang, then self-host | rustc (OCaml→Rust), Go (C→Go), TS | Dogfoods the language; seed compiler retired but archived. |
| Metacircular interpreter | Lisp `eval`, PyPy (RPython), Squeak | Language defines itself; needs a host to run the metacircle. |
| Permanent host implementation | CPython (C), CRuby (C), V8 (C++) | No self-host; portable via one C toolchain; less dogfooding. |
| Bytecode VM + kernel-in-language | Smalltalk, Phalcom | Tiny native VM; core classes authored in the language. |
| Bootstrap from a partial subset | Oberon, PL/0, Ur-Scheme | Seed compiles a subset; full language built atop it in-language. |

**Impl.** Stage0 = a throwaway compiler in a mature host lang emitting the first self-hosting binary; metacircle = an interpreter whose primitives the host provides.
**Hazard — self-host seed ⊗ reproducibility/trust (CROWN JEWEL).** A lost or backdoored seed binary makes the whole toolchain unauditable — you can no longer rebuild from source without an existing binary. Mitigate by archiving seeds + diverse double-compilation.
**Hazard — self-host ⊗ new-feature use in own source.** A compiler that starts using a feature it just added can't be built by the prior seed; you must stage the feature in one release before consuming it. → overlay

## Self-hosting stages
| Option | Langs | Consequence |
|---|---|---|
| stageN compiles stageN+1, fixpoint check | rustc, Go, GCC | 3-stage build; stage2==stage3 proves compiler is a fixpoint. |
| Single self-compile, no fixpoint | small self-hosters | Cheaper; can't detect a compiler that miscompiles itself. |
| Cross-compiler seed from foreign arch | Go (bootstrap toolchain), GCC | New port seeded by cross-compiling; seed must be reproducible. |
| mrustc-style alt-impl seed | Rust (mrustc) | Independent seed in another lang breaks binary-only trust chain. |

**Impl.** Reproducible build = pin seed hash + deterministic codegen; stage2/stage3 byte-identical output is the self-consistency gate.
**Hazard — bootstrap chain ⊗ archival.** If any link (a specific old compiler version) is unarchived, the chain from source is broken; future builds silently depend on a binary nobody can regenerate. → overlay
**Hazard — nondeterministic codegen ⊗ fixpoint gate.** Hashmap-ordered symbol emission or embedded timestamps make stage2≠stage3 even for a correct compiler; the self-consistency check then flags false failures.

## Kernel written in the language itself
| Option | Langs | Consequence |
|---|---|---|
| Full core library in-language | Smalltalk image, Phalcom `core.ph` | Max dogfooding; startup must load/compile kernel before user code. |
| Minimal native runtime + language stdlib | Java (rt.jar over JVM), Ruby | Primitives native; most stdlib in the language. |
| Everything native | CPython C stdlib, Lua C libs | Fast cold start; stdlib can't be reshaped/introspected as objects. |
| Hybrid: native primitives, self-defined control | Smalltalk `ifTrue:`, Phalcom blocks | Control flow is library-level messages atop VM-blessed `Boolean`/`Block`. |

**Impl.** VM-blessed set = only what can't be expressed in-language: object allocation, primitive arithmetic, the `Boolean`/`Block`/`nil` roots, `become:`, message-send. Everything else is method source loaded at boot.
**Hazard — native-vs-library boundary creep.** Each primitive pulled into the VM "for speed" shrinks the dogfooded surface and freezes semantics the language could otherwise redefine; the boundary ratchets one way. → overlay
**Hazard — kernel method ⊗ primitive it depends on.** A `core.ph` method that sends a message whose only implementor is a not-yet-installed primitive fails at load, not at call; primitive registration must precede kernel source. → overlay

## Object-model / metaclass bootstrap
| Option | Langs | Consequence |
|---|---|---|
| Allocate-then-patch the apex | Smalltalk, Phalcom, Ruby | Root objects created uninitialized, then wired by hand; no ctor runs. |
| Reflective `type`-closes-loop | Python (`type` is-a `type`) | `type.__class__ is type`; interpreter special-cases the fixpoint at init. |
| Ordinary construction, no cyclic apex | Java (`Class` is native, not cyclic) | No hand-wiring; also no uniform metaclass tower. |

**Impl.** The cycle `Metaclass instanceOf itself`, `Object`↔`Class`, `Class isKindOf Object` can't be built by normal `new` (each needs the others). Allocate raw cells, then backpatch `class` pointers and superclass links before any user allocation.
**Hazard — metaclass tower ⊗ inheritance direction.** A metaclass's superclass must mirror the instance-side super (`Point class` super is `Object class`), but `Object class` super is `Class` — the tower's top link breaks the mirror and must be special-cased. → overlay
**Hazard — primitive bootstrap cycle (CROWN JEWEL).** The metaclass apex, `nil`, and `true`/`false` cannot be produced by ordinary construction — building them needs classes whose own fields default to objects not yet built. Must VM-bless the roots and backpatch. Guard the finished graph with a `verify_invariants` pass (every object has a class; tower closes). → overlay
**Hazard — patch order ⊗ verify timing.** Running `verify_invariants` before every backpatch completes reports spurious failures; running it never lets a missed link ship — gate it exactly once at end-of-bootstrap. → overlay

## Absence / primitive bootstrap cycle
| Option | Langs | Consequence |
|---|---|---|
| VM-bless the singletons | Smalltalk `nil`/`true`/`false`, Phalcom | Interpreter mints them before class loading; classes patched in later. |
| Immediate/tagged, no allocation | Lua `nil`, tagged `false` | Absence is a bit pattern, not an instance; sidesteps the cycle entirely. |
| Interned literal, class assigned late | Ruby `nil` (`NilClass`) | Singleton exists early; `NilClass` back-links once `Class` exists. |

**Impl.** If `nil`/`true`/`false` are real objects, constructing them needs `NilClass`/`Boolean`, whose slots default to `nil` → cycle. Bless the singleton values in native code, defer their `class` backpatch. See [recipes.md#option-niche](recipes.md#option-niche) for representing absence without an allocation.
**Hazard — absence cycle ⊗ default field init.** Any class whose instance-var default is `nil` cannot be the class that defines `nil`; ordering the singleton before its class is mandatory, not stylistic. → overlay
**Hazard — singleton identity ⊗ multiple blessings.** Blessing `nil`/`true`/`false` more than once (VM init + kernel re-declaration) mints distinct objects; `==` on absence then fails intermittently. → overlay

## Image-based vs source-based startup
| Option | Langs | Consequence |
|---|---|---|
| Serialized live-object image | Smalltalk (`.image`), Pharo | Instant resume of a live world; state drifts from source over time. |
| Re-parse/compile source each boot | Phalcom `core.ph`, Lua, Ruby | Reproducible from text; pays parse+compile cost every startup. |
| Compiled startup snapshot | V8 startup snapshot, Node SEA | Fast boot + reproducible-from-source if snapshot is a build artifact. |
| Precompiled bytecode cache | Python `.pyc`, JVM CDS | Skips parse; cache invalidation keyed on source mtime/hash. |

**Impl.** Image = heap serialized to disk and mmap'd back; source-based = deterministic rebuild but slower cold start; snapshot = image built deterministically at compile time.
**Hazard — image staleness.** A live image accumulates state (open files, patched methods, ad-hoc objects) that no source file records; rebuilding from source yields a different world, and the image becomes the only source of truth. → overlay
**Hazard — snapshot ⊗ layout change.** A serialized heap encodes concrete slot offsets and class shapes; changing instance layout invalidates every old image/snapshot unless a migration/version tag is embedded.

## Trusting trust
| Option | Langs | Consequence |
|---|---|---|
| Trust the seed binary | most self-hosters | A compromised seed can inject a self-propagating backdoor invisibly. |
| Diverse double-compilation (DDC) | GCC/Rust audits | Compile with two independent compilers; identical output refutes the attack. |
| Independent alt-implementation seed | Rust (mrustc), Scheme (many) | A from-source seed in another lang breaks the binary-only trust chain. |
| Fully bootstrappable from tiny binary | GNU Mes / live-bootstrap | Auditable chain from a ~500-byte seed up; maximal trust, high effort. |

**Impl.** DDC: build compiler C with foreign compiler A and with itself; if `C_A` and `C_C` produce byte-identical stage2, a seed-resident backdoor could not have survived both paths.
**Hazard — DDC ⊗ nondeterminism.** Diverse double-compilation's proof rests on byte-identical output; any nondeterministic codegen defeats the comparison and hides the very attack DDC exists to catch.
**Hazard — self-host seed ⊗ trust (CROWN JEWEL).** The Thompson attack hides in the binary, not the source — reading the source proves nothing. Only an independent seed or DDC detects it; a single trusted binary is an unbounded trust assumption. → overlay

## Bootstrap ordering of the stdlib
| Option | Langs | Consequence |
|---|---|---|
| Explicit dependency DAG, topo-loaded | Smalltalk kernel, Phalcom | Load order fixed; `Object`→`Behavior`→`Class`, `Boolean` before control flow. |
| Two-pass: declare all, then define | many linkers/loaders | Forward refs allowed; bodies resolved after all names exist. |
| Lazy/on-demand class init | Java `<clinit>`, .NET | Class initialized at first use; init cycles can deadlock or see half-built state. |
| Flat prelude, author-ordered | Lua, small langs | Simple; one misordered definition = load-time failure. |

**Impl.** Kernel DAG: `Object` → `Behavior`/`ClassDescription` → `Class`/`Metaclass`; `Boolean` before `ifTrue:`-style control; collections before any method taking varargs/keyword bundles; reflection last (it consumes the finished tower).
**Hazard — kernel load order ⊗ inter-class deps (CROWN JEWEL).** A class referenced before its dependency is defined fails at image-build time — e.g. defining a method that sends a collection message before the collection class exists, or using `true`/`false` before `Boolean`. One wrong edge in the DAG = hard boot failure with no user-code frame to blame. → overlay
**Hazard — lazy init ⊗ boot cycle.** On-demand class initialization can enter a class that is mid-init (its `<clinit>` triggered the reference); the runtime hands back a half-built class rather than deadlocking, and callers see missing methods.
