The system is converging toward something unusually clean: not “an OO language with a static type checker,” and not “Haskell types grafted onto Smalltalk objects,” but one runtime value/object universe with a separate, mathematically disciplined semantic classification system that can itself be reified into that value universe.

The design should preserve that distinction aggressively.

My recommended north star is:

```text
                     RUNTIME / OBJECT AXIS

expression ──evaluate──▶ value ──.class──▶ class ──.class──▶ metaclass
                           │
                           │ may denote / reify
                           ▼
                     semantic entity


                     STATIC / SEMANTIC AXIS

value ─────────────── : ───────────────▶ proper type
                                             │
                                             │ ::
                                             ▼
                                            kind


                     REFLECTION BRIDGE

Class object Int ─────denotes─────▶ Int :: Type

Class object List ────denotes─────▶ List :: Type -> Type

AppliedType value ────denotes─────▶ List<Int> :: Type

AtomicKind value ─────reifies─────▶ Type

FunctionKind value ───reifies─────▶ Type -> Type
```

The language stays unified because types and kinds can become ordinary values. It stays mathematically sane because reification never makes:

```text
Class == Type
Type == Kind
.class == :
: == ::
Type :: Type
```

true.

That is the foundation from which I would derive essentially every remaining decision.

---

# 1. Core philosophical laws I would freeze

These should become language invariants rather than implementation preferences.

1. **Every observable Phalcom entity is a value and participates in the object model.** Some values are represented immediately by the VM and some by heap objects; representation does not change semantics.

2. **Classes are runtime behavioral objects, not synonymous with types.** A class object may additionally denote a nominal type or type constructor.

3. **Types classify values. Kinds classify type-level forms.** These relations are distinct from runtime `.class`.

4. **Every canonical type-level form has exactly one kind.** A `TypeId` may denote `Int`, `List`, `List<Int>`, a type parameter, a union, a callable type, etc. Whether it is a proper value type is determined by its kind.

5. **Only forms of kind `Type` classify ordinary runtime values.**

6. **Type metadata never participates in ordinary selector identity, ordinary method dispatch identity, runtime instance layout, or allocation semantics.**

7. **Reflection exposes existing semantics. It never reconstructs or defines the type system independently.**

8. **Static uncertainty is not a type.** `Unknown` remains analyzer knowledge. `Dynamic` remains an explicit static escape. Neither should become a counterfeit nominal class.

9. **Canonical semantic identity is structural and independent of presentation strings.**

10. **Type-level computation remains intentionally smaller than arbitrary Phalcom computation until dependent typing is explicitly chosen.**

Those ten rules eliminate a remarkable number of future traps.

---

# 2. Terminology: `TypeForm` is the right common abstraction

I recommend keeping the name we arrived at:

```text
TypeForm
```

rather than `TypeExpression`.

`TypeExpression` sounds syntactic. It suggests an AST construct.

But:

```text
Int
List
List<Int>
Int | String
(Int, String)
Int -> String
```

are semantic entities even when synthesized by inference and never literally written.

I would use this vocabulary consistently:

| Name | Meaning |
|---|---|
| `Type` | The atomic kind of proper value types |
| `TypeForm` | Semantic/behavioral role of something denoting a type-level form |
| `TypeDescriptor` | Future runtime implementation base for synthetic reflected type forms |
| `KindForm` | Future runtime behavioral abstraction for reflected kinds |
| `KindDescriptor` | Future runtime implementation base for synthetic kind values |
| `KindScheme` | Quantified classification such as `∀k. (k -> Type) -> k -> Type` |
| `TypeParameter` | Stable bound type-level variable |
| `KindParameter` | Stable generalized kind variable |
| inference variable | Ephemeral analyzer metavariable, never reflected |

Crucially:

```text
TypeForm
```

should not be inserted into the class inheritance tower.

Eventually it can be a protocol/capability implemented by several otherwise unrelated runtime objects:

```text
                          TypeForm
                      /      |       \
                     /       |        \
                class      synthetic   type
                object      descriptor parameter
                  │             │
                 Int        List<Int>
                 List       Int | String
```

The existing class/metaclass hierarchy remains untouched.

---

# 3. The kind system: Haskell semantics, but a much smaller language

For the first implemented kernel:

```text
K ::= Type
    | K -> K
```

is enough.

Examples:

```text
Int                 :: Type
String              :: Type

List                :: Type -> Type
Option              :: Type -> Type

Map                 :: Type -> Type -> Type

List<Int>           :: Type

Map<String>         :: Type -> Type
Map<String, Int>    :: Type

Higher              :: (Type -> Type) -> Type
```

I would continue storing arrow kinds canonically in n-ary form internally:

```text
Type -> Type -> Type

≈

Arrow(
    parameters = [Type, Type],
    result = Type
)
```

while presenting them right-associatively.

This gives easy prefix application while preserving the normal mathematical reading.

Haskell/GHC demonstrates how far this model can scale: with `PolyKinds`, constructors can generalize over kind variables, e.g. patterns like `(k -> Type) -> k -> Type`.

But I would stop far earlier than modern GHC initially.

Specifically, do not copy `DataKinds` yet. GHC can promote ordinary datatype constructors into the kind/type level, which is powerful but moves directly toward dependent type-level programming.

Phalcom should first become excellent at:

```text
Type
arrow kinds
HKTs
kind inference
prenex kind polymorphism
```

without admitting arbitrary promoted values or dependent normalization.

---

# 4. Kind polymorphism: yes, prenex, but keep three identities separate

I would ratify prenex kind polymorphism.

Eventually:

```text
Proxy     :: ∀k. k -> Type

Compose   :: ∀a b.
             (b -> Type)
          -> (a -> b)
          -> a
          -> Type
```

But there must be three separate notions:

```text
KindId
    canonical monomorphic kind structure

KindParameterId
    rigid variable bound by a generalized scheme

KindVarId
    ephemeral solver metavariable
```

Conceptually:

```rust
struct KindScheme {
    parameters: Box<[KindParameterId]>,
    body: KindId,
}
```

and solver state approximately:

```text
KindVarId
   ↓ solve
KindId / rigid KindParameterId structure
```

The important escape rule is:

```text
KindParameterId
    MAY enter declaration/interface schemes.

KindVarId
    MUST NEVER enter module interfaces,
    reflection metadata,
    compiled descriptors,
    or semantic snapshots as a finalized answer.
```

That is exactly the distinction GHC-style inference needs without making the public system depend on solver implementation.

I would not implement this during the current two-axis tower work. But I would architect around it.

---

# 5. Do not copy Scala's `AnyKind` representation

Scala 3 supports abstraction across arbitrary kinds through `AnyKind`. Scala describes it as a special synthesized top-like type compatible with all kinds, but heavily restricts what an any-kinded parameter may do.

It works within Scala's type hierarchy.

I would not reproduce that encoding.

Phalcom should prefer:

```text
kind schemes
kind parameters
kind unification
```

over:

```text
special supertype representing every kind
```

Why?

Because Phalcom has already cleanly separated:

```text
subtyping:
    relationships between proper types

kinding:
    classification of type-level forms
```

Making kind polymorphism emerge from a special subtype lattice would unnecessarily intertwine those relations again.

Take Scala's ergonomics, not that representation.

---

# 6. Generic declaration syntax

The most natural surface syntax is still:

```phalcom
class Box<T> {
}
```

with ordinary type parameters defaulting to:

```text
T :: Type
```

Therefore:

```phalcom
class Box<T>
```

means semantically:

```text
Box :: Type -> Type
```

Variance should use the already-ratified syntax:

```phalcom
class Producer<+T> {
}

class Consumer<-T> {
}

class Cell<T> {
}
```

meaning:

```text
+T   covariant
-T   contravariant
 T   invariant
```

I strongly prefer this to `out T` / `in T`.

It is compact, mathematically recognizable, and mirrors Scala's useful declaration-site variance without importing Scala's verbosity.

---

# 7. Higher-kinded parameter syntax

For a higher-kinded parameter I recommend:

```phalcom
class Transformer<F: Type -> Type> {
}
```

and:

```phalcom
class Higher<H: (Type -> Type) -> Type> {
}
```

This uses `:` contextually as “classified by.”

That produces a nice conceptual symmetry:

```phalcom
value: Int
```

means:

```text
value is classified by type Int
```

whereas:

```phalcom
F: Type -> Type
```

inside a type-parameter binder means:

```text
F is classified by kind Type -> Type
```

In formal specification we can continue writing:

```text
value : Int
F :: Type -> Type
```

to distinguish the levels visually.

The source language itself does not necessarily need a second `::` annotation operator.

I prefer that economy.

---

# 8. Bounds and constraints should not overload kind syntax

Once:

```phalcom
F: Type -> Type
```

means kind ascription, do not also make:

```phalcom
T: Number
```

mean subtype constraint.

That becomes confusing immediately.

I recommend putting semantic constraints in `where` clauses.

Conceptually:

```phalcom
class Sorted<T>
  where T <: Comparable {
}
```

and perhaps protocol constraints:

```phalcom
class Encoder<T>
  where T conforms Serializable {
}
```

Then the grammar cleanly distinguishes:

```text
<T>
    generic binder

<F: Type -> Type>
    kind annotation

<+T>
    variance

where T <: U
    subtype bound

where T conforms P
    protocol constraint
```

That gives us room for multiple constraints without torturing the parameter-list grammar.

This is a proposal, not something I would silently bake into the implementation before you approve it.

---

# 9. Higher-kinded variance: restrict it initially

I would allow:

```phalcom
class Box<+T>
```

where:

```text
T :: Type
```

but initially reject:

```phalcom
class Wrapper<+F: Type -> Type>
```

Higher-order variance is considerably less obvious.

Named higher-kinded parameters should initially be invariant.

Similarly, type-lambda binders should not carry `+`/`-` initially. Scala 3 likewise does not permit variance annotations on type-lambda parameters.

We can later introduce higher-order variance only if there is a concrete use case.

---

# 10. Variance validation must be semantic, not trusted declaration text

A declaration saying:

```phalcom
class Producer<+T>
```

must not automatically make `T` covariant.

The compiler must verify that all instance-observable uses of `T` satisfy covariance.

For example:

```text
return position       +
parameter position    -
function parameter    composition reverses
function result       composition preserves
invariant constructor forces invariance
```

So:

```text
F<+T>
G<-T>
```

compose in the expected sign algebra.

This is one place where the mathematical model should actually guide implementation.

One subtle point: class-side methods should generally not determine the variance safety of instance specialization.

For example:

```text
List<+T>
```

could have a class-side factory accepting `T` without making instances of `List<T>` mutable.

Variance is fundamentally about substitutability of:

```text
List<A>
List<B>
```

as value types.

That distinction matters because Phalcom's class side is a different runtime receiver.

---

# 11. Type application: make it unary at the primitive semantic level

This is a refinement I strongly recommend.

Surface syntax:

```phalcom
Map<String, Int>
```

should conceptually lower to repeated unary application:

```phalcom
Map.<>(String).<>(Int)
```

not to a family of selectors:

```text
<>(_)
<>(_, _)
<>(_, _, _)
...
```

This is superior for several reasons.

First, there is one canonical type-application operation:

```text
<>(_)
```

Second, partial application falls out automatically:

```text
Map
    :: Type -> Type -> Type

Map.<>(String)
    :: Type -> Type

Map.<>(String).<>(Int)
    :: Type
```

Third, arbitrary constructor arity does not create selector families.

Fourth, the kind calculus and the object façade match precisely.

The compiler is free to batch:

```text
apply_type_form(Map, [String, Int])
```

internally for performance.

Semantically it is equivalent to left-to-right unary application.

This fits Phalcom unusually well.

---

# 12. `<>` should be a real but privileged operation

I still recommend that `<...>` correspond conceptually to a genuine operation:

```phalcom
List<Int>

≈

List.<>(Int)
```

But `<>(_)` should be non-overridable for type-denoting values.

Otherwise a user could make:

```phalcom
List.<>(Int)
```

execute arbitrary code and cease to mean canonical type application.

That would make static typing depend on executing arbitrary runtime behavior.

The correct compromise is:

```text
real selector
real reflected operation
ordinary value receiver

BUT

privileged semantic implementation
non-overridable
canonical
kind checked
```

That keeps one language without making the type checker an interpreter for arbitrary user programs.

---

# 13. Partial type application should be a real semantic value

I recommend eventually permitting:

```phalcom
Map<String>
```

as a valid type-level form.

Its kind is:

```text
Type -> Type
```

It simply cannot be used where a proper value type is required.

Thus:

```phalcom
const x: Map<String>
```

is an error:

```text
expected a proper type of kind Type
found type constructor of kind Type -> Type
```

while:

```text
const F = Map<String>
```

can eventually be valid once type-form values are runtime-reflectable.

That distinction is mathematically clean:

```text
existence of a TypeForm
    !=
eligibility to classify a runtime value
```

---

# 14. Type lambdas: support them, but delay choosing syntax

Phalcom eventually needs anonymous type constructors.

For example:

```text
λT. Result<T, Error>
```

with kind:

```text
Type -> Type
```

Scala 3 demonstrates the practical utility of making these explicit; its spelling is:

```scala
[X] =>> Map[String, X]
```


There are several plausible Phalcom spellings:

| Option | Example | Assessment |
|---|---|---|
| Scala-like | `[T] =>> Result<T, Error>` | Precise but stylistically foreign |
| Generic-like | `<T> => Result<T, Error>` | Compact, fits `<T>`, introduces `=>` |
| Explicit | `type<T> { Result<T, Error> }` | Clear but creates separate type-language feel |
| Reuse closure syntax | language-specific binder form | Most philosophically attractive if it stays unambiguous |

I would not ratify the spelling yet.

I would ratify the semantic form now:

```text
TypeLambda {
    parameters,
    body
}
```

with the restriction that type lambdas may close over type-level bindings, not arbitrary runtime values.

Otherwise Phalcom crosses from HKTs into dependent typing.

---

# 15. Do not allow arbitrary type-level execution

This deserves a hard boundary.

“One language” should not mean:

```phalcom
const x = readNetwork()
type T = if x { Int } else { String }
```

and therefore the compiler must execute arbitrary Phalcom to discover `T`.

That creates:

```text
compile-time side effects
nontermination
environment-dependent types
undecidable equality
nondeterministic builds
```

The unification principle should instead be:

> Type-level forms use the same semantic entities and object-facing operations as ordinary Phalcom values, but static normalization is restricted to a trusted, terminating type-form calculus.

Initially that calculus is:

```text
nominal forms
parameters
application
union
tuple
record
callable
kind arrows
eventually type lambdas
```

No arbitrary message send.

That is how Phalcom can remain unified without becoming accidentally dependent.

---

# 16. Type equality, identity, equivalence, subtyping, assignability and conversion must remain distinct

I recommend documenting five separate relations.

```text
canonical identity
semantic equivalence
subtyping
assignability
conversion
```

Canonical identity asks whether two forms normalize to the same canonical representation.

For example:

```text
Int | String
String | Int
```

should canonicalize identically.

Record field ordering similarly should not matter.

Tuple order absolutely should.

Applied forms should flatten:

```text
apply(apply(Map, String), Int)
```

and:

```text
apply(Map, [String, Int])
```

to the same form.

Semantic equivalence may eventually be broader than canonical structural identity—especially after aliases or more advanced normalization appear.

Subtyping is the pure mathematical relation:

```text
A <: B
```

Assignability is contextual:

```text
Can evidence/value known as A be accepted here as B?
```

It may involve:

```text
Dynamic
Unknown
proof authority
flow information
```

Conversion asks whether execution can explicitly transform one value into another representation.

Never make:

```text
Int assignable to Float
```

because an implicit numeric conversion exists.

Those are different relations.

---

# 17. Keep canonicalization independent of the inheritance hierarchy

A subtle implementation recommendation:

Do not make the core interner normalize:

```text
Int | Number
```

to:

```text
Number
```

merely because:

```text
Int <: Number
```

That would make canonical identity depend on hierarchy state and potentially module/loading context.

Canonicalization should remain structural:

```text
flatten
sort
deduplicate identical forms
eliminate Never where algebraically valid
normalize application grouping
normalize record order
```

Subtype-aware simplification belongs to the relation/proof layer.

That keeps `TypeStore` deterministic and cheap.

---

# 18. Generic application is not specialization

This distinction should remain absolute.

```phalcom
List<Int>
```

does not create:

```text
new runtime Class
new method dictionary
new metaclass
new class-side fields
new instance layout
```

It creates a semantic applied type form.

Eventually it may reify into an `AppliedType` descriptor value.

But:

```text
List<Int>.origin === List
```

and all runtime instance behavior still belongs to `List`.

This is one of the most important places where Phalcom can improve on languages that blur parameterized typing objects with runtime class machinery.

Python, for example, explicitly represents `list[int]` as a `GenericAlias` object whose `__origin__` is `list` and whose `__args__` are the supplied parameters; it is not itself a class.

Phalcom should preserve that useful separation while improving canonicalization and kind semantics.

---

# 19. Do not automatically forward class-side messages through `List<Int>`

This is a tempting feature I recommend rejecting.

One might want:

```phalcom
List<Int>.new()
List<Int>.empty()
```

to magically forward to:

```phalcom
List.new()
List.empty()
```

with generic substitution.

But that makes the synthetic type descriptor participate as a proxy in ordinary class dispatch.

It would blur exactly the boundary we have fixed:

```text
type metadata must not alter ordinary dispatch
```

Therefore I recommend:

```text
List
    runtime class object
    receives List class-side messages

List<Int>
    reflected type-form value
    receives TypeForm reflection messages
```

Not:

```text
List<Int> automatically masquerades as List
```

Instance creation remains an origin-class operation.

Static expected types can infer:

```text
List<Int>
```

for the resulting instance without changing runtime dispatch.

---

# 20. Runtime membership tests must respect erasure

Another trap to avoid:

```phalcom
x is List<Int>
```

looks attractive.

But ordinary runtime list instances do not carry `Int`.

Therefore Phalcom must not pretend it can dynamically prove this.

A runtime test:

```phalcom
x is List
```

can use runtime class behavior.

A runtime test:

```phalcom
x is List<Int>
```

cannot generally inspect erased generic arguments.

Options are:

| Model | Consequence |
|---|---|
| Pretend generic runtime checking exists | Unsound |
| Store generic tokens on every instance | Violates current runtime philosophy |
| Treat it as static-only refinement | Possible |
| Reject runtime parameterized `is` | Safest baseline |

I recommend the last two depending context: the static checker may use known semantic facts, but runtime `is` must only promise what runtime evidence can prove.

Do not add a universal:

```phalcom
TypeForm.matches(value)
```

until runtime evidence semantics are explicitly designed.

---

# 21. Record rows: yes, but record-specific

Closed structural records remain:

```text
#{name: String, age: Int}
```

with width subtyping:

```text
#{name: String, age: Int}
    <:
#{name: String}
```

That is good but insufficient for generic field-preserving transformations.

So I recommend a second atomic kind later:

```text
RecordRow
```

Then:

```text
R :: RecordRow
```

and a type such as conceptually:

```text
#{name: String, ...R}
```

means:

```text
known field name:String
plus some unknown remainder R
```

The overall record still has kind:

```text
Type
```

The row variable itself has kind:

```text
RecordRow
```

I would reuse stable `TypeParameterId` for row binders because it is already a general type-level binder with an associated kind:

```text
TypeParameterId(R)
kind(R) = RecordRow
```

But inference should have a distinct ephemeral:

```text
RowVarId
```

rather than pretending an unsolved row is a proper `TypeData::Infer`.

---

# 22. Row syntax options

I recommend eventually choosing:

```phalcom
#{name: String, ...R}
```

but I would not ratify it until syntax review.

Alternatives:

| Syntax | Strength | Weakness |
|---|---|---|
| `#{name: String, ...R}` | Matches value spread vocabulary | `...` gains type-level meaning |
| `#{name: String \| R}` | Mathematically concise | Confusable with union |
| `#{name: String, ..R}` | Familiar from row languages | New token convention |

`...R` is the strongest Phalcom choice because the semantic meaning really is “the rest of this structural record.”

---

# 23. Row extension must carry lacks constraints

Consider:

```text
#{name: String, ...R}
```

If `R` itself already contains `name`, blindly combining them is ambiguous.

The solver needs an implicit constraint:

```text
name ∉ R
```

or an explicit duplicate-field resolution law.

I recommend the former.

Known fields remain canonical and sorted.

The row tail remains separate.

Conceptually:

```rust
struct RecordType {
    fields: CanonicalFieldMap,
    tail: RecordTail,
}

enum RecordTail {
    Closed,
    Parameter(TypeParameterId),
    Infer(RowVarId),
}
```

Row unification handles:

```text
equality
extension
field extraction
lacks constraints
```

Do not use rows for nominal object fields.

---

# 24. Do not unify record rows, variant rows and effect rows prematurely

Mathematically they are all “row-like,” but their meanings differ:

```text
record row
    extensible unordered product

variant row
    extensible unordered sum

effect row
    extensible effect set
```

They may later reuse solver infrastructure.

They should not initially share one public `Row` kind.

I would rather have:

```text
RecordRow
```

and potentially later:

```text
VariantRow
EffectRow
```

than create one abstraction whose laws we haven't fully designed.

OCaml is a useful lesson here more generally: its module/functor subsystem remains deliberately distinct from ordinary type constructors; not every abstraction benefits from being collapsed into one generic mechanism. OCaml functors are module-to-module functions, not Haskell functors or higher-kinded types.

Phalcom should preserve that discipline.

---

# 25. Numeric literals: runtime spelling wins

Freeze this:

```phalcom
1
```

is `Int`.

```phalcom
1.0
```

is `Float`.

No expected type may silently reinterpret:

```phalcom
const x: Float = 1
```

as a different runtime value class.

If accepted at all, this must involve an explicit conversion rule that is visible in semantics.

Otherwise:

```phalcom
1.class
```

would depend on static context.

That violates the object model.

Haskell intentionally goes the other direction: numeric literals can be overloaded and unresolved numeric types may later default. GHC documents numeric type defaulting as a type-inference mechanism.

That is useful for Haskell's semantics.

It is not a good fit for Phalcom's reflective object identity.

---

# 26. Exact literal knowledge belongs in evidence, not ordinary types

The analyzer should still remember:

```text
ordinary type:
    Int

constant fact:
    exactly 1
```

Conceptually:

```rust
enum ConstantFact {
    Int(BigInt),
    Float(...),
    Bool(bool),
    Symbol(...),
    // ...
}
```

This enables:

```text
constant folding
branch proving
range inference
exhaustiveness
contract proving
refinement
```

without making every local integer type:

```text
1
2
42
```

a singleton `TypeId`.

If explicit refinement types arrive later, the analyzer can promote constant facts into them.

Until then:

```text
1 : Int
```

remains the public type.

---

# 27. Numeric promotion belongs to methods, not the subtype lattice

Do not define:

```text
Int <: Float
```

merely because arithmetic can mix them.

If Phalcom wants:

```phalcom
1 + 2.5
```

the declared operation surface can say conceptually:

```text
Int.+(Float) -> Float
```

or use a common numeric protocol.

That preserves:

```text
runtime message semantics
explicit class identity
normal subtype meaning
```

There is no hidden conversion relation masquerading as inheritance.

---

# 28. Functor and Monad should not be language primitives

Phalcom should support them brilliantly without knowing what they are.

A `Functor` or `Monad` is a generic abstraction over a constructor such as:

```text
F :: Type -> Type
```

The type checker needs higher kinds.

It does not need a hardcoded `Monad` concept.

Eventually users should be able to express the semantic equivalent of:

```text
Functor<F>
Monad<F>
```

using ordinary protocol/constraint machinery.

Haskell's `Constraint` kind is a useful conceptual model: it separates things that classify values (`Type`) from things that act as constraints. GHC also documents why unrestricted constraint machinery can cause type checking to loop, which is precisely why Phalcom should not immediately turn every predicate into unrestricted type-level computation.

My preferred eventual direction is:

```text
Type
RecordRow
possibly Constraint later
```

but do not introduce `Constraint` merely to make `Monad` look Haskell-like.

First settle protocol/constraint semantics.

---

# 29. Haskell typeclasses are not the only way to model Functor/Monad

This is important for Phalcom.

We should not reflexively copy:

```haskell
class Functor f where ...
class Monad m where ...
```

because Phalcom already has:

```text
classes
protocols
class-side behavior
attributes
contracts
reflection
```

The right representation may be a protocol/constraint over a type-form parameter rather than a separate typeclass mechanism.

Conceptually:

```phalcom
// illustrative, not ratified syntax

@protocol
class Functor<F: Type -> Type> {
    ...
}
```

or:

```text
where F satisfies Functor
```

The key design requirement is simply:

```text
F must have the expected kind
```

The core language should not care whether the protocol is called:

```text
Functor
Monad
Applicative
Parser
Container
Effect
```

That should be library-level abstraction.

---

# 30. Do not copy OCaml module functors as HKT substitutes

OCaml's module system is extremely powerful precisely because it provides another level of abstraction. Module expressions evaluate to modules, and functors map modules to modules.

The lesson for Phalcom is:

```text
type constructors
    Type -> Type

modules
    module abstraction

do not collapse them
```

Phalcom should have genuine HKTs and genuine modules.

Neither should simulate the other.

---

# 31. No monadic `do` syntax in the core type-system design

I would not add `do` merely because Monad becomes expressible.

If a later ergonomic feature is desirable, it should desugar through ordinary operations and perhaps leverage Phalcom's attribute/macro machinery.

The type system's job is:

```text
recognize F :: Type -> Type
validate constraints
infer applications
```

not to privilege one FP abstraction.

This is another useful lesson from Scala: FP patterns can live in libraries while the language provides the generic abstractions they need.

---

# 32. Totality: partial correctness by default

Ordinary callable semantics should be:

```text
A -> B
```

means:

> If execution returns normally, the result conforms to B.

It does not promise termination.

That is the correct default for a language with:

```text
general recursion
loops
dynamic sends
fibers
open-world native code
I/O
```

Totality becomes an additional contract requirement.

Conceptually keep separate:

```rust
enum TerminationRequirement {
    Partial,
    Total,
}

enum TerminationKnowledge {
    ProvenTerminates,
    Unknown,
    // potentially later:
    ProvenDiverges,
}
```

This is preferable to one enum mixing:

```text
what the signature requires
```

with:

```text
what the analyzer currently knows
```

The rule is then simple:

```text
Partial requirement
    no termination proof required.

Total requirement
    requires ProvenTerminates.
```

---

# 33. `Never` does not mean divergence

Freeze this distinction.

```text
Never
```

means:

> This expression has no normal returned value.

That can happen because execution:

```text
throws
diverges
terminates the process
performs a nonlocal exit
reaches an impossible path
```

These are semantically different.

So eventually a callable has orthogonal information:

```text
return type
effect summary
control-flow exits
termination requirement
termination evidence
```

Do not attempt to encode all of that into the return type.

---

# 34. Totality should probably be an attribute/contract property

If we eventually expose source syntax, an attribute fits Phalcom better than modifying selector or callable-type syntax.

Possible spelling:

```phalcom
@total
foo(...) {
}
```

or:

```phalcom
@terminates
foo(...) {
}
```

I prefer `@total`.

It says what the contract promises, not how the proof is obtained.

Crucially:

```text
@total
```

never participates in selector identity.

A function requiring totality can include that property in semantic callable conformance without changing runtime dispatch.

---

# 35. Proof artifacts: persistent and independently auditable

The prover should not merely emit:

```text
yes
```

and throw its reasoning away.

I recommend content-addressed persistent artifacts.

Something like:

```rust
struct ProofArtifact {
    obligation: VerificationConditionFingerprint,
    evidence: ProofEvidence,
    assumptions: Arc<[TrustBoundary]>,
    backend: BackendIdentity,
    backend_version: BackendVersion,
    semantic_version: SemanticModelVersion,
    checker_version: ProofKernelVersion,
}
```

with:

```rust
enum ProofEvidence {
    Certificate(Certificate),
    TrustedBackendAttestation(BackendAttestation),
    Counterexample(CounterexampleModel),
}
```

The trust status must remain explicit:

```text
KernelChecked
TrustedBackend
AssumedAxiom
```

These are not equivalent claims.

---

# 36. `Proven` must mean something precise

I would not allow the semantic database to flatten:

```text
solver returned UNSAT
```

and:

```text
small trusted kernel independently checked certificate
```

into one indistinguishable state.

Instead:

```text
ProofResult
    status
    trust level
    artifact
```

can say:

```text
proved / kernel checked
proved / trusted backend
assumed
disproved / counterexample
unknown
```

That is far more useful for serious verification tooling.

And proof caches must depend on:

```text
VC fingerprint
assumptions
referenced declaration fingerprints
semantic model version
solver version
proof-kernel version
```

not merely the text of the VC.

---

# 37. Proof artifacts may later be reflected—but remain evidence values

A runtime/tooling value might expose:

```phalcom
proof.status
proof.assumptions
proof.backend
proof.obligation
```

But that object is:

```text
a value representing evidence
```

not:

```text
a TypeId
a KindId
a proof term
a selector component
dispatch authority
```

Reification does not grant semantic authority.

Only a verified artifact or explicitly trusted policy does.

This follows exactly the same architecture as reflected types.

---

# 38. Do not introduce `Prop` or proof terms now

The full Curry–Howard direction:

```text
propositions as types
proofs as values
Prop
dependent function types
proof normalization
```

is a completely different scale of language commitment.

Nothing about Phalcom's current contract/proof aspirations requires it.

Keep the architecture open enough that a future proof-term system is not impossible.

Do not implement one by accident.

---

# 39. Runtime type reflection taxonomy

When reflection eventually lands, I recommend this object-facing architecture:

```text
EXISTING RUNTIME OBJECT MODEL
────────────────────────────────────

Object
  │
  ├── existing class hierarchy
  │      └── Class objects such as Int, List
  │
  ├── TypeDescriptor
  │      ├── AppliedType
  │      ├── UnionType
  │      ├── TupleType
  │      ├── RecordType
  │      ├── CallableType
  │      └── possibly TypeLambda
  │
  └── KindDescriptor
         ├── AtomicKind
         └── FunctionKind
```

The runtime classes above are implementation classes.

They are **not** the semantic type hierarchy.

For example:

```text
List<Int>.class == AppliedType
```

does not mean:

```text
List<Int> <: AppliedType
```

in the semantic type relation.

One relation classifies the descriptor object.

The other compares what that descriptor denotes.

---

# 40. Runtime protocols/capabilities

I would eventually expose:

```text
TypeForm
KindForm
```

as reflective protocols/capabilities.

Then:

```text
class object Int
class object List
AppliedType
UnionType
TupleType
CallableType
TypeParameter
```

can satisfy `TypeForm`.

While:

```text
Type
Type -> Type
RecordRow
```

as reflected kind values satisfy `KindForm`.

This allows reflective APIs such as:

```phalcom
inspectType(t: TypeForm) {
    t.kind
}
```

without putting `TypeForm` into the inheritance ancestry of `Class`.

---

# 41. Do not add a semantic kind-of-kind tower

This is a refinement from some earlier exploratory examples.

I recommend stopping semantic classification at kinds.

Thus:

```text
Int :: Type
List :: Type -> Type
```

but do not introduce:

```text
Type :: Kind
Kind :: SuperKind
...
```

unless a future dependent/universe design genuinely needs it.

The reflected runtime object:

```phalcom
Type
```

still has an ordinary runtime class:

```text
Type.class == AtomicKind
```

and ordinary static value type:

```text
AtomicKind
```

while denoting/reifying the semantic kind `Type`.

Likewise:

```text
(Type -> Type).class == FunctionKind
```

There is no need for:

```text
(Type -> Type).kind
```

in the semantic sense.

This avoids an infinite universe tower while preserving full runtime introspection.

---

# 42. Type reflection API

I would aim for an API approximately like this conceptually.

For every `TypeForm`:

```phalcom
t.kind
t.proper
t.equivalentTo(other)
t.subtypeOf(other)
t.display
```

For declaration-originating forms:

```phalcom
t.declaration
t.genericParameters
```

For applied forms:

```phalcom
t.origin
t.arguments
t.remainingParameters
```

For unions:

```phalcom
t.members
```

For tuples:

```phalcom
t.elements
```

For records:

```phalcom
t.fields
t.rowTail
```

For callables:

```phalcom
t.parameters
t.returnType
```

For a generic parameter descriptor:

```phalcom
p.name
p.owner
p.index
p.kind
p.variance
p.bounds
```

The exact selector names can be tuned later.

The semantic distinctions matter more than the naming.

---

# 43. `origin`, `arguments`, and parameters need careful definitions

Python's generic aliases expose `__origin__`, `__args__`, and `__parameters__`, which is a good baseline introspection model.

Phalcom should improve the naming clarity.

For:

```text
Map<String, V>
```

I recommend:

```text
origin
    Map

arguments
    [String, V]

declaredParameters
    [K, V]

remainingParameters
    [V]
```

For saturated:

```text
Map<String, Int>
```

`remainingParameters` is empty.

This distinction becomes extremely useful for partial application and reflection.

---

# 44. Reflection should expose semantic equality directly

I would provide:

```phalcom
T.equivalentTo(U)
T.subtypeOf(U)
```

rather than forcing users to infer semantics from runtime object equality.

Separately, I recommend canonical descriptor reification, so identical type forms evaluate to the same live descriptor object where possible:

```phalcom
List<Int> === List<Int>
```

can then naturally be true.

Nominal forms already have this property:

```phalcom
Int === Int
```

For synthetic descriptors the runtime registry should intern them.

A weak interning cache is sufficient if it guarantees that while a descriptor is live, reification of its semantic key returns that descriptor.

Semantic equivalence must nevertheless remain a defined relation independent of GC/object identity.

---

# 45. Runtime type constructors should be immutable

There should be no public:

```phalcom
AppliedType.new(...)
UnionType.new(...)
FunctionKind.new(...)
```

that can manufacture malformed descriptors.

Construction must go through canonical operations:

```text
type application
union formation
arrow formation
tuple/record formation
```

Descriptor fields are immutable.

That protects:

```text
canonical identity
hash stability
kind correctness
cache correctness
```

and greatly simplifies reflection.

---

# 46. Class objects remain live origins

Suppose reflective mutation adds a method to `List`.

`List<Int>` should not own an old copied method table.

Its semantic/member view still points at:

```text
origin = List
substitution = T -> Int
```

so future member queries see the current origin surface.

This preserves the earlier “live view” principle.

Generic specialization is a view, not a behavioral fork.

---

# 47. Runtime reflection must use stable structural metadata, never compiler `TypeId`

The compiler/static analyzer uses:

```text
TypeId
KindId
```

that are only meaningful inside a semantic store/generation.

Compiled metadata uses structural representations:

```text
CompiledTypeRef
CompiledKindRef
```

Runtime loads those and re-interns them into runtime-local IDs.

Correct pipeline:

```text
TypeId
  ↓ export
CompiledTypeRef
  ↓ load
RuntimeTypeId
  ↓ reify lazily
ordinary TypeDescriptor value
```

Never:

```text
TypeId(42)
stored in heap forever
```

This is essential for incremental compilation, LSP generations, hot code evolution and reproducible artifacts.

---

# 48. Lazy reflection means lazy objects, not absent semantics

A useful distinction:

The compiler may need to retain compact structural metadata.

That does not mean runtime heap descriptor objects must already exist.

For example:

```text
compiled artifact:
    stores declaration kind scheme
    stores relevant annotations/signatures

runtime:
    allocates List<Int> descriptor only when observed
```

That matches Phalcom's philosophy very well:

> compute and allocate reflective machinery only when observation requires it.

Later, metadata retention modes can be considered:

```text
minimal
public reflection
full debug/proof reflection
```

but object allocation should remain lazy in all cases.

---

# 49. Do not expose `.type` on every runtime value

This is an API trap.

If:

```phalcom
xs.class
```

returns:

```text
List
```

users may expect:

```phalcom
xs.type
```

to somehow return:

```text
List<Int>
```

But the runtime instance usually does not know that.

I recommend no universal runtime `.type`.

Keep:

```phalcom
x.class
```

for runtime classification.

Static type information belongs to semantic/tooling reflection:

```text
semanticInfo(x).type
declaration.signature
binding.declaredType
```

depending on eventual reflection APIs.

That prevents erased static information from masquerading as runtime state.

---

# 50. Separate runtime reflection from source-semantic reflection

There should really be two reflection/query surfaces.

Runtime reflection answers:

```text
what object is this?
what class?
what TypeForm value is this descriptor?
what kind does this TypeForm have?
what generic parameters does this declaration expose?
```

Compiler/LSP semantic queries answer:

```text
what type did the checker infer here?
why?
what expected type influenced inference?
what declaration supplied this member?
what flow refinement applied?
what constant fact is known?
what proof obligation exists?
what proof artifact discharged it?
what termination/effect knowledge exists?
```

Do not ship every occurrence-level analyzer fact into the executable merely to call it “reflection.”

That would explode metadata and blur phases.

---

# 51. Recommended semantic tooling API

Conceptually the shared semantic snapshot should eventually answer:

```text
typeAt(sourceOccurrence)
denotationAt(sourceOccurrence)
kindOf(TypeId)
provenanceAt(sourceOccurrence)
constantFactAt(sourceOccurrence)
dispatchResolutionAt(messageSend)
constraintsFor(declaration)
effectsOf(callable)
terminationOf(callable)
proofsFor(obligation)
```

The LSP, CLI and compiler should consume these exact same facts.

This is where the Pyrefly-inspired semantic-query architecture becomes valuable.

---

# 52. Hover should visibly show both axes when useful

For a class reference:

```phalcom
List
```

a rich hover could say:

```text
class List<T>

runtime value:
    class object List

denotes:
    List

kind:
    Type -> Type
```

For:

```phalcom
List<Int>
```

eventually:

```text
type form List<Int>

runtime representation:
    AppliedType

origin:
    List

arguments:
    Int

kind:
    Type
```

For an ordinary variable:

```phalcom
xs
```

show:

```text
static type:
    List<Int>

runtime class:
    List
```

when both facts are known.

That presentation teaches the ontology instead of hiding it.

---

# 53. Kind reflection and kind schemes need separate APIs

When kind polymorphism arrives, a polymorphic declaration does not have one monomorphic kind.

For example:

```text
Proxy :: ∀k. k -> Type
```

Therefore distinguish:

```text
kind
kindScheme
```

For monomorphic:

```phalcom
List.kind
// Type -> Type
```

For polymorphic:

```phalcom
Proxy.kindScheme
// forall k. k -> Type
```

Do not pretend a quantified scheme is itself just another `KindId`.

A scheme contains kinds; it is not a kind.

This distinction matters enormously once inference arrives.

---

# 54. Generalization boundaries should be declaration/interface boundaries

When kind polymorphism arrives, infer kind variables locally.

Then generalize only at permitted stable boundaries:

```text
type constructor declaration
type lambda
module/public semantic interface
possibly explicitly generalized local binding
```

Unsolved metavariables never leak.

I would not silently default unconstrained kind variables to `Type` at public boundaries.

If generalization is allowed, generalize.

If it is not, report ambiguity.

This avoids losing useful polymorphism silently.

---

# 55. `RecordRow` changes the kind grammar—but don't overabstract now

Once record rows are actually implemented, the current:

```rust
KindData::Type
KindData::Arrow
```

may naturally evolve toward:

```rust
enum KindData {
    Atom(KindAtom),
    Parameter(KindParameterId),
    Arrow {
        parameters: Box<[KindId]>,
        result: KindId,
    },
}

enum KindAtom {
    Type,
    RecordRow,
    // future, only when ratified:
    Constraint,
    EffectRow,
}
```

But I would not perform this refactor today merely because it might eventually be useful.

The current two-axis milestone only needs:

```text
Type
Arrow
```

Refactor when the second atomic kind becomes real.

Avoid architecture cosplay.

---

# 56. Callable types and kind arrows should share notation but not semantics

These are structurally analogous:

```text
Int -> String
```

means:

```text
CallableType :: Type
```

whereas:

```text
Type -> Type
```

means:

```text
FunctionKind
```

The surface operator can be the same:

```text
->(_)
```

and remain non-overridable.

But operand level decides semantic construction.

For callable types:

```text
left/right operands must be proper Type forms
```

For kind arrows:

```text
left/right operands are KindForms
```

No mixed-level formation.

---

# 57. Callable arrows should remain right-associative

I recommend:

```text
A -> B -> C
```

means:

```text
A -> (B -> C)
```

A two-argument callable is expressed as:

```text
(A, B) -> C
```

not:

```text
A -> B -> C
```

This preserves normal function-type mathematics while avoiding confusion with Phalcom's selector arity.

Kinds likewise present right-associatively:

```text
Type -> Type -> Type
```

while the implementation may store an n-ary parameter vector.

---

# 58. Reflection of callable types

A callable descriptor should expose actual parameter structure rather than pretending the arrow itself captures all message-selector details.

Conceptually:

```phalcom
sig.parameters
sig.returnType
```

with each parameter exposing:

```text
position
label
type
rest?
```

because Phalcom selectors have positional/named/arity semantics.

Type metadata remains outside selector identity.

This lets reflection explain a typed method without changing how methods dispatch.

---

# 59. Generic method parameters must not alter selectors

Eventually one might write something like:

```text
map<T>(...)
```

if that syntax is ratified.

But selector identity remains:

```text
map(...)
```

not:

```text
map<T>(...)
```

Generic binders belong to declaration semantic metadata.

This is non-negotiable under the already accepted constraints.

I would actually defer explicit call-site method type arguments until there is a concrete need, because syntax like:

```phalcom
obj.map<Int>(...)
```

can easily make users believe type arguments influence runtime dispatch.

Inference should handle most generic call sites.

---

# 60. Keep class-object types internal initially

The checker needs:

```text
ClassObject(Int)
ClassObject(List)
```

as proper static value types.

I would not immediately expose source syntax:

```text
Class<Int>
```

for them.

The internal form solves:

```text
What is the ordinary value type of the class object Int?
```

Reflection/tooling can present it as:

```text
class object Int
```

rather than exposing implementation vocabulary.

If a genuine need for first-class metatype annotations appears later, design them separately.

Swift and Scala both show how quickly explicit metatype/type-object syntax becomes its own subsystem. Phalcom can postpone that.

---

# 61. TypeForm itself can eventually be an ordinary reflective annotation

Once runtime reflection exists, this is useful:

```phalcom
inspect(t: TypeForm) {
    t.kind
}
```

because both:

```text
Int
List
List<Int>
Int | String
```

are values satisfying that capability.

This does not conflict with:

```text
Type
```

being the atomic kind.

That is precisely why renaming the old `Type` protocol was important.

---

# 62. `KindForm` should likewise be a runtime reflection capability

For example:

```phalcom
inspectKind(k: KindForm) {
}
```

could accept:

```text
Type
RecordRow
Type -> Type
(Type -> Type) -> Type
```

when runtime reflection lands.

Again, `KindForm` is a proper runtime protocol type for descriptor values.

It is not itself the semantic kind category.

One word can refer to runtime behavior; another to mathematical role.

---

# 63. Reflection errors should be typed and precise

Dynamic reflective application:

```phalcom
List.<>(42)
```

should fail because `42` is not a TypeForm.

Dynamic:

```phalcom
List.<>(List)
```

should fail because:

```text
expected argument kind Type
actual kind Type -> Type
```

That should produce a dedicated reflective error such as conceptually:

```text
TypeApplicationError
KindMismatchError
```

Static code catches the same problem earlier.

The runtime path and static path should reuse the same semantic rules, not duplicate them.

---

# 64. Reflection descriptors should be canonical but cheap

The runtime registry should hold compact structural nodes:

```text
RuntimeTypeId
RuntimeKindId
```

and descriptor objects should carry only the corresponding ID.

Conceptually:

```rust
struct AppliedTypeObject {
    id: RuntimeTypeId,
}
```

not:

```rust
struct AppliedTypeObject {
    origin: Value,
    args: Vec<Value>,
    fields: ...
    cached_name: String,
    ...
}
```

The registry owns canonical structure.

Objects are lightweight façades.

---

# 65. Proof/type reflection should be optionally retained independently

I recommend separating compiled metadata categories:

```text
runtime type metadata
    declaration kinds
    generic signatures
    stable type forms required for reflection

debug semantic metadata
    source provenance
    inference traces
    explanations

proof metadata
    VCs
    certificates
    assumptions
    solver identities
```

A release executable may need type reflection but not proof provenance.

A verification build may retain proof artifacts.

A development LSP has all of it without shipping any of it into the executable.

This is much more scalable than one monolithic “reflection metadata on/off” switch.

---

# 66. Type-directed optimization may use semantic facts—but must remain unobservable

Static type knowledge can legitimately help the compiler:

```text
devirtualization
dead-branch elimination
known-selector resolution
proof-based check elimination
constant folding
```

provided runtime observable semantics remain unchanged.

Given your explicit constraint, I would not let type specialization alter Phalcom's language-level allocation, class identity, object layout, or selector behavior.

Backend optimizations remain implementation artifacts.

---

# 67. Type-based multimethods remain a separate opt-in abstraction

Phalcom may eventually support things like the previously discussed `@typecase`.

That should not cause the ordinary method system to become type-dispatched.

The distinction remains:

```text
ordinary method:
    selector dispatch

@typecase / multimethod abstraction:
    explicitly opts into typed secondary dispatch machinery
```

The core typing system merely provides the type facts such a facility may consume.

This preserves the purity of the existing object model.

---

# 68. Error diagnostics should teach kinds explicitly

A bad application:

```text
List<List>
```

should not report:

```text
invalid generic argument
```

It should say something like:

```text
List expects an argument of kind Type.

List has kind:
    Type -> Type

The supplied argument `List` has kind:
    Type -> Type

Expected:
    Type

Received:
    Type -> Type
```

Likewise:

```text
const x: Map<String>
```

should say:

```text
`Map<String>` is a valid type constructor, but it is not a complete value type.

Its kind is:
    Type -> Type

This annotation requires:
    Type
```

That is vastly more educational than calling it “not a type.”

It *is* a type-level form.

It is simply not a proper type.

---

# 69. Diagnostic vocabulary should use “proper type”

I recommend standard terminology:

```text
type form
proper type
type constructor
kind
kind mismatch
unsaturated constructor
```

Avoid saying:

```text
Map is not a type
```

because mathematically it is a type constructor and semantically a `TypeForm`.

Prefer:

```text
Map is not a proper value type; it has kind Type -> Type.
```

This wording will matter once users start learning HKTs.

---

# 70. LSP should make kind information normal, not exotic

Autocomplete/hover could eventually show:

```text
List<T>
Type -> Type
```

directly beside the declaration.

For:

```text
F
```

inside:

```phalcom
class Transformer<F: Type -> Type>
```

hover:

```text
type parameter F
kind: Type -> Type
```

For row parameter:

```text
row parameter R
kind: RecordRow
```

This makes kinds feel like normal type-level shape information rather than an advanced hidden compiler feature.

---

# 71. Reflection display syntax should be source-like, but never identity-bearing

A type descriptor should print something like:

```text
List<Int>
Int | String
#{age: Int, name: String}
(Int, String) -> Bool
```

The display should be deterministic.

But never use the string as:

```text
type identity
cache key
linker identity
serialization meaning
```

Identity remains canonical structural data.

Strings are presentation.

---

# 72. Alpha-equivalent schemes should be semantically equal

Once schemes exist:

```text
forall k. k -> k
```

and:

```text
forall j. j -> j
```

are equal.

Binder names are documentation, not semantic identity.

Therefore scheme equality and hashing should use:

```text
binder position / de Bruijn-like identity / stable parameter indexing
```

not user parameter spelling.

Reflection may preserve/display source names separately.

---

# 73. Type parameters and inference variables must never be conflated

This same principle recurs everywhere:

```text
TypeParameterId
    declared stable binder

InferVarId
    local type inference metavariable

KindParameterId
    generalized stable kind binder

KindVarId
    kind inference metavariable

RowVarId
    row inference metavariable
```

The stable ones can appear in interfaces/reflection.

The ephemeral ones cannot.

This separation will make incremental compilation and proof caching much easier.

---

# 74. Record rows should use the same stable type-parameter abstraction

A declared:

```text
R :: RecordRow
```

does not need a fundamentally different stable binder identity from:

```text
T :: Type
F :: Type -> Type
```

All are type-level parameters classified by kinds.

So:

```rust
TypeParameterData {
    name,
    kind,
    ...
}
```

is the right general abstraction.

Only solver metavariables need domain-specific IDs.

This is a nice example of the kind system actually simplifying implementation.

---

# 75. `Constraint` should probably become a kind eventually—but only when the system needs first-class constraints

GHC's separate `Constraint` kind is conceptually valuable because it says:

```text
this expression is a requirement/proposition,
not a type inhabited by runtime values
```


Phalcom may eventually benefit from:

```text
Type
RecordRow
Constraint
```

Then something conceptually like:

```text
Serializable<Int> :: Constraint
```

could exist.

But I would not add it merely because Haskell has it.

Wait until:

```text
protocol constraints
equality constraints
bounds
proof obligations
constraint composition
```

are sufficiently mature that making constraints first-class genuinely improves the model.

Otherwise constraints can remain checker predicates.

---

# 76. Avoid `TypeInType`/universe collapse

GHC's historical `TypeInType` direction deliberately blurred types and kinds and has since been deprecated as a feature umbrella; modern GHC keeps the underlying richer kind system while distinguishing the extensions involved.

Phalcom does not need that complexity.

Keep:

```text
Type
RecordRow
possibly Constraint
arrow kinds
kind schemes
```

and stop.

No:

```text
Type :: Type
```

No arbitrary universe hierarchy.

No term-dependent kinds.

This preserves exactly the clean stratification we've built.

---

# 77. A compact formal grammar I recommend targeting

Long-term, the semantic grammar could approximately become:

```text
TypeForm F ::=
      Nominal(D)
    | ClassObject(D)               // internal value type only
    | Parameter(P)
    | Apply(F, F)
    | Union(F...)
    | Tuple(...)
    | Record(fields, rowTail)
    | Callable(parameters, result)
    | TypeLambda(parameters, body) // later


Kind K ::=
      Type
    | RecordRow                    // later
    | Constraint                   // possibly later
    | KindParameter(KP)            // kind polymorphism
    | K -> K


KindScheme S ::=
    forall KP*. K


Knowledge ::=
      Known(proper TypeForm)
    | Unknown(reason)
    | Dynamic(reason)


Denotation ::=
      none
    | TypeForm(TypeId)
    | Kind(KindId)
```

Notice what is absent:

```text
arbitrary values inside types
arbitrary Phalcom method execution
runtime classes masquerading as type algebra
solver metavariables in exported semantics
```

That absence is deliberate.

---

# 78. The mathematical laws worth putting in the specification

The core kind-application judgment:

```text
F :: K1 -> K2
A :: K1
────────────────
F<A> :: K2
```

Proper-value typing:

```text
T :: Type
v : T
```

No value may have an unsaturated constructor as its ordinary type.

Partial application:

```text
F :: K1 -> K2 -> K3
A :: K1
────────────────────
F<A> :: K2 -> K3
```

Variance:

```text
A <: B
F covariant
────────────
F<A> <: F<B>
```

Contravariance:

```text
A <: B
F contravariant
───────────────
F<B> <: F<A>
```

Invariant:

```text
F<A> <: F<B>
only when A ≡ B
```

Record width:

```text
{name:A, age:B}
    <:
{name:A}
```

provided field variance/mutability laws permit depth relationships.

Row extension:

```text
R :: RecordRow
────────────────────────
{name:String, ...R} :: Type
```

These laws are small enough that users can reason about the system without memorizing compiler exceptions.

---

# 79. Lessons I would explicitly take from other languages

Haskell gives us the best basic kind calculus and demonstrates that kind polymorphism can make abstractions substantially more reusable. It also demonstrates how quickly promotion, type families, `ConstraintKinds`, dependent features and unrestricted type-level evaluation make the system significantly harder to reason about. Phalcom should take `Type`, arrow kinds, inference and prenex polymorphism, while postponing the rest.

Scala 3 demonstrates that HKTs can coexist with an OO language and that explicit type lambdas are worth having. Its `[X] =>>` syntax solves a real problem. Its `AnyKind` mechanism is useful but tied deeply to Scala's subtype system; Phalcom can implement cleaner explicit kind schemes instead.

OCaml demonstrates the value of not collapsing every abstraction into ordinary types. Modules, module types and functors form a distinct abstraction layer, and first-class modules allow controlled reification where needed. Phalcom should preserve its own module system independently from HKTs.

Python demonstrates an attractive reflective property: parameterized generic expressions are actual values with inspectable origin/arguments, and they are not themselves classes. Phalcom should take that ergonomic idea but use canonical semantic identity, real kinds, explicit partial application and a stronger static/runtime boundary.

---

# 80. My recommended staged destination

I would think of the system as four increasingly powerful layers.

| Layer | Capability | Recommendation |
|---|---|---|
| Semantic kernel | `Type`, arrow kinds, canonical TypeForms, application, denotation | Implement now |
| Generic algebra | parameters, `+T/-T`, substitution, bounds, HKTs, partial application | Implement next |
| Advanced kind/type algebra | type lambdas, prenex kind polymorphism, record rows | Architect now, implement after kernel |
| Verification/reflection | effects, totality, proof artifacts, runtime TypeForm/KindForm values | Semantics now where necessary; runtime presentation later |

What I would explicitly not let block the first two layers:

```text
dependent types
DataKinds-style promotion
Prop
proof terms
universal row calculus
kind universes
runtime reified generics on instances
type-based ordinary dispatch
```

---

# 81. Decisions I would ask you to ratify next

Most of the mathematical architecture is strong enough to treat as settled. The remaining decisions that are genuinely worth your comment rather than being silently selected by implementation are these:

| Decision | My recommendation |
|---|---|
| Generic declaration syntax | `class Box<T>` |
| Kind annotation syntax | `F: Type -> Type` |
| Bounds/constraints | `where` clauses, not overloaded `:` |
| Primitive type application | unary privileged `<>(_)`; `Map<A,B>` lowers left-to-right |
| Partial application surface | legal as a TypeForm; illegal where a proper value type is required |
| Higher-kinded variance | invariant initially; `+/-` initially restricted to Type-kinded parameters |
| Type lambda semantics | yes |
| Type lambda syntax | defer; `<T> => ...` is my current favorite among standalone syntaxes |
| Record rows | yes |
| Record-row syntax | `#{field: Type, ...R}` is my favorite, but defer ratification |
| `Constraint` kind | eventual candidate, not yet |
| Numeric literals | spelling fixes runtime class |
| Constant singleton knowledge | analyzer evidence, not ordinary TypeId |
| Totality | partial by default, explicit totality requirement |
| Totality source syntax | likely `@total` |
| Proof evidence | persistent checkable artifacts |
| `Prop` / proof terms | no |
| Runtime TypeForm reflection | yes, later |
| Runtime `.type` on arbitrary values | no |
| Runtime test `x is List<Int>` | no general promise under erased generics |
| `List<Int>.new()` forwarding | no automatic class-side forwarding |
| Runtime semantic kind-of-kind | no |
| Kind polymorphism | prenex, inferred initially |
| Type-level arbitrary Phalcom execution | no |

The overarching principle connecting all of these is:

> **Phalcom should make semantic structure first-class without making runtime behavior type-directed.**

That is the balance I would protect above everything else. It gives you Haskell's mathematical type algebra, Scala's OO/HKT coexistence, Python's reflective accessibility, and Smalltalk's object uniformity—without inheriting the worst complexity of any of them.