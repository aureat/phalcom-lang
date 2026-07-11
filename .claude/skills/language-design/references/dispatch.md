# Dispatch

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing method lookup, selectors/signatures, overloading, multimethods, keyword/default/variadic args, MRO/super, or missing-method hooks.

## Contents
- Lookup mechanism: message-send vs vtable vs hashtable
- Selector / signature identity
- Single vs multiple dispatch
- Keyword args & parameter labels
- Default arguments
- Variadics / rest params
- Method resolution order & super
- Missing-method hooks

## Lookup mechanism
| Option | Langs | Consequence |
|---|---|---|
| Message-send + method dict walk | Smalltalk, Ruby, Self | Fully dynamic; every send is a lookup unless inline-cached. |
| Fixed vtable index | Java, C++, Swift(non-final) | O(1) slot; no runtime method add; devirtualizable when sealed. |
| Hashtable on class + PIC | JS (V8), Lua | Dict lookup, hidden behind polymorphic inline caches for speed. |
| Static dispatch / monomorph | Rust, OCaml (non-virtual), C | Resolved at compile/link; zero runtime dispatch cost, no late binding. |
| Typeclass/witness dict passing | Haskell, Rust traits | Dispatch = implicit dictionary arg; resolved by type, not value, at instantiation. |

**Syntax.** `recv.msg(a)` (Java/JS) · Smalltalk `recv msg: a` · Lua `t:m(a)` · Rust `x.m(a)` (trait) · Haskell `m x` (class-constrained).
**Impl.** Message-send walks the method dict behind an inline cache ([recipes.md#inline-cache](recipes.md#inline-cache)); vtable = index load; static = direct call; typeclass = dictionary-passing.

**Hazard — vtable index ⊗ runtime method add.** Fixed vtable slots assume a closed method set; a language that also allows runtime `addMethod` cannot use pure vtable indexing without rebuild. → overlay

## Selector / signature identity
| Option | Langs | Consequence |
|---|---|---|
| Name only | Python, JS, Lua, Ruby | One name = one method; redefinition overwrites; no arity overloading. |
| Name + arity + kind (encoded selector) | Smalltalk, Phalcom-style | `foo` and `foo:` and `foo(_)` are distinct selectors; coexist cleanly. |
| Keyword-message selector | Smalltalk (`at:put:`), ObjC | Selector = concatenation of keyword parts; argument order fixed by name. |
| Name + full static type signature | Java, C++, Swift overloads | Overload set resolved by static arg types; needs a type system to pick. |
| Symbol interning of selectors | Smalltalk, Ruby symbols | Selectors are interned → identity compare + fast dict key. |

**Syntax.** Smalltalk `at:put:` · ObjC `[d setObject:o forKey:k]` · Swift `move(to:)` · Java overloads `foo(int)`/`foo(int,int)` · Python `foo` (one name).
**Impl.** Interned selector symbol keys the dict; name+arity+kind fused into one symbol; static overload = name-mangled, resolved at compile time.

**Hazard — name-only ⊗ overloading.** Name-only selector identity makes true overloading impossible; the "second" definition silently clobbers the first (Python, JS). → overlay
**Hazard — static-type selector ⊗ dynamic receiver.** Overload picked by *static* arg types but method picked by *dynamic* receiver = the Java "overload chooses at compile time, override at runtime" split-brain surprise. → overlay
**Hazard — keyword selector ⊗ arg reorder.** With `at:put:`-style selectors the keywords ARE the identity; you cannot pass args in another order or omit one without naming a different (possibly nonexistent) selector. → overlay

## Single vs multiple dispatch
| Option | Langs | Consequence |
|---|---|---|
| Single (receiver only) | Smalltalk, Java, Ruby, Swift | Dispatch on `self`; second-arg type handled by manual double-dispatch. |
| Multiple (all args) | CLOS, Julia, Dylan | Method chosen by all arg types; no visitor boilerplate; symmetric ops clean. |
| Static overload (compile-time multi) | Java, C++, Swift | Looks multi but resolved on static types → no runtime specialization. |
| Predicate/multimethod dispatch | Cecil, CLOS `eql`-specializers | Dispatch on arbitrary value predicates; powerful, hard to cache/verify total. |

**Syntax.** CLOS `(defmethod draw ((s shape)(c canvas)) ...)` · Julia `draw(s::Shape, c::Canvas)` · Java `v.visit(this)` (double-dispatch) · Smalltalk `shape drawOn: canvas`.
**Impl.** Single = one class-keyed lookup; multi = arg-type tuple → applicable methods sorted by specificity, cached on the tuple.

**Hazard — multimethods ⊗ modularity/encapsulation.** A multimethod belongs to no single class, so it can't be a private member and any module can add a specialization — ambiguity errors surface at load time, far from either arg's definition. → overlay
**Hazard — single dispatch ⊗ binary operators.** `a + b` dispatching only on `a` forces coercion protocols (`coerce`, `__radd__`) to give `b` a say; asymmetry leaks into every numeric tower. → overlay
**Hazard — multimethods ⊗ inline cache.** Caching keys on the receiver's class alone; a multimethod's target depends on *all* arg types, so a single-key inline cache is unsound and must widen to a tuple key or bail. → overlay

## Keyword args & parameter labels
| Option | Langs | Consequence |
|---|---|---|
| Positional only | C, Lua, OCaml(default), Go | Call site is order-coupled; no self-documentation at call. |
| Keyword args (caller optional) | Python, Ruby 2+, Kotlin | Args passable by name; adding a param with default is source-compatible. |
| External vs internal labels | Swift (`move(to target:)`) | Call-site label differs from body name; label is part of the selector. |
| Keyword message = selector | Smalltalk, ObjC | Labels aren't optional sugar — they constitute the method name itself. |
| Keyword-only / positional-only markers | Python (`*`, `/`) | Forces call style; changes what refactors are source-compatible. |

**Syntax.** Python `f(x, key=1)` · Swift `move(to: p, by: d)` · Smalltalk `p moveTo: x by: y` · Ruby `f(key: 1)` · Kotlin `f(key = 1)`.
**Impl.** Labels folded into the selector symbol (Swift/ObjC/Smalltalk); Python/Ruby bind kwargs into a dict/hash at call; positional-only compiled to slot fill.

**Hazard — external labels ⊗ selector identity.** When the call-site label is part of the selector (Swift/ObjC), renaming a label is a breaking API change even though the internal parameter name is untouched. → overlay
**Hazard — kwargs ⊗ ordering guarantees.** Languages mixing positional and keyword args must freeze evaluation order and forbid duplicate binding; `f(x, x=1)` and later-added positional params are silent breakage. → overlay

## Default arguments
| Option | Langs | Consequence |
|---|---|---|
| Defaults evaluated at def-time | Python | Mutable default shared across calls — the classic `def f(x=[])` trap. |
| Defaults evaluated at call-time | Ruby, Swift, Kotlin, C++ | Fresh each call; but each default is a distinct hidden call shape. |
| No defaults; overload set instead | Java | N defaults → N overloaded stubs; combinatorial method count. |
| Default = synthesized arity family | Swift, Scala | One source method expands to many callable arities at the ABI. |

**Syntax.** Python `def f(x=[])` · Swift `func f(x: Int = 0)` · C++ `void f(int x = 0)` · Kotlin `fun f(x: Int = 0)` · Java (none → overloads).
**Impl.** Def-time default = value captured once in the function object; call-time = default expr inlined at each call site; Java synthesizes N overload stubs.

**Hazard — default args ⊗ selector identity (the arity-family trap).** Under name+arity selectors, a method with k defaults isn't one selector — it's a *family* `f(_)…f(_,_,_)` that all must route to one body. Either the compiler knows the static callee and fills defaults (impossible under fully dynamic dispatch), or every arity in the family needs its own dictionary entry → arity-family blowup and ambiguity with genuinely-different-arity methods of the same name. → overlay
**Hazard — call-time default ⊗ multiple dispatch.** A default filled at the call site changes the effective arg tuple *before* multimethod selection, so which specialization fires depends on defaults the caller never saw. → overlay

## Variadics / rest params
| Option | Langs | Consequence |
|---|---|---|
| Rest param collects to array/list | JS, Ruby(`*a`), Python(`*args`) | One variadic method spans all arities ≥ min; can't key a fixed-arity table. |
| Typed variadic | Java (`T...`), Swift | Sugar over array; overload resolution prefers fixed-arity over variadic. |
| Spread at call site | JS, Ruby, Python | Arity unknown until runtime → defeats static arity-based selector dispatch. |
| No variadics; pass explicit list | OCaml, Rust(mostly) | Caller builds the collection; dispatch stays fixed-arity and cache-friendly. |

**Syntax.** JS `f(...args)` · Python `def f(*args)` · Ruby `def f(*a)` · Java `f(T... xs)` · Swift `f(_ xs: Int...)`.
**Impl.** Rest collected into an array/list at the prologue; spread expands an iterable at the call; overload resolution ranks fixed-arity above variadic.

**Hazard — variadic ⊗ arity-keyed dispatch.** If selectors encode arity, a variadic method occupies an open-ended arity range; it collides with fixed-arity same-name methods and forces a runtime fallback path the inline cache can't monomorphize. → overlay
**Hazard — spread ⊗ overload resolution.** A spread call's arity is unknown statically, so the compiler cannot pick among fixed-arity overloads — resolution degrades to a runtime dispatch or an ambiguity error. → overlay

## Method resolution order & super
| Option | Langs | Consequence |
|---|---|---|
| C3 linearization | Python, Dylan, Raku | Monotonic, respects local precedence; deterministic diamond order. |
| DFS left-to-right (old) | Python 2 classic, Perl 5 default | Diamond visits a base twice / skips overrides; non-monotonic. |
| Single-parent chain | Java, Smalltalk, Swift | `super` is unambiguous; no linearization needed. |
| Linearized mixin chain | Ruby, Scala | `super` walks the *linearized* ancestors, not lexical parent. |
| Explicit next-method | CLOS (`call-next-method`) | Caller chooses whether/when to chain; no implicit walk. |

**Syntax.** Python `super().m()` · Ruby `super` / `super(args)` · Java `super.m()` · CLOS `(call-next-method)` · Smalltalk `super m`.
**Impl.** C3 precomputes a per-class ancestor list; `super` = the entry after the current class in that list; single-parent = direct parent slot.

**Hazard — C3 ⊗ super arg forwarding.** In cooperative-`super` chains every method must forward compatible args (`**kwargs`) or the chain breaks midway; one non-forwarding override silently drops later mixins. → overlay
**Hazard — linearization ⊗ non-monotonic bases.** If two classes disagree on base order, C3 *fails to linearize* (raises), whereas DFS silently builds a wrong order — a correctness-vs-fragility trade at class-def time. → overlay
**Hazard — `super` ⊗ mixin reorder.** Because `super` targets next-in-linearization, changing `include`/`with` order silently redirects which implementation `super` reaches. → overlay

## Missing-method hooks
| Option | Langs | Consequence |
|---|---|---|
| `doesNotUnderstand:` | Smalltalk | Reifies the failed send as a `Message`; enables proxies/DSLs. |
| `method_missing` / `respond_to_missing?` | Ruby | Must pair both or reflection/`respond_to?` lies. |
| `__getattr__`/`__getattribute__` | Python | `__getattribute__` intercepts *every* access — trivially catastrophic if wrong. |
| `Proxy` traps | JS | Per-operation traps; each trap is an un-cacheable indirection. |
| Compile error, no hook | Java, Rust, OCaml | Unknown method = static error; no runtime interception, fully cacheable. |

**Syntax.** Ruby `def method_missing(name, *args)` · Python `def __getattr__(self, name)` · Smalltalk `doesNotUnderstand: aMessage` · JS `new Proxy(t, {get})`.
**Impl.** Reify the failed send as a Message / args tuple; the hook runs on negative lookup, so the inline cache cannot record a miss ([recipes.md#inline-cache](recipes.md#inline-cache)).

**Hazard — dNU/method_missing ⊗ inline cache (the fallback trap).** A missing-method hook means "method not found" is no longer terminal — it's a live code path. Every negative lookup must call the hook rather than cache a miss, so inline caches can't record absence, and a hot dynamic-proxy call site stays megamorphic and slow. → overlay
**Hazard — `method_missing` ⊗ `respond_to?`.** Defining `method_missing` without updating `respond_to_missing?` makes the object handle a message it claims (via reflection) not to understand — breaks duck-typing checks and serializers. → overlay
**Hazard — `__getattribute__` ⊗ recursion.** Intercepting all access then touching `self.x` inside the hook re-enters the hook; infinite recursion unless you route through the base implementation. → overlay
