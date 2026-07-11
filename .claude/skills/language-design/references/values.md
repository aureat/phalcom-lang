# Values, Absence & Equality

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** deciding value representation, null/absence, truthiness, the numeric model, mutability, interning, or equality/hashing.

## Contents
- Axis 1 — Runtime value representation
- Axis 2 — Absence (null / Option / nil)
- Axis 3 — Truthiness
- Axis 4 — Numeric model
- Axis 5 — Immutability & value vs reference
- Axis 6 — Symbols & interned identity
- Axis 7 — Equality ladder

## Axis 1 — Runtime value representation
| Option | Langs | Consequence |
|---|---|---|
| Tagged union / enum | Rust, OCaml, Zig, most static | Explicit, exhaustive; pointer-sized+tag; no bit tricks |
| NaN-boxing (payload in float bits) | JS (V8/SpiderMonkey), LuaJIT, Wren | Fast doubles + 48-bit ptrs; 64-bit-only; brittle |
| Pointer low-bit / fixnum tagging | Smalltalk, Ruby, OCaml ints, Lisp | Unboxed small ints; loses ≥1 int bit; masks everywhere |
| Niche / nullable-pointer | Rust `Option<&T>`, Swift | Free absence via forbidden bit-patterns; needs invalid states |
| Uniform boxed pointers | CPython, Scheme, Java objects | Simplest GC/dispatch; every int allocates; slow, cache-hostile |

**Impl.** NaN-boxing packs ptr/immediate into a double's NaN payload (see recipes.md#nan-boxing) · low-bit fixnum tagging masks the tag on every deref · enum = discriminant + union.

**Hazard — NaN-box ⊗ new immediate type.** Every added unboxed immediate (symbol, char, small-string) must claim a distinct tag in the NaN payload space; the space is finite and 64-bit-locked, so a late "just one more immediate" can force re-tagging all reads. → overlay
**Hazard — fixnum tag ⊗ FFI/bit-ops.** Low-bit tagging means raw machine integers must be shifted/masked at every native boundary and bitwise op; forgetting one mask silently corrupts a pointer. → overlay

## Axis 2 — Absence (null / Option / nil)
| Option | Langs | Consequence |
|---|---|---|
| Null-everywhere (any ref nullable) | Java, C#, Go, Hoare's "billion-dollar mistake" | Every deref a latent NPE; no type-level tracking |
| `undefined` + `null` (two absences) | JS | Absent-vs-explicit-empty distinction; perpetual `== null` confusion |
| Option / Maybe (absence in the type) | OCaml, Haskell, Rust, Swift | Total, checked; forces unwrap; monadic plumbing cost |
| nil-as-object (nil responds to messages) | Smalltalk, CL nil-punning | No crash on nil-send; errors swallowed, propagate far |
| No null at all | Haskell (sans `Maybe`), Kotlin non-`?` | Absence must be modeled explicitly; migration friction |

**Syntax.** Rust `Option<T>`/`Some(x)`/`None`/`x?` · Haskell `Maybe a`/`Just x`/`Nothing` · Swift `T?`/`x!`/`x?.f` · Smalltalk `nil` (real object) · JS `null`+`undefined`.
**Impl.** Niche/nullable-pointer stuffs `None` into a forbidden bit-pattern for zero overhead (see recipes.md#option-niche) · nil = one preallocated singleton.

**Hazard — Option/nil bootstrap cycle.** If `None`/`nil` is itself a heap object whose uninitialized fields default to absence, defining absence needs absence to already exist. Break it: make nil an immediate/singleton constructed before any field-defaulting, or a distinct tag — never an ordinary instance. → overlay
**Hazard — nil-punning ⊗ truthiness ⊗ Option.** When `nil` both is falsey and answers messages, `Option`-style checks and truthiness collapse into one ambiguous test; you cannot tell "absent" from "present-but-empty" from "present-false". → overlay

## Axis 3 — Truthiness
| Option | Langs | Consequence |
|---|---|---|
| Everything truthy but a falsey set | JS, Python, Lua, Ruby, CL | Terse conditionals; `0`/`""`/`[]` bugs; each lang's set differs |
| Only `false`/`nil` falsey (rest truthy) | Ruby, Lua | Minimal surprises vs JS/Python; still no `0`-guard |
| Strict typed `Bool` only | Java, Swift, Haskell, OCaml, Kotlin | `if` demands `Bool`; no `if(x)` on non-bool; verbose but safe |
| No truthiness, dynamic lang | (target for many new dynlangs) | Must reject `if(non-bool)` without a static type checker |

**Syntax.** JS `if (x)` any value · Python `if xs:` · Lua `if v then` (only `nil`/`false` falsey) · Java/Swift `if b {` Bool-only · Haskell `if b then … else`.

**Hazard — truthiness-ban without flow analysis.** In a dynamically typed lang, banning `if(option)` can't be a compile-time type error — there are no types. Enforce it at the branch opcode: `jump-if-false` traps unless the operand is the `Bool` singleton, turning misuse into a runtime `MustBeBoolean` error instead of silent coercion. → overlay
**Hazard — falsey `nil` ⊗ absence semantics.** Making `nil` falsey means `if(dict[k])` conflates missing-key, present-nil, and present-false; any "get or default" pattern silently misbehaves. → overlay

## Axis 4 — Numeric model
| Option | Langs | Consequence |
|---|---|---|
| Single float (all numbers doubles) | JS, Lua ≤5.2 | One type, no int/float bugs; 2^53 integer ceiling; no true ints |
| Int / Float split | Java, Go, Rust, Swift, OCaml | Predictable, fast; overflow wraps/traps; `1/2==0` surprises |
| Full numeric tower | Scheme, CL | Exact rationals/bignums, contagion rules; heavy, slow generic ops |
| Fixnum tag + bignum promotion | Smalltalk, Ruby, Python 3 int | Unbounded ints, fast small case; promotion branch every arithmetic op |

**Syntax.** Scheme `1/3`, `(+ 1/3 0.5)` exact→inexact · Python `2**100` auto-bignum · Rust `1i64`/`1.0f64`/`1u8` · JS `0.1+0.2` all double · Smalltalk `1000 factorial`.
**Impl.** Fixnum + overflow-check→bignum-box on the arithmetic path · tower dispatches on numeric-type contagion.

**Hazard — fixnum promotion ⊗ representation tagging.** Auto-promoting fixnum→bignum on overflow requires an overflow check and a boxed fallback on the hot arithmetic path; combined with tag masking (Axis 1) each `+` becomes untag→add→overflow?→retag or allocate. → overlay
**Hazard — single-float ⊗ array/index identity.** With only doubles, array indices, hash keys, and bit flags all live in float mantissa; `-0`, `NaN` keys, and >2^53 ids break equality and lookups (see Axis 7). → overlay

## Axis 5 — Immutability & value vs reference
| Option | Langs | Consequence |
|---|---|---|
| Deep immutability default | Haskell, Erlang, Clojure | Free sharing/concurrency; update = copy/persistent structure |
| Mutable default, opt-in freeze | Ruby (`freeze`), JS (`Object.freeze`), Python tuples | Ergonomic; freeze is shallow, runtime-checked, easy to forget |
| Value vs reference type split | Swift `struct`/`class`, C#, Go | Copy semantics chosen per type; accidental-copy vs aliasing traps |
| `const`/`let` binding-immutability | Rust, JS `const`, Kotlin `val` | Binding not value frozen; `const xs=[]; xs.push()` still mutates |
| Interned/shared immutables | small ints, `""`, symbols | Identity == value for free; must guarantee no in-place mutation |

**Syntax.** Rust `let`/`let mut` · Kotlin `val`/`var` · JS `const xs=[]` (binding only) · Ruby `x.freeze` · Swift `struct` (value copy) vs `class` (ref).

**Hazard — shallow freeze ⊗ interning.** Interned immutables (small ints, cached strings) are shared singletons; if the "frozen" guarantee is shallow or bypassable, one in-place mutation corrupts every holder of that shared value process-wide. → overlay
**Hazard — value-type copy ⊗ reference identity.** When a type has value/copy semantics, `identity-equal` and `==` diverge silently: two copies are `==` but never `identity`, breaking identity-keyed caches. → overlay

## Axis 6 — Symbols & interned identity
| Option | Langs | Consequence |
|---|---|---|
| Interned symbols distinct from strings | Smalltalk, Ruby, Lisp, Scheme | O(1) identity compare; method keys; two string-vs-symbol worlds |
| Symbols == interned strings | JS `Symbol` (unique, not string), Erlang atoms | Cheap dispatch keys; atom table can leak if dynamically created |
| No symbols; hash strings | Python (str keys), Lua | One type, simpler; every dispatch/key does string hash+compare |
| Selector symbols encode arity/kind | Smalltalk, Phalcom-style | `foo` and `foo(_)` coexist as distinct keys; selector-building rules |

**Syntax.** Ruby `:foo` · Lisp/Scheme `'foo`/`(quote foo)` · Smalltalk `#foo`/`#at:put:` · Erlang `ok` (atom) · JS `Symbol("x")`.
**Impl.** Intern table maps name→unique id; identity compare is pointer/id equality; selector ids key method dispatch (see recipes.md#inline-cache).

**Hazard — dynamic symbol/atom creation ⊗ GC.** If symbols are interned forever and can be minted from runtime strings (`"x".to_sym`, `list_to_atom`), untrusted input grows the intern table unboundedly — a memory-exhaustion DoS (Erlang's classic atom-table crash). → overlay
**Hazard — selector symbol ⊗ arity encoding.** If the dispatch key encodes arity/kind, the parser and the reflective `perform:`/`send` path must build byte-identical selectors, or a method defined one way is unreachable by the other. → overlay

## Axis 7 — Equality ladder
| Option | Langs | Consequence |
|---|---|---|
| Identity only builtin (`eq`) | Scheme `eq?`, Smalltalk `==` | Fast, unambiguous; user must define value equality per class |
| `==` value + `identity` separate | Java `equals`/`==`, Python `==`/`is`, Ruby | Two operators; `==` overridable; identity for caches/nil |
| Structural / deep default | Haskell `Eq`, OCaml `=`, Rust `PartialEq` | Compares contents; cyclic data loops; costly on big graphs |
| Coercing `==` + strict `===` | JS | `==` type-juggling footguns; `===` the real one |

**Impl.** Identity = pointer/tag compare · structural = recursive field walk with cycle guard · hash must canonicalize `-0`→`+0` and special-case `NaN` keys.

**Hazard — `NaN`/`-0` ⊗ hashing & equality.** `NaN != NaN` breaks reflexivity, so hash-set membership and dedup silently fail on `NaN` keys; `-0 == +0` yet may hash differently — normalize (canonicalize `-0`→`+0`, treat `NaN` keys specially) at the hash boundary. → overlay
**Hazard — overridable `==` ⊗ hash contract.** If users override value `==` but not `hash` (or vice versa), hash-based collections violate the `a==b ⇒ hash(a)==hash(b)` invariant and lose/duplicate entries; the equality and hash methods must be co-defined or co-derived. → overlay
