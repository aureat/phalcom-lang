# Object Model

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing class, metaclass, inheritance, object-layout, or equality/identity features.

## Contents
- Object genesis: class-based vs prototype vs hybrid
- Metaclass strategy
- Behavior / method-dictionary / class-as-object split
- Inheritance topology (single / multiple / mixin / trait / interface)
- Class-hierarchy mutability
- Instance layout
- Identity vs structural equality

## Object genesis: class-based vs prototype vs hybrid
| Option | Langs | Consequence |
|---|---|---|
| Pure class-based | Java, Swift, Kotlin, OCaml | Shape fixed at class-def; no per-object structure edits. |
| Pure prototype | Self, JS(pre-class) | Objects clone/delegate; no class needed, but no shared shape guarantee. |
| Prototype + delegation table | Lua (metatables), JS `__proto__` | Behavior is a field lookup up a chain; fully mutable, slow-cold. |
| Class sugar over prototypes | JS `class`, TS | `class` is a `prototype` object; `static` lives on the constructor fn. |
| Class + open metaobject | Smalltalk, Ruby, Python | Classes are first-class objects; runtime introspection/reshape. |
| Trait-object / no nominal self | Rust | "Object" = data struct + trait impls; no inheritance, dispatch via vtable. |

**Syntax.** Smalltalk `Object subclass: #Point` · Ruby `class Point < Object` · Python `class Point:` · Rust `struct Point; impl Point` · Lua `setmetatable(t,{__index=Proto})` · Self `(| x. y |)` (clone).
**Impl.** Class-based → static shape descriptor + shared vtable; prototype → per-object delegation pointer walked at lookup; Lua `__index` chain.

**Hazard — prototype mutation ⊗ shared shape.** Assigning a new field on one prototype-delegated object silently changes lookup results for every object still delegating to that prototype. → overlay
**Hazard — class-sugar ⊗ `static`.** JS `static` members live on the constructor function, not the prototype chain, so subclass instances don't see them via normal lookup. → overlay

## Metaclass strategy
| Option | Langs | Consequence |
|---|---|---|
| Parallel metaclass tower | Smalltalk, Pharo, Newspeak | Every class has a unique metaclass mirroring the hierarchy; class-side methods inherit. |
| Eigenclass / singleton class | Ruby | Per-object hidden class holds "class methods"; uniform but 2× class count. |
| Type-is-a-class, `type` reflexive | Python (`type`) | Metaclasses are just callable `type` subclasses; one knob, sharp edges. |
| Class-as-value, flat | Lua, JS | No true metaclass; "static" is just a table/constructor property. |
| None — static namespace | Java, C++, Kotlin | `static` is a compile-time namespace, not dispatchable, not inherited polymorphically. |

**Syntax.** Smalltalk `Point class` (the metaclass) · Ruby `class << obj` (eigenclass) · Python `class C(metaclass=M)` with `class M(type)` · Java `static` members.
**Impl.** Smalltalk auto-creates one metaclass object per class; Ruby inserts a singleton class into the ancestry; Python metaclass = the class's own `type`.

**Hazard — metaclass tower ⊗ bootstrap cycle.** `Class` is an instance of `Metaclass` which is an instance of its own metaclass — the tower must close the loop by hand-wiring a few objects before any allocation works. → overlay
**Hazard — metaclass tower ⊗ multiple inheritance.** Parallel metaclasses must linearize in lockstep with instance-side MI; inconsistent metaclass MRO is a classic bootstrap paradox. → overlay
**Hazard — Python metaclass ⊗ MI.** Combining bases with different metaclasses raises `metaclass conflict` unless one metaclass is a subclass of all others. → overlay
**Hazard — eigenclass ⊗ introspection.** Ruby's singleton classes make `obj.class` ≠ the class that actually holds the method; reflection/serialization tools that assume `class` owns behavior break.

## Behavior / method-dictionary / class-as-object split
| Option | Langs | Consequence |
|---|---|---|
| `Behavior`→`ClassDescription`→`Class` layering | Smalltalk | Method dict + layout live on `Behavior`; `Class` adds naming/metaclass. |
| Class object holds a mutable method dict | Ruby, Python (`__dict__`) | Methods addable/removable at runtime; enables monkey-patch + reload. |
| Vtable baked at link time | Java, C++, Swift(final) | No method dict object; fast, but no runtime method injection. |
| Method table = plain hashmap value | Lua, JS | "Class" is a table; dispatch = key lookup; trivially reflective, uncached-slow. |

**Syntax.** Ruby/Python reopen `class C; def m; end; end` · Smalltalk `C compile: 'm ...'` (`C >> #m`) · Lua `function C:m() end`.
**Impl.** Open dict = hashmap `symbol→method` on the class, read via inline cache ([recipes.md#inline-cache](recipes.md#inline-cache)); vtable = fixed slot array resolved at link.

**Hazard — open method dict ⊗ inline cache.** A monkey-patch mutates the dict a call site cached against; every cache keyed on class-version must invalidate or it dispatches stale. → overlay
**Hazard — layout on `Behavior` ⊗ subclass slot offsets.** If instance-variable offsets are computed on the class, adding a slot to a superclass shifts every subclass's offsets; live instances and compiled accessors must be recomputed. → overlay

## Inheritance topology
| Option | Langs | Consequence |
|---|---|---|
| Single inheritance + interfaces | Java, Swift, Kotlin, C# | No diamond on state; interfaces give type-only multiple supertyping. |
| Multiple inheritance (state) | C++, Python, CLOS | Diamond on fields; needs virtual bases or C3 to dedupe. |
| Mixins (linearized modules) | Ruby (`include`), Scala | MI-of-behavior without MI-of-state; order determines override winner. |
| Traits (stateless, conflict-explicit) | Self-traits, Rust, PHP, Pharo traits | Composition requires manual conflict resolution; no implicit override. |
| Interfaces w/ default methods | Java 8+, Kotlin | Diamond returns for defaults; compiler forces explicit override to break tie. |

**Syntax.** Java `class C extends B implements I` · Ruby `include M` · Rust `impl Trait for C` · Scala `class C extends B with T` · C++ `class C: public A, public B`.
**Impl.** Single = parent pointer; MI = C3 or virtual-base offset tables; mixin = module spliced into linearization; trait = compile-time flatten with explicit conflict check.

**Hazard — default methods ⊗ diamond.** Two interfaces supplying the same default method reintroduce the diamond the "interfaces-only" model was meant to avoid; caller must override to disambiguate. → overlay
**Hazard — mixin ordering ⊗ super.** With linearized mixins, `super` targets the *next in linearization*, not the lexical parent; reordering `include`s silently changes which method `super` hits. → overlay
**Hazard — trait state ⊗ statelessness.** Traits that pretend to be stateless but require an accessor create hidden coupling; two traits demanding the same slot name collide at compose time.

## Class-hierarchy mutability
| Option | Langs | Consequence |
|---|---|---|
| Fully reshapeable at runtime | Smalltalk, CLOS | Add/remove slots on a live class; instances migrate via `update-instance`. |
| Open classes, methods only | Ruby, Python | Reopen to add methods; changing layout of live instances is unsafe/ad hoc. |
| Sealed after definition | Wren, most compiled | Class shape final; enables flat slot layout + monomorphic call sites. |
| Sealed unless opted-in | Kotlin (`open`), Swift(`final` default in modules) | Closed-by-default lets compiler devirtualize; `open` re-enables override. |

**Syntax.** Smalltalk `Point subclass:#P instanceVariableNames:'x y'` · Ruby reopen `class C ... end` · Kotlin `open class C` · Swift `final class C`.
**Impl.** Reshape = instance migration (`become:` / `update-instance-for-redefined-class`); sealed = frozen vtable the compiler can devirtualize.

**Hazard — live reshape ⊗ existing instances.** Redefining a class with new slots orphans already-allocated instances; without an instance-migration protocol they read garbage or crash. → overlay
**Hazard — sealed ⊗ open method dict.** Sealing for speed contradicts a runtime-addable method dict; you can't both devirtualize and allow monkey-patch on the same class. → overlay

## Instance layout
| Option | Langs | Consequence |
|---|---|---|
| Fixed ordered slot map | Smalltalk, Java, Swift, OCaml records | O(1) indexed field access; layout frozen at class-def. |
| Per-object hash/dict | Python (`__dict__`), Ruby ivars, Lua | Any field anytime; pointer-chase + hash per access. |
| Hidden classes / shapes | V8, Self maps, SpiderMonkey | Dict-like flexibility with slot-map speed *if* shapes stay stable. |
| `__slots__` opt-in fixed | Python | Trades dict flexibility for memory + speed on declared fields. |
| Delegation (no own storage) | Self, JS proto | Fields resolve up the chain; write creates own copy (shadowing). |

**Impl.** Fixed map = offset-indexed struct; dict = per-object hashmap; V8 hidden classes = shape objects transitioned on each field-add and cached ([recipes.md#inline-cache](recipes.md#inline-cache)); immediates via tagging/[recipes.md#nan-boxing](recipes.md#nan-boxing).

**Hazard — shapes ⊗ field-add order.** Adding fields in different orders across constructors mints divergent hidden classes; polymorphic call sites de-optimize (V8 "megamorphic"). → overlay
**Hazard — dict ivars ⊗ `==`/hash.** Per-object dict layout means structural-equality and hashing must iterate keys; a mutated field silently changes a live hash-map key. → overlay

## Identity vs structural equality
| Option | Langs | Consequence |
|---|---|---|
| Identity default, `==` overridable | Java (`equals`/`==`), Smalltalk (`=`/`==`) | Two knobs; `hashCode`/`identityHash` must track whichever you override. |
| Structural default for values | Rust (`PartialEq` derive), Swift structs, OCaml | Deep compare by default; identity only via explicit reference/box. |
| `===` identity vs `==` coercing | JS, PHP | Coercing equality is a footgun; `NaN`, `-0`, object refs surprise. |
| Value/reference split by type | Swift (struct vs class), C# | Same `==` syntax means copy-equality or ref-equality by declaration site. |
| `eq?`/`eql?`/`equal?` tiers | Scheme, CLOS, Ruby | Multiple graded predicates; picking the wrong tier breaks hashing. |

**Syntax.** JS `a === b` vs `a == b` · Java `a.equals(b)` vs `a == b` · Rust `a == b` (`#[derive(PartialEq)]`) · Scheme `(eq? a b)`/`(equal? a b)` · Ruby `equal?`/`==`/`eql?`.
**Impl.** Identity = pointer / `identityHash`; structural = recursive field compare; `hash` must be redefined in lockstep with `==`; small-int/symbol interning collapses the two.

**Hazard — override `==` ⊗ hash contract.** Redefining equality without updating `hash`/`identityHash` in lockstep corrupts every hash-set/dict keyed on the type. → overlay
**Hazard — mutable key ⊗ structural hash.** Structural equality + mutable object = hash changes after insertion; the entry becomes unreachable in its own table. → overlay
**Hazard — identity dispatch ⊗ interning.** If small integers/symbols are interned but larger ones boxed, `==` identity gives inconsistent results across the boundary (Java `Integer` cache 127). → overlay
