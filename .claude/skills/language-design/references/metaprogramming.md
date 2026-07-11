# Metaprogramming & Reflection

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing macros, reflection, the metaobject protocol, compile-time staging, attributes, or dynamic code generation.

Orienting fact for a message-passing runtime (Phalcom): reflection is not a bolt-on — it *falls out* of the object model. Metaclass tower = a MOP; `doesNotUnderstand`/`perform` = message interception; `#sym`/`Family` = reified selectors; `@` = attributes. You get most metaprogramming "for free" via send interception — the bill is paid at the optimizer (see the CROWN JEWEL below).

## Contents
- Macro system kind
- Hygiene
- Compile-time evaluation / staging
- Runtime reflection / introspection
- Metaobject protocol (MOP)
- Dynamic modification
- Annotations / attributes / decorators
- Quotation

## Macro system kind
| Option | Langs | Consequence |
|---|---|---|
| None | Go, Java (pre-annotations), Smalltalk | Metaprogramming shifts entirely to runtime reflection. |
| Textual / preprocessor | C/C++ `#define` | Token substitution, no scope/type awareness; capture-prone, untooled. |
| Reader macros | Common Lisp `set-macro-character` | Extend the *reader* → new surface syntax; unbounded, hard to tool. |
| Syntactic / hygienic | Scheme `syntax-rules`/`syntax-case`, Rust `macro_rules!` | Pattern→template on AST; auto-hygiene; limited to declarative shapes. |
| Procedural | Rust proc-macro, Template Haskell, Elixir | Arbitrary code transforms AST/tokens; full power, own compile phase. |
| AST-transform / homoiconic | Julia `macro`, Elixir `defmacro`, Lisp `defmacro` | Macro sees & returns the language's own AST; power ≈ reader, better scoped. |

**Syntax.** C `#define M(x) …` · Scheme `(define-syntax m (syntax-rules () …))` · Rust `macro_rules! m {…}` + `#[proc_macro]` · Julia `macro m(ex) … end` / `@m expr` · Lisp `` (defmacro m (x) `(… ,x)) ``.
**Impl.** Textual = pre-tokenize splice; syntactic = pattern-match on syntax objects carrying scope sets; procedural = a compiler-invoked function over token/AST → new AST, run in [compiler.md](compiler.md)'s expand phase before typeck/lowering.
**Hazard — power ⊗ tooling.** Procedural/reader macros can mint syntax the IDE, formatter, and incremental compiler can't see through; more power = less static analyzability. → overlay
**Hazard — macro ⊗ pipeline phase.** Expansion must sit at a defined phase; a macro that needs type info (Rust `macro_rules!` can't see types) either runs too early or forces a second late-macro stage. See [compiler.md](compiler.md).

## Hygiene
| Option | Langs | Consequence |
|---|---|---|
| Fully hygienic (scope sets) | Racket, Scheme `syntax-rules` | Introduced bindings can't capture/be captured; referentially transparent. |
| Hygienic + controlled break | Rust `macro_rules!`, `syntax-case` | Hygienic by default; explicit `$crate`/`datum->syntax` to inject on purpose. |
| Gensym discipline (manual) | Common Lisp `defmacro` + `gensym` | Author must mint fresh names; forget once = capture bug. |
| Unhygienic | C `#define`, naive `defmacro` | Any introduced identifier can collide with user code silently. |

**Syntax.** Racket `(with-syntax …)` auto-renames · Lisp `` (let ((g (gensym))) `(let ((,g …)) …)) `` · Rust `$x:ident` binders tagged with call-site hygiene · C — none.
**Impl.** Attach a scope set / hygiene mark to every introduced identifier at expansion; resolution compares marks so macro-introduced `tmp` and user `tmp` resolve to distinct bindings; gensym = runtime-unique symbol as a poor-man's mark.
**Hazard — unhygienic macro ⊗ name capture. (CROWN JEWEL)** A macro that expands to `let tmp = …` captures — or is captured by — a user identifier of the same name, so the macro silently reads/writes the wrong binding; the entire reason hygiene exists. Unhygienic `swap(a, tmp)` breaks the day a caller has their own `tmp`. Phalcom has no macro layer today, so any future macro/quotation feature inherits this obligation up front. → overlay
**Hazard — hygiene ⊗ intended capture.** `anaphoric` macros (bind `it`) *want* capture; a fully-hygienic system forbids it, forcing an explicit unhygienic escape hatch that reopens the capture risk locally.

## Compile-time evaluation / staging
| Option | Langs | Consequence |
|---|---|---|
| None (all runtime) | Smalltalk, Ruby, Python | Constants computed at runtime; no guaranteed compile-time folding. |
| `const`/`constexpr` functions | Rust `const fn`, C++ `constexpr` | Pure subset runs at compile time; two-colored function world. |
| Forced compile-time | C++ `consteval`, Zig `comptime` | Value/type *must* materialize at compile time; strong, error locality poor. |
| Multi-stage / quasiquote staging | MetaOCaml, Terra, Template Haskell | Explicit stage annotations generate & splice code across phases. |
| Partial evaluation / JIT specialization | Julia, PyPy, Truffle | Specialize on constants/types seen; runtime-driven, not user-declared. |

**Syntax.** Rust `const fn f()` · C++ `consteval int f()` · Zig `comptime x` / `inline fn` · MetaOCaml `.< e >.` / `.~e` (splice) · TH `$(…)` splice.
**Impl.** Compile-time interpreter (miri-like) or the same VM run at build time over a *pure* effect-restricted subset; staged code is quoted AST spliced into the next stage and recompiled.
**Hazard — staging ⊗ error locality.** Errors surface *inside generated code* the author never wrote; the diagnostic points at a splice site or synthetic span, not the logical mistake — spans must be threaded through generation or the message is useless. See [compiler.md](compiler.md).
**Hazard — comptime ⊗ effect purity.** Compile-time eval must forbid I/O, ambient mutation, and nondeterminism, or the build stops being reproducible; the line between "const-evaluable" and "runtime-only" leaks into every signature. → overlay

## Runtime reflection / introspection
| Option | Langs | Consequence |
|---|---|---|
| None / static only | C, Rust(mostly), OCaml | No `obj.class` at runtime; monomorphizable, un-introspectable. |
| RTTI / limited | C++ `typeid`, Go `reflect`, Java reflection | Type identity + member enumeration; opt-in, boxed, slow path. |
| Mirror-based (capability) | Newspeak, Java `MethodHandles`(partly) | Reflection is an object you must hold; decoupled, denyable, secure-able. |
| Pervasive self-reflection | Smalltalk, Ruby, Python | `class`, `respondsTo:`, method/ivar enumeration everywhere; total openness. |

**Syntax.** Smalltalk `obj class` / `obj respondsTo: #m` / `class selectors` · Ruby `obj.class` / `respond_to?` / `instance_variables` · Python `type(o)` / `getattr` / `dir` · Java `o.getClass().getMethods()` · Phalcom `obj.class` / `respondsTo(#sel)` / `perform(#sel)`.
**Impl.** Pervasive model = the class object *is* the reflective API (method dict + slot map already live there, see [object-model.md](object-model.md)); mirror model routes access through a separate capability object so base objects expose nothing.
**Hazard — pervasive reflection ⊗ optimization. (CROWN JEWEL)** If any code can enumerate/rewrite methods and slots at runtime, the compiler can never prove a class shape or method set is closed — it defeats sealing, devirtualization, and AOT. This is the same wound as the IC ⊗ mutable-hierarchy hazard: openness the reflective API guarantees is exactly what the inline cache and [performance.md](performance.md) need forbidden. → overlay
**Hazard — mirror ⊗ ergonomics.** Capability-safe mirrors are secure but verbose; teams route around them, and the "secure by default" reflection ends up unused. See [security.md](security.md).

## Metaobject protocol (MOP)
| Option | Langs | Consequence |
|---|---|---|
| Full AMOP (dispatch/alloc/slots customizable) | CLOS (`compute-applicable-methods`, `slot-value-using-class`) | User code redefines the object system itself; ultimate power. |
| Metaclass tower | Smalltalk, Pharo, Phalcom | Classes are objects with a parallel metaclass; class-side behavior reifies. |
| Singleton/eigenclass MOP | Ruby (`class << obj`), `define_method` | Per-object metaobject; open but doubles class count. |
| `type`-as-metaclass | Python (`__new__`, `__init_subclass__`, descriptors) | One reflexive knob (`type`) customizes creation & attribute access. |
| None (fixed object model) | Java, Go, C++ | Object semantics are the compiler's; not user-reprogrammable. |

**Syntax.** CLOS `(defmethod slot-value-using-class …)` · Smalltalk/Phalcom `Point class` (the metaclass), `#sel`/`Family` reify selectors · Ruby `define_method(:m){…}` · Python `class M(type): def __new__ …`.
**Impl.** Reify classes/methods/messages as first-class objects; dispatch/allocation become overridable methods *on the metaclass*. Phalcom's tower (`Behavior`→`Class`/`Metaclass`, `(X class).super == (X.super) class`) is a MOP: class-side `construct`/`static` inherit through it.
**Hazard — MOP power ⊗ runtime invariants. (CROWN JEWEL)** Letting user code override allocation, slot access, or dispatch lets it violate the runtime's *own* invariants — e.g. Phalcom's "`nil` never leaks into `Some`", fixed slot vectors, or the metaclass bootstrap loop. A user `allocate` that skips slot init, or a custom dispatch that resurrects a dead home frame, corrupts guarantees the VM assumes hold. The `verify_invariants()` guard exists precisely because the apex is hand-wired. → overlay
**Hazard — metaclass MOP ⊗ bootstrap.** A reprogrammable metaclass must itself be created *by* the object system it reprograms; the tower has to close its own loop before any MOP hook can run (see [object-model.md](object-model.md), [bootstrapping.md](bootstrapping.md)). → overlay

## Dynamic modification
| Option | Langs | Consequence |
|---|---|---|
| `eval` of source strings | Ruby, Python, JS, Lisp | Runtime string → running code; max flexibility, injection & no-AOT. |
| Method synthesis | Ruby `define_method`, Python `setattr`, JS proto assign | Add methods at runtime; every add is a dict mutation → cache bust. |
| Open classes / monkey-patch | Ruby, Python, Smalltalk | Reopen any class incl. core; global, non-local, load-order-sensitive. |
| Send interception | Smalltalk `doesNotUnderstand`, Ruby `method_missing`, Python `__getattr__`, JS `Proxy` | Absence becomes a live code path; proxies/DSLs for free, cache never records a miss. |
| Sealed / no runtime add | Java, Rust, Wren | Method set fixed post-load; fully cacheable, devirtualizable. |

**Syntax.** Ruby `class C; define_method(:m){…}; end` · Python `setattr(C,'m',fn)` · Smalltalk `doesNotUnderstand: aMessage` · Phalcom `perform(#sel, args)` / `doesNotUnderstand(_)` / `SEND_DYNAMIC` · JS `new Proxy(t,{get})`.
**Impl.** Reify the failed/dynamic send as a `Message` (selector symbol + args); the hook fires on negative lookup, so the inline cache can't cache absence. See [dispatch.md](dispatch.md#missing-method-hooks) for the dNU/proxy interaction in full.
**Hazard — reflection/open-classes ⊗ optimization.** Same crown wound as pervasive reflection: `method_missing`/monkey-patch/`eval` mean any call site can go megamorphic and any class version can change under a cached send; ties [performance.md](performance.md) and the IC ⊗ mutable-hierarchy hazard. Phalcom keeps dNU but defers IC population so adding caches later isn't a redesign. → overlay
**Hazard — eval/dynamic-codegen ⊗ security. (CROWN JEWEL)** Compiling or `eval`-ing a string built from runtime (esp. external) data is a code-injection vector — the metaprogramming analogue of SQL injection. Phalcom has no surface `eval` today; `perform` takes *interned selector symbols only*, not arbitrary source, which is a deliberate narrowing of this attack surface. Any future `eval` must treat its input as untrusted. Ties [security.md](security.md). → overlay
**Hazard — monkey-patch ⊗ load-order determinism.** Two libraries reopening the same core class make behavior depend on require order; last patch wins, non-locally, with no diagnostic.

## Annotations / attributes / decorators
| Option | Langs | Consequence |
|---|---|---|
| Runtime-visible metadata | Java `@Ann`(RUNTIME), C# `[Attr]`, Python `@deco` | Read via reflection at runtime; enables frameworks, costs a reflective scan. |
| Compile-time / erased | Java `SOURCE`-retention, Rust `#[derive]`/attribute macros | Consumed by the compiler/macro; zero runtime footprint. |
| Executable decorators | Python `@deco` (wraps), TS decorators | Decorator *is* a function that rewrites the decorated object at def time. |
| Language-blessed attributes | Rust `#[…]`, Phalcom `@attr` | Fixed set with compiler meaning (`#[repr]`, `@construct`/`@get`/`@set`). |

**Syntax.** Python `@deco` above `def` · Java `@Retention(RUNTIME) @interface A` · C# `[Obsolete]` · Rust `#[derive(Clone)]` / `#[attr]` · Phalcom `@construct` / `@get` / `@set` (planned relationship to hand-written accessors TBD).
**Impl.** Attribute parsed as metadata attached to the AST node; either (a) lowered by a macro/derive at [compiler.md](compiler.md) expand time into generated members, or (b) serialized into class metadata for runtime reflection. Phalcom `@` is lexically distinct from `#` (symbols) and `::` (`Family`).
**Hazard — attribute processing order ⊗ determinism.** Stacked decorators/derives apply in an order the source doesn't make obvious (Python bottom-up; multiple derives unspecified inter-order); two attributes that both rewrite the same member give order-dependent results. → overlay
**Hazard — runtime-retained attributes ⊗ cost.** RUNTIME retention forces a reflective metadata scan on the hot path (annotation-driven DI/serialization); the "declarative" feature is a hidden reflective lookup. See [performance.md](performance.md).

## Quotation
| Option | Langs | Consequence |
|---|---|---|
| None | Java, Go, C | No code-as-data; metaprogramming only via strings or bytecode APIs. |
| Homoiconic quote/quasiquote | Lisp/Scheme `` ` `` `,` `,@`, Elixir `quote`/`unquote` | Code *is* the data structure; macros are list/AST transforms, trivially. |
| Typed quotation | Template Haskell `[| |]`/`$(…)`, MetaOCaml `.<>.`/`.~` | Quoted code is type-checked; splices composable & scope-aware. |
| Reified-AST objects | Julia `:(…)`/`quote`/`Expr`, Rust `quote!`/`TokenStream` | AST is a concrete type you build/pattern-match; not homoiconic but close. |

**Syntax.** Lisp `` `(a ,b ,@xs) `` · Elixir `quote do … end` / `unquote(x)` · TH `[| e |]` / `$(e)` · Julia `:(x + $y)` · Rust `quote!{ … #var … }`.
**Impl.** Quasiquote builds an AST/list literal with holes; unquote/splice evaluate sub-expressions and stitch results in; splicing (`,@` / `#(…)*`) flattens a sequence into the surrounding form. Hygiene marks (above) ride along on quoted identifiers.
**Hazard — splice ⊗ hygiene.** Unquoted user expressions carry *their* scope into the macro's template; without scope tracking the splice either captures template bindings or loses its own — quotation and hygiene are one problem, not two. → overlay
**Hazard — quotation ⊗ non-homoiconic surface.** In a non-Lisp surface syntax, `Expr` objects diverge from what the programmer typed (desugaring, operator forms); pattern-matching quoted code must target the *post-parse* AST, so macro authors work against an representation the language docs don't show. See [parsing.md](parsing.md).
