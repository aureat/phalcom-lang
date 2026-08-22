Yes: both belong on the semantic side of the boundary, not in Phaldoc. But I would treat them differently.

Your existing principle—

> Phaldoc explains. Metadata specifies.

—handles this case extremely well. The deciding question is not “would users like to see this in documentation?” It is:

> Could a compiler, runtime, LSP, checker, optimizer, caller, or reflective tool make a correctness-relevant decision from this fact?

For both suspension and message interception, the answer is clearly yes.

The interesting part is deciding whether each fact should be explicitly declared, inferred, or both.

## `@suspends`: semantic callable metadata

I would make suspension a method/callable semantic property, not a Phaldoc directive.

Conceptually:

```phalcom
@suspends
read(buffer) {
    ...
}
```

or eventually, if Phalcom develops a generalized effect system:

```phalcom
@effects(suspend)
read(buffer) {
    ...
}
```

The exact syntax can remain open, but the fact itself should unquestionably be machine-readable.

Why? Because “this call may suspend execution” affects far more than documentation:

- whether it is legal in a non-suspending context;
- scheduler interaction;
- fiber/runtime behavior;
- cancellation boundaries;
- lock/critical-section analysis;
- resource lifetime analysis;
- debugger stepping;
- LSP diagnostics;
- call-graph analysis;
- optimization;
- whether an override is behaviorally compatible with its parent;
- whether a callback is acceptable to an API requiring a non-suspending operation.

That makes this categorically different from:

```phalcom
/// This operation may take some time.
```

The prose explains the consequence. `@suspends` specifies the semantic fact.

Generated documentation should simply harvest it:

```text
fetch(_)

(URL) -> Result<Response, NetworkError>

suspends · io

Fetches the resource at `url`.
```

### But I would go one step further: permit inference

I would not require users to manually annotate every transitive suspension point forever.

Suppose:

```phalcom
foo() {
    bar()
}

@suspends
bar() {
    Fiber.yield()
}
```

If `foo()` unconditionally contains a call to a suspending operation, the compiler/checker should often be capable of inferring:

```text
foo: suspends
```

The useful distinction is therefore:

```text
declared suspension
inferred suspension
effective suspension
```

For example:

```rust
pub struct CallableSemantics {
    pub suspension: SuspensionSemantics,
}

pub struct SuspensionSemantics {
    pub declared: Option<SourceRange>,
    pub inferred: bool,
    pub effective: bool,
}
```

This becomes especially important once types participate in correctness. A public API may want an explicit declaration even where it is inferable, while private implementation code may rely on inference.

So I would eventually consider a policy roughly like:

```text
public API:
    explicit @suspends recommended or required when suspension is part
    of the callable's contract

private/internal callable:
    inference sufficient where exact

unknown/dynamic dispatch:
    conservatively considered potentially suspending unless proven otherwise
```

That is a typing/effect-system decision rather than a Phaldoc decision.

## `@intercepts`: semantic class capability, but preferably derived

This one is more subtle.

If a class overriding something like:

```phalcom
__intercept__
```

changes the fundamental semantics of message dispatch, that is absolutely machine-relevant.

The LSP may need to know:

```text
normal member lookup is not the whole story here
```

Reflection may need to know it. Static reasoning may need to become conservative. The optimizer may have to abandon normal send specialization. Debugger tooling may need to expose intercepted sends differently.

So it should not be:

```phalcom
/// @intercepts
class Proxy
```

Phaldoc cannot be the canonical source.

But I would hesitate to require this:

```phalcom
@intercepts
class Proxy {
    __intercept__(...) { ... }
}
```

because then you have two declarations of the same fact.

If implementing/overriding `__intercept__` is itself what gives the class interception semantics, then:

```text
class defines effective __intercept__
        ↓
semantic model derives
        ↓
intercepts_message_sends = true
```

is superior.

In other words, I would distinguish:

```text
@intercepts            ← probably unnecessary
intercepts = true      ← derived semantic fact
__intercept__ override ← authoritative source
```

This follows the exact same rule you have already applied elsewhere:

> Do not ask the programmer to state metadata that follows deterministically from an actual declaration.

For example:

```phalcom
class Proxy {
    __intercept__(message, receiver) {
        ...
    }
}
```

could yield:

```text
ClassSemantics {
    intercepts_message_sends: true,
    interceptor: MethodKey(
        Proxy,
        Instance,
        "__intercept__(_,_)"
    ),
}
```

And generated documentation could display:

```text
Proxy

message-intercepting

A transparent proxy around another object.

Message dispatch
    Overrides the standard message-send pipeline through
    Proxy#__intercept__(_,_).
```

The author writes no redundant `@intercepts`.

## This gives you a useful three-way distinction

I think Phaldoc would benefit from explicitly adopting this classification.

| Kind | Example | Canonical source |
|---|---|---|
| Declared semantic fact | `@suspends`, `@requires(...)`, stability | semantic attribute/declaration |
| Derived semantic fact | “class intercepts sends because it overrides `__intercept__`” | semantic analysis |
| Human explanation | “Calls may resume on another scheduler turn” | Phaldoc |

That is a very strong architectural boundary.

Consider suspension:

```phalcom
@suspends
/// Downloads the requested resource.
///
/// Suspension may occur while waiting for network readiness.
fetch(url: URL) -> Result<Response, Error>
```

Although you'd actually place the doc first according to your attachment rules:

```phalcom
/// Downloads the requested resource.
///
/// Suspension may occur while waiting for network readiness.
@suspends
fetch(url: URL) -> Result<Response, Error>
```

Here:

```text
@suspends
    fact

"Suspension may occur while..."
    explanation
```

And interception:

```phalcom
/// A forwarding proxy around `target`.
///
/// Most unknown sends are forwarded to the wrapped object. Sends for
/// introspection selectors are handled locally instead.
class Proxy {
    __intercept__(message) {
        ...
    }
}
```

Here:

```text
__intercept__ implementation
    source semantic construct

intercepts message sends
    derived fact

"Most unknown sends..."
    explanation
```

That is exactly the separation you want.

## I would not let Phaldoc acquire `@suspends` or `@intercepts`

Doing so would introduce several problems.

First, it would undermine your rule that Phaldoc diagnostics cannot affect program correctness. If:

```phalcom
/// @suspends
foo() {}
```

is only documentation, then a scheduler-aware checker cannot rely on it. If the checker can rely on it, Phaldoc has ceased to be documentation trivia.

Second, it would create two semantic channels:

```text
attributes
Phaldoc directives
```

That becomes unpleasant almost immediately. Does this mean:

```phalcom
@suspends
/// @suspends
foo()
```

is valid? Redundant? An error? Which one wins?

Avoid the problem entirely.

Third, it creates documentation drift:

```phalcom
/// @intercepts
class Proxy {
    // __intercept__ removed during refactoring
}
```

Generated docs would lie.

With derived semantics, removing the interceptor automatically removes the classification.

## I would actually tighten section 28 of the proposal

Right now it says:

> Effects do not belong in Phaldoc.

I would expand that into a broader normative rule:

> **Behavioral semantic declarations do not belong in Phaldoc.**

That includes not merely classic “effects,” but properties such as:

```text
suspends
blocks
performs IO
allocates
mutates
throws/raises
cancellable
atomic
transactional
unsafe
trusted
intercepts message sends
customizes lookup
customizes equality/hash semantics
customizes iteration
compiler intrinsic
runtime primitive
```

But there is an important qualification:

Some are explicitly declared:

```text
@suspends
@unsafe
@atomic
```

Some are derived:

```text
intercepts message sends
provides custom equality
provides custom iteration protocol
```

And some may be both inferred and optionally asserted.

Phaldoc simply renders and explains them.

## Beware of turning every interesting behavior into a class attribute

This is the opposite failure mode.

You probably do not want this:

```phalcom
@intercepts
@customEquality
@customHash
@customIteration
@customConstruction
@customLookup
class Foo
```

if all of those facts can be learned from the class's method/protocol implementation.

That becomes a semantic version of Java annotation soup.

A good rule for Phalcom would be:

> Explicit semantic attributes should state information that cannot be obtained reliably from the declaration itself, or should impose/check a contract beyond what the declaration naturally says.

So:

```phalcom
@suspends
```

makes sense because the method body and arbitrary dynamic callees may not provide a locally obvious stable API contract.

But:

```phalcom
@intercepts
```

usually does not, because:

```phalcom
__intercept__
```

is itself already the declaration of interception behavior.

Similarly, if Phalcom eventually has:

```phalcom
__iter__
__hash__
__equals__
__call__
```

you generally should not also require:

```phalcom
@iterable
@hashable
@equatable
@callable
```

unless those attributes have independent semantics.

## There is another important distinction: capability versus implementation

This becomes useful for inheritance.

Imagine:

```phalcom
class Proxy {
    __intercept__(...) { ... }
}

class LoggingProxy < Proxy {
}
```

Does `LoggingProxy` intercept sends?

Obviously yes.

But it does not declare `__intercept__`.

Therefore the semantic model should probably distinguish:

```text
declares interceptor
inherits interceptor
effectively intercepts
```

Something like:

```rust
pub struct MessageInterception {
    pub effective: bool,
    pub origin: Option<MethodKey>,
    pub inherited: bool,
}
```

Then the docs can say:

```text
LoggingProxy

message-intercepting

Message interception inherited from Proxy#__intercept__(_,_).
```

That is richer and more accurate than an `@intercepts` marker could ever be.

The same applies to overrides:

```phalcom
class TransparentProxy < Proxy {
    __intercept__(...) {
        super(...)
    }
}
```

The class still intercepts at the semantic machinery level even if its implementation merely forwards.

So you don't want tooling trying to inspect whether the method “actually modifies” behavior. The relevant semantic fact is:

```text
this class participates in the interception pathway
```

rather than:

```text
we somehow proved it changes observable behavior
```

The latter is generally undecidable or at least not worth trying to define.

## `@suspends` should probably interact with overriding

This is one reason it must be semantic metadata.

Suppose:

```phalcom
class Reader {
    read() {
        ...
    }
}
```

and:

```phalcom
class NetworkReader < Reader {
    @suspends
    read() {
        ...
    }
}
```

Can a non-suspending base method be overridden by a suspending method?

That is a real language-design question.

If callers statically know only:

```phalcom
reader: Reader
reader.read()
```

then allowing the subtype to introduce suspension potentially violates the caller's assumptions.

So Phalcom may eventually need an effect-subtyping rule analogous to return/parameter compatibility:

```text
non-suspending implementation
    ≤
possibly-suspending contract
```

but not necessarily:

```text
possibly-suspending implementation
    ≤
non-suspending contract
```

In other words:

```phalcom
class Reader {
    @suspends
    read()
}
```

could permit both implementations:

```phalcom
class MemoryReader < Reader {
    read() { ... }               // stronger guarantee: never suspends
}

class NetworkReader < Reader {
    @suspends
    read() { ... }
}
```

Whereas a base declaration with a non-suspending contract might prohibit `NetworkReader`'s override.

That is exactly the sort of correctness rule you lose if `@suspends` lives in Phaldoc.

## It may eventually be better modeled as an effect than an isolated attribute

Given the concurrency work we've been doing, I would avoid prematurely making `@suspends` an architectural one-off.

You may initially expose:

```phalcom
@suspends
```

while internally modeling:

```text
CallableEffects {
    suspension: MaySuspend,
    ...
}
```

Then later:

```text
IO
blocking
mutation
allocation
suspension
nondeterminism
...
```

can share an effect architecture without forcing the surface syntax to become:

```phalcom
@effects(...)
```

immediately.

The public syntax and semantic representation do not need to be identical.

For example:

```phalcom
@suspends
foo()
```

can normalize internally to:

```text
Effects {
    suspension = MaySuspend
}
```

That preserves room for a coherent future effect system.

## Likewise, interception deserves a generalized runtime-semantics model

I would not normalize this merely to:

```text
intercepts: bool
```

because there may eventually be several customization hooks:

```text
message-send interception
missing-message handling
method lookup customization
class-side lookup customization
construction
attribute/property lookup
conversion/coercion
iteration
callability
```

You don't need to design all those now. But a representation like:

```rust
pub struct ClassBehavior {
    pub message_interception: Option<MessageInterception>,
    // future hooks
}
```

is preferable to accumulating flags directly on `Class`.

And, critically, the source of this metadata remains the special method declaration.

## Phaldoc can still give these features first-class presentation

“Not a Phaldoc directive” does not mean “buried somewhere in metadata.”

In fact, I would make Phaldoc rendering especially aware of important semantic facts.

For a suspending method:

```text
NetworkStream#read(_)

(Buffer) -> Result<Int, IOError>

suspends · io

Reads data into `buffer`.

Suspension
    May suspend while waiting for the stream to become readable.
```

The first line is generated machine metadata.

The explanatory paragraph might come from ordinary Phaldoc:

```phalcom
/// Reads data into `buffer`.
///
/// May suspend while waiting for the stream to become readable.
@suspends
read(buffer: Buffer) -> Result<Int, IOError>
```

You do not necessarily even need a “Suspension” section unless the author adds one:

```phalcom
/// ## Suspension
///
/// The current fiber is parked until the descriptor becomes readable.
```

Likewise:

```text
Proxy

message-intercepting

A transparent forwarding proxy.

Message interception
    Proxy#__intercept__(_,_)
```

can be generated from semantic information.

Phaldoc can explain the policy:

```phalcom
/// ## Message interception
///
/// Unknown application messages are forwarded to `target`; reflection
/// messages are handled by the proxy itself.
```

Again, fact versus explanation.

## I would therefore add a “semantic badges” concept to generated docs

You already have:

```text
stable · pure · native
```

That can generalize naturally:

```text
public · stable · suspends · io
```

or for classes:

```text
public · message-intercepting
```

These aren't Phaldoc syntax. They're a documentation projection of the semantic model.

Potential categories:

```text
Callable
    pure
    suspends
    blocking
    io
    unsafe
    native
    intrinsic

Class
    abstract
    sealed
    message-intercepting
    native
    generated

Declaration
    experimental
    deprecated
    internal
```

You probably want the renderer to decide which facts deserve badges rather than blindly dumping every semantic bit.

## My recommended decision

I would ratify these rules:

1. `@suspends` is a semantic callable declaration, not Phaldoc. Internally, model it as suspension/effect metadata. Permit inference where possible, while preserving explicit API contracts.

2. Message interception is semantic class behavior, not Phaldoc. If interception is established by implementing/overriding a distinguished method such as `__intercept__`, do not add a redundant `@intercepts` attribute. Derive the class capability from effective method resolution.

3. Phaldoc may explain either behavior in prose or arbitrary Markdown sections, but cannot declare either behavior.

4. Generated documentation should prominently display both facts from semantic metadata.

5. The normalized documentation system should therefore consume more than just “attributes + Phaldoc.” It consumes the whole semantic declaration model:

```text
source declaration
      │
      ├── explicit semantic attributes
      │       @suspends
      │       @requires
      │       ...
      │
      ├── derived semantic properties
      │       message interception
      │       inherited capabilities
      │       synthesized contracts
      │
      ├── type information
      │
      └── Phaldoc
              human explanation
              examples
              semantic links
              @param / @returns / @raises / @see

                    ↓

          Documentation Record

                    ↓

        HTML / LSP / CLI / index
```

I think this also exposes one useful improvement to the proposal itself: replace the narrower doctrine “Effects do not belong in Phaldoc” with the more general rule **“Machine-actionable behavioral semantics do not belong in Phaldoc.”**

That gives you a durable answer not just for `suspends` and interception, but for the next twenty similar questions that will inevitably appear as Phalcom gains concurrency, typing, metaprogramming, contracts, and deeper runtime hooks.