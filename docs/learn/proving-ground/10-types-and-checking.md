# 10 — Types and Checking

Rejecting programs before they run, and paying for the ones you cannot. The through-line:
*every type system is a bet about which errors are worth the annotations, and every one of
them cheats somewhere.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Nominal and structural, from the library author's chair

Two libraries, written by strangers, each declare a type with the same shape:

```go
// library A                      // library B
type Closer interface {           type Closer interface {
    Close() error                     Close() error
}                                 }
```

In Go these are the same type. In Java, Rust, or C# two identically-shaped `interface`/
`trait` declarations are unrelated and always will be.

1. You are the author of a published structural interface and you want to add a method.
   Describe what happens to your consumers, and explain why the structural answer is
   *worse for you* and *better for them* than the nominal one.
2. Structural typing makes accidental conformance possible. Give a case where accidental
   conformance is a genuine bug rather than a convenience, and say what the nominal
   discipline was actually encoding that the shape did not.
3. Rust and Haskell are nominal at the type level, yet you can attach behaviour to a type
   long after it was defined (`impl`, `instance`). Why is that not structural typing, and
   what does the difference buy you at codegen time?

### Q2 — Where Hindley-Milner stops

This is fine in ML and Haskell:

```haskell
let f = \x -> x in (f 1, f True)
```

This is not — GHC rejects it without a signature:

```haskell
data Nest a = Nil | Cons a (Nest [a])

size Nil        = 0
size (Cons _ r) = 1 + size r     -- recursive call is at Nest [a], not Nest a
```

1. Name the property of Damas-Milner that makes the first program inferable, and name the
   problem the second one reduces to. "It's polymorphic recursion" is not an answer;
   say what the *solver* is being asked to do.
2. HM inference is worst-case exponential in the size of the program, and nobody
   complains. Reconcile that.
3. Overloading — one name, several unrelated types — also breaks HM. Explain why, and
   explain why Haskell's type classes do *not*, despite looking exactly like overloading.

### Q3 — The value restriction

```ocaml
let r = ref []          (* what type does this get? *)
```

If `let`-generalization applied here as it does everywhere else, `r : 'a list ref`.

1. Write the two-line program that then segfaults (or, in a safe runtime, reads an integer
   as a string), and say exactly which step in inference was the lie.
2. The fix in SML and OCaml is *syntactic*: generalize only if the right-hand side is a
   syntactic value. Why syntactic rather than an effect analysis that asks "does this
   allocate a mutable cell?" — and name a perfectly safe program the syntactic rule rejects.
3. OCaml relaxes the restriction and generalizes some non-values anyway. State the
   criterion and argue its soundness.

### Q4 — Rank

```haskell
apply2 :: (forall a. a -> a) -> (Int, Bool)
apply2 g = (g 1, g True)
```

Delete the signature and GHC cannot recover it.

1. Why can it not be inferred? Be specific about *which* rank is the cliff and what is
   known about the ranks either side of it.
2. With the signature present, checking the body succeeds. Describe the mode switch a
   bidirectional checker performs, and say what changes about the treatment of `g`'s type
   variable inside the body.
3. Higher-rank types are not a theoretical toy. Name a real API that is impossible without
   them and explain what the rank is protecting.

### Q5 — Why almost nobody chose global inference

Haskell and OCaml infer whole programs. Rust, Swift, Kotlin, Scala, C#, and TypeScript all
require signatures on function boundaries and infer only inside them.

1. Name three language features these languages have that make global HM impossible, and
   for each say *why* it breaks — one sentence of mechanism each, not a list of names.
2. Even in Haskell, top-level signatures are considered mandatory in serious code.
   Give the two engineering reasons that have nothing to do with decidability.
3. Bidirectional checking rejects programs a global inferencer would accept. Construct
   one, and say who pays and how.

### Q6 — Java's covariant arrays

```java
Object[] objs = new String[1];
objs[0] = Integer.valueOf(42);   // compiles cleanly; throws ArrayStoreException
```

1. This rule exists. Reconstruct why, in terms of what Java-of-1995 did not have. What
   library code would have been unwritable otherwise?
2. Name the runtime mechanism that keeps it memory-safe, and state what it costs on the
   array stores that are provably fine — including how the JIT tries to claw it back.
3. Generics went the other way: `List<String>` is not a `List<Object>`. Given that the
   designers clearly knew better by then, why is `String[]` still an `Object[]`?

### Q7 — Where variance is declared

```kotlin
interface Source<out T> { fun next(): T }        // declaration-site (Kotlin, Scala, C#)
```

```java
void drain(List<? extends Number> src)           // use-site (Java wildcards)
```

1. Each moves a decision to a different person. Say who decides what under each, and name
   the class of API that only declaration-site variance can express cleanly.
2. `List<? extends Number>` will not let you call `add(someNumber)` even though `Number`
   is the bound. Explain the mechanism — the compiler is not being conservative for fun,
   it is doing something specific to the wildcard.
3. C# added declaration-site variance late and only for interfaces and delegates, never
   for classes. Reconstruct the restriction.

### Q8 — Erasure and reification

```java
new ArrayList<String>().getClass() == new ArrayList<Integer>().getClass()   // true
```

```csharp
typeof(List<int>) == typeof(List<string>)   // false
```

1. What did erasure buy Java at the moment generics shipped, and what did it cost
   permanently? The permanent cost is not "you can't write `new T[]`" — that is a symptom.
2. What does reification buy the CLR at *codegen* time specifically? Name the concrete
   thing that gets faster and why erasure cannot have it.
3. Bridge methods: reconstruct why erasure forces the compiler to synthesize them, and say
   where in the program a heap-pollution bug actually surfaces as a result.

### Q9 — Subtyping meets parametric polymorphism

`<T extends Comparable<? super T>>` is a signature people copy without reading. Rust,
notably, has no subtyping at all outside of lifetimes.

1. Adding subtyping to HM destroys principal types. Explain the mechanism — what does the
   solver produce instead of a substitution, and why does that stop being a *type*?
2. Java's generic subtyping has been shown to be undecidable — you can encode a Turing
   machine in it. Reconstruct roughly which two features combine to give it that power.
3. Rust's decision to have no general subtyping is usually described as a simplicity
   choice. State what it actually buys the *checker*, and name the ergonomic bill Rust
   pays for it.

### Q10 — Type classes are not interfaces

```haskell
class Monoid a where
  mempty :: a                      -- no argument of type a anywhere
```

```java
interface Monoid<A> { A empty(); }  // you need an instance before you can call it
```

1. Name the capability the type class has and the interface structurally cannot, and give
   the mechanism that makes it possible — the mechanism is one sentence about *what
   selects the implementation*.
2. Dictionary passing and vtable dispatch look alike at runtime. Say where each table
   comes from, and explain why the dictionary can usually be optimized away entirely while
   the interface's vtable usually cannot.
3. Coherence and the orphan rule: construct a program that goes wrong without them, and
   say what Scala gives up by not having them.

### Q11 — What crossing the boundary costs

A gradually typed program. A typed module imports an untyped one and receives a value the
typed side has annotated `(Int -> Int)`.

1. Why can the boundary not simply check the value once and be done? Name the object that
   gets created instead, and say what it costs per call and what it does to object identity.
2. Blame is usually sold as an error-message feature. Give the operational reason blame
   assignment matters, and say what makes tracking it expensive.
3. Sound gradual typing has been measured into the ground and some configurations are
   catastrophic. The field took two escapes. Name both and say precisely what each gives up.

### Q12 — Unsound on purpose

```ts
interface Handler  { on(e: MouseEvent): void }
interface Handler2 { on(e: Event): void }

declare let h: Handler, h2: Handler2;
h  = h2;   // sound: h2.on accepts more
h2 = h;    // ALSO allowed: TypeScript methods are bivariant
```

1. Reconstruct why TypeScript made method parameters bivariant. What extremely common
   JavaScript pattern would strict contravariance have broken?
2. `strictFunctionTypes` makes function-typed *properties* contravariant but deliberately
   exempts *method* declarations. Reconstruct the argument for the exemption.
3. Java's arrays, TypeScript's methods, and Dart's covariant generics are three holes.
   One of them pays for its hole with runtime checks. Say which, and rank all three by what
   the language got in exchange.

### Q13 — What a dynamic runtime is actually buying

```python
def f(a, b):
    return a + b
```

versus the same in a language where `a` and `b` are known to be 64-bit integers.

1. Enumerate what the dynamic runtime does at `a + b`. Count the *dependent* memory loads
   and the allocations, and say why the serialization matters more than the count.
2. "Dynamic typing is just static typing with one type." What does that framing get right,
   and what does it hide from an implementer?
3. Inline caches and speculative JITs recover nearly all the dispatch cost. Name the
   residual cost that speculation cannot remove, and say why not.

### Q14 — `any`, `unknown`, `never`

```ts
declare let a: any, u: unknown, n: never;
let s1: string = a;   // ok
let s2: string = u;   // error
let s3: string = n;   // ok
let x: any = u;       // ok
```

1. Place the three in the subtype lattice, and then explain why `any` does not actually
   fit in a lattice at all.
2. `never` is the empty union. Derive three separate observable behaviours from that one
   algebraic fact.
3. `unknown` was supposed to retire `any`. It did not. Name where `any` is structurally
   load-bearing and cannot be replaced.

### Q15 — Untagged unions and the code generator

```ts
type V = string | number          // TypeScript: untagged
```

```rust
enum V { S(String), N(i64) }      // Rust: tagged
```

1. What does "untagged" mean *operationally*, and why does an untagged union type force
   the host runtime to already carry type information? Say what that forecloses for a
   language that wanted to compile to unboxed data.
2. Rust's tagged enums get niche optimization — `Option<&T>` is one pointer wide. Describe
   the mechanism and name what it forecloses.
3. Scala 3 has untagged union types on the JVM. What does `A | B` erase to, and name two
   things that stop working as a direct result.

### Q16 — Narrowing, and what unnarrows it

```ts
function f(o: { x: string | null }, g: () => void) {
  if (o.x !== null) {
    g();
    o.x.length;    // TypeScript: accepted
  }
}
```

1. That is unsound. Show how, then give the reason TypeScript accepts it anyway — the
   reason is about what the alternative would do to ordinary code.
2. Kotlin refuses the analogous smart cast in a specific enumerable set of cases. List
   them, and state the single property Kotlin is demanding that TypeScript is not.
3. Typed Racket calls this occurrence typing and gives predicates *latent propositions*.
   TypeScript's `x is T` is the userland version. Name the crucial difference in what the
   checker does with the predicate's body, and say what that costs.

### Q17 — Effects are the colouring problem with a type on it

```java
list.stream().map(x -> Files.readString(x));   // does not compile: checked exception
```

1. State precisely why `Stream.map` cannot propagate the lambda's checked exception, and
   name the missing type-system feature in general terms.
2. Koka and Eff track effects in the type and have effect polymorphism. Name the cost in
   signatures and in inference — and then explain why OCaml 5 shipped effect *handlers*
   with the effects deliberately untracked.
3. Rust has `async`, `const`, `unsafe`, and `?`-on-`Result`: four ad-hoc effects with four
   ad-hoc mechanisms. Say what unifying them would buy and name the concrete blocker.

### Q18 — Soundness, completeness, and which one you sell

A checker rejects this, and is right to:

```
fn f(): Int {
    if (never_true()) { return "not an int" }
    return 0
}
```

1. State soundness and completeness precisely for a type checker, then argue informally
   that a decidable checker for a Turing-complete language cannot have both.
2. Which one does essentially every practical checker give up? Name two widely used
   systems that gave up the *other* one deliberately, and give each one's stated reason.
3. A checker is unsound in exactly one well-known place versus unsound in ten small ones.
   Argue which is worse *for the user* — not for the theorist — and name the property that
   actually decides it.

---

## Answers

### A1 — Nominal and structural, from the library author's chair

**1.** Adding a method to a structural interface silently *un-conforms* every existing
implementer that did not happen to have that method — and they never wrote your type's
name anywhere, so the breakage appears at unrelated call sites in unrelated packages, with
an error that says "does not implement" rather than "you added a method". Under nominal
typing the compiler at least knows who claimed to implement you, and a language can offer
default methods keyed on that declaration (Java did exactly this in 8, to evolve
`Collection` without breaking the world). So it is worse for the author: you have no list
of your implementers and no hook to give them a default. It is better for the consumer:
they can satisfy your interface without depending on you at all, which means your library
does not become a node in their dependency graph. Go's whole `io.Reader` ecosystem is that
property cashed in.

**2.** Two `struct(f64)` newtypes for metres and feet have identical shape; a structural
system will accept either where the other is wanted, which is the failure mode the
newtype existed to prevent. Same shape, incompatible meaning. What the nominal declaration
encodes is not the shape but the *claim* — "I intend to be usable as a `Closer`, and I
accept the obligations that come with it, including ones the signature cannot state"
(`Close` is idempotent, `Close` after `Close` is not an error, `Read` returns `io.EOF` and
not a wrapped error). Structural typing checks the part of the contract that is a shape and
is silent about the part that is a promise.

**3.** Because the *type* is still nominal — `impl Display for MyType` does not make
`MyType` a member of some shape-defined set, it registers an explicit, named association
between a nominal type and a nominal trait. There is no search over shapes; resolution is a
lookup in a table keyed by (trait, type). At codegen that is decisive: the compiler can
monomorphize, because for any concrete `T` there is exactly one implementation and it is
known statically. A structural system must either carry the shape at runtime or resolve by
searching, and neither monomorphizes.

**Trap.** "Structural typing is just duck typing with a compiler." It is not — duck typing
resolves per call site at runtime, structural typing resolves per type at compile time and
so still has to answer subtyping and variance questions that duck typing never poses. Go
has a real subtype relation between interfaces; Python does not.

### A2 — Where Hindley-Milner stops

**1.** The first program works because of **principal types under prenex (rank-1)
quantification**: unification over first-order type terms is decidable and produces a most
general unifier, so every typable term has a single most general type that every other
valid type is an instance of. Algorithm W just walks the term, unifies, and generalizes the
free variables at `let`. The second program breaks it because the recursive occurrence is
used at a type that is an *instance* of the type being inferred, so the constraint is not
"these two types are equal" but "this type is a substitution instance of that one". That
is **semi-unification**, and semi-unification is undecidable. So type inference for ML with
polymorphic recursion is undecidable, and the way out is to make the user write the
signature — which turns inference back into checking, where the recursive use has a known
polytype to instantiate.

**2.** The exponential blowup requires deeply nested `let`s each of which doubles the size
of the inferred type — the classic `let x1 = (x0,x0) in let x2 = (x1,x1) in ...` shape.
Real programs do not have that structure, because programmers write signatures and because
types in real code stay small. The complexity bound is driven by the *size of the printed
type*, and the size of the printed type is bounded in practice by human patience. This is
the standard shape of a "bad worst case, irrelevant in practice" result — the same shape as
Hindley-Milner's cousin, the union-find inverse-Ackermann bound, running the other way.

**3.** Overloading breaks HM because it destroys principality: if `+` can be
`Int -> Int -> Int` or `Float -> Float -> Float`, then `\x y -> x + y` has two incomparable
types and no most general one, so there is nothing for the algorithm to return. Every use
site would have to be resolved by search, and search over a program is exponential and
non-modular. Type classes avoid this by **not** overloading the type — `(+)` has exactly
one type, `Num a => a -> a -> a`. The polytype is unique; what varies is a *constraint*
carried along with it, and constraints are collected and solved separately from
unification. Principality is preserved because the type is principal *modulo* a constraint
set, and the constraint set is deterministic. That is the whole trick, and it is why type
classes were invented — they were the answer to "how do we get overloading without losing
inference".

### A3 — The value restriction

**1.**

```ocaml
let r = ref []            (* claimed: 'a list ref *)
let () = r := [1]         (* instantiate 'a := int  *)
let s : string = List.hd !r   (* instantiate 'a := string *)
```

The lie was **generalization at the `let`**. `ref []` is not a value; it is a computation
that allocates one specific cell. Generalizing gave every use site its own private `'a`,
as if there were a fresh cell per use, when in fact all uses share one cell. Generalization
is sound exactly when the right-hand side denotes something that can honestly be
*re-elaborated* at each instantiation; a heap cell cannot.

**2.** Syntactic because the property being approximated — "does evaluating this expression
create observable mutable state that the generalized variable can flow into" — is not
decidable, and any analysis that tries becomes part of the language definition: users must
be able to predict whether their code generalizes, and an analysis they cannot run in their
head is a worse user experience than a rule they can. So the rule is "generalize iff the
RHS is a syntactic value: a literal, a variable, a lambda, a constructor applied to
values." It rejects, among many others, `let f = compose g h` — a perfectly pure partial
application that produces a function and touches no state — because application is not a
syntactic value. That is the tax: eta-expand (`let f x = compose g h x`) and it generalizes
again. Every OCaml programmer has done this and most do not know why.

**3.** OCaml's **relaxed value restriction** (Garrigue) generalizes type variables that
appear only in *covariant* positions of the inferred type. A variable that occurs only
covariantly can never be used to *put* a value of that type into the structure — it is
output-only — so no two instantiations can meet inside the same cell. Formally the
soundness argument is that such a variable can be safely instantiated to the empty type and
then subsumed upward, so generalizing it introduces no new well-typed programs that could
go wrong. The practical effect: `let r = ref []` is still monomorphic (`'a` is invariant
under `ref`), but `List.map f []` and many pure applications generalize again.

**Trap.** "The value restriction exists because of `ref`." It exists because of anything
that can create a shared, type-instantiable location — `ref`, arrays, mutable record
fields, and in the original ML formulation, exceptions and lazy cells. Naming only `ref`
suggests you have memorized the example rather than the invariant.

### A4 — Rank

**1.** Type inference for System F — unrestricted rank — is undecidable (Wells). The cliff
is at **rank 3**: inference is decidable for rank ≤ 2 and undecidable for rank ≥ 3
(Kfoury and Wells). Rank-2 decidability is not much comfort — the algorithm is not
practical and the error behaviour is bad — so GHC does not attempt inference at any rank
above 1. The reason inference dies is that once a `∀` can appear to the left of an arrow,
unification is no longer being asked "are these two type terms equal" but "is there an
instantiation making this one at least as polymorphic as that one", which is a
subsumption/matching question and it does not have most general solutions.

**2.** The checker is in **checking mode** on the body because the signature supplied an
expected type; it pushes that expected type inward. At the binder `g`, the parameter type
is `forall a. a -> a`, which is a *known polytype*, so `g` is entered into the environment
at that polytype rather than at a fresh unification variable. Now each occurrence of `g`
in the body **instantiates** the `∀` afresh — `a := Int` at `g 1`, `a := Bool` at `g True`.
That is the entire content of the mode switch: in synthesis mode `g`'s type would have been
a single meta-variable that both uses would unify against, and `Int ~ Bool` fails. Rank-N
in GHC is precisely "you supply the polytype, we do the instantiation."

**3.** `runST :: (forall s. ST s a) -> a`. The rank-2 `∀ s` is what makes it impossible for
a mutable reference created inside the computation to escape into `a`: `a` is fixed
*outside* the quantifier, so any type mentioning `s` cannot be `a`. The rank is enforcing a
region discipline through nothing but scoping — this is the same trick as Rust's `for<'a>`
higher-ranked trait bounds, where `for<'a> Fn(&'a str) -> &'a str` means the closure must
work for *every* lifetime, not for one the caller picked. In both cases what the rank
protects is that the callee cannot choose the type/lifetime, so it cannot correlate it with
anything it owns.

### A5 — Why almost nobody chose global inference

**1.** (a) **Subtyping.** Unification asks for equality; subtyping asks for a bound, so the
solver produces a constraint set rather than a substitution and there is no most general
solution to report as "the type". (b) **Overloading / ad-hoc method resolution.** A call
`x.foo()` where `x`'s type is unknown cannot be resolved at all — with nominal methods the
receiver's type *is* the dispatch key, so inference and resolution become mutually
dependent, and you get the loop that makes Scala's and Swift's inference occasionally blow
up. (c) **First-class polymorphism and implicit conversions / defaults.** Anything that
lets the elaborator insert a coercion means the term you typed is not the term you check,
so there is no single most general type for the original term.

**2.** **Error locality** and **API stability.** Without signatures, a unification failure
surfaces wherever the two conflicting constraints happen to meet, which can be several
modules away from the mistake — the notorious HM error experience. And without signatures,
a function's type is whatever its *body* implies, so an innocuous edit inside a body
silently changes the exported type and breaks callers. Signatures turn that into a local
error at the definition. (Third reason, real but engineering-adjacent: signatures make
compilation separable, because a module can be checked against its neighbours' signatures
without their bodies.)

**3.** Anything whose most general type is not the "obvious" one, and where the obvious one
is what you would have written. Classic shape: a helper used at two different types inside
one function.

```haskell
f xs ys = (g xs, g ys) where g = length     -- HM generalizes g; fine
```

Bidirectional systems that only generalize at explicit signatures will infer `g` at a
single type from its first use and then reject the second. The bill is paid by the
*programmer*, in annotations on local helpers, and by the language designer, in "why does
this need a type here" bug reports. Everyone concluded that bill is cheaper than the
error-message bill, and they were right.

**Trap.** "Rust has type inference like Haskell's, just with mandatory signatures." Rust has
no HM at all in the relevant sense: there is no let-generalization, so a local binding is
never polymorphic. `let f = |x| x;` gets one concrete closure type fixed by its first use,
and a second use at a different type is an error — a program HM accepts without complaint.
Rust infers *types for expressions*; Haskell infers *polymorphic schemes for definitions*.
Those are different problems, and conflating them is the tell.

### A6 — Java's covariant arrays

**1.** Java 1.0 had no generics. Without array covariance you could not write
`void sort(Object[] a)` and call it on a `String[]`, or `System.arraycopy`, or
`Arrays.fill`, or any generic-over-element-type utility — you would need one overload per
element type, forever. Covariance was the only way to have a reusable array API in a
language with no parametric polymorphism. It was a deliberate, well-understood unsoundness
traded for the existence of `java.util.Arrays`.

**2.** A **store check on every `aastore`**: the VM loads the runtime element type from the
array header and checks that the stored reference is an instance of it, throwing
`ArrayStoreException` otherwise. That is a load of the array's class, a load of its element
class, and a subtype test, on every single reference store into any array — including
`String[] s; s[0] = "x";` where nothing can possibly go wrong. JITs claw it back by
speculating: if the array's exact type is known at the store (from an allocation in the same
compilation unit, or from a profile plus a guard), the check folds away. That works well in
hot loops and not at all across a non-inlined call, so the residual cost is real and is
part of why `ArrayList<T>` with an `Object[]` backing store can outperform naïve array code
in some shapes — the erased array's element type is `Object`, so the check trivially passes
and the JIT eliminates it easily.

**3.** Because removing it would break every program written since 1995, and there is no
migration path: the fix is not a new API alongside the old one, it is a change to the
subtype relation, which invalidates existing *compiled* code and existing source
simultaneously. Java's compatibility rule forecloses it permanently. This is the general
lesson worth stating out loud: a soundness hole in the *subtype relation* is unfixable in a
way that a soundness hole in a library is not, because the subtype relation is the thing
every other decision was made relative to.

**Trap.** "It's safe because of the runtime check." It is *memory*-safe. It is not type-safe
in any useful sense: the static type said this store could not fail, and it failed. The
distinction matters because the check catches the error at the store, arbitrarily far from
the covariant assignment that caused it, which is exactly the debugging property a type
system is supposed to buy you out of.

### A7 — Where variance is declared

**1.** Declaration-site variance puts the decision on the **type author**, once, at the
declaration: `Source<out T>` says "this type is a producer of `T` and will never consume
one", and every use site gets the subtyping for free. Use-site variance puts it on the
**consumer**, at every signature: the author of `List<T>` says nothing, and each API that
wants covariance writes `? extends`. Declaration-site is the only one that expresses
cleanly a type that is *inherently* a producer or consumer — `Iterator<out T>`,
`Comparator<in T>`, `Function<in A, out B>` — because the property belongs to the type, and
under use-site variance the author cannot say it, so every one of their users must
re-derive and re-write it. Java's `Function<? super T, ? extends R>` littering its own
standard library is that failure, visible.

**2.** **Capture conversion.** `List<? extends Number>` is an existential: "there exists
some type `X <: Number` such that this is a `List<X>`". At each use the compiler *captures*
the wildcard into a fresh, unnameable type variable `CAP#1 <: Number`. `get()` returns
`CAP#1`, which is safely widened to `Number`. But `add(E)` requires an argument of type
`CAP#1` exactly, and you cannot produce one — you do not know what `CAP#1` is, and a
`Number` is not necessarily a `CAP#1`. So it is not conservatism: the parameter type is a
type variable you have no way to inhabit. That is also why `List<?>` accepts `add(null)`
and nothing else: `null` inhabits every reference type.

**3.** Because variance is only sound when the parameter occurs in consistently-signed
positions, and a class **has fields**. A field of type `T` is both readable (covariant) and
writable (contravariant), so it forces `T` invariant — a class with a `T` field can never
be declared `out T`. Interfaces and delegates have no state, so every occurrence of `T` is
in a method signature where the sign is checkable. There is a runtime dimension too: the
CLR reifies generics, so variance has to be enforceable by the runtime's cast machinery,
and variant *interface* casts were a bounded change to that machinery in a way that variant
class layouts would not have been.

### A8 — Erasure and reification

**1.** Erasure bought **migration compatibility**: `List` and `List<String>` are the same
runtime type, so pre-generics code and generic code interoperate in both directions, a
generified library keeps its old binary signature, and the entire existing ecosystem kept
working without recompilation. That was the explicit design goal and it was probably right
in 2004. The permanent cost is that **types are not available where decisions are made at
runtime**: you cannot dispatch on, reflect over, allocate for, or specialize on a type
parameter. Everything downstream — no `new T[]`, no `instanceof List<String>`, no
overloading on `List<String>` vs `List<Integer>`, no unboxed `List<int>` — is a consequence
of that one fact. Project Valhalla's difficulty is that same bill, twenty years later.

**2.** **Specialization over value types with no boxing.** The CLR JIT generates distinct
native code per value-type instantiation, so `List<int>` stores raw `int`s in a raw `int[]`,
and `Dictionary<int, long>` has no per-element object header, no allocation, and no pointer
chase. Reference-type instantiations share one code body, so the code-size cost is bounded.
Erasure structurally cannot have this: with one code body for all instantiations, the
element storage must be a uniform representation, which for anything primitive means boxing.
That is the actual performance gap between `List<T>` in the two ecosystems, and it is not
recoverable by a better JIT. What reification forecloses: erasure's migration story (the
CLR shipped a whole parallel `System.Collections.Generic` namespace instead), and cheap
retroactive change to the generic system, since instantiations are baked into metadata and
the runtime must agree with the compiler about them.

**3.** A class implementing `Comparable<String>` writes `compareTo(String)`. The interface
method, after erasure, is `compareTo(Object)`. Those are different descriptors, so the JVM
would not see an override — the class would be abstract and virtual dispatch through the
interface would find nothing. The compiler therefore synthesizes a **bridge method**
`compareTo(Object)` that casts its argument to `String` and delegates. The cast is the
important part: it is the checkcast that fails when heap pollution has put an `Integer`
into a `List<String>`. So the `ClassCastException` surfaces *inside a synthetic method
nobody wrote*, at a line that does not exist in the source, arbitrarily far from the
unchecked operation that caused it. That is the debugging signature of erasure, and
recognizing it in a stack trace is a real skill.

**Trap.** "Erasure means Java throws generic type information away." It keeps a great deal
of it — generic signatures survive in the `Signature` attribute of the class file, which is
how `javac` type-checks against a compiled library and how frameworks read
`List<String>` off a field or method via reflection. What is erased is the type of an
*instance*: the class file knows the field was declared `List<String>`, and the object on
the heap does not. Getting this backwards leads to confident wrong claims about what
reflection can and cannot see.

### A9 — Subtyping meets parametric polymorphism

**1.** Unification returns a substitution — a function from variables to types — and
composing substitutions is closed, which is what makes "the principal type" a type. With
subtyping the solver returns a **set of inequality constraints** (`α <: Number`,
`String <: β`, `α <: β`), and the honest principal result is the original type *together
with* that constraint set. That object is not a type in the surface language; it is a
constrained type scheme, and it grows with the size of the term. Systems that do infer them
(MLsub, algebraic subtyping) must aggressively simplify or the inferred types become
unreadable and unprintable. So you either publish an incomprehensible type, or you pick
some non-principal instance of it and lose the guarantee that the inferred type is the most
general one. Every practical language picks the latter and demands annotations.

**2.** **F-bounded polymorphism** (`<T extends Comparable<T>>` — the bound mentions the
variable) plus **wildcards with capture** (contravariant occurrences producing fresh
existentials). Together they let a subtyping query on one type spawn subtyping queries on
freshly constructed types of greater depth, with no bound on the depth, so the subtype
checker's recursion can encode unbounded computation — Grigore showed you can compile a
Turing machine into Java class declarations such that `javac`'s subtype check simulates it.
Kennedy and Pierce had earlier shown that nominal subtyping with variance plus expansive
inheritance is undecidable in general. Real compilers survive by imposing a depth cutoff and
reporting an error, which is why `javac` and Scala's checker can both report "cyclic" or
depth-limit errors on pathological hierarchies.

**3.** It buys the checker **the equality solver**. With no subtyping, trait resolution and
generic inference are unification problems, so they have most general solutions, terminate
predictably, and produce errors that name a single conflicting pair. It also buys
monomorphization: with no subsumption, a generic function at a concrete type has exactly
one meaning, so it can be compiled as if hand-written. The bill: no implicit widening
anywhere. `&Vec<T>` does not become `&[T]` by subtyping — Rust needs `Deref` coercions,
`AsRef`, `Into`, and a hand-written coercion table in the compiler to recover the
ergonomics, plus explicit `dyn Trait` with unsizing coercions to get anything resembling
subsumption. Rust did not eliminate subtyping's complexity; it moved it out of the type
relation and into a pile of named, individually-specified coercions — which is a defensible
trade precisely because each one is nameable and none of them recurse.

### A10 — Type classes are not interfaces

**1.** **Return-type polymorphism** — dispatch on a type that appears nowhere in the
arguments. `mempty`, `read`, `fromInteger`, `Default::default()`, `"x".parse::<T>()`. The
mechanism: an interface selects the implementation from *a value* (the receiver's runtime
class), so there must be a value; a type class selects the implementation from *a type*,
resolved statically at the call site from the expected type. That is why `mempty :: Monoid a
=> a` is callable with no arguments and `interface Monoid<A> { A empty(); }` is not — the
Java version needs an instance of the very thing it is trying to construct.

**2.** A vtable comes from the **object**: it is a pointer in the object header, installed
at allocation, so it can only be read by first having the value, and its contents are not
known until you know the runtime class. A dictionary comes from the **call site**: the
compiler resolved `(Monoid a)` at instantiation and passes the record of functions as an
extra argument. Because it is an argument determined statically, monomorphization or
specialization can constant-fold it: GHC's `SPECIALISE`/inlining and Rust's
monomorphization replace the dictionary with direct calls and then inline them, and the
dictionary vanishes entirely. The vtable cannot vanish unless the compiler can prove the
receiver's exact class, which in an open world it usually cannot — hence speculation and
guards rather than elimination. This is why `dyn Trait` and `impl Trait` have genuinely
different performance in Rust while looking similar in source.

**3.** Without coherence you can have two `Ord` instances for the same type in one program.
Insert into a `Set<T>` under one ordering in module A, look up under the other in module B:
the set's internal invariant was established under an ordering the lookup does not use, and
the lookup silently returns "not found" for an element that is present. No type error, no
runtime error, wrong answer. The **orphan rule** (an impl must be in the crate defining
either the trait or the type) is what makes coherence checkable *modularly* — without it,
two crates that never see each other can each define a valid impl and the conflict only
exists in a program that links both. Scala's implicits have no coherence: resolution is
lexically scoped, so the same expression can mean different things in different files. In
exchange Scala gets local, ad-hoc instances — you can supply an ordering for a type you do
not own, in the scope where you need it, with no newtype wrapper. Rust makes you write the
newtype. That is the trade, and Scala 3's `given`/`using` redesign tightened the scoping
rules without adopting coherence, because coherence would have broken the ecosystem.

**Trap.** "Dictionary passing is just a vtable passed explicitly." Operationally similar,
but the dictionary is chosen by a *static* type and the vtable by a *dynamic* value, and
that single difference is what gives type classes return-type polymorphism, multi-parameter
dispatch, and full erasure at monomorphization — none of which an interface can have.

### A11 — What crossing the boundary costs

**1.** Because the value is a **function**, and you cannot check a function's type by
inspecting it — its type is a statement about all the calls it will ever participate in.
So the boundary creates a **wrapper** (a proxy / contract / chaperone) that checks the
argument on the way in and the result on the way out, at every call, forever. Costs: an
indirection and two checks per call, so the wrapped function is permanently slower than
either side would have been; the checks are themselves higher-order if the arguments are
functions, so wrappers *nest* as values cross back and forth and you get a value wearing
twenty layers; and it breaks **`eq?`/reference identity**, because the wrapped function is
not the same object as the original — which shows up as a bug in any code that memoizes on
identity or uses a function as a hash key. Racket's chaperone/impersonator machinery exists
specifically to bound the identity damage.

**2.** Operationally: without blame, the error tells you a check failed at a boundary, and
in a program with wrappers on wrappers you have no idea which *module* violated its
contract — the failure is reported at the innermost check, which is typically in library
code far from the untyped module that supplied a bad value. Blame makes the error
actionable by naming a party, and "well-typed programs can't be blamed" (Wadler and
Findler) is the theorem that makes the naming meaningful: the fully typed side is never
blamed, so blame always points at code a human has to fix. It is expensive because every
wrapper must carry a label, labels must be *swapped* when a wrapper is applied
contravariantly (an argument crossing inward blames the other party), and the labels have to
survive composition — so the wrapper's payload grows with the boundary history, not just the
type.

**3.** (a) **Transient / shallow checking** (Reticulated Python's transient semantics):
check types at each *use* rather than wrapping values at boundaries. No wrappers means no
per-call indirection, no identity damage, and constant-factor overhead — but you lose
precise blame (a failure names a use site, not a violating party) and you lose the
guarantee that a typed function's argument was ever checked at all against a deep type.
(b) **No runtime checks at all** — TypeScript, mypy, Sorbet in most configurations, Flow:
types are erased and the boundary is unchecked. Zero cost, zero soundness; a value annotated
`Int` from an untyped source can be a string, and every downstream use is running on a lie.
A third answer worth naming: **nominal gradual typing** (Nom), where casts are O(1)
nominal tag checks rather than structural or higher-order ones, which shows the catastrophic
overheads were partly a consequence of *structural* types rather than of soundness itself.

### A12 — Unsound on purpose

**1.** Because of **event handlers and callbacks with narrower parameter types**, which is
essentially all of DOM and Node code:

```ts
elem.addEventListener("click", (e: MouseEvent) => { ... })
```

`addEventListener` wants `(e: Event) => void`. A `MouseEvent`-taking handler is *not* a
valid `Event`-taking handler under contravariance — it would be rejected. The idiom is
ubiquitous, correct in practice (the runtime really does pass a `MouseEvent` for `"click"`),
and cannot be expressed soundly without overloads or dependent parameter types. TypeScript's
mandate was to type existing JavaScript, and existing JavaScript is full of this. So
bivariance was the price of the language being adoptable at all.

**2.** The argument is about **inheritance and interface implementation**. Method
declarations are how classes implement interfaces and how subclasses override, and strict
contravariance would make a large fraction of existing hierarchies fail — most notably,
`Array<T>` is covariant in TypeScript, and its methods (`push`, `indexOf`) take `T` in
parameter position, so contravariant methods would make array covariance immediately
inconsistent. Rather than fix array covariance (unadoptable) or make methods
contravariant (unadoptable), they scoped the strictness to function-typed *properties*,
where the pattern is rarer and where the user wrote a function type explicitly and can
reasonably be held to it. It is a deliberately drawn line between "syntax people use to
describe a callback" and "syntax people use to declare a class member".

**3.** **Dart pays with runtime checks.** Dart 2 kept covariant generics (`List<Cat>` is a
`List<Animal>`) for exactly Java's reason — ergonomics and existing code — and recovered
soundness by inserting checks at method entry on covariantly-used parameters, so a bad call
throws a `TypeError` at the boundary. That is the same deal as Java's `ArrayStoreException`
generalized from arrays to all generics.

Ranking by what was bought: **Java arrays** bought the most — the existence of a generic
array API in a language with no generics, an actual impossibility otherwise — and pays with
a per-store check plus errors far from their cause. **Dart** bought conventional
object-oriented ergonomics in a language that wanted to be sound, and pays honestly with
localized runtime checks and localized errors. **TypeScript methods** bought adoptability
of the entire existing JavaScript ecosystem and pays with *nothing at runtime*, which means
the hole is real and silent — the strongest thing bought and the weakest recovery. Note the
pattern: the two systems that get away with it are the two that check at runtime; the one
that does not is the one that had already given up soundness elsewhere and had nothing more
to lose.

### A13 — What a dynamic runtime is actually buying

**1.** In CPython, roughly: load `a` (a pointer), dereference to its `ob_type`, load the
type's number-protocol table, load the `nb_add` slot, call it; inside, re-check both
operand types, unbox both values out of their objects, do one machine `add`, then
**allocate** a result object, initialize its header and refcount, and return the pointer.
Plus a reference-count increment/decrement on each operand. So: five or six loads and one
allocation where the static version has one instruction. The count is not the story —
**they are dependent**. Each load's address comes from the previous load's result, so the
chain cannot be overlapped by the out-of-order engine; a modern core can issue several
independent loads per cycle and gains nothing here. You are paying latency, not throughput,
and latency does not amortize.

**2.** It gets the *semantics* right: a dynamically typed language is a statically typed
language with one recursive sum type, and its "type errors" are pattern-match failures in
the injection/projection of that sum. This is genuinely clarifying — it explains why
untyped languages are not "unityped by accident", and why gradual typing works by refining
that one type. What it hides is that the single type has a **runtime representation cost**
that the framing makes invisible. Every value must be tagged; every field must be uniform
width; every object must carry a shape; a struct of three floats cannot be three floats.
The implementer's problem is not the type discipline, it is that the universal type
forecloses layout decisions — which is exactly what the theory abstracts away.

**3.** **Representation and layout.** An inline cache removes the *lookup*; it does not
remove the fact that the value in the field might change type tomorrow, so the field cannot
be an unboxed 8-byte float in an array of structs. You keep pointer chasing, per-object
headers, poor cache density, and no vectorization, because none of those are decisions a
guard can make locally — they are decisions about a whole heap made before any code ran.
On top of that, speculation has its own irreducible costs: guards on every specialized
site, deoptimization metadata proportional to the optimized code, a code cache, and a
compiler resident in the process. This is why the fastest dynamic runtimes get within a
small factor of C on scalar-dispatch-heavy code and stay far away on data-layout-heavy
code — and why the interesting work is object shapes, hidden classes, and unboxed storage
rather than better caches.

**Trap.** "Static types make programs faster." Types as *written by the programmer* do
nothing; what makes programs faster is the compiler *knowing* the representation, and it can
learn that from inference, from profiles, or from speculation just as well as from an
annotation. TypeScript's types are erased and buy zero performance. Conversely a JIT that
proves a field is always a small integer gets the layout win in a language with no type
annotations at all. The performance argument for static types is really an argument for
*checked declarations the code generator is allowed to rely on* — which is why it applies to
Rust and C# and not to TypeScript or mypy.

### A14 — `any`, `unknown`, `never`

**1.** `never <: T <: unknown` for all `T` — `never` is the bottom type (the empty set of
values, so vacuously a subtype of everything), `unknown` is the top type (every value, so a
supertype of everything, and nothing can be done with it without narrowing). `any` is
assignable **to** everything and **from** everything. That makes it simultaneously a top and
a bottom, which is only consistent in a one-element lattice. Concretely it destroys
transitivity of assignability as a useful property: `string → any → number` is a valid chain,
so if you accept `any` as a lattice element you have proved `string <: number`. `any` is not
a type in the order; it is a hole punched in the order, and TypeScript's checker special-
cases it rather than positioning it.

**2.** (a) **`T | never = T`** — `never` is the identity of union, so it disappears from
unions and from the type of a `filter`ed array. (b) **The return type of a function that
never returns** is `never`, because the set of values it can return is empty — hence
`function fail(): never { throw ... }`, and hence control-flow analysis knows code after a
`never`-returning call is unreachable. (c) **Exhaustiveness checking falls out for free**:
after narrowing a discriminated union in a `switch`, the scrutinee in the `default` branch
has the union of the un-handled cases, which is `never` iff all cases were handled — so
`const _: never = x` in the default is a compile error exactly when you forgot one. All
three are the same fact.

**3.** `any` is the only type that is assignable in **both** directions, and that is what
makes **incremental migration** possible. Typing one file of a large JavaScript codebase
means its boundary values come from and go to untyped code; with `unknown` every one of
those crossings needs an explicit narrowing at the call site, so adopting types in one file
forces edits in files you did not want to touch. `any` localizes the cost to zero. It is
also the escape hatch for expressing things the type system genuinely cannot — variance
workarounds, higher-kinded encodings, `Function`, and reflective code — and the pragmatic
reality is that a system without a hole is a system people cannot adopt into an existing
codebase. `unknown` is the *correct* top type and its job is to be what you reach for in
new code; retiring `any` would have retired the migration story that made TypeScript win.

### A15 — Untagged unions and the code generator

**1.** Untagged means the union type adds **no runtime representation**: a `string | number`
value is just the string or just the number, with nothing attached saying which. So the
only way to tell them apart at runtime is to ask the *value* — `typeof x === "string"` —
which requires the host runtime to already carry a type tag on every value. TypeScript can
have untagged unions precisely because JavaScript already tags everything. A language
compiling to unboxed data cannot: `i32 | f32` has no way to discriminate two four-byte
values, so an untagged union either forecloses unboxed representations entirely or
forecloses discrimination (which is what C unions do, and why C unions are unsafe). That is
the whole reason ML-family languages and Rust chose *tagged* variants: the tag is what makes
the values unboxable.

**2.** Rust's enum is a tag plus a payload, but the compiler looks for an **invalid bit
pattern in the payload — a niche —** and encodes the tag into it. `&T` is never null, so
`Option<&T>` uses the all-zeros pointer as `None` and needs no separate tag word; the same
applies to `NonZeroU32`, `bool`'s 254 unused byte values, and enums with unused
discriminants. What it forecloses: **layout guarantees**. You cannot rely on where the tag
is, you cannot `transmute` an enum meaningfully, and you cannot pass one across FFI without
`#[repr(C)]` or `#[repr(u8)]`, which switch the optimization off. The general shape here is
worth naming — every representation optimization is paid for in ABI freedom.

**3.** `A | B` erases to the **least upper bound of the erasures** — usually `Object`.
Consequences: (a) you cannot overload on unions, because `f(A | B)` and `f(Object)` have the
same JVM descriptor, so the two methods clash; (b) pattern matching on `A | B` requires a
synthesized sequence of `instanceof` tests, so the discrimination the type system knew
statically has to be re-derived at runtime — and it cannot be derived at all for erased
generic components, so `List[Int] | List[String]` is undiscriminable. Union types on a
nominal, erased runtime are a static-only convenience; the moment you want to *branch* on
one you are back to the host's runtime type information and its limits.

### A16 — Narrowing, and what unnarrows it

**1.** `g` can be a closure over `o` that does `o.x = null`. TypeScript's narrowing of
`o.x` survives the call, so the subsequent `o.x.length` is a null dereference the checker
approved. The reason it is accepted: the sound rule is "invalidate every narrowing of every
property on every function call", and the vast majority of real code calls functions
between a check and a use. Under the sound rule, ordinary defensive code becomes a forest
of re-checks or local temporaries, and the checker would be blamed for the noise rather than
credited for the safety. TypeScript chose a rule that is wrong in a case people rarely hit
over a rule that is right in a case people hit constantly. Note the shape: the unsoundness
is not laziness, it is a considered bet on the distribution of real programs.

**2.** Kotlin smart-casts only when it can prove **no intervening write between the check and
the use**. It refuses for: a `var` local captured *and modified* by a lambda; any `var`
property; any property with a custom getter (the getter can return different values on
successive calls); and any property — even a `val` — declared in another module, because
Kotlin cannot rely on cross-module compilation to have kept it a `val` and cannot see a
custom getter added later. Open `val`s with overridable getters fall in the same bucket. The
property Kotlin demands is **stability**: the expression must be guaranteed to evaluate to
the same value on the second read as on the first. TypeScript demands nothing; it just
assumes it. Kotlin's rule is why you write `val x = maybeNull ?: return` so often — you are
manually creating a stable binding.

**3.** Typed Racket **checks the predicate's body** against the proposition it claims. A
function declared `(-> Any Boolean : String)` must actually be proved to return true only
for strings; the latent proposition is derived from and verified against the implementation,
so the narrowing it licenses is sound. TypeScript **trusts** `x is T` — the body is checked
only as an ordinary boolean-returning function, so `function isString(x: unknown): x is
string { return typeof x === "number" }` compiles and lies to every caller. What Typed
Racket's version costs: the checker must reason about the *logical* content of arbitrary
boolean expressions, which means a proposition language, `and`/`or`/`not` on propositions,
and inference through them — a substantially larger and slower checker, and one that will
fail to verify predicates that are correct for reasons it cannot express. TypeScript's
version costs nothing and buys nothing; it relocates a cast and calls it a check.

**Trap.** "Narrowing is just flow-sensitive typing, which is standard dataflow analysis."
The dataflow part is easy. The hard and language-defining part is deciding *what
invalidates a fact*, and that is not a dataflow question — it is an aliasing and
effect-tracking question, which is why every language answers it differently and why the
sound answer is unaffordable.

### A17 — Effects are the colouring problem with a type on it

**1.** `map`'s signature is `<R> Stream<R> map(Function<? super T, ? extends R> f)`, and
`Function.apply` is declared to throw nothing. There is no way for `map`'s return type or
throws clause to *depend on* the exceptions of its argument, because `throws` clauses are
not part of the generic system — you cannot write `<R, E extends Exception> Stream<R> map(
ThrowingFunction<T, R, E> f) throws E` and have it compose (you can approximate it for a
single exception type, which is exactly the trick libraries use, and it collapses as soon as
you need two or zero). The missing feature is **effect polymorphism**: the ability to
quantify over an effect and have a higher-order function's effect be a function of its
argument's. Java's checked exceptions are an effect system with no polymorphism, which is
why they compose so badly with lambdas that the standard library's own functional interfaces
declare no exceptions at all.

**2.** Cost in signatures: every higher-order function grows an effect variable, and every
concrete function grows an effect row, so `map : (a -> b <e>) -> list<a> -> list<b> <e>` is
the *simple* case. Users see and must sometimes write effect variables that are pure
plumbing. Cost in inference: effect rows need row unification with row-polymorphic
variables, and the inferred rows must be simplified and printed, which is the same
readability problem as constrained types in A9. OCaml 5 shipped handlers untracked because
the alternative was to colour the entire existing ecosystem: adding an effect to a type
would have made every signature in every library incomplete, and every higher-order function
in `Stdlib` would have needed effect polymorphism retrofitted. They took **unchecked
effects** — an unhandled effect is a runtime error, `Effect.Unhandled` — as the price of
adding effect handlers to a twenty-five-year-old language without a flag day. That is the
same trade as TypeScript's: soundness sacrificed for adoptability, made by people who knew
exactly what they were giving up.

**3.** Unifying them would let you write a *single* `map`, `Iterator`, and trait definition
that works in async, const, fallible, and unsafe contexts, instead of the current parallel
universes (`Iterator`/`Stream`, `Fn`/`AsyncFn`, `const fn` duplicates, `try_` variants) —
this is the "keyword generics" / "effect generics" work. The concrete blocker is that the
effect variable has to appear on **traits, trait methods, closures, and every bound**, and
Rust monomorphizes: each effect instantiation is a distinct compiled body, so the
combinatorics multiply against existing generic parameters. It also collides with
coherence — is `impl<E> Trait for T` with an effect variable one impl or many? — and with
the object-safety rules for `dyn Trait`. The feature is not blocked on wanting it; it is
blocked on the fact that Rust's other two commitments (monomorphization, coherence) each
multiply its cost.

### A18 — Soundness, completeness, and which one you sell

**1.** **Soundness**: if the checker accepts a program, the program does not go wrong —
formally, progress (a well-typed term is a value or can step) plus preservation (stepping
preserves the type). No false negatives. **Completeness**: if a program does not go wrong,
the checker accepts it. No false positives. Now: "does not go wrong" — say, "never
dereferences null at runtime" — is a non-trivial semantic property of programs, so by
Rice's theorem it is undecidable for a Turing-complete language. A checker is by
construction a total, decidable procedure. A sound *and* complete checker would decide that
property. Therefore no such checker exists, and every real system chooses which side of the
approximation to land on.

**2.** Practical checkers give up **completeness**: they reject programs that would in fact
never fail, and the language is designed so that the rejected ones are rewritable. The
`if (never_true())` example is the trivial case; the real cases are things like a `match`
the compiler cannot prove exhaustive, or a borrow the checker cannot prove disjoint (Rust's
pre-Polonius rejection of correct programs is the canonical modern example, and NLL was
literally a project to recover completeness without losing soundness). The two systems that
gave up **soundness** instead: **TypeScript**, whose stated non-goal is to be sound because
the goal is to type existing JavaScript and productivity was ranked above correctness; and
**Java's arrays**, where soundness was traded for the existence of a generic array API in a
language without generics. Both were deliberate, both are documented, and both were
correct decisions given their constraints — which is the point of stating soundness
precisely rather than treating it as a virtue.

**3.** For the user it depends almost entirely on whether the unsoundness is **local and
predictable**. `any` in TypeScript is a single, named, greppable construct: you can find
every one, and its failure mode is confined to values that flowed through it. Java's array
covariance is a single hole too, but its failure mode is *action at a distance* — the
error is thrown at a store, possibly in library code, arbitrarily far from the covariant
assignment that made it possible, with a stack trace that does not contain the mistake. Ten
holes that each fail immediately and locally are more manageable than one hole that fails
late. The deciding property is not the *count*, it is **whether the unsoundness composes**:
a hole that can be laundered through generics, or through an inferred type, or through a
narrowing, stops being one hole and becomes a property of the whole program. That is the
question to ask of any proposed exception — not "how big is the hole" but "can a value that
went through it be indistinguishable from one that did not."

**Trap.** "Rust is sound, so Rust has no holes." Rust's guarantee is conditional on `unsafe`
blocks upholding their invariants, and soundness bugs in the checker itself have existed and
been fixed. The honest claim is that Rust's unsoundness is *confined to a syntactically
marked region*, which is a much stronger and much more interesting property than "sound" —
and stating it that way is the difference between having read about soundness and having
thought about it.
