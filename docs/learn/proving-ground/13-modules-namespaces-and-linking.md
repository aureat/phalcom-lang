# 13 — Modules, Namespaces, and Linking

Names, scopes, and what identity means across compilation units. The through-line: *a
module system is an answer to "when are two things the same thing", and every other
question here is a corollary of that answer.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Four things called "module"

Four systems, four answers to what a module is:

- **Python** — a module is a *file*, and `sys.modules` is a dictionary keyed by dotted name.
- **OCaml** — a module is a *value* with a signature type, and functors are functions over modules.
- **Java** — the compilation unit is the class file; packages are naming; JPMS modules are a
  fourth, later layer.
- **Rust** — modules are a namespace tree *inside* a crate, and the crate is the compilation
  unit and the linking unit.

1. Pick any two and show a concrete consequence downstream — something a user hits — that
   follows directly from the choice, not from the language's other features.
2. Rust separates the namespace unit from the compilation unit; Python fuses them. Name one
   thing each gains that the other cannot have.
3. `sys.modules` is keyed by *name*, not by file path or inode. Construct the bug this
   permits, and say why it is the same bug as Q4's.

### Q2 — When is a functor's output type the same type

```ocaml
module F (X : S) : T = struct type t = ... end

module A = F(Arg)
module B = F(Arg)
(* is A.t = B.t ? *)
```

Two disciplines: **generative** functors mint a fresh abstract type per application;
**applicative** functors give equal types for equal arguments.

1. Give a program that is well-typed under one and rejected under the other, and say which
   answer you want for a `Set` functor specifically.
2. Applicative functors need a notion of "equal arguments". What is that equality actually
   *on*, and why does it get hard the moment functor application can have side effects?
3. First-class modules (OCaml's `(module M : S)`, or a record of functions) let a module be
   chosen at runtime. Name the thing you lose the moment a module is a runtime value.

### Q3 — Wildcard imports

```cpp
using namespace std;
```

```python
from numpy import *
```

```rust
use foo::*;
```

1. A library adds one new public name in a patch release and downstream code stops
   compiling — with no change on the downstream side. Reconstruct the mechanism, and say
   why this is a *versioning* failure and not a style complaint.
2. Rust's glob imports have a shadowing rule that makes them substantially safer than C++'s.
   State the rule and what it buys.
3. Re-exports (`pub use`, `export * from`) are wildcard imports pointed outward. What do
   they commit you to that a plain `pub fn` does not?

### Q4 — Two copies of the same library

Four systems, same problem:

- npm nests `node_modules`, so two versions of a package can both be loaded.
- Java's runtime type identity is (defining classloader, fully-qualified name).
- Cargo allows semver-incompatible majors of a crate to coexist in one binary.
- Python's `sys.modules` allows exactly one module per name.

1. React errors out with a message about more than one copy of React being loaded, and a
   servlet container throws `ClassCastException: Foo cannot be cast to Foo`. These are the
   same bug. State it in one sentence that covers both.
2. Rust reports "expected `foo::Bar`, found `foo::Bar`". Explain why the compiler is right
   and why deduplicating to one version would be *unsound*, not merely inconvenient.
3. Python's single-copy rule avoids all of this. Name precisely what it costs, and why that
   cost lands on the package manager rather than on the language.

### Q5 — The double-import

```
project/
  main.py          # run as `python project/main.py`
  models.py
  worker.py        # does `import models`
```

`main.py` does `import models` too, and also `from models import Thing`. In some layouts,
`isinstance(x, Thing)` returns `False` for an object that is obviously a `Thing`.

1. Explain the mechanism. Your explanation must name what the two module objects are keyed
   under.
2. Class identity is not the only casualty. Name two other kinds of state that silently
   duplicate, and why one of them is much worse than the class-identity bug.
3. Node's ESM/CJS "dual package hazard" is the same failure in a different system. What
   makes it *structurally harder* to avoid than Python's?

### Q6 — Circular imports, three answers

```js
// a.mjs
import { b } from './b.mjs';
export const a = 'a';
console.log('a sees', b);

// b.mjs
import { a } from './a.mjs';
export const b = 'b';
console.log('b sees', a);

// main.mjs
import './a.mjs';
```

1. Predict the output of `node main.mjs`. Then say what changes if both `const`s become
   `function` declarations, and why — the folklore "circular imports work if you only use
   functions" is actually correct here, and you should say why.
2. CommonJS `require` on the same cycle does not throw. Explain what it does instead, and
   argue which failure mode you would rather ship.
3. Go bans import cycles outright at compile time. Name two things that ban buys the
   implementation, and the one thing it costs the user that no amount of tooling fixes.

### Q7 — The static initialization order fiasco

```cpp
// a.cpp
Logger logger;
// b.cpp
Registry registry;   // Registry's constructor calls logger.log(...)
```

1. State the exact guarantee C++ does and does not give, and why the standard could not
   simply have specified an order.
2. Go, Java, and Rust each avoid or contain this. Give each one's mechanism in a sentence,
   and say which of the three actually *solves* it versus merely *containing* it.
3. Rust's "no life before `main`" forecloses a widely used pattern. Name it, name the crates
   that reintroduce it, and say what makes the reintroduction fragile.

### Q8 — The deadlock nobody can see

Two Java classes, each with a static initializer that touches the other. Two threads, each
first touching one of them. The process hangs. `jstack` shows both threads in
`Class.forName`-ish frames, and no monitor is reported as held by either.

1. Reconstruct the mechanism from the class initialization procedure. Name the state each
   class is in and what each thread is waiting on.
2. Why is this not reported by ordinary deadlock detection, and what does that tell you
   about where the lock lives?
3. Lazy initialization is what makes this possible. Eager initialization would not have this
   bug. Say what eager initialization costs, and why the JVM chose lazy anyway.

### Q9 — What must a module export to be compiled against

Three points on a line:

- **Java** erases generics; a `.class` file is a complete, sufficient interface.
- **Rust** monomorphizes generics; a downstream crate instantiates the generic *body*.
- **C++** puts templates in headers, and the `export` keyword for templates was specified,
  implemented essentially once, and then removed.

1. State what each system must ship to a consumer, and derive the recompilation
   consequences of each from that.
2. Erasure buys separate compilation and a stable class-file format. Name the two concrete
   things it costs, and why one of them is still unresolved twenty years on.
3. Why did `export template` fail? Answer in terms of what the two-phase name lookup rules
   require an implementation to have retained.

### Q10 — API stable, ABI broken

A C++ library adds a private data member and reorders nothing public. A library ships a new
`std::string` layout. glibc changes `memcpy`'s overlap behaviour.

1. For each of the three, say what breaks and *when* the user finds out — the "when" is the
   whole point.
2. Symbol versioning lets one shared object export two incompatible implementations of one
   name. Sketch the mechanism and say what problem it solves that a version-in-the-soname
   does not.
3. Swift's library evolution mode and CPython's limited API/stable ABI are the same idea.
   State the idea, the runtime cost it imposes, and the escape hatch each provides for
   performance-critical types.

### Q11 — Interposition

```
LD_PRELOAD=./mymalloc.so ./app
```

1. Explain the resolution rule that makes this work on ELF. Then explain how the same rule
   produces a production bug where a program calls a function it never linked against.
2. macOS uses a two-level namespace: each undefined symbol records the library it is
   expected to come from. Name one thing this makes safer and one legitimate technique it
   breaks.
3. A library wants to be immune to interposition of its *internal* calls. Name two
   mechanisms, and the cost each imposes on the library's own users.

### Q12 — A registry keyed by name

A runtime keeps `classes: Map<String, ClassId>`. Two modules each declare `class Point`.

1. Say exactly what happens, and why the *silence* is the serious part rather than the
   collision.
2. Give three candidate identity schemes for a class, and for each say what breaks under
   reloading, serialization, and separate compilation.
3. Objective-C's class registry is genuinely flat and global, and duplicate class names
   produce a runtime warning saying one of the two will be used and which is undefined. Why
   is a *warning* the right level here, and what would it take to make it an error?

### Q13 — "Private" needs a boundary

```rust
mod inner { struct S { x: i32 } }   // x is private — to whom?
```

```java
package com.acme;  // package-private — to whoever claims to be com.acme
```

```python
class C:
    def _helper(self): ...   # private by convention
```

1. For each, name *whose* access the visibility rule actually restricts, and identify which
   one can be defeated by an attacker who merely controls a file's location.
2. Reflection defeats visibility in Java, C#, and Python. Argue that this is not a bug in
   reflection but a consequence of where the boundary was drawn.
3. You want `private` to be enforceable — not advisory. Name the two things the runtime must
   own for that to be true, and one entire feature category you must give up.

### Q14 — Feature flags are a combinatorial module system

```toml
[dependencies]
serde = { version = "1", default-features = false }
```

Another crate deep in your graph depends on `serde` with `features = ["std"]`.

1. Explain what your crate now gets, and why Cargo's designers chose that rule despite it
   producing exactly this surprise.
2. Features are supposed to be **additive**. Give a concrete way a crate violates
   additivity, and say why the violation is undetectable by the package manager.
3. `#[cfg]`/`#ifdef` code paths that no CI configuration compiles are a known rot source.
   Name the testing strategy that addresses it and the reason it does not scale.

### Q15 — The package manager is the real module system

Four resolution disciplines: npm's nesting, Cargo's one-per-major, Go's minimal version
selection, and pip's flat single-version namespace.

1. Rank them by how hard the *solver's* job is, and explain the ranking with a single
   structural property of each.
2. SemVer is described as a technical contract. Argue it is a social one, then name the two
   real systems that made some part of it technically enforced and say what each enforces.
3. A lockfile pins exact versions. Name the thing a lockfile does *not* pin that has broken
   real builds, and the mechanism that closes it.

### Q16 — Retrofit: CommonJS to ES modules

Node shipped `require` first and ES modules a decade later.

1. `import { readFile } from 'some-cjs-package'` requires Node to know the package's named
   exports before executing it. Explain why that is hard, and what Node actually does.
2. `require()` is synchronous; ESM evaluation can be asynchronous. Name the language feature
   that makes this a genuine impedance mismatch rather than an implementation detail.
3. The dual-package hazard (Q5.3) is the ecosystem-level cost. Name the two package-level
   mitigations and say why neither is fully satisfactory.

### Q17 — Retrofit: Java 9 modules

`--add-opens java.base/java.lang=ALL-UNNAMED` is a permanent fixture of production build
files, a decade after JPMS shipped.

1. Split packages — one package spanning two modules — were banned. Reconstruct why the
   ban is *forced* by the design rather than chosen, and name the ecosystem pattern it
   killed.
2. Strong encapsulation broke reflection-heavy frameworks. Identify what those frameworks
   were actually relying on, and why "just fix the frameworks" was not available as an
   answer.
3. Draw the general lesson about retrofitting a boundary. What must be true at v1 for a
   module boundary to be addable later without an ecosystem break?

---

## Answers

### A1 — Four things called "module"

**1.** Two good pairs. **Python vs Rust on namespace/compilation fusion**: because a Python
module is a file *and* a runtime object *and* an execution unit, `import x` has side
effects — arbitrary top-level code runs — so import order is observable, circular imports
are a semantic problem (Q6), and "which file am I" (`__name__ == "__main__"`) is a runtime
question. In Rust, `mod` is pure namespacing with no execution, so the equivalent questions
do not exist; the corresponding Rust pain is elsewhere, in crate-granularity compilation.
**OCaml vs Java on module-as-value**: because an OCaml module has a *type*, you can write a
function from modules to modules and have the compiler check it, which is how `Set.Make` and
`Map.Make` are parameterised — the abstraction is checked at the module level. Java has no
such thing, so the same parameterisation is done with generics and interfaces at the *type*
level, and abstract type members (a `Set` whose element type and comparison are fixed
together, opaquely) are simply not expressible.

**2.** Rust gains **compilation-unit-sized optimisation and linking**: a crate is optimised
as a whole, generic instantiation and inlining cross `mod` boundaries freely, and the
namespace tree costs nothing at runtime. It loses **incremental granularity** — touching one
function invalidates work across the crate, which is why "split your crate" is standard Rust
build advice, and it loses the ability to load a module dynamically. Python gains **dynamic
loading and reloading, and per-module identity at runtime** — a module is an object you can
inspect, patch, or replace — at the cost of never being able to close the world or optimise
across the boundary.

**3.** Keying by name means the identity of a module is *the string used to import it*, not
the file it came from. So the same file imported under two names is two module objects, with
two copies of every class, function, and piece of module-level state. Q4's problem is the
same shape one level up: identity of a *library* determined by something other than the
artifact's actual identity. In Python it is the import name; in Java it is
(classloader, name); in npm it is the path in the tree. Every one of these systems has a
bug family where "the same thing" turns out not to be, and every one traces to the key.

**Trap.** "A module is just a namespace." It is a namespace in Rust, a runtime object with
an execution history in Python, a typed value in OCaml, and a linking unit in C. The word is
the same and almost nothing transfers; every cross-language argument about modules that goes
nowhere is two people using one word for four things.

### A2 — When is a functor's output type the same type

**1.** Under applicative semantics, `A.t = B.t`, so a value made by `A` can be passed to a
function expecting `B.t`. Under generative semantics they are distinct and that program is
rejected. For a `Set` functor you want **applicative**: `Set.Make(String)` used in two
different files must produce interoperable set types, or every library that exposes a
`StringSet` exposes one nobody else can consume. The generative answer is what you want when
the functor's application allocates or names a *resource* — a fresh symbol table, a fresh
counter, a fresh database handle — because then two applications genuinely are two different
things and letting their types unify would let you mix them.

**2.** It is equality **on the module path** — a syntactic/structural notion of "the same
module expression" — not on the module's contents, because module contents are not
comparable and structural equality on signatures would be both undecidable in general and
wrong (two structurally identical modules with different invariants are not
interchangeable). It gets hard with side effects because `F(Arg)` twice would then be two
*evaluations*, and if the functor allocates, the two results are observably different
entities whose types the applicative rule has just declared equal — which is unsound in
exactly the way the generative rule exists to prevent. This is why OCaml's rule is
essentially "applicative when the functor is pure enough to be given a path, generative when
you ask for it," and why the feature came with an explicit `()`-taking generative form.

**3.** You lose **type-level knowledge of which module you got**, and therefore the ability
of the type checker to connect two uses. A first-class module must be unpacked before use,
and the abstract types it introduces are scoped to that unpacking — so two unpackings of
what is at runtime the same module yield incompatible abstract types, and you cannot express
"these two values came from the same instance" without extra machinery. The general law:
**making an abstraction a runtime value moves its identity from compile time to run time,
and the type system can no longer speak about it.** Practically this is why runtime-selected
modules are usually reduced to a record of functions over a *concrete* type, giving up the
abstraction that motivated the module in the first place.

### A3 — Wildcard imports

**1.** A glob import makes the set of names in your scope a function of the *library's*
contents. Adding `std::byte`, or a `numpy` function named `any`, introduces a name that
either collides with one of your locals or with another glob's, and the resolution becomes
ambiguous or silently changes which entity a name refers to. The classic C++ instance is
adding a `std` overload that then wins overload resolution over the one you meant. This is a
versioning failure because the library made a change it is entitled to call
backwards-compatible — it only *added* — and broke a consumer. It means "additive change" is
not a safe category under glob imports, which removes the entire foundation SemVer minor
versions rest on. That is a much stronger claim than "globs are untidy."

**2.** In Rust, a glob-imported name has **lower precedence than an explicitly imported or
locally defined name**, and two globs bringing in the same name is only an error if that
name is actually *used* ambiguously. So adding a name to a glob-exporting library cannot
shadow anything you wrote yourself, and cannot break you unless you were relying on a name
that is now ambiguous at a use site. It converts the failure from "silently resolves
differently" to "either fine, or an error at the exact use." That is the entire difference,
and it is why glob imports are acceptable in Rust and a code-review flag in C++.

**3.** A re-export commits you to the *stability of the re-exported item's entire public
surface*, including its type identity — because downstream code now names your path but gets
the dependency's type. Consequences: bumping that dependency to a semver-incompatible
version is a breaking change **for you**, even though your own signatures did not change; a
downstream crate can end up needing to depend on the dependency directly to name a type that
appears in your API; and a wildcard re-export means you cannot enumerate your own API. A
plain `pub fn` commits you only to a signature you wrote and can preserve by adapting. The
rule of thumb: re-exporting a type from a dependency makes that dependency part of your
public API whether or not your manifest says so.

**Trap.** "Globs are fine if you control both sides." You control both sides *today*; the
failure arrives with a version bump you did not make, in a transitive dependency, and it
presents as a compile error in code nobody touched. The problem is not authorship, it is
that you have made your scope depend on a set you do not version.

### A4 — Two copies of the same library

**1.** In both, an object was created by one copy of a definition and is being checked
against a *different* copy of the same-named definition — so the check fails even though the
two definitions are textually identical, because runtime type identity is keyed on the
loaded entity, not on the name or the source. React's hook dispatcher is module-level state
in one copy that the component from the other copy never sees; the servlet `Foo` was defined
by two classloaders, so there are two `Class` objects and the cast between them is genuinely
a cast between unrelated types.

**2.** Because the two versions' `Bar` types really are different types: their field layouts
may differ, their invariants may differ, and their trait impls certainly may differ. If the
compiler unified them, a value constructed under v1's invariants could flow into v2's code,
which is a memory-safety hazard the moment layout differs — this is not "inconvenient", it
is UB. The coherence angle sharpens it: if `foo v1` and `foo v2` both define trait `T`, and
you unified the *types* but not the *traits*, or vice versa, you would either get two impls
of one trait for one type (violating coherence, so overlapping impls with different
behaviour) or a type that implements a trait it was never checked against. Cargo's rule —
distinct majors are distinct crates, one instance per major — is the only sound rule that
also allows the graph to resolve.

**3.** It costs **the ability for the dependency graph to be satisfiable when two of your
dependencies need incompatible versions of a third**. Under a single-copy rule, that is not
a resolution difficulty, it is a *hard conflict with no solution* — someone must change
their code. The cost lands on the package manager because the language has removed the
escape hatch: npm can always nest, Cargo can always duplicate a major, so their solvers can
answer "yes" more often; pip's solver must return "no" and hand a conflict message to a human.
This is the real reason Python dependency hell has a different character than Node's: not
worse tooling, a strictly harder problem with strictly fewer legal answers.

### A5 — The double-import

**1.** `sys.modules` is keyed by the *name under which the module was imported*. Running
`python project/main.py` inserts that file as `__main__` and puts its directory on
`sys.path`. When `worker.py` does `import models`, Python inserts a second module object
under the key `models` — and if any path leads to the main script also being imported as
`main`, you get the file executed twice under two keys. Two module objects means two
executions of the class statement, means two distinct class objects. `isinstance` compares
class identity, not name or structure, so it says `False` for an object that is a `Thing`
from the other copy.

**2.** (a) **Module-level mutable state**: caches, registries, connection pools, `logging`
configuration — each copy gets its own, so a registration performed against one copy is
invisible to code holding the other. (b) **Singletons and side effects at import time**:
a module that opens a file, starts a thread, or installs a signal handler does it twice.
(a)/(b) are much worse than the class-identity bug because the class-identity bug produces a
*visible* failure at a specific line, whereas a duplicated registry produces "the plugin
isn't registered" or "the second call went to a different cache" — symptoms with no obvious
connection to imports, that reproduce only under a particular entry point, and that pass
under `pytest` and fail under `python -m` or the reverse.

**3.** Because in Node the two copies do not come from a *layout accident* — they come from
the package deliberately shipping two entry points (`main` for CJS, `exports`/`module` for
ESM) and different consumers legitimately reaching for different ones. The Python bug is
fixed by fixing your entry point; the Node hazard cannot be fixed by any single consumer,
because a package deep in the graph doing `require('pkg')` while your code does
`import 'pkg'` will load both, and neither party is wrong. It is a property of the graph, not
of any file. That is what makes it structurally harder: the mitigation has to be published by
the *package*, not applied by the *consumer*.

### A6 — Circular imports, three answers

**1.** `main` imports `a`; `a`'s dependency `b` is instantiated and evaluated first; `b`
imports `a`, which is already in the module map mid-instantiation, so linking succeeds — the
binding for `a` exists but is **uninitialised**. `b` runs, assigns its own `b`, then reads
`a` and throws `ReferenceError: Cannot access 'a' before initialization`. The temporal dead
zone, across a module boundary. With `function` declarations instead, the output is
`b sees undefined`-free and both logs run: **function declarations are hoisted and
initialised during instantiation**, before any module body evaluates, so the binding is
already live when `b` runs. The folklore is right and this is why — it is not "functions are
called later", it is that function declarations are initialised in a different phase from
`const`.

**2.** CommonJS `require` returns the **partially populated `exports` object** as it stands
at that moment. So `b` gets an object missing `a`, reads `undefined`, and continues — no
error, wrong value, and the wrongness depends on which module the cycle was entered from.
Ship the ESM behaviour. The CJS failure is silent, order-dependent, and surfaces arbitrarily
far from the cycle as an `undefined is not a function`; the ESM failure names the variable,
names the moment, and is deterministic for a given entry point. A loud failure at the cycle
beats a quiet wrong value anywhere.

**3.** The ban buys: (a) **a topological order for package initialisation**, so `init()`
functions and package-level variables have a well-defined, dependency-respecting execution
order with no partial-initialisation state to specify (contrast A7); and (b) **a DAG for the
build system**, so packages can be compiled and cached independently, in parallel, and an
export-data file for a package is complete when written — which is a large part of why Go
builds are fast. It costs the user **the ability to express genuinely mutually recursive
definitions across a package boundary**, and no tooling fixes it because it is a
well-formedness rule, not a diagnostic. The user's only recourse is to restructure — extract
the shared piece into a third package, or invert a dependency through an interface — which is
often the right design and sometimes just tax.

**Trap.** Saying ESM cycles "work because of live bindings." Live bindings are why a cycle
*can* work at all — the importer sees the binding, not a snapshot — but they do not save you
from the TDZ, which is what actually fires here. Live bindings and initialisation order are
two separate mechanisms and only one of them is on your side.

### A7 — The static initialization order fiasco

**1.** C++ guarantees that within a single translation unit, non-local objects with static
storage duration are dynamically initialised **in declaration order**; across translation
units the order is **unspecified**. The standard could not specify it because the order would
have to be a whole-program property, and C++'s compilation model deliberately compiles each
TU with no knowledge of the others — an ordering would require either a link-time analysis of
initialisation dependencies (which are not declared anywhere and are not statically
derivable, since a constructor can call anything) or a language-level dependency declaration
C++ has never had. The unspecified order is not an oversight; it is the price of the
compilation model.

**2.** **Go**: initialises packages in dependency order, and within a package orders
variable initialisation by reference dependencies, then runs `init()` — solves it, and the
import-cycle ban (A6.3) is what makes the ordering well-defined. **Java**: initialisation is
lazy, triggered on first active use, with a documented per-class procedure — *contains* it,
because you never observe an uninitialised class, but pushes the problem into ordering
questions at runtime and buys the deadlock of Q8. **Rust**: statics must be const-evaluable,
so there is no dynamic initialisation to order — solves it by removing the feature, and
pushes anything that needs runtime setup into explicit lazy initialisation (`OnceLock`,
`LazyLock`) whose ordering is determined by first access. Go and Rust solve; Java contains.

**3.** It forecloses **self-registration**: the pattern where a translation unit registers
itself into a global table at startup by declaring a static object whose constructor does the
registration, so that merely linking a file makes its plugin/test/handler available. Every
C++ test framework and plugin system is built on it. Rust crates that reintroduce it —
`ctor` (runs a function before `main` using platform init-array sections), `inventory` and
`linkme` (collect items into a custom linker section) — are fragile because they depend on
**linker behaviour that is not part of the language**: sections get garbage-collected by
`--gc-sections`, behave differently across ELF/Mach-O/PE/wasm, interact badly with static
linking into an `rlib` where the linker may drop an object file nothing references, and give
no ordering guarantees among themselves. The pattern was foreclosed at the language level,
so its reintroduction lives at a level where nobody promised anything.

### A8 — The deadlock nobody can see

**1.** Java's initialisation procedure: a thread acquires the class's initialisation lock,
observes the state, sets it to "being initialised by this thread", releases the lock, and
runs `<clinit>`. Any *other* thread that touches the class while it is in that state
**blocks on the initialisation lock until initialisation completes**. So: thread 1 begins
initialising `A`, and `A`'s `<clinit>` touches `B`; thread 2 has already begun initialising
`B`, and `B`'s `<clinit>` touches `A`. Thread 1 waits for `B`'s init to finish; thread 2
waits for `A`'s init to finish; neither `<clinit>` can complete. A textbook lock-order
inversion where the locks are per-class initialisation locks. Note the self-recursive case
is *not* a deadlock — a thread re-entering a class it is already initialising is defined to
proceed, which is why single-threaded circular static init merely gives you a partly
initialised class and a `null`, not a hang.

**2.** Because the initialisation lock is **an internal runtime lock, not a Java monitor**.
`jstack`'s deadlock detector reports cycles over monitors and `java.util.concurrent` locks
because those are the ones the JVM exposes to it; the class-init lock is not one of them, so
the threads look like they are simply blocked with no lock attribution. What this tells you
generally: **deadlock detection only covers the lock kinds the runtime chose to instrument**,
and every runtime has internal locks — class init, classloader locks, JIT compilation queues,
GC handshakes — that will never appear. If you build a runtime with an internal lock on a
path user code can reach, you have created a hang class your own tooling cannot see.

**3.** Eager initialisation costs **startup time proportional to the whole reachable class
graph, whether or not it is used** — you pay to initialise every class in every library you
link, including the ones a given run never touches — and it *reintroduces the ordering
problem*, because eager means you must pick an order, and A7 says there is no correct one to
pick in the presence of cycles. The JVM chose lazy because dynamic class loading is a
first-class feature: the set of classes is not known up front, classes arrive from the
network and from generated bytecode, so "initialise everything eagerly" is not even a
well-defined instruction. Lazy is forced by open-world class loading, and the deadlock is
its bill.

**Trap.** "Static initializers are single-threaded, so this cannot happen." Each `<clinit>`
runs on one thread, which is the guarantee people remember — but two *different* classes'
initializers run on two threads concurrently, and that is all the deadlock needs. The
per-class guarantee says nothing about the order in which classes initialize relative to
each other.

### A9 — What must a module export to be compiled against

**1.** **Java** ships a class file: signatures, plus enough metadata to type-check and to
emit a call. A consumer compiles against it and links at load time; changing a method *body*
requires recompiling nothing downstream. **Rust** ships crate metadata including the **MIR
for generic (and `#[inline]`) functions**, because the consumer must instantiate the generic
at the consumer's types — the code does not exist until the consumer's compilation. So
changing a generic function's *body* is a recompilation trigger for every dependent, and
generic code cannot sit behind a stable ABI at all. **C++** ships the template's **source**,
in a header, textually included and re-parsed by every consumer. Consequence: compile times
scale with (number of TUs × header size), and changing a template body forces rebuilding
every TU that included it — which is why `#include` graphs are a build-performance discipline
in C++ and why modules were eventually added.

**2.** Erasure costs (a) **no specialisation over primitives** — `List<int>` is
inexpressible, so you box, paying an allocation and a pointer chase per element and losing
the flat memory layout; and (b) **no runtime access to type arguments**, so
`new T[]`, `x instanceof List<String>`, and reflection over generic instantiations do not
work, which pushes libraries into passing `Class<T>` tokens around by hand. (a) took two
decades because fixing it is not a generics change, it is a *value-representation* change —
you need value types with defined layout, a way to specialise or to represent both erased and
specialised instantiations, and total backwards compatibility with every existing erased
class file. That is Valhalla, and its long timeline is evidence for how deep a
representation decision reaches.

**3.** Because two-phase lookup means a template's meaning depends on names visible at the
**point of definition** (for non-dependent names) *and* names found by argument-dependent
lookup at the **point of instantiation** (for dependent ones). To compile an exported
template separately, the exporting implementation must therefore retain essentially the full
parsed representation *plus the definition context's name-lookup environment*, and then
merge it with each consumer's instantiation context at instantiation time — which is a
cross-TU semantic merge, not a link step. In practice that meant shipping the front end's
internal representation as an interchange format, which is exactly the thing the separate
implementations could not agree on. `export` was removed not because the idea was wrong but
because the effort it required was indistinguishable from shipping a module system, which is
what eventually happened.

### A10 — API stable, ABI broken

**1.** **Private data member added**: `sizeof` and every member offset after it change. Any
consumer compiled against the old header allocates the wrong size and reads the wrong
offsets. The user finds out **at run time, as memory corruption** — no linker error, no
compile error, just a heap smash somewhere unrelated. **`std::string` layout change**:
libstdc++'s C++11 change (COW to SSO) altered the type's representation, so an object passed
across a boundary between differently-built code is misinterpreted; here the vendor added a
**mangled-name distinction** (the `__cxx11` inline namespace, selected by
`_GLIBCXX_USE_CXX11_ABI`), which converts the disaster into a **link error** — undefined
symbol naming a type that "obviously exists". Vastly better: found at link time. **glibc
`memcpy` overlap behaviour**: code that relied on the old implementation's incidental
copy direction breaks; the user finds out **at run time on a machine with a newer glibc**,
which is the worst of the three because the artifact that changed is not one the developer
built or tested against.

The ranked lesson: run time < link time < compile time, and ABI work is mostly the craft of
moving failures leftward.

**2.** Each exported symbol carries a version label, and the object's version-definition
section declares which labels it provides and which is the default. A consumer's undefined
reference records the label it was resolved against at *link* time, so an old binary keeps
resolving `foo@VER_1` while a newly linked binary gets `foo@VER_2` — and both implementations
live in one `.so`. What that buys over bumping the soname: **one shared library serving both
generations in one process**. With sonames, a process that transitively loads both old and
new consumers loads two copies of the entire library, with two copies of its global state —
which for something like the C library is not merely wasteful, it is incorrect (two mallocs,
two locale tables, two `errno` regimes). Symbol versioning is what lets glibc never break and
never fork.

**3.** The idea: **make the consumer stop depending on the provider's layout and internal
decisions, by routing everything through an interface the provider can reimplement**. Swift's
library evolution makes struct layout and enum cases opaque across a resilience boundary, so
field access goes through value-witness/metadata indirection instead of a fixed offset, and
adding a field or a case is non-breaking. CPython's limited API hides `PyObject` struct
internals behind functions and hides the `Py_ssize_t`-laden macro layer, so a wheel built
once works across minor versions — at the cost that every field read becomes a call. Escape
hatches: Swift's `@frozen` (and `@inlinable`) opt a type back into layout-visible, fast,
never-changeable; CPython's answer is that you can simply not use the limited API and rebuild
per version, which is what most performance-sensitive extensions do. Both prove the same
thing: **resilience is an indirection you pay for, so every such system needs a way to
declare "this one is final and fast."**

**Trap.** "We keep ABI stable by not changing public headers." Inline functions and
templates in headers are compiled *into the consumer*, so their bodies are ABI even though
they look like implementation. Changing an inline function's body without a version bump
gives you a process containing two different implementations of one function, with whichever
the linker picked winning — the ODR violation family, and it is silent.

### A11 — Interposition

**1.** ELF uses a **flat, global symbol namespace with first-definition-wins ordering**: the
dynamic loader searches objects in load order (the executable, then `LD_PRELOAD` objects,
then `DT_NEEDED` dependencies breadth-first), and the first object defining a name satisfies
every reference to it, from anywhere. Preloading places your object early, so your `malloc`
wins for the whole process, including calls from inside libc-adjacent libraries — that is the
capability, and it is what ASan, jemalloc, and every LD_PRELOAD-based shim rely on. The
production bug is the same rule with no preload: two unrelated libraries both export
`parse_config`, one is earlier in the load order, and *both* libraries' internal calls
resolve to that one. Neither author did anything wrong, nothing warns, and the symptom is a
function behaving as though it were a different function — which it is. `-fvisibility=hidden`
exists exactly to shrink the surface of this bug.

**2.** Two-level namespace records, for each undefined symbol, the *install name of the
library it was found in* at link time, so resolution at load time is (library, symbol), not
just symbol. Safer: two libraries exporting the same name never collide, and adding a symbol
to a system library cannot hijack a reference intended for yours — the entire bug class in
part 1 is structurally absent. Broken: **global function interposition**, so the
preload-a-replacement-`malloc` technique does not work by default. macOS supports it only via
`DYLD_INSERT_LIBRARIES` plus explicit `__interpose` sections (and, more recently, only where
platform hardening permits it at all), which is a deliberate narrowing: interposition becomes
a declared, per-symbol act rather than an emergent consequence of name collision.

**3.** (a) **Hidden visibility** (`-fvisibility=hidden` plus explicit export annotations):
internal symbols never enter the dynamic symbol table, so calls to them are resolved at link
time and cannot be interposed. Cost to users: anything not explicitly exported is genuinely
unreachable, so the debugging/monkey-patching/LD_PRELOAD-shim tricks users relied on stop
working, and profilers see fewer symbol names. (b) **`-Bsymbolic` / protected visibility**:
references within the library bind to its own definitions even for exported symbols. Cost:
you break the *intended* case too — a user who legitimately wants to replace your allocator
or your logger for the whole process now finds your internal calls unaffected, producing a
process where half the calls go one way and half the other, which is worse than either
consistent behaviour. There is also a real hazard with protected visibility and copy
relocations that has historically produced address-comparison inconsistencies for the same
function. The honest summary: interposition-proofing is undoing the flat namespace one
symbol at a time, and it makes your library less debuggable in exact proportion to how much
it protects.

### A12 — A registry keyed by name

**1.** The second declaration either overwrites the first, or is rejected, or — worst and
most common in a first implementation — is silently unified with the first, so both modules'
code now operates on **one** class whose method table is the merge of two independent
definitions in load order. The silence is the serious part because the failure has no locus:
module A's `Point` behaves correctly until module B is loaded, and then a method A never
wrote appears on A's objects, or A's method is replaced by B's. Nothing in A changed, A's
tests pass in isolation, and the bug's trigger is "someone else's module was imported first."
A collision *error* is a two-minute fix; a collision *merge* is a bug whose reproduction
depends on import order and whose blast radius is every instance in the process.

**2.** (a) **Name only** — reloading works trivially (redefinition finds the same entry),
serialization works trivially (the name is the wire format), separate compilation is
impossible to do safely because two units cannot both define the name. (b) **Fresh opaque id
per definition** — separate compilation is safe and collisions are impossible; reloading
breaks, because the reloaded class gets a new id and existing instances belong to the old
one; serialization breaks, because the id is meaningless across processes and you must
maintain a side mapping. (c) **Module path + name** — separate compilation is safe,
serialization works (the path is stable and meaningful), collisions are impossible *within*
a coherent module graph; reloading works if you define reload as "same path, new definition",
which then reintroduces the instance-migration problem of a live redefinition. (c) is the
right default and (b) is the right internal representation, with (c) as the key that maps to
it — which is exactly what Java does with (classloader, name), and Java's classloader is the
identity component pretending to be a namespace.

**3.** A warning is right because the runtime **cannot distinguish the accident from the
intent**. The same duplicate-class condition arises from a genuine mistake (two frameworks
vendoring the same library) and from deliberate, working practice (a test bundle overriding a
class, a hot-patch, a dylib loaded twice by different paths in a plugin host) — and
Objective-C's model has no module boundary to arbitrate between them, because classes are
registered into one flat table at image load. Making it an error would break shipped
software for a condition the runtime cannot prove is wrong. To make it an error you would
need **a namespace in the identity** — classes owned by a module, resolution scoped to a
module graph — which is exactly what Swift did with module-qualified mangled names, and note
that Swift had to introduce the boundary at v1 rather than retrofit it into Objective-C.

### A13 — "Private" needs a boundary

**1.** **Rust**: restricts other *modules within the same crate* and everything outside it;
the boundary is the module tree, checked entirely by the compiler over source it can see.
**Java package-private**: restricts code not in the same package — and "in the same package"
is a *claim a source file makes about itself*. Anyone who can add a class file declaring
`package com.acme` gets access, which is the classic package-injection attack and the reason
`sealed` JARs and, later, JPMS exist. That is the one defeated by controlling a file's
location. **Python**: restricts nobody; `_helper` is a convention plus a rule that `import *`
skips it, and even name mangling (`__helper` → `_C__helper`) is a renaming, not a
restriction.

**2.** Because visibility was drawn as a **compile-time property of names**, while
reflection operates on the **runtime representation of objects**, and nothing in the design
made the runtime representation obey the compile-time rule. A private field is a field; once
you can enumerate fields at runtime, privacy was never encoded in the thing you are
enumerating. So reflection is not violating the rule, it is operating in a layer where the
rule was never expressed. The only way for reflection to respect visibility is for the
*runtime* to carry and enforce the boundary — which is precisely what JPMS added, and note
what it took: a new boundary concept, a new access check inside `setAccessible`, and command
line flags to opt out.

**3.** The runtime must own (a) **object representation** — no way to obtain a field's value
except through an operation the runtime mediates, so no raw memory access, no
`sun.misc.Unsafe`, no C extension holding a struct pointer, no debugger API that bypasses the
check; and (b) **the loading and naming path** — the runtime must know which boundary the
requesting code belongs to and be unable to be lied to about it, which means code identity is
established at load time and is not forgeable. Give up: **unrestricted reflection-driven
frameworks** — serialisation that reads private state, ORMs that populate private fields,
dependency injection into private members, mocking libraries that replace private methods.
Capability-style languages and the object-capability model take this seriously and pay
exactly that price; the E/Newspeak lineage is the honest precedent. Everyone else keeps
advisory privacy and a reflection hole, and then discovers they cannot optimise on the basis
of encapsulation either.

### A14 — Feature flags are a combinatorial module system

**1.** You get `serde` **with `std` enabled**, despite asking for `default-features = false`:
Cargo builds one copy of each crate version per unique feature-set-plus-target, and **unifies
features across the graph by union**. Resolver `"2"` — the default since edition 2021 —
carves out three cases where it deliberately does *not* unify: build-dependencies and
proc-macros, platform-specific deps for targets you are not building, and dev-dependencies.
None of them help here, since both `serde` requirements are ordinary dependencies. The
designers chose union because the alternative —
building `serde` twice, once with `std` and once without — reintroduces exactly the two-copies
problem of Q4: two `serde::Serialize` traits, two incompatible sets of impls, and a type
implementing "the wrong one". Union guarantees one copy and therefore one coherent set of
trait impls, and it makes resolution monotone and deterministic. The surprise is the price of
coherence, which is why the rule is defensible even though the `no_std` breakage it causes is
genuine and unfixable from your side.

**2.** A crate violates additivity whenever a feature *changes* behaviour instead of adding
it: switching a numeric backend so results differ, changing a default (a hash function, a
serialisation format), replacing an implementation with a stricter one that now rejects
previously accepted input, or — the sharpest case — mutually exclusive features where
enabling both fails to compile. It is undetectable by the package manager because the
manager sees feature names as opaque strings gating `cfg` blocks; "this feature only adds
API" is a semantic property of the source, not a structural one, and nothing in the manifest
format can express or check it. The manager cannot even tell that two features are meant to
be exclusive until the build fails.

**3.** The strategy is **feature-powerset testing** — build and test every subset
(`cargo hack --feature-powerset`, or the `#ifdef`-configuration equivalents). It does not
scale because the number of configurations is 2^n in the feature count *per crate*, and the
interesting failures are cross-crate, so the real space is a product over the graph. Practical
mitigations are all approximations: test the powerset up to pairs (most real bugs are
pairwise, which is the standard combinatorial-testing result), test a curated set of named
configurations, or reduce n by refusing to add features. The honest position is that feature
flags are a module system with an exponential number of modules and a testing budget that is
linear, and every codebase resolves that by leaving most of its configurations
uncompiled — which is precisely why `#ifdef` branches rot.

**Trap.** "`default-features = false` means I get the minimal build." It means *your*
declaration requests the minimal build. Feature resolution is a property of the whole graph,
not of your manifest, and one transitive dependency's opinion overrides your intent with no
warning anywhere. Reading your own `Cargo.toml` to determine what you compiled is reading
the wrong file; only the resolved graph knows.

### A15 — The package manager is the real module system

**1.** Easiest to hardest: **npm**, **Go MVS**, **Cargo**, **pip**. npm is easiest *for
ordinary `dependencies`*, because nesting means a conflict is never fatal — the solver hands
that consumer its own copy, so resolution is nearly local. Its one genuinely hard constraint
is `peerDependencies`, which exist precisely because nesting is *not* an acceptable answer
there; since npm 7 a peer conflict is a fatal `ERESOLVE`, which is why `--legacy-peer-deps`
is a fixture of real build files. Go's MVS is easy for a
different reason: it is not a search at all, it is a **maximum over the requirements** —
each module states a minimum, the answer is the largest minimum, which is computable in one
pass with no backtracking, and is reproducible without a lockfile. Cargo is harder: it must
find a single version per semver-compatible range, which is a genuine constraint problem with
backtracking, though the escape hatch of allowing distinct majors to coexist prunes a large
class of conflicts. pip is hardest because the single flat namespace means **one version per
package for the entire program**, with no escape hatch at all, so every constraint must be
satisfied simultaneously — a real SAT-shaped problem where "unsatisfiable" is a frequent,
legitimate answer. The structural property that determines the ranking is simply **how many
copies of one package the runtime will tolerate**: more copies, easier solver, worse identity
(Q4).

**2.** It is social because **nothing checks it**: a maintainer decides whether a change is
breaking, and the version number records that opinion. Adding a public name is a minor bump
by the rules and breaks glob importers (A3); adding a trait impl is additive by the rules and
breaks inference; changing timing or error text breaks someone's test suite. Two systems made
part of it technical: **Elm** computes the API diff between the published and proposed
versions and *forces* the version number the diff implies, so the major/minor/patch decision
is not a human judgement — at the cost of only being able to see type-level changes, so
behavioural breakage still slips. **Go** encodes the major version *in the import path*
(`example.com/m/v2`), which makes a major bump a different module by construction — so a
"breaking change" is not a promise about compatibility at all, it is a rename, and old and
new can coexist. Cargo's `cargo-semver-checks` is a third, weaker instance: a linter that
catches a known catalogue of breaking changes, not a proof.

**Trap.** "A lockfile makes the build reproducible." It makes *dependency selection*
reproducible, which is one input among several. Say "a lockfile makes resolution
deterministic" and you are right; say "reproducible" and the interviewer will ask about the
toolchain, the system libraries, and the build scripts, and you will have claimed something
only a content-addressed build system delivers.

**3.** A lockfile pins versions; it does not pin **the artifact the version resolves to** or
**the build environment**. The artifact problem is republication — a registry entry mutated
or a Git tag moved (the `left-pad` unpublish and the general dependency-confusion/typosquat
family live here) — closed by recording a **content hash** per entry, which every modern
lockfile now does. The environment problem is broader and mostly still open: the compiler
version, the platform's system libraries, build scripts that read the network or the clock,
and — per A14 — the *feature resolution*, which a lockfile does record in Cargo but which many
ecosystems do not. That is what the reproducible-builds and content-addressed-store work
(Nix, Bazel) is for, and it is a strictly larger problem than dependency pinning.

### A16 — Retrofit: CommonJS to ES modules

**1.** Hard because ESM requires the **imported module's export names to be known before any
module in the graph is evaluated** — instantiation links bindings first, evaluates second —
while a CJS module's exports are *whatever it assigns to `module.exports` when it runs*,
which can be computed, conditional, or produced by a loop. Determining the names statically
is undecidable in general. Node's answer is a **heuristic static analyser**
(`cjs-module-lexer`) that recognises the common assignment patterns and extracts names from
them; anything it does not recognise simply has no named exports, and the default export —
the whole `module.exports` object — is always available. So `import { readFile } from` a CJS
package works when the package is written in a recognised style and fails with "does not
provide an export named" when it is not, which is a confusing error precisely because the
export *does* exist at runtime.

**2.** **Top-level `await`.** Without it, an ESM graph's evaluation could be argued to be
finishable synchronously and `require(esm)` would be a scheduling detail. With it, a module's
evaluation genuinely may not complete without returning to the event loop, and `require`
must return a value *now* — so `require` of a graph containing top-level await is not an
implementation gap, it is a request to synchronously wait for something that by construction
may depend on the event loop, i.e. a deadlock. That is why the eventual relaxation permitted
`require` of ESM only for graphs *without* top-level await: the restriction tracks the
feature, not the effort.

**3.** (a) **A single entry point** — ship only ESM (or only CJS), so no consumer can reach a
second copy. Unsatisfactory because it forces every consumer of the other kind to change, and
in a transitive graph you do not control your consumers. (b) **The "wrapper" pattern** — one
implementation module plus a thin shim for the other format, so both entry points resolve to
the same state. Unsatisfactory because it only works for the CJS→ESM direction cleanly (an
ESM wrapper can `import` the CJS implementation and re-export; the reverse requires
synchronous `require` of ESM, i.e. part 2), and because any consumer that reaches past the
entry point with a deep import bypasses the shim. A third, partial answer is to make the
duplication harmless by **having no module-level state** — no singletons, no registries, and
`instanceof` replaced by structural checks — which is real advice and an admission that the
module system cannot give you identity.

### A17 — Retrofit: Java 9 modules

**1.** Because the module system's central promise is that **a package is owned by exactly
one module**, which is what makes access checks decidable at load time: given a package name,
the runtime must be able to say which module exports it and therefore whether a given reader
may access it. If two modules could contribute to one package, package-private access would
straddle a module boundary — code in module M could see package-private members in module N
purely by declaring the same package, which is the package-injection hole of A13.1 preserved
at the module level and would make strong encapsulation unenforceable by construction. The
pattern it killed: **splitting an API across artifacts** — the long-standing practice of one
jar declaring `javax.something` interfaces and another jar adding classes to the same
package, which is how a great deal of the Java EE / annotation-API ecosystem was packaged,
and why those artifacts all had to be repackaged or relocated.

**2.** They were relying on **reflective access to non-public members of classes in packages
they did not own** — setting private fields (ORMs materialising entities, DI containers
injecting, serialisers reconstructing), calling package-private methods, and defining classes
into another package's namespace (proxy and bytecode-generation libraries doing
`defineClass` into the target's package to get package-private access). "Just fix the
frameworks" was unavailable for two reasons: the capability had no supported replacement for
several of those uses at the time (defining a class into another module's package, in
particular, required new API — the `MethodHandles.Lookup` privilege-granting machinery grew
to cover it), and the *deployment* reality is that a user upgrading their JDK does not get to
upgrade every transitive library simultaneously. That is why `--illegal-access=permit` shipped
as the default, was degraded over releases, and why `--add-opens` became permanent build
furniture rather than a migration step.

**3.** The lesson: **a boundary that does not exist at v1 will be load-bearing by v9**.
Everything built in the interval is entitled to assume the boundary's absence, and the
assumption is invisible — nobody writes down "I depend on there being no module system."
For a boundary to be addable later, two things must be true at v1: (a) the *capability* it
would restrict must already require an explicit, greppable act — a marked reflective API, a
declared permission, an `unsafe`-like marker — so that the future boundary has a finite, known
set of sites to grandfather rather than an unbounded set of ordinary-looking code; and (b)
the naming/identity scheme must already be able to *express* the boundary, so that adding it
is a restriction on an existing coordinate rather than the invention of a new one. Java had
neither: `setAccessible` was ordinary API and package identity had no owner. Contrast a
language that ships crate/module-qualified identity and an explicit unsafe marker from v1;
tightening those later is a lint, not an ecosystem event.

**Trap.** Framing JPMS as a failure. It largely achieved its actual goals — the JDK itself
is modularised, which is what enabled `jlink`, ahead-of-time-friendly images, and the removal
of internal APIs on a schedule. What failed was the *migration path for the ecosystem*, and
saying "modules were a mistake" hides the transferable lesson, which is about the timing of
boundaries, not their value.
