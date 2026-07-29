```markdown
# Phalcom User-Defined Message Interception and Dynamic Dispatch Specification

## Status

**Proposal**

## Scope

This specification defines:

- user-defined message interception;
- the `intercept(message:proceed:)` protocol;
- the `Message` meta-object;
- the `Dispatch` continuation capability;
- interaction with ordinary method lookup;
- interaction with `doesNotUnderstand`;
- typing and reflection semantics;
- optimization constraints;
- recursion rules;
- security considerations;
- restrictions on global interception.

The goal is to provide powerful meta-object capabilities while preserving Phalcom's core principles:

- message-oriented object semantics;
- selector-based dispatch;
- reflective runtime;
- optimizable method lookup;
- explicit dynamic behavior;
- trustworthy type information.

---

# 1. Design Principles

## 1.1 Message sending is the fundamental operation

Phalcom is message-oriented.

A send:

```phalcom
receiver.save(value: item)
```

is conceptually:

```text
receiver
    receives selector
        save(value:)
    with arguments
        item
```

The runtime performs:

```text
receiver
    ↓
class lookup
    ↓
method resolution
    ↓
method invocation
```

User-defined interception inserts an explicit meta-object layer into this process.

---

# 2. Dispatch Pipeline

The complete dispatch pipeline is:

```text
Message Send
    |
    v
Evaluate receiver
    |
    v
Evaluate arguments
    |
    v
Check receiver interception policy
    |
    +----------------+
    |                |
    | No interceptor |
    |                |
    +----------------+
            |
            v
    Ordinary selector lookup
            |
            +----------------+
            |                |
            | Found          | Missing
            |                |
            v                v
        Method call     doesNotUnderstand(message)


    With interceptor:

Message Send
    |
    v
intercept(message:proceed:)
    |
    +----------------------------+
    |                            |
    | proceed.invoke()            |
    |                            |
    +----------------------------+
                 |
                 v
        Ordinary selector lookup
                 |
                 +----------------+
                 |                |
                 | Found          | Missing
                 |                |
                 v                v
             Method call     doesNotUnderstand(message)


    Without proceed:

Message Send
    |
    v
intercept(message:proceed:)
    |
    v
Interceptor return value
```

---

# 3. `doesNotUnderstand`

## 3.1 Purpose

`doesNotUnderstand` handles messages that have no concrete implementation after normal lookup.

It is the final fallback mechanism.

Example:

```phalcom
class Proxy {

  doesNotUnderstand(message: Message) {
    return message.forward(to: _target)
  }

}
```

A missing send:

```phalcom
proxy.download(path: "/tmp/file")
```

becomes:

```text
lookup download(path:)
        |
        v
missing
        |
        v
doesNotUnderstand(Message)
```

---

# 3.2 Relationship with Python

Conceptually:

| Python | Phalcom |
|---|---|
| `__getattr__` | `doesNotUnderstand` |
| `__getattribute__` | `intercept` |
| `__setattr__` | setter selectors + interception |
| `__delattr__` | not generally applicable |

The difference:

Python intercepts attribute operations.

Phalcom intercepts messages.

A Phalcom message includes:

- selector;
- arity;
- labels;
- arguments;
- receiver;
- source metadata.

---

# 3.3 `doesNotUnderstand` is not universal interception

This:

```phalcom
doesNotUnderstand(message)
```

only runs after lookup fails.

It does not observe:

```phalcom
object.existingMethod()
```

when:

```text
existingMethod()
```

already exists.

For observing or replacing existing behavior, use `intercept`.

---

# 4. User-Defined Interceptor

## 4.1 Declaration

A class opts into interception explicitly.

```phalcom
@interceptsMessages
class LoggingProxy {

  intercept(
    message: Message,
    proceed: Dispatch
  ) -> Any {

    System.print(
      "Calling \(message.selector)"
    )

    return proceed.invoke()
  }

}
```

The attribute is mandatory.

Defining:

```phalcom
intercept(...)
```

without:

```phalcom
@interceptsMessages
```

does nothing special.

---

# 4.2 Why explicit activation is required

Automatic interception based on method naming would:

- surprise users;
- invalidate optimizer assumptions;
- make reflection ambiguous;
- allow accidental semantic changes.

This is forbidden:

```phalcom
class Account {

  intercept(message, proceed) {
    ...
  }

}
```

The runtime treats it as an ordinary method.

---

# 5. Message Object

## 5.1 Definition

The interceptor receives a structured immutable message.

Conceptually:

```phalcom
@data
@immutable
class Message {

  const receiver: Object

  const selector: Selector

  const arguments: List<Argument>

  const kind: MessageKind

  const source: Option<SourceLocation>

}
```

---

# 5.2 Message contents

A message contains:

## Receiver

The original target:

```phalcom
message.receiver
```

---

## Selector

Complete selector identity:

Examples:

```text
name

save(_)

save(value:)

at(_)

atPut(_:value:)
```

Selector identity includes:

- base name;
- positional arity;
- labels.

---

## Arguments

Arguments are already evaluated.

Example:

```phalcom
object.process(expensive())
```

Evaluation order:

```text
1. evaluate object
2. evaluate expensive()
3. create Message
4. intercept
```

Interception does not create call-by-name semantics.

---

## Source information

Optional:

```phalcom
message.source
```

Used for:

- diagnostics;
- tracing;
- security auditing;
- debugging.

---

# 6. Dispatch Continuation

## 6.1 Purpose

`Dispatch` represents the operation that would have occurred without interception.

Example:

```phalcom
intercept(message, proceed) {

  Audit.record(message)

  return proceed.invoke()

}
```

---

# 6.2 Invocation semantics

Calling:

```phalcom
proceed.invoke()
```

means:

> Continue normal message dispatch from the current interception point.

It performs:

```text
skip current interceptor
        |
        v
ordinary lookup
        |
        v
method or doesNotUnderstand
```

---

# 6.3 Single-use rule

`Dispatch` is single-use.

Valid:

```phalcom
return proceed.invoke()
```

Invalid:

```phalcom
const a = proceed.invoke()
const b = proceed.invoke()
```

Result:

```text
DispatchAlreadyInvokedError
```

Reason:

Repeating a message may repeat side effects:

```phalcom
account.withdraw(amount)
```

Calling twice is not equivalent to one call.

---

# 6.4 Lifetime rule

`Dispatch` cannot escape the interceptor.

Invalid:

```phalcom
class Example {

  var saved

  intercept(message, proceed) {

    saved = proceed

    return None
  }

}
```

Later:

```phalcom
saved.invoke()
```

fails:

```text
ExpiredDispatchError
```

Reason:

`Dispatch` may reference:

- stack state;
- temporary VM objects;
- exception context;
- invocation metadata.

---

# 7. Interceptor Outcomes

An interceptor has three possible outcomes.

---

# 7.1 Continue

```phalcom
intercept(message, proceed) {

  return proceed.invoke()

}
```

Meaning:

```text
perform original operation
```

---

# 7.2 Replace

```phalcom
intercept(message, proceed) {

  return Cache.get(message)

}
```

Meaning:

```text
do not call original operation
return this value instead
```

---

# 7.3 Reject

```phalcom
intercept(message, proceed) {

  throw AccessDenied.new()

}
```

Meaning:

```text
abort operation
```

---

# 8. Intentional Suppression

A bare return can accidentally swallow a message:

```phalcom
intercept(message, proceed) {

  Audit.record(message)

}
```

This returns:

```phalcom
None
```

and suppresses the original call.

Therefore tooling should warn:

```text
warning:
interceptor exits without invoking proceed

Possible intentions:
- return replacement value
- throw error
- call proceed.invoke()
```

---

# 9. Recommended Suppression API

Optional explicit API:

```phalcom
return proceed.suppress(with: value)
```

Example:

```phalcom
intercept(message, proceed) {

  if message.selector == #delete {

    return proceed.suppress(
      with: false
    )

  }

  return proceed.invoke()

}
```

Purpose:

- documentation;
- static analysis;
- tooling.

It does not change semantics.

---

# 10. Recursion Rules

## 10.1 Interceptor lookup bypass

The runtime must invoke:

```text
intercept(message, proceed:)
```

through a raw VM method handle.

Otherwise:

```text
call interceptor
    |
    v
interceptor lookup
    |
    v
interceptor
    |
    v
infinite recursion
```

---

# 10.2 Messages inside interceptor remain intercepted

Example:

```phalcom
intercept(message, proceed) {

  self.log(message)

  return proceed.invoke()

}
```

The call:

```phalcom
self.log(message)
```

is itself intercepted.

This is intentional.

---

# 10.3 Field access bypasses interception

Inside a class:

```phalcom
_count = _count + 1
```

is direct field access.

It is not:

```phalcom
self._count()
```

or:

```phalcom
self.setCount(...)
```

Reason:

The interceptor itself needs a stable internal state mechanism.

---

# 11. Interaction With `doesNotUnderstand`

Ordering:

```text
interceptor
    |
    v
proceed.invoke()
    |
    v
ordinary lookup
    |
    +-------------+
    |             |
 found       missing
    |             |
    v             v
method    doesNotUnderstand
```

Example:

```phalcom
class Proxy {

  intercept(message, proceed) {

    return proceed.invoke()

  }


  doesNotUnderstand(message) {

    return Remote.send(message)

  }

}
```

Unknown messages still reach:

```text
doesNotUnderstand
```

because the interceptor continued dispatch.

---

# 12. Reflection Rules

## 12.1 `respondsTo`

This:

```phalcom
object.respondsTo(#save)
```

means:

> Does ordinary method lookup find `save`?

It does not mean:

> Could an interceptor handle `save`?

---

# 12.2 Dynamic capability query

A separate API may exist:

```phalcom
object.acceptsMessage(#save)
```

Possible results:

```phalcom
Implemented

Intercepted

Fallback

Rejected

Unknown
```

---

# 12.3 Protocol conformance

Interception does not automatically satisfy protocols.

Invalid assumption:

```phalcom
Strategy<Int>.conforms(proxy)
```

because:

```text
proxy can intercept draw()
```

A protocol describes actual capability, not arbitrary message fabrication.

---

# 13. Typing Model

## 13.1 Interceptor result type

Because an interceptor may replace any operation:

```phalcom
intercept(message, proceed)
```

returns:

```phalcom
Any
```

by default.

Example:

```phalcom
class BadInterceptor {

  intercept(message, proceed) {

    return "wrong type"

  }

}
```

The runtime permits this.

---

# 13.2 Static checking

The checker should warn:

```text
interceptor result cannot be proven compatible
with original method result
```

Example:

Original:

```phalcom
balance() -> Int
```

Interceptor:

```phalcom
return "hello"
```

Diagnostic:

```text
possible type violation:
interceptor replaces Int result with String
```

---

# 13.3 Typed interceptors

Future extension:

```phalcom
intercept<T>(
  message: Message<T>,
  proceed: Dispatch<T>
) -> T
```

Benefits:

- preserves return typing;
- improves static analysis.

Limitations:

- unknown selectors;
- `doesNotUnderstand`;
- dynamically generated messages.

Therefore this should be optional.

---

# 14. Optimization Rules

## 14.1 Non-intercepted objects

Fast path:

```text
receiver class
      |
      v
inline cache
      |
      v
method
```

No Message allocation.

---

## 14.2 Intercepted objects

Path:

```text
receiver class
      |
      v
interceptor check
      |
      v
Message creation
      |
      v
intercept()
```

---

# 14.3 Inline cache guards

A cached call requires:

```text
receiver class unchanged

AND

dispatch epoch unchanged

AND

interception policy unchanged
```

---

# 14.4 Global interception forbidden

This is rejected:

```phalcom
Object.intercept(...)
```

Reason:

It destroys:

- inline caches;
- method specialization;
- reflection assumptions;
- kernel reliability.

---

# 15. Scope of Interception

## Supported

Interception applies to:

- methods;
- getters;
- setters;
- operators;
- indexing;
- labeled messages;
- inherited methods.

---

## Not supported

Interception does not apply to:

- local variables;
- lexical fields;
- GC operations;
- allocation internals;
- stack manipulation;
- VM bookkeeping.

---

# 16. Class Inheritance

Interception is inherited.

Example:

```phalcom
@interceptsMessages
class Proxy {

  intercept(...) {}

}


class RemoteProxy is Proxy {

}
```

`RemoteProxy` remains intercepting.

---

# 16.1 Class closure

Once a class closes:

- interceptor status is fixed;
- interceptor method is fixed;
- dispatch caches are valid.

Dynamic mutation of interception policy after closure is forbidden.

---

# 17. Recommended User Model

The hierarchy of dynamic behavior is:

```text
                    Message Send
                         |
                         v
              User Interceptor (optional)
                         |
                         v
              Ordinary Method Dispatch
                         |
                         v
              doesNotUnderstand fallback
```

Use cases:

| Feature | Mechanism |
|-|-|
| Missing methods | `doesNotUnderstand` |
| RPC objects | proxy + `doesNotUnderstand` |
| Logging | interceptor |
| Authorization | interceptor |
| Caching | interceptor |
| Mocking | interceptor/proxy |
| Lazy objects | interceptor |
| Debug tracing | VM observer |
| Profiling | VM observer |

---

# 18. Final Recommendation

Phalcom should provide:

## Required

```phalcom
doesNotUnderstand(message: Message)
```

with structured forwarding support.

## Advanced

```phalcom
@interceptsMessages
intercept(
    message: Message,
    proceed: Dispatch
)
```

with:

- explicit activation;
- single-use `proceed`;
- no global `Object` interception;
- optimizer-aware dispatch guards;
- reflection separation;
- static warnings.

The result is a Smalltalk-inspired meta-object protocol adapted for a modern optimizing VM:

- dynamic enough for proxies and metaprogramming;
- explicit enough for reasoning;
- constrained enough for performance.
```