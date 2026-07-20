# 12 — Metaprogramming and the Open World

Programs that reshape themselves while running. The through-line: *every construct that
lets a program rewrite itself is a promise the optimizer is no longer allowed to make.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Hygiene has two directions

A `swap!` macro, written naively:

```scheme
(define-macro (swap! a b)
  `(let ((tmp ,a))
     (set! ,a ,b)
     (set! ,b tmp)))
```

Two separate things go wrong:

```scheme
(swap! x tmp)          ; case A
(let ((list 5)) ...)   ; case B — for a macro whose template calls `list`
```

1. State precisely what breaks in case A and what breaks in case B. They are not the same
   failure, and one of them is not fixed by `gensym`.
2. `gensym` is the classic Lisp discipline. Say exactly which half of the problem it
   solves and why it is structurally incapable of solving the other half.
3. A hygienic expander solves both. Name the two distinct guarantees it must provide, and
   say which one is about *binders* and which is about *free references*.

### Q2 — The macro ladder, rung by rung

```c
#define SQUARE(x) x * x
#define MAX(a, b) ((a) > (b) ? (a) : (b))
```

versus a Lisp `defmacro` operating on cons cells, versus Scheme `syntax-rules`.

1. `SQUARE(1 + 2)` and `MAX(i++, j)` each fail, for *different* reasons. Name both
   failure classes, and say which one parenthesisation fixes and which one it cannot.
2. Moving from text to s-expressions eliminates a whole failure class outright. Which one,
   and what does it *not* eliminate?
3. Moving from `defmacro` to `syntax-rules` eliminates another. What did `syntax-rules`
   give up to get it, and name a macro you provably cannot write in it.

### Q3 — Why `syntax-case` exists

`syntax-rules` is a pattern language: patterns in, templates out, hygiene automatic.
`syntax-case` lets the right-hand side be arbitrary Scheme running at expansion time, with
`datum->syntax` available to *deliberately* break hygiene.

1. Give a macro that fundamentally requires computation at expansion time — not just
   convenience — and say what in `syntax-rules` blocks it.
2. `datum->syntax` takes an existing syntax object as its first argument, not just a
   symbol. Why does it need that argument at all? Answer in terms of what an identifier
   actually *is* in a hygienic expander.
3. Anaphoric macros (an `it` the user did not write but can reference) are the canonical
   intentional-capture case. Explain why a well-designed system makes this *possible but
   loud*, rather than banning it.

### Q4 — Implementing hygiene

Two real implementation strategies for hygiene: Kohlbecker's "rename everything, then
selectively unrename," and the marks-and-substitutions approach (Dybvig et al.), later
generalised to Racket's **scope sets**.

1. In a scope-sets model, an identifier is not a symbol — it is a symbol plus a set. What
   is in the set, and what is the *resolution rule* that turns an identifier into a
   binding?
2. Explain why that single rule subsumes both directions of Q1, rather than needing two
   mechanisms.
3. What does hygiene cost you at the tooling level — specifically, what does "go to
   definition" have to do that it would not have to do in an unhygienic system?

### Q5 — Rust's partial hygiene

```rust
macro_rules! make {
    () => {
        let x = 5;          // does NOT collide with caller's `x`
        struct Helper;      // DOES land in the caller's namespace
        println!("{}", helper_fn());  // resolved where?
    };
}
```

`macro_rules!` is hygienic for local variables and loop labels, and *not* hygienic for
items, types, modules, or macro names.

1. Why is this asymmetry a defensible engineering position rather than an unfinished job?
   Argue from what macros are actually used for in Rust.
2. `$crate` exists. Reconstruct the bug it fixes, precisely, and say why the macro author
   cannot fix it by writing a full path by hand.
3. A macro expands to `let x = ...;` and the caller then writes `x`. Under Rust's rules
   the caller cannot see it. Name a real macro pattern this forecloses, and the escape
   hatch Rust offers instead.

### Q6 — Proc macros run too early

A Rust derive macro sees `TokenStream` in and emits `TokenStream` out. It runs after
parsing, before name resolution and type checking.

```rust
type MaybeInt = Option<u32>;

#[derive(MyDerive)]
struct S { a: Option<u32>, b: MaybeInt }
```

1. `MyDerive` wants to treat optional fields specially. Explain why it will treat `a` and
   `b` differently, and why this is not a bug in the macro author's code but a
   consequence of pipeline position.
2. Give the standard workaround derive macros use, and say what it costs in generated-code
   size and error-message quality.
3. Why can a proc macro not simply be moved later in the pipeline — say, after type
   checking — so it can ask real questions? Name the circularity.

### Q7 — `eval` is a tax on everything

```js
function f(obj) {
  let n = 0;
  for (const k in obj) n += obj[k];
  return n;
}
function g(obj, src) {
  let n = 0;
  eval(src);              // direct eval
  for (const k in obj) n += obj[k];
  return n;
}
```

1. Name three distinct compiler optimisations that `f` gets and `g` does not, and for each
   say what the presence of `eval` makes unsound.
2. In Python, `locals()` inside a function returns a dict, and writing to that dict does
   not change the local. Explain the representation decision that makes this true, and
   connect it to why Python 3 made `exec` a function rather than a statement.
3. "We have `eval`" is often described as a library feature. Argue that it is a
   *whole-implementation* decision by naming what it forecloses at the VM level — not just
   in the compiler.

### Q8 — Staging: when does the metaprogram run

Zig's `comptime`, C++ `constexpr`/`consteval`, Template Haskell splices, and Julia's
`@generated` functions all run user code during compilation.

1. Template Haskell has a **stage restriction**: you cannot splice a function defined in
   the same module. Reconstruct why that restriction is forced, not arbitrary.
2. Zig's `comptime` uses the same language at compile time and runtime; C++ built
   `constexpr` as a growing subset. Name the concrete thing the subset approach must keep
   re-litigating, and the concrete thing the one-language approach must solve that
   `constexpr` gets for free.
3. Compile-time execution and cross-compilation interact badly. State the problem
   precisely, and say how it constrains what a compile-time metaprogram may observe about
   the machine.

### Q9 — Keeping caches correct in an open world

A method cache says "for receiver class `C`, selector `foo` resolves to method `M`."
Then someone reopens `C` and redefines `foo`.

1. Three real invalidation strategies: a global epoch counter, per-class version stamps,
   and dependency lists recorded by the optimizer. Rank them by invalidation *precision*
   and by *bookkeeping cost*, and say which one a first implementation should ship.
2. HotSpot devirtualizes a call using class hierarchy analysis, then someone loads a class
   that adds an override. Describe the mechanism that keeps this sound, including where
   execution resumes.
3. Julia refuses to call a method defined by `eval` from inside the function that ran the
   `eval` — it reports the method as too new. Explain what invariant this "world age"
   discipline is protecting, and why "just look it up again" is not an acceptable fix.

### Q10 — Monkey-patching versus ahead-of-time devirtualization

A static compiler sees exactly one implementation of `Shape#area` in the whole program and
inlines it. Then a user writes, at runtime:

```ruby
class Circle
  def area; @cached ||= super; end
end
```

1. Explain why the inline is now unsound, and be specific: the problem is not "the answer
   changed", it is something structurally worse.
2. GraalVM native-image, R8/ProGuard, and Swift's whole-module optimisation all close the
   world in some way. Name the *user-visible* thing each one breaks, and the escape valve
   each provides.
3. A dynamic runtime can keep the inline and stay correct. Name the mechanism, and state
   the one property the machine code must have for that mechanism to be implementable at
   all.

### Q11 — When a lookup miss becomes a call

Ruby `method_missing`, Smalltalk `doesNotUnderstand:`, Python `__getattr__`, and JS
`Proxy` all turn a failed lookup into user code.

1. In a VM with monomorphic inline caches, what happens to the cache on a
   `method_missing` hit? Give the two design options and say what each one gets wrong.
2. Ruby requires you to override `respond_to_missing?` as well as `method_missing`, and
   Smalltalk has the same problem with `respondsTo:`. Explain why this is not an API wart
   but a genuine consequence of the design.
3. A JS `Proxy` must uphold invariants — it cannot report a different value for a
   non-configurable, non-writable own property of its target. Why does the spec impose
   this, and what does it buy an engine that would otherwise be lost?

### Q12 — Annotation, or transformation

```python
@memoize
def f(x): ...
```

```java
@Transactional
public void f() { ... }
```

These look alike and are not remotely alike.

1. State the operational difference in terms of what exists after the declaration is
   processed.
2. Java annotation processors (JSR 269) and C# Roslyn source generators are both
   deliberately **additive only** — they may create new sources, not modify existing ones.
   Reconstruct why, from what a compiler needs in order to terminate and to be
   incremental.
3. Given that restriction, how does `@Transactional` actually take effect? Name at least
   two distinct implementation strategies and the pipeline stage each occupies.

### Q13 — Where the weave happens

A production bug: `@Transactional` on a method has no effect, but only when that method is
called from another method of the same class.

```java
class Svc {
  public void outer() { this.inner(); }     // no transaction!
  @Transactional public void inner() { ... }
}
```

1. Diagnose it. Name the weaving strategy that produces exactly this hole, and say why the
   hole is unavoidable under that strategy.
2. Name a weaving stage that does not have this hole, and the three concrete costs of
   moving there.
3. Load-time weaving sits between the two. What does it buy, and what new class of failure
   does it introduce that compile-time weaving does not have?

### Q14 — Three levels of code generation, three levels of error message

A user makes a typo inside: (a) a C preprocessor macro invocation, (b) a Rust `macro_rules!`
invocation, (c) a template-string code generator that emits source text.

1. Rank the three by the quality of the error the user sees, and explain the ranking with
   the *one* piece of data that determines it.
2. A proc macro that emits a token with `Span::call_site()` and one that emits the same
   token with the span of the user's input produce different diagnostics. Explain what a
   span carries besides a source location in a hygienic system.
3. Why does a string-template generator so often ship a source map, and why is a source map
   strictly weaker than what an AST macro has?

### Q15 — Reflection versus everything the optimizer wants

```java
Class.forName(config.get("handler")).getDeclaredConstructor().newInstance();
```

1. Name three separate things this single line defeats in a build-and-deploy toolchain, and
   for each say what the tool would otherwise have been allowed to conclude.
2. `setAccessible(true)` reaches private state. Java 9 tried to stop it and the ecosystem
   revolted. Argue both sides, then commit: should a language's reflection be able to
   violate its own visibility rules?
3. Deserialization gadget chains (Java's `ObjectInputStream`, .NET's `BinaryFormatter`,
   JNDI-driven remote class loading) turn reflection into remote code execution. Identify
   the *single* capability that makes the whole family possible, and the design rule that
   removes it.

### Q16 — Reshaping a live hierarchy

In a running Smalltalk image, someone adds an instance variable to a superclass with ten
thousand live instances across forty subclasses.

1. Enumerate what must happen for the image to remain consistent. There are at least three
   distinct obligations, and one of them is not about the objects.
2. Smalltalk's `become:` swaps the identity of two objects — every reference to A now
   points at B and vice versa. Give the two ways to implement it and the cost each imposes
   on *every other* operation in the system.
3. Reparenting a class (changing its superclass at runtime) triggers an invalidation
   cascade. Describe the cascade's shape and say why the cost is not proportional to the
   number of subclasses.

### Q17 — Internal DSL, external DSL

```ruby
task :build => [:compile, :test] do
  sh "make"
end
```

versus a purpose-built `.build` file with its own parser.

1. An internal DSL inherits the host's parser, tooling, and package manager for free. Name
   precisely what it *steals* in exchange — and it is not "syntax flexibility".
2. Scala's implicit-heavy DSLs and C++'s expression templates produce famously bad error
   messages. Name the shared structural cause; it is the same cause in both languages.
3. You must ship a configuration language. Argue for the external DSL using an argument
   that is *not* about syntax.

---

## Answers

### A1 — Hygiene has two directions

**1.** Case A is **introduced-binder capture**: the macro's template binds `tmp`, and the
user passed an expression that *refers* to their own `tmp`. The expansion's `let` shadows
it, so `,b` — which was the user's `tmp` — now reads the macro's temporary, and the swap
silently does the wrong thing. Case B is **free-reference capture**, the opposite arrow:
the macro's template *refers* freely to `list`, and it lands inside a user scope that binds
`list` to 5. The macro's own reference is hijacked by user code. In A the macro captures
the user; in B the user captures the macro.

**2.** `gensym` fixes only A. It works because A is a problem about *names the macro
chooses* — the macro is the author of `tmp`, so it can choose an unforgeable one instead.
B is a problem about *names the macro merely mentions*, and the macro does not get to
choose those: it has to say `list` because `list` is what it means. There is no fresh
symbol you could substitute; the whole point is to refer to the existing binding. `gensym`
operates on the wrong side of the arrow. The only fix for B is to make the *reference*
resolve somewhere other than the use site — which is a property of the expander, not
something a macro author can express with a naming trick.

**3.** (a) **Hygiene proper**: identifiers introduced as binders by a macro template do not
capture identifiers that came from the macro's arguments. (b) **Referential transparency**:
free identifiers in a macro template refer to bindings visible at the macro's *definition*
site, regardless of what is in scope at the use site. (a) is about binders, (b) is about
free references. Systems that provide only (a) — which is what "hygiene" is loosely taken
to mean — are half-hygienic, and the half they are missing is the one that breaks silently
rather than loudly.

**Trap.** Saying "hygiene means the macro renames its temporaries." That is one direction
and the easy one. If you cannot state the second direction, you cannot explain why
`syntax-rules` needs to track a definition environment at all, and you will design a macro
system whose failures are invisible.

### A2 — The macro ladder, rung by rung

**1.** `SQUARE(1 + 2)` expands to `1 + 2 * 1 + 2` = 5: a **precedence/structure** failure,
because the macro operates on text with no notion of the argument being a complete
expression. `MAX(i++, j)` expands to a form that mentions `i++` twice: a **multiple
evaluation** failure, because the argument's *effect* is duplicated. Parenthesising fixes
the first completely — `((x) * (x))` — and cannot touch the second, because parentheses
change grouping, not the number of times a subexpression appears. GNU C's statement
expressions with a local temporary exist specifically to patch the second, and drag in a
non-standard extension plus the capture problem of Q1.

**2.** Operating on cons cells eliminates the structure class entirely and for free: an
argument arrives as a tree, so it is a single node no matter what it looks like, and there
is no such thing as it "regrouping" against the surrounding template. It does *not*
eliminate multiple evaluation — `` `(if (> ,a ,b) ,a ,b) `` still names `,a` twice — and it
does not eliminate capture. `defmacro` gives you structure but leaves effects and names to
discipline.

**3.** `syntax-rules` eliminates the capture class, by construction. What it gave up is
**arbitrary computation at expansion time**: the right-hand side is a template, not a
program. You cannot construct a new identifier from parts (no `foo` + `-bar` →
`foo-bar`), you cannot iterate over a list to a fixed point, you cannot inspect an
identifier's binding, and you cannot make a decision by comparing two arbitrary
identifiers. A `define-record-type`-style macro that synthesises accessor names from a
field list — `(define-struct point x y)` generating `point-x` and `point-y` — is the
canonical thing you cannot write, because name synthesis is exactly the thing the pattern
language cannot express.

### A3 — Why `syntax-case` exists

**1.** Name synthesis, per A2 — a struct macro that must produce `point-x` from `point` and
`x`. `syntax-rules` blocks it because its right-hand side is a template into which
sub-forms are substituted; there is no place to *run* string or symbol manipulation. Other
genuine cases: a macro that must sort or deduplicate its inputs, one that must compute a
jump table at expansion time, and one that must dispatch on whether an identifier is
currently bound as a macro or a variable.

**2.** Because an identifier in a hygienic expander is not a symbol — it is a symbol plus
the lexical context it was written in (marks, or a scope set). `datum->syntax` is being
asked to manufacture an identifier that did not appear in any source text, so it has no
context of its own, and *which* binding it should see is not determined by its spelling.
The first argument is a **context donor**: "make this new identifier behave as though it
had been written where *that* one was written." That is precisely how an anaphoric `it`
becomes visible to the user's body — it is stamped with the user's context — and it is why
the operation is unavoidably explicit.

**3.** Because intentional capture is genuinely useful (anaphora, implicit `self` in a
class-body macro, a loop macro that exposes `break`) and any rule banning it also bans a
pile of legitimate designs. The right shape is: hygiene is the *default* and violating it
requires naming the context you are injecting into, so the violation appears in the macro's
source as a distinct, greppable operation. Compare `unsafe` in Rust: the point is not that
the operation is forbidden, it is that the audit surface is finite and marked.

### A4 — Implementing hygiene

**1.** The set contains **scopes**: one for each binding form the identifier appears
inside, plus one fresh scope introduced for each macro expansion, applied to every piece of
syntax that came *out* of the macro. So an identifier written by the user carries the
user's scopes; an identifier introduced by the template carries the user's scopes *plus*
the expansion's fresh scope. The resolution rule: among all bindings for that symbol whose
scope set is a **subset** of the identifier's scope set, pick the one with the **largest**
such set; ambiguity is an error. That is the whole algorithm.

**2.** Because both directions of Q1 reduce to the same subset test. In case A, the macro's
`tmp` binder carries the expansion scope and the user's `tmp` reference does not; the
binder's scope set is therefore *not* a subset of the reference's, so the reference cannot
see it — no capture. In case B, the macro's free `list` reference carries the expansion
scope and also the macro's definition scopes, so the binding it resolves to is the one at
the definition site; the user's local `list` binding carries the user's scopes but not the
macro definition's, so it loses the subset test. One rule, both arrows. This is why scope
sets replaced the earlier marks-and-substitutions machinery: not because they were faster,
but because they were a *single* explanation for phenomena that previously needed two.

**3.** Every tool that maps a name to its definition has to work on **expanded** syntax with
the scope/mark information intact, and then map back to the surface. "Go to definition" on
an identifier inside a macro invocation must answer: which of the possibly several
identically-spelled bindings, in which expansion, at which stage. An unhygienic system's
tooling can cheat with textual scoping heuristics and be right most of the time; a hygienic
one cannot, so it must either run the expander inside the IDE or persist the expansion's
binding table. Racket does the former and this is a large part of why its macro-aware
tooling is unusual.

### A5 — Rust's partial hygiene

**1.** Because the two categories have opposite ergonomic requirements. Local variables in
a macro body are *implementation detail* — the caller must never see them, so hygiene is
what you want. Items are *the deliverable* — the overwhelming use of Rust macros is to
generate a `struct`, an `impl`, a function, or a test, and the caller must be able to name
what was generated. A fully hygienic item namespace would make `macro_rules!` unable to
define anything the caller can use, which is most of its job. So the split tracks the
distinction between what a macro *uses* and what a macro *produces*.

**2.** The bug: a macro defined in crate `A` expands to `::helper::do_it()` or
`std::vec::Vec::new()`, and is invoked in crate `B`. Path resolution happens at the
expansion site, in `B`'s namespace — so if `B` has no `helper` in scope, or has a *different*
`helper`, the expansion breaks or silently calls the wrong thing. The author cannot fix it
by hand because they cannot write the absolute path to their own crate: the crate's name in
the dependent's namespace is chosen by the *dependent's* manifest and can be renamed. Only
the compiler knows that mapping at expansion time, so `$crate` is a token the expander
substitutes with a path that resolves back to the defining crate whatever it is called
downstream. Note this is exactly the referential-transparency half of Q1 — Rust provides it
for paths via an explicit marker rather than automatically.

**3.** It forecloses the "macro that introduces a binding for you to use" pattern — a
`let!`-style macro, or a `with_context!` that gives you a `ctx` variable. The escape hatch
is to make the macro take the identifier as an *argument* (`with_context!(ctx, { ... })`),
so the name is written in the caller's source and therefore carries the caller's context.
This is the same trick as `datum->syntax`'s context donor, degraded to "make the user supply
the token" — less expressive, but it requires no hygiene-breaking API and it is honest at
the call site about what is being bound.

### A6 — Proc macros run too early

**1.** The macro sees tokens. For field `a` it sees the token sequence `Option < u32 >` and
its "is this optional?" check — a textual/structural match on the path's last segment —
succeeds. For field `b` it sees `MaybeInt`, one identifier, and the check fails. The macro
has no way to learn that `MaybeInt` *is* `Option<u32>`, because type aliases are resolved by
a later phase that has not run and cannot be invoked from here. This is not sloppiness: the
information genuinely does not exist yet at this pipeline position. Every derive that
special-cases `Option` has this hole, and the standard bug report is "your derive doesn't
work with my type alias."

**2.** The workaround is to stop asking and instead **generate code that makes the compiler
decide** — emit a trait-based dispatch (a blanket impl plus a specialised impl, or a
helper trait implemented for `Option<T>` and for `T`) so that type resolution, which does
have the answer, picks the branch. Cost: substantially more generated code per field,
slower compiles, and — the real cost — **error messages that point at synthesised trait
bounds the user never wrote**. A failure that would have been "field `b` must be `Option`"
becomes "the trait bound `MaybeInt: IsOptional` is not satisfied", inside a span the user
does not recognise.

**3.** Circularity: name resolution and type checking need the full set of items, and proc
macros *produce* items. To type-check you must have expanded; to expand-with-type-info you
must have type-checked. Any language that wants type-informed macros has to break the cycle
somewhere — by stratifying into stages with an explicit ordering (Template Haskell's stage
restriction, Q8), by restricting type-aware generation to a phase that cannot introduce new
names, or by iterating to a fixed point and accepting non-termination as a real
possibility. Rust chose the cheapest cut: macros never see types, and the compiler stays a
straight pipeline.

**Trap.** "The macro should just resolve the alias itself." There is nothing to resolve
against — the macro is handed a token stream for one item, not a symbol table, and the
alias may be defined in another crate, behind a `cfg`, or by another macro that has not run
yet.

### A7 — `eval` is a tax on everything

**1.** (a) **Scope analysis / slot allocation.** In `f`, the compiler proves the set of
variable names in scope is fixed, so `n` and `k` become stack slots or registers. Direct
`eval` can introduce new bindings into the enclosing scope in sloppy mode, so `g`'s
variables must live in a heap-allocated scope object addressable by name. (b) **Closure
conversion.** Deciding what a closure must capture requires knowing which inner references
are free; `eval` can create a reference to any name at runtime, so the conservative answer
is "capture the whole scope chain," which keeps otherwise-dead objects alive and turns a
cheap closure into an environment pointer. (c) **Inlining and constant propagation** past
the `eval` site: the compiler cannot prove any local's value is unmodified across it,
because the evaluated string can assign to any visible name. Add a fourth if you like:
`eval` can shadow a global the surrounding code reads, which invalidates any global-slot
caching.

**2.** Python function locals are **array slots**, resolved to indices at compile time
(`LOAD_FAST i`), not dictionary entries. `locals()` therefore has nothing to hand back but
a materialised snapshot dict built by copying the slots out; writing to the dict writes to
the copy. The connection to `exec`: in Python 2, `exec` was a *statement*, and the compiler
detected it syntactically and demoted the whole function to dictionary-based locals — a
real, silent performance cliff triggered by a keyword. Making `exec` a function in Python 3
removed the compiler's ability *and obligation* to detect it, which is precisely what allows
fast locals to be unconditional. The cost is that `exec` in a function can no longer create
locals, which is a documented, accepted regression in expressiveness bought for a uniform
representation.

**3.** Because `eval` requires the **compiler to be present in the shipped runtime**, and
that is a VM-level fact, not a compiler-level one. Consequences: you cannot ship an
ahead-of-time-only artifact; you cannot close the world for whole-program optimisation; you
cannot claim a memory ceiling (compilation allocates); you cannot run on a platform that
forbids generating executable pages (iOS's W^X restrictions are why JavaScriptCore on iOS
historically ran interpreted for third-party apps); and every cache keyed on "the program's
shape" needs an invalidation path. It also permanently changes your security posture: any
path from untrusted input to `eval` is remote code execution, so the feature is a
threat-model line item. This is why "we have `eval`" belongs in the same decision tier as
"we have a GC."

### A8 — Staging: when does the metaprogram run

**1.** Because a splice *runs* the function, and running it requires compiled code, and
compiling the module requires expanding the splice. The stage restriction is the cycle
break: the spliced function must come from a module that is already through the pipeline,
so its compiled form exists before this module's expansion begins. The restriction is
exactly A6's circularity with the boundary drawn at module granularity instead of being
banned outright — which is the honest general answer: every compile-time-execution feature
must define a *stratification*, and the visible restriction is where that stratum boundary
was placed.

**Trap.** Calling the stage restriction an implementation limitation a smarter compiler
would lift. Lifting it means running a definition while the module containing it is still
being elaborated, so the module's meaning depends on executing code whose meaning is not yet
fixed. That is not an engineering gap; it is a request for the compiler to be a fixed-point
solver over its own input.

**2.** The subset approach must keep re-litigating **which constructs are admitted**:
`constexpr` began by forbidding loops and local variables, then admitted them, then
admitted allocation-with-deallocation-in-the-same-evaluation, then `constexpr` virtual
calls, and each admission is a standards cycle plus two divergent implementations. The
one-language approach must solve **the compile-time/runtime boundary in the type system and
the semantics**: what happens when comptime code touches something that only exists at
runtime, how a value crosses the boundary, and what "the same type" means when types are
first-class comptime values. `constexpr` gets that boundary for free precisely because the
subset is defined to exclude everything that would raise the question.

**3.** Compile-time code runs on the **host**, but is generating code for the **target**.
So any observation of the machine — pointer width, endianness, alignment, floating-point
rounding, `sizeof`, available intrinsics, the filesystem, environment variables — is a
question with two different correct answers, and taking the host's is a silent
miscompilation. The constraint: a compile-time metaprogram may only observe properties of
the *target* as modelled by the compiler, never the host it happens to be running on. This
is why compile-time floating-point evaluation is specified so carefully, why compile-time
file I/O is either banned or heavily restricted (Zig's `@embedFile` is a controlled,
declared version of it), and why any macro that shells out to the environment breaks
reproducible and cross builds.

### A9 — Keeping caches correct in an open world

**1.** **Global epoch**: one counter, bumped on any method (re)definition anywhere; every
cache entry stores the epoch it was filled at and is considered stale if it differs. Least
precise — one unrelated definition invalidates every cache in the system — and cheapest by
a wide margin: one integer, one comparison, no data structures. **Per-class version
stamps**: bump the stamp on the class whose method dictionary changed; a cache entry
records the class and stamp. Much more precise, but wrong on its own the moment
inheritance is involved — redefining a method on a superclass must invalidate entries keyed
on subclasses, so you need either version propagation down the hierarchy or a stamp on a
shared lineage object. **Dependency lists**: the optimizer records, for each compiled unit,
the exact set of assumptions it made ("no subclass of `C` overrides `foo`"), and each
assumption is registered with the entity it depends on. Most precise, and by far the most
bookkeeping — you need a reverse index from every mutable entity to every compiled unit that
depends on it, and it must survive collection of those units.

Ship the global epoch first. It is correct, it is a day's work, and its imprecision only
costs you throughput after a redefinition, which is rare in steady state. Every mature
runtime started here — CRuby's global method cache being the standard example — and moved
to finer granularity only when profiling showed redefinition-heavy workloads (test suites
loading mocks, hot reload) thrashing.

**2.** The JIT devirtualizes on the CHA fact "`Shape` currently has exactly one
implementor of `area`", emits a direct, inlined call, and **registers a dependency** on
that fact with the class hierarchy. Loading a class that overrides `area` invalidates the
assumption, which marks every compiled method holding it as **not entrant** — new calls go
to the interpreter or a fresh compile. But existing *activations* are the hard part: a frame
may currently be executing inside the now-wrong inlined code. So the runtime must
**deoptimize on-stack**: at the next safepoint, rewrite that native frame into the
equivalent interpreter frame(s) using the scope-descriptor metadata recorded at compile
time, and resume in the interpreter at the corresponding bytecode index. Execution resumes
*in the interpreter, mid-method*, not at the method's entry — reconstructing that mid-method
state is the entire difficulty, and it is why the compiler must emit deopt metadata at every
point it makes a speculative assumption.

**3.** It protects **the consistency of a running computation's view of the method table**.
If a function could call methods defined after it began, then which methods a call site
resolves to would depend on when it executed, and the runtime could not (a) cache
resolution for the duration of a call, (b) type-infer and specialise a function against a
fixed method set, or (c) reason about a call graph at all — inference results would be
invalidated by code the inferred function itself ran. "Just look it up again" fails because
the problem is not staleness of a lookup, it is that the *type inference and specialisation*
built on top of that lookup would have to be redone mid-execution, which is on-stack
replacement in the general case for every call site. World age buys a cheap answer: a
running function sees a frozen snapshot, and new definitions become visible at a defined
boundary (returning to top level, or an explicit `invokelatest`).

**Trap.** Claiming per-class version stamps are strictly better than a global counter.
They are more precise per invalidation, but they cost a stamp check *on every cache hit*
against a value that may be several dereferences away, whereas a global epoch is one load
of a hot, always-cached word. The precise scheme can be slower in aggregate on the workload
that matters, which is the one where nothing is being redefined.

### A10 — Monkey-patching versus ahead-of-time devirtualization

**1.** The problem is not that the compiled code returns a stale answer — that would at
least be a value bug. It is that the inlined body was compiled **under assumptions borrowed
from the callee**: registers allocated across the inline boundary, the callee's `self`
proven non-null, its field offsets baked in, exception edges elided, surrounding code
rescheduled through it. There is no "the answer changed" — there is a machine-code region
whose *shape* is a theorem about a program that is no longer the program. You cannot patch
the return value; you have to invalidate the region and reconstruct the state of anything
currently executing inside it. That is why unsound devirtualization is a correctness
catastrophe rather than a staleness annoyance.

**2.** **GraalVM native-image** closes the world at build time: reflection, dynamic proxies,
`Class.forName` on a computed string, and runtime class loading stop working; the escape
valve is explicit reachability metadata (configuration files, now often generated by a
tracing agent). **R8/ProGuard** shrink and rename: anything referenced only by string —
reflection, serialised class names, XML-configured handlers — silently disappears or is
renamed out from under the string; the valve is `-keep` rules. **Swift whole-module
optimisation** devirtualizes and specialises across files in a module; the valve is that
the closure is only over the module, plus `@objc`/`dynamic` to opt individual members back
into dynamic dispatch, plus library-evolution mode to opt out of layout assumptions across
a module boundary. The common shape: **close the world, then sell back openness as
configuration**, and the configuration is always the part that rots.

**3.** The mechanism is **guarded inlining plus deoptimization** — inline speculatively,
record the assumption, and on invalidation rewrite live frames back to a canonical
representation (A9.2). The required property of the machine code is that at every point
where a speculative assumption is live, the runtime can **reconstruct the abstract-machine
state** — every local, every temporary, every virtual frame that inlining erased — from the
physical registers and stack. That is the deopt map, and it must be emitted at compile
time, at every such point, for every inline level. If your code generator cannot emit it,
you cannot speculate at all, and everything above is unavailable to you. The decision to
speculate is therefore made in the register allocator's design, not in the optimizer's.

**Trap.** "Add a guard — check the method is still the one you inlined and branch to the
slow path if not." That handles the *entry*, and it is genuinely how guarded inlining
starts. It does nothing for a frame already executing inside the inlined body when
invalidation happens, and long-running loops are exactly where the inline mattered. Guards
make new calls correct; only deoptimization makes in-flight ones correct.

### A11 — When a lookup miss becomes a call

**1.** Option one: **don't cache the miss** — treat `method_missing` as the slow path and
re-run full lookup every time. Correct, but it makes a dispatch style that some libraries
use for *every call* (ActiveRecord-style dynamic finders, RPC stubs, `OpenStruct`)
permanently uncached, so the pattern's cost is O(hierarchy depth) forever. Option two:
**cache the miss** — record "class `C`, selector `foo` → dispatch to `method_missing`" as a
negative entry. Fast, but now you must invalidate that entry when someone *later defines*
`foo` on `C` or any of its ancestors, which means your invalidation scheme must fire on
definition of a selector that previously did not exist anywhere — a case a naive per-class
stamp scheme will miss, because the class whose stamp changed may be an ancestor you never
consulted during the failed lookup. Negative caching is where most method-cache
invalidation bugs live.

**2.** Because `method_missing` makes the object's *behaviour* larger than its **reflected
interface**, and `respond_to?` reports the interface. The interface is not derivable from
the implementation: `method_missing` is a function whose domain is "every selector for
which the body does not call `super`", and that set is decidable only by running the body,
which is not a query. So the language must ask the author to declare the domain separately.
Any design where behaviour is a *procedure* and introspection is a *table* has this
obligation — Python's `__getattr__` versus `__dir__`, JS `Proxy`'s `get` versus its `has`
and `ownKeys` traps. The wart is not the extra method; the wart would be pretending one
method could serve both.

**3.** Because without the invariants, the object model's guarantees would be unenforceable
in the presence of any proxy, and those guarantees are what downstream code and engines rely
on. `Object.freeze` would mean nothing if a frozen-looking proxy could report changing
values; `Object.getOwnPropertyDescriptor` could not be trusted to describe reality. What an
engine gets: it can still trust the *shape* conclusions it draws from non-configurable,
non-writable properties, and it can trust extensibility answers — so the general path
through a proxy stays a general path, but the surrounding code's assumptions about frozen
objects and about `Object.prototype`'s integrity survive. Note the engine still loses the
big prize: property access on a proxy cannot be inline-cached to an offset, because there is
no offset — every access is a call into a trap, so proxy-heavy code runs on the generic path
by construction. The invariants preserve *soundness of other people's* optimisations, not
speed of the proxy.

**Trap.** "`Proxy` is just `__getattr__` with more traps." The difference that matters is
that `__getattr__` fires only on a *miss*, so a normal attribute access on a
`__getattr__`-having object is still a plain dict/slot lookup and stays fast. A `Proxy`
intercepts *every* operation including hits, so it has no fast path at all. Miss-only
interception is the design that composes with caching; total interception is not.

### A12 — Annotation, or transformation

**1.** After Python processes `@memoize`, the name `f` is bound to **a different object** —
whatever `memoize` returned. The original function may be unreachable except through a
closure. The decorator is an ordinary call executed at definition time, and there is no
record of it anywhere except by convention (`functools.wraps` copying `__name__` exists
precisely because the record is otherwise lost). After Java processes `@Transactional`, the
method is **byte-for-byte the method the programmer wrote**, plus an entry in the class
file's annotation table. Nothing has been transformed. One is a transformation with no
metadata; the other is metadata with no transformation.

**2.** Because a processor that could modify existing sources creates a fixed-point problem
with no natural termination: processor A rewrites a class, which changes what processor B
sees, which may cause B to rewrite it again, feeding A. Round-based additive generation
terminates because the set of declarations only grows and each round consumes the previous
round's output; there is a monotone quantity. It is also what makes the model tractable for
**incremental** compilation and for IDEs: the compiler can treat generated sources as
ordinary inputs with a known provenance, and can reason about which inputs a regeneration
depends on. A rewrite-in-place model gives the IDE a file on disk that does not correspond
to what is compiled, which is the thing every user complains about in systems that do it
anyway.

**3.** (a) **Runtime dynamic proxy**: the container reads the annotation reflectively at
wiring time and hands out a generated proxy object that wraps the real one, opening a
transaction before delegating. Stage: runtime, at object-graph construction. (b) **Build-time
bytecode weaving**: a post-compile step rewrites the method body to insert the transaction
begin/commit/rollback. Stage: after `javac`, before packaging. (c) **Load-time weaving**: a
JVM agent transforms the class bytes as they are loaded. Stage: class loading. (d) Worth
naming: **compile-time generation of a wrapper** via a source generator, where the
annotation drives creation of a new decorating class the framework instantiates instead —
Roslyn generators and Dagger-style Java processors both do this, and it is the only one of
the four that stays inside the additive rule.

### A13 — Where the weave happens

**1.** **Runtime proxy weaving** (Spring's default AOP model). The container gives your
*callers* a proxy that implements the same interface and delegates to the real `Svc`
instance; the advice lives in the proxy. `outer()` executes inside the real object, so
`this` is the real object, not the proxy — and `this.inner()` is a direct virtual call that
never crosses the proxy boundary. The hole is unavoidable because the proxy's entire
mechanism is *interception at the reference*, and a self-call does not go through a
reference the container controls. Same reason `@Transactional` on a private method silently
does nothing: there is no interceptable call edge. The general law: **any weaving strategy
that operates on the object's exterior cannot see calls that never leave it.**

**2.** **Compile-time (or bytecode) weaving** — AspectJ's model — inserts the advice into
the method body itself, so it fires however the method is reached, including self-calls,
private methods, and reflective invocation. Costs: (a) you need a second compiler or a
bytecode-rewriting build step, which fights every tool that expects `javac` output to
correspond to source; (b) debugging and stack traces now show code nobody wrote, and line
tables must be maintained through the rewrite or breakpoints land wrong; (c) it is
**whole-artifact**: you cannot weave a dependency you consume as a compiled jar unless you
also rewrite it, so the aspect's reach stops at your build boundary.

**3.** Load-time weaving buys reach into **third-party and already-compiled code** without
owning their build — the agent transforms classes as the JVM loads them, so you can advise a
library you did not compile. The new failure class is **ordering and coverage dependent on
class-loading order**: classes loaded before the agent is installed are never woven, classes
loaded by a loader the agent does not see are missed, and the same application can behave
differently depending on which code path warms first. Compile-time weaving's coverage is a
static, inspectable set; load-time weaving's coverage is an emergent property of a
particular run. That is also why load-time weaving interacts so badly with anything that
caches or shares class data across runs.

### A14 — Three levels of code generation, three levels of error message

**1.** Best to worst: Rust `macro_rules!`, C preprocessor, string-template generator. The
single determining datum is **whether the generated token carries a pointer back to the
user-written source that caused it** — a span. Rust's expander propagates spans from the
macro's input into its output, so an error inside an expansion is reported at the user's
argument with an expansion backtrace. The C preprocessor discards structure and emits text;
the compiler reports a column in a post-expansion line, which is why `#line` directives and
"in expansion of macro" notes exist as bolt-on reconstruction. A string generator emits a
fresh file with no relationship to any input at all, so the error is reported at a location
in a file the user never wrote and cannot meaningfully read.

**2.** A span carries a **syntax context** as well as a location: which expansion introduced
this token, and therefore which scope its identifiers resolve in. That is the same mark /
scope-set information from A4, stored on the token. So `Span::call_site()` does two things
at once — it makes errors point at the caller *and* makes identifiers resolve in the
caller's scope (unhygienically); a definition-site span points errors at the macro and
resolves in the macro's scope. The two are not separable, which is a genuine design wart:
you sometimes want the error location of one and the hygiene of the other, and the API
gives you a single knob.

**3.** Because a source map is the *cheapest possible reconstruction* of the span
information the generator threw away: a side table from generated position back to source
position. It is strictly weaker because (a) it is positional only — it carries no scope or
hygiene context, so it cannot tell a tool which binding an identifier refers to; (b) it is
per-character, so it cannot express "this token was synthesised and corresponds to no
input"; and (c) it lives outside the artifact, so every downstream consumer must opt in to
honouring it, and the ones that do not (a stack trace from a runtime that never loaded the
map, a linter, a grep) see only generated text. AST macros do not need a map because the
provenance is *in* the tree and cannot be dropped by a tool that did not know to look.

### A15 — Reflection versus everything the optimizer wants

**1.** (a) **Dead-code elimination / tree shaking**: the shrinker would otherwise conclude
the handler class is unreachable and delete it, because no call edge points at it. (b)
**Name minification**: the renamer would otherwise conclude the class name is an internal
detail it may rewrite; the config string then names a class that no longer exists. (c)
**Closed-world AOT compilation**: native-image would otherwise conclude the set of
instantiated types is known, which is what lets it pre-initialise the heap, devirtualize
globally, and omit a runtime class loader entirely. A fourth, if you want it: **class
initialisation ordering** — the compiler would otherwise know statically when each class's
static initialiser runs.

**2.** For: without it, dependency injection, ORMs, serialisation, mocking frameworks, and
test infrastructure are all impossible or require every class to be written to accommodate
them; the entire Java ecosystem was built on the assumption and Java 9 discovered that
"revolt" was not hyperbole — `--add-opens` became a permanent fixture of build files rather
than a migration step. Against: a `private` field that anyone can read is not private, so
the language is lying in its own type system; you cannot maintain an invariant, you cannot
optimise on the basis of encapsulation, and every security boundary drawn with visibility
is decorative. **Commit:** reflection must not be able to violate visibility *by default*,
but the capability must exist and must be **grantable at the module boundary** — which is
exactly what Java 9 built and what it should have built in 1996. The revolt was not evidence
the design was wrong; it was the cost of adding a boundary twenty years late, which is the
real lesson: **a boundary retrofitted is a boundary that breaks everyone.**

**3.** The capability is **deserialising a graph that names its own types, and instantiating
those types by name, with side effects during construction**. Once untrusted bytes can
select which class gets instantiated, and construction can run arbitrary code (constructors,
`readObject`, static initialisers, property setters), the attacker is programming your
process using your own classpath as an instruction set — the gadget chain is just a program
in that language. JNDI-driven remote class loading is the same bug with the classpath itself
supplied by the attacker. The design rule: **never let the data choose the type.** A
deserialiser must be given the expected type by the *program*, and must validate against it —
which is why schema-first formats and allow-list-based deserialisers are the accepted
answer, and why `BinaryFormatter` was deprecated outright rather than patched.

### A16 — Reshaping a live hierarchy

**1.** (a) **Every existing instance must be migrated** to the new layout — a new object
with the extra slot, old field values copied, new one defaulted. (b) **Every subclass must
be recompiled**, because in most implementations a subclass's methods have the superclass's
field offsets baked into their bytecode or machine code; adding a slot at the superclass
shifts every subclass field index. (c) — the one that is not about objects — **every method
cache, inline cache, and compiled-code assumption keyed on the old class shape must be
invalidated**, including caches held by code that is *currently on the stack*. And a fourth
in practice: **all references to each old instance must be redirected to its replacement**,
which is the `become:` problem below, and is why the two features are always implemented
together.

**2.** (a) **Object table indirection**: every object reference is an index into a global
table of real addresses, so `become:` is swapping two table entries — O(1), and it makes
the operation nearly free. The cost is paid by *every field access and every method call in
the system*, which now pays an extra indirection and loses the ability to keep an object's
address in a register across operations. (b) **Heap scan**: walk all of memory (and all
stacks, and all registers) rewriting every pointer to A into a pointer to B. O(heap) per
`become:`, but ordinary access stays direct. The trade is stark and it is the same trade as
handles-versus-direct-pointers in a moving GC: you can make one rare operation cheap by
taxing every common one, or keep the common one fast and accept that the rare operation is
a full-heap event. Most modern systems chose (b) and made `become:` correspondingly rare and
expensive.

**3.** The cascade is over the **transitive subclass closure** for shape, and over the
**transitive set of compiled code and cache entries that recorded any assumption touching
that lineage** for code — which is a strictly larger and differently-shaped set. Cost is not
proportional to subclass count because the expensive term is the second set: a single
widely-used superclass may have three subclasses but be mentioned in the dependency lists of
thousands of compiled methods (every call site that devirtualized on "no subclass of `C`
overrides `foo`" — including call sites in classes that are not subclasses of `C` at all).
The invalidation is proportional to *how much the optimizer speculated about this lineage*,
which is a function of how hot the code was, not how deep the hierarchy is. This is why hot
reload gets slower the longer a process has been running well.

**Trap.** "Reparenting is just fixing up the superclass pointer and flushing the method
cache." It is that plus a layout migration of live instances, plus recompilation of code
holding baked offsets, plus on-stack deoptimization of any activation currently inside
speculatively compiled code for that lineage. Systems that shipped the cheap version have a
characteristic bug: hot reload works fine until the code is hot, and then it corrupts.

### A17 — Internal DSL, external DSL

**1.** It steals **the host's error reporting, and with it the ability to say anything about
the DSL's own rules**. A typo in a `task` declaration is not "unknown task attribute on line
4"; it is a Ruby `NoMethodError` raised inside the DSL library, with a stack trace through
the library's internals, pointing at a line in the user's file only if you are lucky. The
DSL has no grammar, so it has no notion of a malformed program, so it cannot diagnose one —
every error is a *runtime* error in the host language, discovered at the moment of use, in
the host's vocabulary. It also steals **static analysis**: nothing can tell you the set of
valid keys, because there is no artifact that lists them, which is why internal DSLs have
such poor completion. Note that both losses are consequences of the same thing you were paid
for: not having a grammar.

**2.** Both push the DSL's rules into a **type-level or overload-resolution search**, and
then report failures in terms of the search rather than the rules. A Scala DSL encodes
"this combination is invalid" as "no implicit instance of `Foo[Bar, Baz]` in scope", and the
compiler faithfully reports the missing instance — a fact about the encoding, not about the
user's mistake. C++ expression templates encode grammar in the type of a partially-built
expression, so a misuse surfaces as a substitution failure deep in a type name that is
hundreds of characters long. The structural cause is **error messages are generated by the
mechanism that failed, and the mechanism is the encoding, not the language the user thinks
they are writing.** `static_assert` with a message, `@implicitNotFound`, and Rust's
`#[diagnostic::on_unimplemented]` are all the same patch: let the DSL author override the
mechanism's message with one in the user's vocabulary. That they all exist is evidence the
problem is structural.

**3.** **Because the configuration must be readable and writable by programs that are not
your language's runtime.** A `.build` file with a real grammar can be parsed by a linter, a
formatter, an editor, a CI system, a migration tool, and a security scanner — none of which
want to instantiate your object graph. An internal DSL can only be understood by *executing*
it, which means understanding it requires a Ruby interpreter, arbitrary code execution, and
whatever side effects the file feels like performing. That is why the ecosystem drifted from
Rakefiles and `setup.py` toward declarative manifests: not because the syntax was nicer, but
because `setup.py` is a program and `pyproject.toml` is data, and every tool that wants to
know your dependencies without running your code needs data. Second acceptable argument:
**version skew** — an external DSL can be versioned and evolved independently of the host
language, while an internal DSL's meaning silently changes when the host changes.

**Trap.** Arguing for the external DSL on the grounds that "you control the syntax." You do,
and it is the least valuable thing you get; the syntax argument also loses badly to the
internal DSL's free tooling. The real currency is *analysability without execution*, and if
you do not say that, the interviewer hears a preference rather than a reason.
