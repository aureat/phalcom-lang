# Type Systems

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing typing — static/dynamic/gradual, inference, nominal/structural, generics, variance, soundness, or runtime type info.

## Contents
- When checking happens: static vs dynamic vs gradual
- Strong vs weak / implicit coercion
- Nominal vs structural
- Type inference
- Parametric polymorphism / generics
- Subtyping & variance
- Soundness & escape hatches
- Runtime type info & reflection

## When checking happens: static vs dynamic vs gradual
| Option | Langs | Consequence |
|---|---|---|
| Static, checked pre-run | ML, Rust, Haskell, Java | Errors at compile; types erasable; strong tooling/refactors. |
| Dynamic, tag-checked at op | Smalltalk, Python, Ruby, JS | Errors at first bad send; every value carries a tag. |
| Gradual, unsound (erase) | TypeScript, Dart | Annotations guide tooling; erased at runtime; no boundary check. |
| Gradual, sound (contracts) | Typed Racket, Sorbet(`typed:`) | Typed↔untyped boundary inserts runtime casts/contracts. |
| Optional retrofit onto dynamic | mypy, Sorbet, TS/JS | Checker is a separate pass; runtime unchanged, lies possible. |

**Syntax.** `let x: int = 1` (ML/Rust) · `int x = 1` (Java) · `x: int = 1` (Python hint) · `let x: number` (TS) · none (Smalltalk/JS) · `(: x Integer)` (Typed Racket).
**Impl.** Static = constraint solve then erase; dynamic = tag on every value, checked at each primitive; gradual = insert `Dyn`↔`T` coercions at boundaries (sound) or drop them (unsound).
**Hazard — gradual ⊗ soundness/performance (CROWN JEWEL).** Sound gradual typing pays the "gradual guarantee" tax: every typed↔untyped crossing needs a runtime cast/contract/wrapper, and higher-order values need proxies — pathological slowdowns (Typed Racket 10–100×). Unsound gradual (TS) skips the tax by erasing and simply *lying* when data crosses the boundary. No middle ground is free. → overlay
**Hazard — dynamic ⊗ refactoring/tooling.** No types means rename/find-callers/dead-code analysis is heuristic; the checker cannot prove a send resolves, so IDEs guess and large-scale refactors are unsafe. → overlay

## Strong vs weak / implicit coercion
| Option | Langs | Consequence |
|---|---|---|
| No implicit coercion | Python, Haskell, Rust | Mismatched op is an error, not a silent convert. |
| Weak numeric/string coercion | JS (`+`), PHP, Perl | `1 + "2"` "works"; source of whole bug classes. |
| Pointer/reinterpret weakness | C, C++ casts | Types reinterpretable at will; memory unsoundness by design. |
| Explicit-conversion only | Go, Rust, OCaml | Every widening/narrowing is written; verbose but unambiguous. |

**Syntax.** `x + y` errors on mismatch (Python) · JS auto: `"a"+1→"a1"`, `1-"a"→NaN` · Go requires `int64(x)+y` · Rust `x as u8` · Haskell `fromIntegral x`.
**Impl.** Strength = whether the compiler/runtime inserts conversions; it is **orthogonal to static/dynamic** — Python is dynamic+strong, C is static+weak, Haskell static+strong, JS dynamic+weak. All four corners exist.
**Hazard — weak coercion ⊗ `==`.** Coercing equality yields intransitive/asymmetric results (JS `0==""`, `0=="0"`, but `""!="0"`); forces a second strict operator (`===`) and permanent two-equality confusion. → overlay

## Nominal vs structural
| Option | Langs | Consequence |
|---|---|---|
| Nominal (name identity) | Java, Rust, Swift, C# | Conformance is declared; no accidental match; explicit intent. |
| Structural (shape identity) | TS, Go interfaces, OCaml objects | Fits if shape fits; decoupled but accidental conformance possible. |
| Duck typing (dynamic structural) | Smalltalk, Python, Ruby | "Responds to the message" checked at send time, per call. |
| Nominal + structural mix | Scala (refinement), TS(classes) | Named types plus width/structural refinements coexist. |

**Syntax.** Java `class C implements I` (declared) · Go `type R interface{ Read(...) }` (implicit) · TS `{ x: number }` structural · OCaml `< m : int >` object type · Ruby: no declaration, just call.
**Impl.** Nominal = compare a declared-supertype set / name; structural = recursively compare fields+method sigs (coinductive for recursive types); duck = negative-lookup at runtime → missing-method hook.
**Hazard — structural ⊗ accidental conformance.** Two unrelated types with the same shape are interchangeable even when semantically incompatible (a `Point{x,y}` satisfies a `Vector{x,y}` API); nominal forbids this by construction. → overlay
**Hazard — duck typing ⊗ error locality.** A shape mismatch surfaces as `doesNotUnderstand`/`AttributeError` deep in the callee, not at the call site that supplied the wrong object. → overlay

## Type inference
| Option | Langs | Consequence |
|---|---|---|
| None / manifest | Java(pre-`var`), C, Go(params) | Every binding annotated; zero inference surprises, high verbosity. |
| Local / bidirectional | Rust, Kotlin, Swift, TS, C#(`var`) | Locals inferred, signatures annotated; predictable, composes w/ subtyping. |
| Global Hindley-Milner | ML, Haskell (no ann needed) | Whole-program principal types; annotations optional but errors non-local. |
| HM + extensions | OCaml, Haskell(GADTs, RankN) | Power features force annotations HM alone can't infer. |

**Syntax.** `let x = 1` (infers) both Rust and ML · Rust needs `fn f(x: T) -> U` sigs · ML infers even top-level sigs · Kotlin `val x = expr` · Java `var x = expr` (locals only).
**Impl.** HM = unification over type variables → principal type (Algorithm W); bidirectional = alternate *check* (type given) and *synthesize* (type derived) modes so annotations are needed only at mode switches.
**Hazard — inference ⊗ subtyping (CROWN JEWEL).** Full global HM and subtyping don't compose: unification wants one equality (`α = τ`) but subtyping wants inequalities (`α <: τ`), giving huge unreadable constraint sets and no principal type. This is precisely why Rust/Swift/TS/Kotlin use *local bidirectional* inference instead of global HM. A dynamic language adding both later inherits the same wall. → overlay
**Hazard — HM ⊗ error locality.** A type mismatch is reported where unification finally fails, often far from the real mistake ("expected int, got string" pointing at an innocent later use). → overlay

## Parametric polymorphism / generics
| Option | Langs | Consequence |
|---|---|---|
| Monomorphization | Rust, C++ templates | One specialized copy per instantiation; fast, code-size blowup. |
| Erasure | Java, Haskell(dicts) | One shared body; no runtime type args; smaller, less specialized. |
| Reified generics | C#/.NET, Dart | Type args retained at runtime; `typeof T`, `new T[]` legal. |
| None (no generics) | Go(pre-1.18), early Java | `interface{}`/`Object` + casts; type info lost at boundary. |

**Syntax.** `Vec<T>` (Rust) · `List<T>` (Java, erased) · `List<T>` (C#, reified) · Haskell `[a]` / `Ord a =>` · Go `[T any]`.
**Impl.** Monomorph = instantiate+specialize per type; erasure = compile to one body, pass a dictionary (Haskell) or just `Object`+casts (Java); reified = carry the type descriptor in the object header / call.
**Hazard — erasure ⊗ runtime dispatch/overloading (CROWN JEWEL).** Erased generics can't dispatch, overload, or reflect on the type argument: `List<String>` and `List<Int>` are indistinguishable at runtime, so `void f(List<String>)`/`f(List<Int>)` won't compile as overloads and the JVM must synthesize *bridge methods* to reconcile erased overrides. `new T[]` is impossible. → overlay
**Hazard — monomorphization ⊗ code size / compile time.** Every distinct instantiation emits a full copy; deep generic stacks explode binary size and compile time (C++ template bloat, Rust build times). → overlay

## Subtyping & variance
| Option | Langs | Consequence |
|---|---|---|
| Invariant | Rust, Java generics(default), C# | `List<Cat>` not a `List<Animal>`; safe, sometimes annoying. |
| Covariant (declaration-site) | Scala(`+T`), Kotlin(`out`), C#(`out`) | Producer position; safe only when read-only. |
| Contravariant (declaration-site) | Scala(`-T`), Kotlin(`in`) | Consumer position; `Comparable`/callback params. |
| Use-site variance | Java wildcards(`? extends`/`? super`) | Variance chosen per use; verbose, PECS ceremony. |
| Unsound covariant builtins | Java/C# arrays, TS | Convenient but holes — checked at runtime or not at all. |

**Syntax.** Kotlin `interface Src<out T>` / `Sink<in T>` · Scala `class L[+A]` · Java `List<? extends Animal>` (use-site) · C# `IEnumerable<out T>` · Rust: variance inferred from usage, no syntax.
**Impl.** Declaration-site = compiler checks each member's position (out=covariant-only, in=contravariant-only); use-site = wildcard restricts the API surface at that reference; array covariance = runtime store-check.
**Hazard — variance ⊗ mutable containers (CROWN JEWEL).** Covariant *mutable* collections are unsound: if `Cat[] <: Animal[]`, then `animals[0] = aDog` type-checks but corrupts the store. Java/C# "solve" it with a runtime `ArrayStoreException` — the classic soundness hole. The rule: covariant reads + contravariant writes ⇒ a read-write container must be invariant. → overlay

## Soundness & escape hatches
| Option | Langs | Consequence |
|---|---|---|
| Sound (well-typed can't crash) | ML, safe Rust, Haskell | Type = a real guarantee; no runtime type errors in safe subset. |
| Deliberately unsound | TS, Dart, Java(arrays) | Ergonomics over guarantee; some "typed" programs still throw. |
| `any` / dynamic escape | TS `any`, C# `dynamic`, Dart | Opts a value out of checking; infects everything it touches. |
| Explicit unsafe block | Rust `unsafe`, Haskell `unsafeCoerce` | Localized, auditable; obligation on the author, not the type. |
| Cast / `Obj` widening | Java cast, Smalltalk (all `Object`) | Runtime-checked (Java) or unchecked (C) hole punched on demand. |

**Syntax.** `x as any` (TS) · `unsafe { *p }` (Rust) · `(Cat) obj` (Java, runtime-checked) · `unsafeCoerce x` (Haskell) · `Object.cast()` no-op in fully-dynamic langs.
**Impl.** Soundness = a proof that reduction preserves types (progress+preservation); escape hatches insert an unchecked or runtime-checked coercion that the static system then *trusts* unconditionally.
**Hazard — `any`/`Obj` ⊗ soundness erosion.** `any` is viral: it disables checking on every value derived from it, and unlike `unknown` it flows outward silently — one `any` at an API seam can void guarantees across a whole subsystem while still "type-checking." → overlay
**Hazard — TS bivariant params ⊗ unsound overrides.** TS method params are bivariant by default, so a narrower-typed override is accepted; callers pass a valid supertype and the override receives a value it wasn't typed for — a known, deliberate hole. → overlay

## Runtime type info & reflection
| Option | Langs | Consequence |
|---|---|---|
| Tagged values / RTTI | Smalltalk, Python, JS, JVM | Every object knows its class; `class`/`isKind:`/`instanceof` cheap. |
| Erased, no runtime type | OCaml, Haskell, Java generics | Nothing to reflect on; can't switch on a type variable. |
| Retained/reified descriptors | C#, Java(classes), Go | `typeof`/`reflect`/`GetType` enumerate fields+methods at runtime. |
| Type-case / pattern on type | Rust(enum match), OCaml, Scala | Dispatch on a *closed* variant set, checked exhaustive. |
| Downcast test | Java `instanceof`, Rust `Any::downcast` | Recover a static type from a dynamic one, guarded. |

**Syntax.** Smalltalk `x isKindOf: Cat` / `x class` · JS `x instanceof C` · Java `x instanceof Cat c` · Rust `match e { Variant(..) => }` · Go `switch v.(type)`.
**Impl.** Tag = class pointer in the object header; dynamic dispatch reads the tag to pick a method — so **types ≠ classes**: a class is a runtime dispatch tag, a type is a static claim, and dynamic languages have only the former. Reflection walks the retained descriptor; erased langs have no descriptor to walk.
**Hazard — types ≠ classes ⊗ dispatch model.** In a class-based dynamic language, "type" is just "class of the receiver at this send"; there is no static type to check against, so the only "type error" is a runtime `doesNotUnderstand`/`method_missing` on negative lookup. Bolting a static checker on later must reconstruct types the dispatch model never recorded. → overlay
**Hazard — erasure ⊗ type-directed reflection.** Serializers/DI/`instanceof T` that need the type argument at runtime cannot get it under erasure; libraries resort to reified type tokens (`TypeToken`, `Class<T>` args) to smuggle it back in. → overlay
