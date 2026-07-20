# 08 — Errors and Unwinding

Leaving a computation early without leaving it broken. The through-line: *an early exit is
a jump plus a promise — the promise is that every scope you skipped got to clean up, and
the whole design space is about who keeps that promise and what they charge for it.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — What "zero-cost" actually costs

A C++ implementation on the Itanium ABI compiles `try`/`catch` with no runtime check on
entry to a `try` block. The claim is "zero-cost exceptions." Throwing then runs a
*two-phase* unwind: a search phase that walks frames asking each personality routine
"would you handle this?", and only afterwards a cleanup phase that walks the same frames
again running destructors.

1. The non-throwing path is genuinely free of instructions. What did it cost instead, and
   in what units — name the artifact that grew.
2. Why two phases? A single pass that unwound and ran cleanups as it looked for a handler
   would visit each frame once. What does the second walk exist to make possible, and what
   would break without it?
3. The JVM does not do this. A `throw` in HotSpot consults a per-method exception table and
   there is no search phase in the Itanium sense. Explain what the JVM has that C++ does
   not, that makes the simpler scheme sound.

### Q2 — Two shapes of failure at the call site

```rust
let f = File::open(path)?;      // failure is a value in the return
```
```cpp
auto f = File::open(path);      // failure is an edge out of this line
```
```c
FILE *f = fopen(path, "r");     // failure is a magic return value plus a global
```

1. For the optimizer specifically: what does each shape do to the control-flow graph of the
   *caller*, and which one constrains code motion around the call site?
2. `Result` makes the error part of the return type, so the return value grows to
   `max(size(T), size(E))` plus a discriminant. Name two concrete costs of that beyond
   bytes, and one thing it buys that an exception cannot.
3. `errno` is thread-local mutable global state consulted after the fact. Give the specific
   composition failure this creates — a case where correct-looking code reads the wrong
   error — and say why the "value in the return" designs are immune.

### Q3 — What checked exceptions got right

Java requires `throws IOException` on every function in the propagation path. Essentially no
language since has copied it. Rust encodes the same information in `Result<T, E>` and
everyone considers that fine.

1. The information content is nearly identical. Name the *structural* difference that made
   one intolerable and the other pleasant — it is not syntax.
2. Higher-order functions are where checked exceptions actually died. Write out why
   `Stream.map` cannot accept a lambda that throws a checked exception, and what Swift's
   `rethrows` does to fix exactly this.
3. Java's escape hatch was `RuntimeException`, and the standard advice became "wrap and
   rethrow." Argue that this was not a failure of discipline but a forced move, then say
   what a language would need to make checked failure work.

### Q4 — Four takes on "run this on the way out"

```java   finally { close(f); }        ```
```ruby   ensure  f.close             ```
```go     defer f.Close()             ```
```cpp    File f(path); // ~File()    ```

1. Rank these by what happens when the *cleanup itself fails*. State concretely: what does
   C++ do if a destructor throws during unwinding, what does Java do if `finally` throws,
   and what does Python's `__exit__` raising do to the original exception?
2. `defer` in Go is function-scoped; `defer` in Zig and Swift is block-scoped. Give the
   observable difference with a loop body, and say which one is harder to implement and why.
3. RAII is the only one of the four that is not a statement. What does that buy — name the
   property the other three cannot guarantee even in principle.

### Q5 — The strong guarantee's copy

C++ names three exception-safety levels: **basic** (no leaks, invariants hold), **strong**
(the operation either completes or has no effect), **nothrow**.

```cpp
v.push_back(x);   // reallocation: allocate new buffer, move/copy n elements, free old
```

1. Explain why implementing the strong guarantee here forces *copying* rather than moving
   the existing elements, unless something specific is true.
2. That "something specific" is `noexcept` on the move constructor, checked via
   `move_if_noexcept`. Why is the absence of a `noexcept` annotation, rather than the
   presence of a throwing move, what triggers the pessimisation — and what does that tell
   you about how safety guarantees compose across a library boundary?
3. Argue that `nothrow` is not a third level on the same ladder but a *precondition* of the
   other two. Use `swap` to make the point.

### Q6 — Panic is not an error

Rust: indexing out of bounds panics; opening a missing file returns `Err`. Go: a nil map
write panics; a missing file returns an `error`. Both languages insist the two are
different in kind.

1. What is the criterion that puts a failure on one side or the other? "Recoverable" is
   circular — give the version that a library author can actually apply.
2. Panics are invisible in signatures: every Rust function may panic and none say so. Argue
   that this is deliberate and correct, then name what it costs — be specific about what
   `catch_unwind` is and is not for.
3. `panic = "abort"` deletes unwinding entirely. Name two things that stop working, and say
   why a server and an embedded target land on opposite sides of that switch.

### Q7 — Unwinding into code that never agreed

```rust
extern "C" fn cmp(a: *const c_void, b: *const c_void) -> i32 {
    let x = deref(a); let y = deref(b);
    x.partial_cmp(&y).unwrap()          // may panic
}
unsafe { qsort(base, n, sz, cmp) };
```

1. The panic unwinds out of `cmp` and into `qsort`'s frames. Name two distinct reasons this
   is undefined behaviour rather than merely rude — one about the frames, one about `qsort`
   itself.
2. Rust shipped its fix in two steps, in the opposite order to what you would guess:
   `extern "C-unwind"` was stabilized first (1.71), and only later did plain `extern "C"`
   change from undefined behaviour to a guaranteed abort (1.81). Why was a second ABI
   necessary — what real pattern would a blanket abort have broken?
3. C++ has the same problem in miniature with `noexcept`. A `noexcept` function that throws
   calls `std::terminate` rather than propagating. Why terminate instead of "just let it
   through, the annotation was only advice"?

### Q8 — Who pays for the stack trace

Java attaches a stack trace at exception *construction*, via `fillInStackTrace`. Rust's
`Result` carries no trace at all; Go's `error` carries none either.

1. Why is construction, not throwing, the capture point in Java — and what surprising cost
   does that create for the "exception as a control-flow signal" pattern?
2. HotSpot will, for a hot implicit exception site, start reusing a preallocated exception
   object with an empty stack trace. Explain why a JIT would do that, and why it is a
   notorious production debugging experience.
3. `Result`-based languages structurally struggle to produce traces. Explain the mechanism
   — why does `?` make a trace hard where `throw` makes it easy — and describe what Zig's
   error return traces do differently.

### Q9 — Values you can ignore

```go
f, err := os.Open(path)
_ = err                    // legal
```
```rust
let r = do_thing();        // warning: unused `Result` that must be used
```
```c
fclose(f);                 // returns int. Nobody checks it. It can fail.
```

1. "Errors are values" and "errors must not be ignorable" are usually presented as opposed.
   Show that they are orthogonal, and name what actually supplies unignorability.
2. Rust's `#[must_use]` is a lint, not a type rule. Name a program that ignores a `Result`
   and compiles clean, and say what a type system would have to look like to make ignoring
   genuinely impossible.
3. Exceptions are unignorable by default — silence propagates. Give the failure mode that
   *this* default creates, and name the language feature invented specifically to stop it.

### Q10 — Handling before unwinding

Common Lisp:

```lisp
(handler-bind ((parse-error (lambda (c) (invoke-restart 'use-value 0))))
  (parse-all lines))
```

The handler runs *on top of* the signalling frame. `parse-all` continues from where it
failed with the value `0`. Smalltalk has the same capability via `Exception>>resume:`.
Almost nothing else does.

1. State the implementation consequence precisely: what must be true of the stack at the
   moment the handler runs, and why that is incompatible with the standard
   search-then-unwind design's *usual* shortcut.
2. Resumption is often described as a handler-side feature. Argue instead that it is a
   property of the *raise site*, and use that to explain why bolting `resume` onto an
   existing exception system produces something nobody can use safely.
3. Name the one piece of the condition system that did get widely copied, in effectively
   every modern language, without the resumption.

### Q11 — How a catch clause matches

```csharp
try { Work(); }
catch (IOException e) when (e.HResult == 0x20) { Handle(e); }
```

```java
try { work(); }
catch (IOException | SQLException e) { handle(e); }
```

1. A catch clause matches by subtype test against an *open* hierarchy, walked in source
   order. Name the two consequences for the programmer, and contrast with matching on a
   closed sum type (OCaml's `exn` is deliberately open — say why).
2. In .NET, a `when` filter runs during the *first* pass — before any intervening `finally`
   block executes. Construct the observable consequence and explain why the two-phase design
   is what makes it possible.
3. Java has no filters. What is the standard workaround, and what does it break that the
   filter version does not?

### Q12 — What `?` hides

```rust
fn load(p: &Path) -> Result<Config, ConfigError> {
    let s = fs::read_to_string(p)?;      // io::Error  -> ConfigError
    let c = toml::from_str(&s)?;         // toml::Error -> ConfigError
    Ok(c)
}
```

1. Name the three things `?` inserts that are not visible in the source, in order.
2. One of them is an implicit `From` conversion. Give a concrete way this bites: a change
   elsewhere in the program that alters what this function does without touching this
   function.
3. `?` pushes every function toward one unified error type, which in practice means
   `Box<dyn Error>` or `anyhow`. Name what is lost when a library does that, and say when
   the loss is correct.

### Q13 — Reifying a native failure

A VM primitive is implemented in the host language. It fails — bad argument type, division
by zero, out of memory. User code must be able to `catch` it as an ordinary object.

1. Walk the steps from "the host function returns a failure" to "a user handler is running
   with an object bound." Name the two things that must already exist before the very first
   such failure can be reported, and why bootstrap makes that a real ordering problem.
2. Constructing the surface error object allocates. Name the one failure for which that is
   circular, and describe the standard fix.
3. Once native errors are matched by class, the set of classes a primitive can raise is
   part of your public API. Say what that forecloses, and give the design move that keeps it
   from ossifying.

### Q14 — Leaving without a handler

Three early exits that are not exceptions:

- A block's `return` in Smalltalk or Ruby returns from the *enclosing method*, past frames
  in between.
- `break` out of a nested loop across a closure boundary.
- An error raised inside a suspended coroutine, whose resumer is on a different stack.

1. Non-local return and exception unwinding share machinery but differ in the matching rule.
   State both rules, and say which one can fail at runtime with no possible handler.
2. Java specifies that a `return` inside `finally` discards a pending exception. Explain
   what situation that rule is adjudicating, and argue whether "discard" was the right call.
3. Lua's `coroutine.resume` returns `false, err` instead of propagating; Python's
   `generator.throw` injects an exception *into* the suspended frame; JS reports an
   unhandled promise rejection to a global hook. These are three answers to one question —
   state the question, and say what each answer assumes about ownership.

### Q15 — The edges you cannot see

The compiler builds a CFG. With exceptions enabled, a call that might throw gets a second
outgoing edge to a landing pad.

1. Name two concrete optimizations that the exceptional edge blocks or degrades, and be
   specific about *why* — it is not "the optimizer gives up."
2. `noexcept` deletes those edges. Explain why annotating a small leaf function `noexcept`
   can speed up code that never throws at all.
3. Embedded and kernel projects compile with `-fno-exceptions` and then reinvent failure
   handling with return codes. Given that "zero-cost" means free on the happy path, what are
   they actually buying — name two things, one of which is not speed.

### Q16 — The third category

```c
assert(idx < len);            // compiled out with NDEBUG
```
```rust
debug_assert!(idx < len);     // compiled out in release
if idx >= len { return Err(OutOfRange) }
```

1. Argue that a failed assertion is neither an exception nor a recoverable error, and derive
   from that argument why catching one is close to meaningless.
2. A compiler may treat an assertion as an *assumption* and optimize on it — this is the
   difference between `assert` and `__builtin_assume`/`std::unreachable`. Name the danger
   this creates, and why the C++ contracts proposals have gone back and forth on exactly
   this point.
3. Java's `assert` is disabled by default and throws a catchable `AssertionError`. Argue
   that this is wrong in *both* directions at once.

---

## Answers

### A1 — What "zero-cost" actually costs

**1.** It cost **binary size and a static data structure**: unwind descriptors in
`.eh_frame` (how to restore registers and the stack pointer for each frame) plus a
language-specific data area, `.gcc_except_table`, describing which call sites lie in which
`try` region, which cleanups apply, and which catch types are present. This is often a
double-digit percentage of a C++ binary. It also cost *compile-time* freedom — see A15.
"Zero-cost" always meant "zero instructions on the non-throwing path," and it has never
meant zero cost.

**2.** The second walk exists so that an exception with **no handler anywhere** can be
reported from the throw site with the stack intact. If you unwound as you searched, by the
time you discovered nothing would catch it, the frames are gone and the debugger shows you
`terminate` with no context. The two-phase design lets `std::terminate` run with the
original stack still live, which is why a core dump from an uncaught C++ exception is
useful. The second thing it enables is *filters* — a handler that inspects the exception
and declines can do so before anything has been destroyed (see A11). Phase one is a pure
query; phase two is the commit.

**3.** The JVM has **exact type information and a managed stack it fully controls**. Every
frame is a JVM frame with known layout and a stack map; there are no C++ destructors with
arbitrary user code to run, and there is no requirement to interoperate with a foreign
unwinder that might also want to inspect the frames. So the JVM can pop frames as it
searches: it looks up the exception table of the current method, and if no entry matches,
it pops and repeats. It cannot lose the trace by doing so because the trace was captured
at construction time (A8) — that is the piece that makes the one-pass design lose nothing.
The two designs are trading against different constraints, not one being smarter.

**Trap.** Saying the two-phase design exists "for performance." It is not faster — it walks
the stack twice. It exists for *observability at the throw point* and for filters. If you
claim performance, the natural follow-up ("so a one-pass unwinder would be slower?") has no
good answer.

### A2 — Two shapes of failure at the call site

**1.** `Result` adds an ordinary conditional branch to the caller's CFG: the edge is
explicit, the optimizer sees it like any other `if`, it can be inlined through, branch
prediction handles it, and the failure path is just a cold block. Exceptions add an
*implicit* edge from the call to a landing pad, which does not appear in the source at all
— every potentially-throwing call becomes a two-successor node. Error codes add nothing to
the CFG at the call itself, only whatever the caller writes.

The exception form is what constrains code motion: a store or a call cannot be sunk past a
possibly-throwing call, because if it throws, the landing pad and everything downstream of
it must observe the state that the source order specifies. Values live at the call must be
recoverable in the landing pad, which pushes them out of registers or forces spills. See
A15.

**2.** Costs beyond bytes: (a) **it is contagious through the type system** — every caller's
signature changes, and generic code has to be parametric over `E` or you get a conversion
at every boundary, which is exactly why `?` needs `From` (A12); (b) **it defeats
sizing intuitions in aggregate types** — a `Result<(), BigError>` returned from a hot inner
function is moved by value on every call, and a fat error type slows the success path, which
is why the advice is to box large errors. What it buys that an exception cannot: an error is
an ordinary value, so it can be **stored, collected, returned from a closure, sent across a
channel, retried against, and matched exhaustively**. An in-flight exception is not a value
you can put in a `Vec` while continuing; you have to catch it to make it one.

**3.** `errno` is only valid to read *immediately* after a failed call, and "immediately"
is not enforceable. Any intervening call — including a logging call, an allocation, a
destructor, or a signal handler on some platforms — may overwrite it. The classic bug:

```c
if (fclose(f) != 0) { log("closing %s", name); perror("fclose"); }
```

`log` may itself call something that sets `errno`, so `perror` reports the wrong failure.
Also, `errno` is only meaningful if you first determined *that* a call failed by some
per-function convention (`NULL`, `-1`, `EOF`, and functions where the valid return range
includes the sentinel, so you must clear `errno` first and check it after). Value-returning
designs are immune because the error and the fact-of-failure are the same object, produced
at the failure and carried by ordinary data flow — nothing can clobber it, and there is no
window.

**Trap.** Claiming `Result` is "zero-cost" because the branch is predictable. The
discriminant test is cheap, but the *return-by-value* cost is real and it is on the happy
path — the exact opposite of exceptions, which are free on the happy path and expensive on
the sad one. The honest framing is that the two designs move the cost to opposite sides,
and which is right depends on whether your errors are rare or routine.

### A3 — What checked exceptions got right

What it got right: **a function's failure modes are part of its interface**, and burying
them in prose documentation is a design error. That claim has aged extremely well; Rust,
Zig, Swift and Go's conventions all agree with it.

**1.** The difference is that Rust's error is a **first-class value in an ordinary type**,
so all the abstraction machinery the language already has applies to it: it can be a generic
parameter, an associated type, a trait object, converted, stored, matched. Java's `throws`
is a **separate, second-class annotation channel** that no other feature can abstract over.
You cannot write a Java generic that is parametric over its throws clause (the
`<E extends Exception>` trick is a late, awkward retrofit and does not compose). So the
moment you build any abstraction, the annotation cannot follow it, and you have to erase it.
Same information; one is in the type system and one is beside it.

**2.** `Stream.map(Function<T,R>)` — `Function.apply` does not declare `throws`, so a lambda
whose body throws `IOException` does not conform. There is no way for `map` to say "I throw
whatever my argument throws," because `throws` cannot be a variable. The whole
`java.util.function` package therefore had to be defined non-throwing, which shoved every
checked exception in every stream pipeline into a wrapping lambda. Swift's `rethrows` is
precisely the missing quantifier: `func map<R>(_ f: (T) throws -> R) rethrows -> [R]` means
"`map` throws if and only if `f` does," letting the checkedness pass through a higher-order
function. That one keyword is the fix Java never got.

**3.** Forced move. Given (2), any code path that goes through an interface you do not
control — a framework callback, a `Runnable`, a stream, a constructor of a class whose
supertype does not declare it — has no legal way to propagate a checked exception. Wrapping
is not laziness; it is the only expressible option. For checked failure to work, a language
needs at minimum: (a) effect polymorphism, so a higher-order function can be generic over
its callee's failures (`rethrows`, or an effect system, or `E` as a type parameter);
(b) a cheap way to *widen* one error type into another, since a caller aggregating three
libraries needs one type — Rust's `From` + `?`, Zig's error-set union and inference;
(c) inference, so you are not writing the set by hand. Zig's inferred error sets are the
cleanest existing demonstration that the idea was sound and the encoding was the problem.

### A4 — Four takes on "run this on the way out"

**1.** In failure-handling order, worst to best:

- **C++**: destructors are implicitly `noexcept` since C++11, so a destructor that throws
  during unwinding calls `std::terminate`. The program dies. This is deliberate: two
  exceptions propagating simultaneously has no defined meaning — which one does the next
  handler see? — so the standard refuses to pick and kills the process instead.
- **Java**: an exception thrown in `finally` **replaces and discards** the in-flight
  exception. The original is silently lost, with no link. This is the worst outcome of the
  four, because it destroys the diagnostic that mattered (the first failure) in favour of
  the one that is usually a consequence of it. `try`-with-resources was added partly to fix
  this, and it attaches the close-failure as a *suppressed* exception on the original —
  which is the right answer, arriving a decade late.
- **Python**: raising from `__exit__` or `finally` also replaces the propagating exception,
  but the original is retained as `__context__` and printed ("During handling of the above
  exception, another exception occurred"). Chain preserved, priority given to the new one.
- **Go**: a panic in a deferred function replaces the panicking value, and `recover` sees
  the last one; the chain is not automatic but `defer`s all still run.
- **Rust**: a panic in a `Drop` during unwinding aborts the process — the C++ answer, for
  the C++ reason.

**2.** Function-scoped `defer` inside a loop accumulates: `for f in files { defer close(f) }`
in Go registers N closures that all run at function exit, so you hold N descriptors open —
a classic Go leak, and the reason the idiom is to wrap the body in a function literal.
Block-scoped `defer` (Zig, Swift) runs at the end of each iteration. Function-scoped is
*harder* to implement, not easier: it needs a runtime list of pending deferred calls with
captured arguments, walked at return and during panic — Go maintains exactly that structure
per goroutine. Block scope is a static, lexical transformation the compiler can lower into
the same landing pads it already emits.

**3.** RAII is a property of a **type**, so cleanup is attached at the point of *definition*
rather than at every point of use. The other three cannot guarantee that a resource is ever
cleaned up at all, because they depend on the caller remembering to write the statement.
That is the entire argument: `finally`, `ensure`, and `defer` are opt-in per use site and
therefore fail open; RAII is opt-in per type and therefore fails closed. The cost is that
RAII requires deterministic destruction, which requires either single ownership with
compile-time lifetimes (C++, Rust) or refcounting — it is unavailable in a tracing-GC
language, which is exactly why Java, C#, Go, and Python all ended up with a
statement-shaped answer plus a convention (`Closeable`, `IDisposable`, `io.Closer`,
`__exit__`).

**Trap.** Saying `defer` and `finally` are "the same thing with different syntax." The
scoping difference is observable (part 2) and the *ordering* differs — Go's defers run LIFO
across the whole function and can modify named return values, which `finally` cannot do in
a comparable way. And a strong candidate will often forget that `finally` interacts with
`return` (A14), which `defer` handles by a completely different rule.

### A5 — The strong guarantee's copy

**1.** Reallocation must produce a state where either the vector holds all `n` elements in
a new buffer, or the vector is untouched. If moving element `k` throws, the first `k`
elements have been *destructively removed* from the old buffer — they are in a moved-from
state — so there is no way back to "untouched." Copying leaves the old buffer intact and
valid the whole time; if a copy throws, you destroy the partial new buffer, free it, and
the vector is exactly as it was. The strong guarantee costs a copy because rollback requires
the source to survive the operation.

**2.** `move_if_noexcept` moves only when the move constructor is `noexcept` (or when no
copy constructor exists, at which point you have no choice and the guarantee degrades to
basic). It keys on the *annotation*, not on any analysis, because the standard library is
compiled against your type through a template and can only see what your declaration says.
Absence of `noexcept` is indistinguishable, to the library, from "this move allocates and
may throw." The lesson generalizes hard: **a safety guarantee that must hold across a
library boundary can only be expressed by something in the interface.** The library cannot
infer it, cannot see your body, and must assume the pessimistic case. This is the same
argument as A3's — the property has to be in the type or it does not exist.

**3.** `nothrow` is not a stronger promise of the same kind; it is the primitive the others
are built out of. The canonical strong-guarantee implementation is copy-and-swap: do all
the fallible work into a temporary, then commit with `swap`. The commit step *must* be
nothrow, because there is no rollback for a failed commit — if `swap` throws halfway, you
have neither the old state nor the new one. Every strong-guarantee operation bottoms out in
some nothrow commit: a pointer assignment, a swap, an atomic store. So the ladder is
really: nothrow operations exist → therefore commits exist → therefore rollback-based
strong guarantees can be built. This is also why the standard requires `swap` to be
nothrow for library types, and why `std::vector::swap` is just three pointer exchanges.

**Trap.** Saying "just mark everything `noexcept`." A `noexcept` that lies calls
`std::terminate` (A7), so the annotation is a *hard* promise with a fatal penalty, not a
hint. And marking a function `noexcept` that legitimately allocates is a correctness bug
that shows up as a process death under memory pressure — the worst possible time.

### A6 — Panic is not an error

**1.** The usable criterion: **is the failure part of the function's specified domain, or
does it mean a precondition was violated?** A missing file is an anticipated outcome of
asking the filesystem for a file — the environment is allowed to be that way, and the caller
was right to call. An out-of-bounds index means the caller passed something the function's
contract forbids: the *program* is wrong, not the world. Restated for a library author: if
the caller could not have prevented it by writing correct code, it is an error value; if
the caller could have, it is a bug and should panic. Corollary — the same underlying
condition can be either, depending on the API: `v[i]` panics, `v.get(i)` returns `Option`,
and that is the *interface* deciding whose fault it is.

**2.** Deliberate and correct because putting panics in signatures reproduces checked
exceptions: every function can overflow, allocate, or be indexed, so every signature would
carry a `panics` marker, the marker would carry zero information, and it could not be
abstracted over in higher-order code (A3). The cost is that you cannot statically know a
region of code is panic-free, which matters for real-time and kernel work — hence external
tools (`no_panic`-style link tricks, panic-freedom analyses) rather than a language feature.
`catch_unwind` is for **isolating a failure domain**: a thread pool that must not die when
one task panics, an FFI boundary that must not unwind (A7), a test harness. It is
explicitly not a general error mechanism: it does not catch aborts, the caught payload is
`Box<dyn Any>` with no useful structure, and the code you interrupted may have left data in
a logically inconsistent (though memory-safe) state — which is what `UnwindSafe` is
gesturing at.

**3.** With `panic = "abort"`: `catch_unwind` stops working, so per-task and per-thread
isolation is gone; a test harness cannot report "this test panicked" and continue; and
destructors on the unwind path never run, so anything relying on `Drop` for cleanup at
failure (releasing a lock, flushing, unregistering) is skipped. A **server** wants unwinding
because one bad request must not kill a process serving thousands of others. An **embedded
target** wants abort because the unwind tables can be a large fraction of a small binary,
the personality machinery pulls in formatting and allocation, and there is nobody to recover
*to* — there is one task and if its invariants are broken the correct response is to reset
the device.

**Trap.** "Rust panics are just exceptions with a different name." Two differences that
matter: they are not part of the type system by design, and the language explicitly refuses
to support them as a control-flow mechanism — you cannot match on the payload usefully, the
unwinder may be compiled out entirely, and the documented intent is process- or
task-abandonment. A design that panics to signal an expected condition is misusing the
mechanism, and the giveaway is `catch_unwind` appearing anywhere other than a boundary.

### A7 — Unwinding into code that never agreed

**1.** (a) **The C frames may carry no usable unwind actions.** Be careful how you state
this: on x86-64 and aarch64 you usually *do* get `.eh_frame`, because those targets default
to `-fasynchronous-unwind-tables` — what `-fexceptions` adds is the personality routine and
the LSDA (`.gcc_except_table`), i.e. the cleanup and catch actions, of which C has none.
Nothing in the C ABI *promises* the unwinder can restore the frame, so you are relying on a
platform accident rather than a guarantee, and (b) below is the stronger reason anyway. (b) **`qsort`'s
own invariants are abandoned.** `qsort` is in the middle of a partition: it may hold a
scratch buffer it allocated, and the array it is sorting is in an intermediate permutation.
Unwinding past it frees nothing and repairs nothing — you leak the buffer and hand back a
partially permuted array, and no amount of unwind metadata would fix that, because the
cleanup logic simply does not exist in C. (b) is why the answer cannot be "just compile the
C with `-fexceptions`."

**2.** Because **unwinding through foreign frames is a real, load-bearing pattern when both
sides agree**. C++ code calling a C callback that calls back into C++ and throws is
longstanding practice with GCC/Clang, where the C is compiled with `-fexceptions` and the
intermediate frames are cleanup-only. Rust needed the same thing for interop: a Rust panic
crossing a C shim into a Rust handler, and a C++ exception crossing Rust frames. A blanket
abort would have made those impossible. `extern "C-unwind"` says "this boundary is
unwind-transparent, and I have taken responsibility for what is in between" — an *opt-in*
where the default is safe. That default direction is the whole design: the dangerous thing
must be spelled.

**3.** Because `noexcept` is not advice — it is a promise the *caller's codegen relies on*.
The compiler deletes the exceptional edges out of a `noexcept` call (A15): no landing pad
was emitted, no cleanup was recorded, the caller may have kept values in registers that a
handler would need. "Just let it through" would unwind into a frame with no unwind actions
for state that requires them, which is silent corruption. Terminating converts an
unrecoverable inconsistency into an immediate, diagnosable stop. Same reasoning as C++'s
throwing-destructor rule and Rust's double-panic abort: when the machine reaches a state the
model does not define, stop rather than continue.

### A8 — Who pays for the stack trace

**1.** Because the trace must be captured while the frames are still live, and Java's
`throw` is a separate statement from `new` — by the time you throw, nothing has unwound yet,
but the language cannot know a given exception object will be thrown from where it was made,
so it captures at construction, in `Throwable`'s constructor. The cost is proportional to
stack depth and is paid *even if the exception is never thrown*. This makes exception
construction, not throwing, the expensive operation — which is why the "exceptions as
control flow" pattern in Java requires overriding `fillInStackTrace` to a no-op or reusing
a static instance, and why frameworks that signal with exceptions (early-exit sentinels in
parsers, JDBC drivers, Hibernate's control flow) all have such a class. Deep stacks in
enterprise frameworks are exactly where this hurts most.

**2.** The JIT compiles an implicit exception site — a null dereference or an array bounds
check — into an implicit trap. Once such a site has thrown many times, HotSpot concludes it
is being used as control flow and swaps in a preallocated, shared exception instance with no
stack trace, avoiding both the allocation and the walk
(`-XX:-OmitStackTraceInFastThrow` disables it). It is notorious because the behaviour is
*load-dependent*: the exception has a full trace in your tests and in the first minutes of
production, and then, once the site is hot, your logs fill with a bare
`java.lang.NullPointerException` and no frames at all — precisely when you most need them.
It is a beautiful illustration of a JIT optimizing a semantic the language never promised
was cheap.

**3.** With `throw`, a runtime *walks the stack* and can record it, because there is a
moment where an unwinder is holding every frame. With `?`, each propagation is an ordinary
`return` — the frame is destroyed by the normal epilogue, no runtime is involved, and there
is no point at which anything sees more than one frame. To get a trace you must capture it
explicitly at construction (`std::backtrace`, `anyhow`'s captured backtrace) at real cost,
or attach context manually at each level, which is what `.context(...)` and Go's
`fmt.Errorf("...: %w", err)` are doing — building a *causal* chain by hand rather than a
*stack* trace. Zig splits the difference: **error return traces** record the chain of
`return`-with-error sites as the error propagates, giving you the path the error took
rather than the stack at its origin. That is often the more useful artifact, and it is
cheap because it appends one address per propagation instead of walking anything.

**Trap.** "Rust could just add backtraces to `Result`." The problem is not effort, it is
that there is no single place to hook: an error value can be constructed, stored, moved,
returned from a closure, or synthesized far from where anything went wrong, so "capture at
construction" is both expensive and frequently captures the wrong stack. The cheap trace is
a property of having a runtime unwinder, and value-based errors deliberately do not have one.

### A9 — Values you can ignore

**1.** They are orthogonal because unignorability comes from **the type system's treatment
of the result**, not from how the failure is represented. Go's `error` is a value and is
ignorable (`_ =`, or just not assigning it, or a bare `f.Close()`). Rust's `Result` is a
value and is nearly-unignorable. C's `int` return is a value and is completely ignorable. In
the other direction, an exception is not a value and *is* unignorable. So the axis is
"errors as values vs. errors as control flow," and orthogonally, "does discarding a result
require an explicit act." The second axis is the one people actually care about, and it is
supplied by must-use rules, linear/affine types, or effect checking.

**2.** Any use that isn't a bare discard defeats it:

```rust
fn f() -> Result<(), E> { g(); Ok(()) }   // if g() returns Result, warns
let _ = g();                              // no warning, explicit discard — fine
if let Ok(_) = g() {}                     // no warning, and silently ignores Err
match g() { _ => () }                     // no warning
```

`#[must_use]` fires on *statement-position discard of an unused value*, so putting the value
anywhere — a wildcard match arm, a tuple, a `let _` — silences it, and the lint can be
downgraded globally. To make ignoring impossible you need the `Result` to be an **affine or
linear type that cannot be dropped without being consumed** by an explicit destructor —
Rust's `Drop` is precisely the escape hatch that makes this impossible today, since every
value can always be dropped. A language with linear types (or Haskell's `Linear`, or an
explicit `#[cannot_drop]`) could make it a type error. Nobody does this for errors, because
the ergonomic cost of a truly linear type is high and the lint catches the common case.

**3.** Exceptions fail by **being caught too broadly**: `catch (Exception e) { log(e); }`,
a bare `except:` in Python, `rescue => e` in Ruby. Silence propagates, so a handler placed
for one error swallows every error including ones it has no idea about — and the code
continues past a violated invariant. The feature invented to stop it is the **distinguished
non-`Exception` root**: Python's `BaseException`, so that `KeyboardInterrupt`,
`SystemExit`, and `CancelledError` are *not* caught by `except Exception`. Java's
`Error`/`Exception` split is the same idea for `OutOfMemoryError` and friends. Both exist
because the default "catch everything" was in practice a bug factory, and the fix was to
carve out the exceptions that must not be caught by accident — which is an admission that
unignorable-by-default has its own, symmetric failure.

### A10 — Handling before unwinding

**1.** At the moment the handler runs, **the signalling frame and every frame between it and
the handler must still be live and mounted**, because the handler may choose to resume, and
resumption means continuing execution inside that frame at the point after the signal. The
usual design's shortcut is to unwind first — pop frames, run cleanups, then execute the
`catch` body with everything below already gone. That shortcut is exactly what makes
resumption impossible. Note the connection to A1: the two-phase Itanium design *does* run a
query with the stack intact, so it has the necessary shape; what it lacks is the ability to
run arbitrary user code in that phase and then *return normally into the signaller*.
Condition systems are two-phase unwinding where phase one can decline to become phase two.

**2.** Because resuming means the signalling code continues with a value supplied from
outside, and only the signalling code knows *what a legal value at that point is* and *what
its invariants require*. `(signal 'parse-error)` with a `use-value` restart is a contract:
the author of `parse-all` wrote a restart because they designed a resumption point with a
defined meaning. A handler cannot invent that. So `resume` bolted onto an existing system
gives you a mechanism to push an arbitrary value back into code that was never written to
receive one — the raise site is typically `throw new IllegalStateException(...)` in the
middle of a half-updated structure, and resuming there resumes into a broken invariant.
Common Lisp is safe because the restarts are declared *by the signaller*, named, and
enumerable; the handler picks from an offered menu, it does not force a value in. Any
language wanting resumption must make it a raise-site feature, which means auditing every
raise site — that is the real reason nobody copied it.

**3.** `unwind-protect` — it became `finally`, `ensure`, `defer`, and `__exit__`, and it is
genuinely in effectively every modern language. Do not claim that status for the condition
system's *other* separable piece, running handler-selection before unwinding: that was barely
copied at all, and .NET's `catch ... when` filters (with SEH's two-phase model beneath) are
the one mainstream survivor. Java, C++, Python, Ruby, JS, Go, and Swift all unwind first.
Also widely copied,
if you squint: the idea that a condition might not be an error at all (warnings, and
`with_handler`-style dynamic hooks — Python's `warnings` module and its filters are
structurally a condition system for the non-fatal case). And in a different lineage,
**algebraic effect handlers** are the condition system's resumption idea rebuilt with types
(see the delimited-continuation material in file 07): a handler that can resume is exactly
`shift`/`reset` with a named effect, and OCaml 5 and Koka are the first mainstream systems to
make it sound by making the resumption a first-class, typed value rather than an implicit
"go back."

**Trap.** Describing restarts as "exceptions you can resume." The distinctive part is that
`handler-bind` runs a handler that *may decline* — it can return normally, at which point
the next handler is tried, and if none handles it you fall into the debugger with the whole
stack alive. That declining behaviour, not resumption, is the primitive; resumption is one
thing a non-declining handler can do.

### A11 — How a catch clause matches

**1.** (a) **Order matters and is a silent hazard**: a `catch (Exception)` above a
`catch (IOException)` shadows it. Java makes an unreachable clause a compile error, which
works only because it can see the whole hierarchy; a language with dynamic class creation
cannot. (b) **You can never be exhaustive**, because the hierarchy is open — a library can
add a new subclass in a point release and your `catch (SqlException)` silently stops
covering a case, or starts covering a new one you have never seen. Matching a closed sum
type gives you exhaustiveness checking and therefore a compile error when the set grows,
which is a large part of why `Result<T, MyError>` with an enum `MyError` feels safer.

OCaml's `exn` is deliberately open (an extensible variant) because exceptions must be
declarable in any module without a central registry — closing it would mean the entire
program's failure modes are declared in one place, which does not survive separate
compilation or libraries. So the openness is not sloppiness; it is the price of modularity,
and closed error enums pay for their exhaustiveness by being per-function, not global.

**2.** Observable consequence:

```csharp
try {
  try { throw new IOException(); }
  finally { Console.WriteLine("finally"); }
} catch (IOException e) when (Log("filter")) { Console.WriteLine("catch"); }
```

prints **filter, finally, catch** — the filter runs before the inner `finally`. Two-phase
unwinding is what makes this possible: the search phase asks each frame's personality
routine whether it will handle, and a filter is user code executed during that query, with
the stack below still intact. This is genuinely useful — a filter that captures diagnostic
state sees the stack as it was at the throw, not after cleanup — and genuinely surprising,
because it violates the naive mental model that `finally` runs "on the way out" before
anything outside sees the exception. It also means a filter must not have side effects it
cannot tolerate running for exceptions it declines.

**3.** The workaround is to catch, test, and rethrow:

```java
catch (IOException e) { if (e.getMessage() == null) throw e; handle(e); }
```

This **unwinds first and then re-raises**, which breaks two things: the stack below the
catch is already gone, so any state you wanted to inspect at the throw point is destroyed
and the trace of the rethrow may differ; and every `finally` between the throw and this
frame has already run, so you decline *after* cleanup rather than before. In .NET terms it
is the difference between "let it keep going" and "stop, then start a new one." That is
also why `throw;` versus `throw e;` matters in C# and why Java's rethrow has never had a
clean equivalent.

### A12 — What `?` hides

**1.** In order: (a) a **branch** on the discriminant; (b) on the error path, a call to
`From::from` converting the callee's error type into this function's error type; (c) an
**early `return`**, which runs every pending `Drop` in scope. There is a fourth, subtler
one: the operator is defined via the `Try` trait, so it also works on `Option` and any
`Try` type, meaning the same glyph means different things depending on the return type.

**2.** Adding a `From<X> for ConfigError` impl elsewhere in the crate can make a previously
non-compiling `?` compile — fine. The bite is the reverse and the ambiguous cases: if
`ConfigError` gains a *second* `From` impl reachable by inference from the same source type
through different paths, or if someone changes `From<io::Error> for ConfigError` to attach
different context or to map to a different variant, then **this function's observable error
changes with no edit to this file and no type error anywhere**. Downstream `match`es on
`ConfigError` variants start taking different arms. The conversion is a globally-defined,
implicitly-invoked coercion, which is exactly the category of thing that makes local reading
unreliable — the same complaint people make about implicit conversions in C++ and implicit
conversions in Scala, arriving here through a door nobody watches.

**3.** You lose **exhaustive matching and programmatic recovery**. A caller of a
`Box<dyn Error>`-returning function cannot enumerate the failures, so it cannot write
"retry on timeout, fail on auth error, ignore not-found" without downcasting by concrete
type — which reintroduces the open-hierarchy problem of A11 with worse ergonomics. The loss
is correct in **applications**, where the only consumers are a log line and an exit code, and
where a rich error enum is pure ceremony. It is wrong in **libraries**, where you do not know
what your caller needs to distinguish. That is the actual `thiserror`-versus-`anyhow` rule,
and it is a statement about who the consumer is, not about taste.

**Trap.** "`?` is just sugar for a match." The `From` conversion is not sugar — it is a
type-directed, globally-resolved call, and it is the reason `?` sometimes fails to compile
with an error about an unsatisfied trait bound that mentions neither the operator nor the
line's apparent types. If your explanation of `?` does not include the conversion, you
cannot explain that error message.

### A13 — Reifying a native failure

**1.** The host function returns a failure sentinel (a `Result`, an out-param, a tagged
enum) to the interpreter loop. The loop must then: allocate an instance of the appropriate
surface error class; populate it with a message and, ideally, the current source location,
which requires mapping the current instruction pointer back to a line via a side table;
push it as the in-flight exception; then walk the frame stack looking for a handler,
running each frame's cleanup blocks. Two things must exist first: **the error class itself
must be registered and instantiable**, and **the mechanism that maps a raise to a handler
must be initialized**. Bootstrap makes this an ordering problem because the core class
hierarchy is typically defined by running code in the language itself, and that code can
fail — so you have a window in which the machinery for reporting failure is not yet built.
The standard resolution is to define a minimal, natively-constructed error class before
executing any surface code, and to treat any failure before that point as a fatal host-level
abort with a distinct message. The alternative — trying to report bootstrap failures through
the surface mechanism — produces the worst debugging experience in any runtime, where the
error about the missing class is itself unreportable.

**2.** **Out of memory.** Allocating an error object to report that allocation failed is
circular, and under real memory pressure it fails again. The standard fix is to
**preallocate** the OOM error (and often the stack-overflow error) during startup, when
memory is available, and hand out that singleton — which is what the JVM does with
`OutOfMemoryError` instances. The consequences are that the object is shared, so it cannot
carry a per-site message or a freshly captured stack trace (see A8's part 2 for the same
compromise arrived at for a different reason), and that it must not be mutable by user code.
Stack overflow has the same shape with an extra twist: you need enough remaining stack to
run the handler, hence guard pages and reserved "red zone" stack in runtimes that report it
as a catchable condition.

**3.** It forecloses **changing which class a primitive raises**, since a user's
`catch (TypeError)` becomes a silent no-catch if you retag it as `ArgumentError`, and adding
a *narrower* subclass is safe while widening is not. The move that keeps it flexible is to
**catch on a stable, small set of public root classes and put the detail in structured
fields**, not in the class: one `TypeError` with a `expected`/`got` pair beats twelve
sibling classes. Second move: reserve a documented internal namespace for classes that are
explicitly not part of the catch contract, so you can raise a private subclass of a public
root — the subclass is free to change because nobody was promised it. This is the same
discipline as versioning an error enum with a `#[non_exhaustive]` marker: keep the matchable
surface deliberately coarse.

### A14 — Leaving without a handler

**1.** Exception unwinding matches by **the value's type against each frame's handler
table** — a search over a predicate. Non-local return matches by **frame identity**: the
block carries a reference to its defining method's activation, and unwinding proceeds until
*that specific frame* is on top. Both run the same cleanup actions on the way. The
identity rule is the one that can fail with no possible handler: if the home method has
already returned, the target frame no longer exists, and you get Smalltalk's
`BlockCannotReturn` / Ruby's `LocalJumpError`. A type-matched exception can always fall
through to a top-level default; an identity-matched return has exactly one legal
destination and it can be gone. This is why every language with non-local return needs a
runtime check and a dedicated error for a dead home frame — it cannot be prevented
statically once blocks are first-class values.

**2.** It is adjudicating **two simultaneous non-local exits with different destinations**:
an exception heading for some outer handler, and a `return` heading for this method's
caller. They cannot both happen. Java picks the `return` — the innermost, most recently
initiated transfer wins — and the exception is discarded entirely, not even chained. The
call was wrong, and the evidence is that every static analyzer flags `return` in `finally`
and no style guide permits it. "Discard" throws away the earlier, causally prior failure in
favour of a control-flow statement whose author almost certainly did not know an exception
was in flight. Python makes the same choice and discards just as silently — its `__context__`
chaining fires only when the `finally` *raises*, never when it returns — though it has since
moved toward C#'s position: PEP 765 makes `return`/`break`/`continue` leaving a `finally` a
`SyntaxWarning` in 3.14. C# forbids it outright, which is
the defensible answer: if two exits conflict, make it a compile error rather than picking a
winner silently.

**3.** The question is: **when a computation fails and its stack is not the stack of whoever
is currently running, who owns the failure?** Lua's `resume` assumes the *resumer* owns it
and converts the failure into an ordinary return value, so the coroutine's stack is
discarded and the caller must check — safe, explicit, and it makes error handling across a
resume look nothing like error handling elsewhere (hence `coroutine.wrap`, which re-raises,
as the ergonomic alternative). Python's `throw` assumes the *coroutine* owns it: the
exception is injected at the suspension point so the coroutine's own `finally` blocks and
handlers get a chance, which is the only way `with` statements inside generators can be
correct. JS's unhandled-rejection hook assumes **nobody** owns it yet — a promise's failure
may be handled later, so the runtime has to wait until the microtask queue drains to decide
it will never be, and even then can only report it to a global handler. That third answer is
forced by detached tasks: with no structural parent, there is no frame to propagate to. It
is also why structured concurrency and `ExceptionGroup`/`except*` exist — once a scope owns
a set of children, multiple concurrent failures have a defined place to land, and the
language needs a way to represent *several* in-flight errors at once, which single-value
exception propagation cannot express.

**Trap.** Treating "throw into a coroutine" as symmetric with "throw out of one." Injecting
runs the coroutine's cleanup on its own stack and is well-defined; propagating outward means
grafting an exception onto a stack that has no causal relationship to the failure, which is
why most systems refuse and convert it to a value instead.

### A15 — The edges you cannot see

**1.** (a) **Register allocation and spilling**: a value live across a possibly-throwing
call must be reconstructible in the landing pad, so it must be in a callee-saved register or
spilled to the stack — the exceptional edge extends live ranges and increases stack traffic
on the *normal* path. (b) **Code motion and CSE across the call**: a store cannot be sunk
past a possibly-throwing call, and a load cannot be hoisted above it, because the landing pad
is a real successor that must observe the source-order state; likewise a redundant
computation cannot be eliminated across the edge if the two paths reach it differently.
Related: the extra edges make many blocks non-single-successor, which blocks tail duplication
and complicates loop transformations when a call sits inside a loop body. None of this is
"the optimizer gives up" — it is the optimizer correctly respecting a control-flow edge you
did not write.

**2.** Because the caller's landing pads, spills, and cleanup records exist for *the
possibility* of a throw, not for actual throws. Marking a leaf function `noexcept` (or, in
Rust terms, having the compiler prove a call cannot unwind) removes its exceptional edge
from every caller, which shortens live ranges, removes cleanup entries from the caller's
LSDA, may make a caller's whole `try` region cleanup-free, and can allow the caller itself
to become non-throwing — which then propagates the same win one level up. The benefit is
transitive and it lands entirely on code that never throws. This is the second, less-cited
reason to mark move constructors `noexcept`, alongside A5's `move_if_noexcept`.

**3.** They buy: (a) **binary size** — `.eh_frame` and `.gcc_except_table` deleted, plus the
personality routine and everything it pulls in from the runtime, which in a freestanding
build can be the difference between fitting in flash and not; (b) **predictability**, which
is the non-speed one — a throw has unbounded, data-dependent latency (table lookup, frame
walking, possibly a lock in the unwinder's frame-descriptor lookup on some platforms), and a
hard-real-time or interrupt-context codebase cannot have an operation whose worst case is
unknown. That second reason is why the rule survives even where binary size is not tight,
and it is the honest answer to "but exceptions are free when you don't throw" — they are
free until you throw, and some code cannot afford the day it does.

### A16 — The third category

**1.** An exception says "the world was not as hoped"; a recoverable error says "this input
or environment is out of range, here is that fact as data." A failed assertion says
**"my model of the program is false"** — the code that would run next was written under an
assumption now known to be wrong. Catching it is close to meaningless because the handler is
part of the same program whose model just proved unreliable: whatever cleanup or fallback it
performs is itself written against assumptions you now have no reason to trust. There is also
nothing to *do* — the caller did not cause it and cannot avoid it, since a violated internal
invariant is not a function of the arguments. Hence `abort` rather than `throw`, and hence
Rust's panic-vs-`Result` line landing in the same place from the other direction (A6).

**2.** The danger: if the compiler is permitted to *assume* the condition, then a false
assertion is **undefined behaviour rather than a diagnostic**. `assert(p != NULL)` compiled
out under `NDEBUG` merely stops checking; `__builtin_assume(p != NULL)` tells the optimizer
to delete the null check you wrote three lines later, so a violation produces silent
miscompilation exactly where you were trying to add safety. This is why the C++ contracts
work went back and forth for a decade on whether a contract may be assumed in a build mode
that does not check it: the "assume" semantics give real optimization wins and turn every
wrong contract into a security bug, which is what got contracts pulled from C++20 in 2019.
Know the landing point — the version adopted for C++26 resolved it by refusing the assume
semantic entirely (`ignore`, `observe`, `enforce`, `quick_enforce`, with `ignore` granting
the optimizer nothing). The committee shipped it only once its failure mode stopped being UB. The general principle worth stating: a check
and an assumption look identical in source and are opposites in effect — one adds behaviour
on violation, the other removes the ability to have any.

**3.** Wrong to be **off by default**: an assertion nobody runs is a comment, and the whole
value of assertions is finding model violations in the environments you did not anticipate,
which is production. Requiring `-ea` guarantees that the deployment where the assertion would
have earned its keep is the one where it is inert. And wrong to be a **catchable
`AssertionError`**: it is an `Error`, so convention says do not catch it, but nothing
enforces that, and a broad `catch (Throwable)` in a framework's task runner will swallow it
— converting "my model is false, stop" into "log and continue," which is the precise outcome
the category exists to prevent. Java managed to make assertions both too weak to fire and too
weak to stop the program.

**Trap.** Claiming assertions are "for debugging, errors are for production." Real systems
run assertions in production deliberately — the argument is not "checks cost too much," it is
"what do you do on failure," and for a distributed system the answer is often "crash this
replica loudly," which is an availability strategy, not a debugging one. Presenting the
categories as a dev/prod split misses that the distinction is about *whose fault it is*, and
that is a property of the condition, not of the build.
