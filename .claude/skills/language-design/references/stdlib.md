# Core & Standard Library

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing what's primitive vs library, the core/std split, prelude, collections, module/namespace layout, or API stability.

## Contents
- Axis 1 — Primitive vs library boundary
- Axis 2 — Core vs std split
- Axis 3 — Batteries-included vs minimal
- Axis 4 — Prelude / auto-import
- Axis 5 — Collection library design
- Axis 6 — Modules / namespaces / imports
- Axis 7 — API stability & evolution

## Axis 1 — Primitive vs library boundary
| Option | Langs | Consequence |
|---|---|---|
| Thin native kernel, rest in-language | Smalltalk image, Lisp, Self | Auditable, hackable, self-hosting; slow hot ops until JIT |
| Native primitives + in-language core file | Phalcom `core.ph`, Ruby (C + `.rb`) | Small trusted base; core needs native hooks to bootstrap |
| Fat native runtime, library is thin veneer | CPython (C builtins), V8 | Fast builtins; behavior frozen in C, hard to override/inspect |
| Everything native, no user-visible core | Go runtime, Lua C core | Predictable perf; no in-language introspection of primitives |

**Must be native (irreducible):** arithmetic on the unboxed number repr, `Object` identity/allocation, `Bool` branch primitive, absence sentinel, block/closure `call`, message-send itself. Everything reachable *from* these can be `.ph`.
**Syntax.** Phalcom `<primitive: name>` / Ruby `def x; end` backed by a C func / Smalltalk `<primitive: 1>` fallback to Smalltalk body.
**Impl.** Primitive = Rust fn in a dispatch table keyed by selector; core class methods either mark a primitive slot or run bytecode.

**Hazard — primitive/library boundary ⊗ bootstrap order (CROWN JEWEL).** A class that *looks* like library (e.g. `List`) is secretly required by the VM before user code runs — `Message.args`, rest/variadic params, and reflective `send` all materialize a `List`. It must be allocated and wired before any feature that consumes it, or bootstrap deadlocks. Order native-allocate → wire supers → load `core.ph`. See object-model.md (bootstrap ordering) + values.md (Option/nil bootstrap cycle). → overlay

## Axis 2 — Core vs std split
| Option | Langs | Consequence |
|---|---|---|
| Layered `core`/`alloc`/`std` | Rust | `no_std` embeddable; churn moving items between layers |
| One monolithic std | Go, Java | Simple mental model; everything assumes an OS/allocator |
| Live image, no file std | Smalltalk | Ship whole world; no minimal deployment, huge image |
| Tiny core + external ecosystem | Lua, JS+npm, C | Maximally embeddable; users assemble a de-facto std |

**Consequence axis:** embeddability & portability. The lower the layer that assumes an allocator/OS, the harder to run on bare metal or as an embedded VM.
**Impl.** Layer boundary = which primitives a module may call: `core` touches no heap; `alloc` may allocate; `std` may syscall. Enforced by what native hooks each layer links.

**Hazard — layer leak ⊗ allocation.** A `core`-layer type that quietly calls an alloc-only primitive (a `Display` that builds a `String`) drags the allocator into the no-alloc layer, silently breaking embeddability — caught only when someone tries `no_std`. → overlay
**Hazard — re-layering ⊗ stability.** Moving a type between `core`/`std` changes its import path; even a pure relocation is a source break unless the old path is re-exported forever (Rust keeps `std` re-exports of `core` items). → overlay

## Axis 3 — Batteries-included vs minimal
| Option | Langs | Consequence |
|---|---|---|
| Big curated std | Go | One blessed answer per task; slow to add, slow to fix |
| Big sprawling std | Python | "where modules go to die"; dead `urllib2`/`asyncore` linger |
| Minimal core + package registry | JS+npm, Rust+crates | Fast evolution; supply-chain surface, `left-pad`, dep churn |
| Std as versioned packages | Rust `std` + crates, Deno | Best of both; governance overhead deciding what's blessed |

**Consequence axis:** dependency culture & security surface. A blessed std suppresses trivial deps but bottlenecks fixes; a thin core pushes everything to a registry you don't control.

**Hazard — batteries ⊗ security surface.** Every shipped std module is attack surface you maintain forever (Python `pickle`, `xml` billion-laughs). A "helpful" std HTTP/deserialize module becomes a CVE channel you can't drop without breaking users. Ties to security.md. → overlay
**Hazard — thin core ⊗ transitive trust.** A minimal core pushes essentials to a registry, so a trivial dep tree pulls dozens of unaudited transitive packages (`left-pad`, `event-stream` backdoor); the "small language" has a huge effective TCB. → overlay

## Axis 4 — Prelude / auto-import
| Option | Langs | Consequence |
|---|---|---|
| Implicit prelude, always in scope | Haskell `Prelude`, Rust `std::prelude` | Ergonomic; adding a name can shadow user identifiers |
| Global flat namespace | Smalltalk, early JS globals | Everything visible; collisions, no encapsulation |
| No prelude, explicit imports | Python (mostly), Java | No surprise names; boilerplate at every file head |
| Versioned/opt-in prelude | Rust editions, Haskell `NoImplicitPrelude` | Evolve without breaking; edition machinery cost |

**Syntax.** Rust auto `Option`,`Vec`,`String` in scope · Haskell `import Prelude hiding (map)` · Smalltalk any global by bare name · Python must `from x import y`.
**Impl.** Prelude = a name set injected into every compilation unit's root scope before user bindings resolve.

**Hazard — prelude ⊗ backward compat (CROWN JEWEL).** Adding a name to an implicit prelude injects it into *every* file in the ecosystem; any user who already bound that identifier now shadows or clashes with the new global. Rust dodges this with editions (new prelude items gated per edition); Haskell needs `hiding`. A flat-global lang can never safely grow the prelude. Design prelude additions as edition-gated or never. → overlay

## Axis 5 — Collection library design
| Option | Langs | Consequence |
|---|---|---|
| Protocol/trait over concrete types | Rust `Iterator`, Ruby `Enumerable`, Haskell `Foldable` | One impl, many types; risk of megamorphic dispatch |
| Concrete blessed types | Go slices/maps, early Java | Fast, monomorphic; every algorithm rewritten per type |
| Persistent/immutable structures | Clojure, Scala | Free sharing/undo; structural-sharing complexity, GC pressure |
| Mutable-by-default containers | Python, Ruby, JS arrays | Ergonomic in-place ops; aliasing bugs, no cheap snapshots |
| One primitive `List` others build on | Lua tables, Smalltalk `OrderedCollection` | Uniform; the VM itself depends on it (variadics, dNU args) |

**Syntax.** Ruby `include Enumerable; def each` · Rust `impl Iterator` + `for` desugar · Clojure `(conj v x)` returns new · Smalltalk `coll do: [:e | …]`.
**Impl.** Protocol = a selector set (`each`/`next`) every conforming class answers; iteration lowers to repeated sends unless inline-cached/specialized.

**Hazard — collection protocol ⊗ dispatch cost (CROWN JEWEL).** Routing every element access through a general `next`/`each` message means a hot loop over N element types is a megamorphic send site — the inline cache thrashes, the branch predictor stalls. Over-abstraction is a perf tax paid per iteration. Mitigate with monomorphic specialization or a native fast path for the blessed `List`. Ties to performance.md (inline caches). → overlay
**Hazard — variadics/rest-params ⊗ the one `List`.** If `foo(*args)` and reflective `send` package arguments as `List`, that class is load-bearing VM infrastructure, not a convenience — see Axis 1 crown jewel. → overlay

## Axis 6 — Modules / namespaces / imports
| Option | Langs | Consequence |
|---|---|---|
| File = module | Python, Rust (roughly), Go pkg=dir | Zero ceremony; structure dictated by filesystem |
| Explicit module declarations | OCaml, Haskell `module M where` | Rename/relocate freely; decl boilerplate |
| Flat image, no modules | Smalltalk (classes are globals) | Live coding; namespace collisions, prefix conventions (`ST-`) |
| Package + explicit exports | JS ESM `export`, Java packages | Encapsulation control; two-list (define + export) drift |

**Syntax.** Python `from pkg.mod import f` · Rust `mod`/`pub`/`use` · JS `export {f}` / `import {f}` · Smalltalk category prefixes, no real import.
**Impl.** Import = resolve module id → its export table → bind selected names locally. Visibility = an export allow-list checked at resolution.

**Hazard — cyclic module imports.** A ⇄ B imports execute one module's top level while the other is still half-initialized; the late binding sees a partial/undefined export (Python returns the half-built module object, Node caches a partial `exports`). Resolution order becomes semantically visible. Break with lazy binding, forward decls, or a dependency-ordered load. → overlay
**Hazard — flat image ⊗ collision.** A module-less image (classes as globals) has no namespace to disambiguate two libraries' `Node`/`Parser`; the only defense is manual prefix convention, which is unenforced and rots. → overlay

## Axis 7 — API stability & evolution
| Option | Langs | Consequence |
|---|---|---|
| "Never break userspace" | Linux syscalls, Go 1.x, Java | Trust + longevity; mistakes ship forever, cruft accretes |
| Stability tiers / feature gates | Rust `#[stable]`/`#[unstable]`, nightly | Experiment before commit; two-tier maintenance burden |
| Deprecate-then-remove windows | Python `DeprecationWarning`, Node | Migration path; long tail of warnings, eventual breakage |
| SemVer + free major breaks | npm, Cargo crates | Honest breakage signal; ecosystem version-skew churn |
| Editions (opt-in break) | Rust editions | Break syntax/prelude without splitting ecosystem; edition machinery |

**Impl.** Feature gate = an attribute the compiler checks against an allow flag; deprecation = a metadata bit emitting a warning at the call site; edition = a per-crate flag switching resolution/prelude/lints.

**Hazard — stdlib in std ⊗ permanence (CROWN JEWEL).** Once a std API ships stable it is effectively un-removable — every future release must carry it (Java `Date`, Python `os.path` vs `pathlib`, Node `Buffer`). A rushed signature or a leaked implementation detail (an exposed field, an over-broad type) becomes a permanent maintenance liability. Gate anything uncertain as unstable/experimental first; treat every stable std addition as forever. → overlay
**Hazard — deprecation ⊗ prelude/auto-import.** Deprecating a name that lives in the prelude warns in *every* file at once, even code that never named it explicitly — the warning is ecosystem-wide and un-silenceable short of an edition bump. → overlay
