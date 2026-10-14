# Functional values

`universe.functional` supplies typed building blocks for explicit state,
configuration environments, immutable updates, and independent validation.
Import the types explicitly:

```phalcom
from universe.functional import State, StateStep, Reader, Lens, Validation
```

All implementations are source-authored Phalcom. They use nominal immutable
data products or sums, ordinary generics, typed callbacks, and matching. They
add no native primitives, implicit globals, scheduler policy, or compiler rules.

## State

`State<S, T>` describes `(S) -> StateStep<S, T>`. `StateStep` has labeled
`state` and `value` components. Construction and composition defer execution;
each `run(initial)` starts a fresh execution. `eval` and `exec` each run the
transition once, so use `run` when both outputs are needed.

```phalcom
let nextId = State<Int, Int>.new(|next| {
  StateStep<Int, Int>(state: next + 1, value: next)
})
let twoIds = nextId.zipWith(nextId, combine: |first, second| { (first, second) })
let allocated = twoIds.run(100)
// allocated.value is (100, 101); allocated.state is 102.
```

`pure` preserves state. `get` is available on `State<S, S>`; `put` and `modify`
are available on `State<S, Unit>`. `inspect` reads a projection without changing
state. `map` transforms the produced value; `flatMap` chooses the next transition
from that value. `then` discards the first value. `zipWith` and `then` run left
to right, passing the successor state to the next transition.

`zoom(through: lens)` runs a transition against a part of a larger state.
`rollbackOnError<U, E>()`, available when `T == Result<U, E>`, retains the
successor state for `Ok` and restores the initial state value for `Error`.
It does not catch exceptions or undo external effects or mutations of objects
inside the state. Transactional value updates require immutable state payloads.

## Reader

`Reader<R, T>` describes `(R) -> T`. `new` supplies the computation; `pure`
ignores the environment, and `ask` returns it on `Reader<R, R>`.

`map`, `flatMap`, and `zipWith` defer execution until `run(environment)`.
Every stage sees the same environment value. `zipWith` runs left to right.
`local` adapts an environment for a computation; `contramap<Q>` adapts an
entirely different environment type. Both invoke the adapter once per run.
These operations are useful for configuration and dependency injection without
global storage.

## Lens

`Lens<S, T>` packages a getter `(S) -> T` and a replacement `(S, T) -> S`.
`modify` reads once, transforms once, and replaces once. `compose` combines
nested focuses. For example:

```phalcom
data Settings(_ retries: Int, _ endpoint: String)
let retries = Lens<Settings, Int>.new(
  get: |settings| { settings.retries },
  set: |settings, value| { Settings(value, settings.endpoint) }
)
let original = Settings(2, "service")
let updated = retries.modify(original, using: |value| { value + 1 })
// updated.retries is 3; original.retries remains 2.
```

For lawful, pure lenses, reading after replacement returns the replacement;
replacing a part with its current value preserves the whole; and the last of
two replacements wins. The library does not prove these callback obligations.

## Validation

`Validation<T, E>` is `Valid(T)` or `Invalid(E)`. `map` transforms a valid value,
`mapErr` transforms an error, and `fold` eliminates either variant. `fromResult`
and `toResult` preserve both payloads.

`zipWith(other, combine: ..., errors: ...)` combines independent checks:

| Receiver | Other | Outcome |
| --- | --- | --- |
| Valid(a) | Valid(b) | Valid(combine(a, b)) |
| Invalid(a) | Valid(_) | Invalid(a) |
| Valid(_) | Invalid(b) | Invalid(b) |
| Invalid(a) | Invalid(b) | Invalid(errors(a, b)) |

Only the selected combining callback runs, once. Both input validations have
already been evaluated before the call. An associative error combiner makes
error accumulation independent of grouping while preserving left-to-right
error order. An error payload may be a string, immutable collection, or report.
Use `Result.andThen` for dependent checks that stop at the first error.

## Persistent sequences and logs

`Chain<T>` is `Empty` or `Link(T, Chain<T>)`, with O(1) prepend and shared
immutable tails. Its map, flatMap, filter, filterMap, fold, reverse, append,
take, drop, find, partition, indexing, and zipWith operations use loops.
Append copies the left spine and shares the right. `zipWith` stops at the
shorter sequence. Negative indexing gives None; nonpositive take/drop counts
give an empty sequence/the original sequence respectively.

`ChainIterator<T>` conforms to Iterator; Chain and `NonEmpty<T>` conform to
Iterable with an exact associated Item type. `fromIterable` requires canonical
equivalence between that associated Item and T. NonEmpty stores a required
head and a Chain tail; map and flatMap preserve nonemptiness, while filtering
returns Chain because every value might be removed.

`Journal<T>` is a persistent concatenation tree. Combining logs is O(1), and
entries/fold traverse the tree using an explicit work stack. `==` and hash use
the ordered entries, independent of tree grouping; exact data identity `===`
still includes the declared tree structure. It is a useful log for Writer
because repeated appends avoid copying the entire accumulated prefix.

## Algebraic accumulation and Writer

Semigroup supplies `combine`; Monoid requires Semigroup conformance and adds
class-side `empty`. Laws are obligations under a chosen equivalence, not
automatically proved facts. Endo uses extensional equivalence for pure total
callbacks; floating-point numeric wrappers can violate associativity through
rounding. Neither conformance nor static type equality proves these laws.

| Wrapper | Combination | Identity |
| --- | --- | --- |
| Additive<T> | component addition | T.zero |
| Multiplicative<T> | component multiplication | T.one |
| AllTrue / AnyTrue | conjunction / disjunction | true / false |
| First<T> / Last<T> | first / last present Option | None |
| Dual<M> | reversed underlying combination | underlying empty |
| Endo<T> | right-to-left function composition | identity function |
| Chain<T> / Journal<T> | ordered concatenation | empty sequence/log |

NonEmpty supplies Semigroup without an empty identity. Numeric wrapper
operations require only their corresponding Addable/Multipliable and Zero/One
capabilities. `Writer<W, T>` stores labeled value/log components, maps either
component, and combines logs left to right in flatMap and zipWith. Those
operations require W |= Semigroup; pure and tell also require W |= Monoid.
listen returns the value and log together while retaining the output log;
censor transforms the final log.

## Kind-indexed effects and typed pipelines

Functor, Applicative, and Monad take `F: Type -> Type` and describe class-side
operations supplied by explicit context dictionaries. IdentityContext,
OptionContext, ChainContext, and NonEmptyContext implement these protocols.
Chain and NonEmpty use Cartesian-product applicative combination and
order-preserving flatMap. Identity carries exactly one value.

Effects.map, lift2, apply, flatten, traverse, and sequence preserve the chosen
F family. For example:

```phalcom
from universe.functional import Chain, Effects, OptionContext
let source = Chain<Int>::Empty.prepend(2).prepend(1)
let result = Effects.traverse<Option, OptionContext, Chain<Int>, Int, Int>(
  source, using: |n| { Option<Int>::Some(n * 2) }
)
// result has type Option<Chain<Int>>.
```

Generic traverse eagerly snapshots a finite source spine, then invokes the
mapping callback through the monad. An absent Option skips later callbacks,
but the snapshot has already consumed the source. This is distinct from the
fail-fast traversal APIs below. Deferred dictionaries may retain nested calls;
generic effect composition does not guarantee constant stack depth.

Arrow<A, B> describes a typed pipeline with compose/then, dimap, product
first/second/split, fanout, and sum routing through Either<L, R>. Product
operations preserve exact Tuple component types. Sum routing executes only
the selected branch. Either is a neutral choice, with fold, side mapping,
bimap, swap, and a merge getter guarded by L == R.

Profunctor takes `P: Type -> Type -> Type`; ArrowContext supplies its dimap
operation. This is a constructor-domain constraint, separate from trait
conformance and declaration variance. No automatic variance or law proof is
introduced.

## Folds, traversal, and error-aware computation

Fold<S, A, B> has a fresh initializer, a step function, and a final projection.
run requires an Iterable whose associated Item is equivalent to A. map,
contramap, dimap, and filter adapt a fold. zipWith pairs accumulator types in an
exact Tuple and runs both folds in one traversal. FoldFactory supplies count,
collect, sum, any, all, first, and last. any/all skip later predicate calls once
decided, but their driver still consumes the source.

Folds.foldMap and combineAll use nominal Semigroup/Monoid constraints.
traverseResult and traverseOption stop at the first failure without asking the
iterator for later inputs. validateAll visits every input and returns either
all values or a NonEmpty collection of errors, preserving encounter order.
Validation.ensure runs its error producer only for a rejected valid value;
combineWith uses Semigroup conformance to combine independent errors.

StateResult<S, T, E> combines deferred state threading with Result. flatMap and
then skip later stages after failure. Failed successor state is retained by
default; recovery runs from it. transactional restores the initial state value
on Result::Error at the explicitly wrapped boundary. ReaderResult<R, T, E>
combines deferred environment access and Result, sharing the same environment
through success and recovery. Neither wrapper catches thrown exceptions.

## Additional optics and structural relationships

Iso<S, T> supplies convert, restore, inverse, composition, modify, and asLens.
Its sameShape factory requires S and T to conform to Product and requires
`<S as Product>::Shape == <T as Product>::Shape`. That constraint consumes
canonical structural reflection; it does not prove round-trip callback laws.
Optional<S, T> modifies a possibly absent focus; Prism<S, T> selects and builds
a sum case. Both compose, preserve an unmatched original value, and skip the
modification callback when absent.

## Callback contract and verification

Data component bindings are immutable, but contained objects and captured
variables can be mutable. Callbacks may throw; exceptions propagate normally,
and later stages do not run after abrupt termination. Results are not cached.
Functor and monad laws require pure callbacks; the wrappers do not claim or
statically prove purity. Deeply nested composition uses ordinary nested calls;
it has no trampoline or unbounded stack-safety guarantee.

Source parsing, module registration, and exports were checked on 2026-10-13.
Behavior and generic type acceptance remain **UNVERIFIED**. Twelve pending
regression fixtures cover kind-indexed effects,
persistent sequences, algebra/logging, folds, error-aware computation, optics,
state ordering, environments, validation, and Option/Result composition. Run the focused
acceptance gate before treating these APIs as behavior-certified:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::functional -- --exact
```
