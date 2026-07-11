# Canonical Reading List

> Primary-source layer of the `language-design` skill. One line each: source + the single idea to steal. Load when you want the original argument behind a design axis, not a summary.

## Object model
- **Goldberg & Robson, *Smalltalk-80* (Blue Book)** — everything is an object; classes are objects; message-send is the only control primitive.
- **Cointe, *Metaclasses are First Class: the ObjVlisp Model* (1987)** — the reflexive `Class`/metaclass loop; how to bootstrap a tower without infinite regress.
- **Bobrow & Kiczales, *The Common Lisp Object System* / metaclass work** — the metaobject protocol makes the class machinery itself programmable.
- **Ungar & Smith, *Self: The Power of Simplicity* (1987)** — prototypes + delegation; no classes needed; behavior is inherited state.
- **Chambers, Ungar, Lee, *Self implementation* / maps** — "maps" (hidden classes) recover slot-map speed from a prototype language — ancestor of V8 shapes.
- **Kiczales, des Rivières, Bobrow, *The Art of the Metaobject Protocol* (AMOP)** — open implementations: expose `compute-discriminating-function`, generic-function dispatch as user-tunable.

## Dispatch & VMs
- **Deutsch & Schiffman, *Efficient Implementation of the Smalltalk-80 System* (1984)** — inline caching + JIT compilation of bytecode; the origin of the monomorphic call-site cache.
- **Hölzle, Chambers, Ungar, *Polymorphic Inline Caches* (1991)** — PICs: cache multiple receiver classes per site; feed type feedback to the optimizer.
- **Ierusalimschy et al., *The Implementation of Lua 5.0*** — register-based bytecode VM + table-driven semantics; why registers beat a stack machine.
- **Nystrom, *Crafting Interpreters*** — a complete bytecode VM end to end: compiler, GC, closures/upvalues, NaN-boxing — the practical reference.
- **Wren source (Nystrom)** — a small class-based VM: sealed classes, fibers, flat slot layout — a readable production-grade Smalltalk-lite.

## Closures & control
- **Abelson & Sussman, *SICP*** — closures = procedures + environments; the environment model of evaluation.
- **Dybvig, *The Scheme Programming Language* / *Three Implementation Models for Scheme*** — proper tail calls and `call/cc` implementation; stack as heap-allocated frames.
- **Appel, *Compiling with Continuations*** — CPS as an IR; every control construct (loops, exceptions, generators) is a continuation.
- **Ierusalimschy, *Closures in Lua* / *Coroutines in Lua*** — open/closed upvalue closing; stackful asymmetric coroutines as the one concurrency primitive.

## Values & types
- **Pierce, *Types and Programming Languages* (TAPL)** — the vocabulary: subtyping, variance, parametricity, algebraic data + pattern matching.
- **Okasaki, *Purely Functional Data Structures*** — persistent structures via lazy evaluation + amortization; how immutable collections stay fast.

## Errors
- **Pitman, *Exceptional Situations in Lisp* / CL Condition System** — separate signaling from handling from restarting; the handler decides, restarts live at the signal site — strictly more expressive than unwind-only exceptions.

## Concurrency
- **Armstrong, *Making Reliable Distributed Systems in the Presence of Software Errors* (Erlang thesis)** — share-nothing processes + let-it-crash + supervision trees; isolation is the reliability primitive.
- **Hoare, *Communicating Sequential Processes* (CSP)** — concurrency via synchronous channels, not shared memory — the model Go's goroutines/channels descend from.
- **Sústrik, *Structured Concurrency* / Beaz’s *Trio* nurseries** — no task outlives its lexical scope; `go`/`spawn` without a join point is the concurrency `goto`.

## Language design (meta)
- **Steele, *Growing a Language* (1998)** — a language must let users grow it; design the extension mechanisms, not just the built-ins.
- **Hoare, *Hints on Programming Language Design*** — orthogonality, security, and "the cost of a feature is what it precludes."
