# Phalcom Mirror: protocol-driven doubles, adapters, and contract tests

The strongest demonstration is a single library—`mirror`—that treats a protocol as an executable behavioral schema.

A protocol already exposes its required selectors, parameter structure, generic substitutions, return annotations, and retained attributes. It is therefore enough information to generate ordinary conforming methods, record invocations, translate calls to incompatible APIs, and derive contract tests. This fits Phalcom’s model particularly well because protocols are first-class reflective descriptors rather than abstract superclasses. Branch · Phalcom Hypothesis Implementation.txtTXT Branch · Phalcom Hypothesis Implementation.txtTXT

The demonstration should use a payment gateway because it has realistic requirements:

- Application code depends on a clean protocol.
- Tests need strict mocks and interaction verification.
- Production must adapt an incompatible third-party client.
- Every implementation must satisfy the same preconditions, postconditions, and behavioral laws.

## 1. Architectural choice

There are three possible implementations.

**Catch-all dispatch** would implement mocks through `doesNotUnderstand` or a future `intercept`. It is easy, but the generated object does not visibly implement the protocol’s methods, conformance diagnostics become weaker, and every invocation travels through exceptional dispatch.

**Compile-time generation** would derive a concrete mock or adapter class from a decorator. It is efficient, but less impressive as a first-class metaprogramming example and cannot naturally consume dynamically constructed applied types.

**Reflective class synthesis** is the recommended model. `Mirror` reflects a protocol, creates a concrete class containing one ordinary method per requirement, attaches the original annotations, mixes in reusable behavior, verifies conformance, and caches the generated class.

This requires one explicit runtime facility:

```
BehaviorBuilder
  .named(name: Symbol)
  .is(superclass: Class)
  .with(*mixins: Class)
  .conforms(*protocols: Type)
  .method(requirement: MethodRequirement, body: Block)
  .build() -> Class
```

This is materially better than catch-all interception. Once built, the methods are ordinary methods. They participate in reflection, dispatch caches, tracing, stack frames, and protocol conformance like handwritten code.

Applied generic types such as `PaymentGateway<ChargeRequest, ChargeReceipt>` remain canonical reflective type objects, not newly generated runtime classes. `Mirror` uses that type object as the key from which it synthesizes and caches a separate implementation class. Branch · Phalcom Hypothesis Implementation.txtTXT

---

# 2. Domain model

```
@data @immutable
class RequestId {
  const _value: String
}

@data @immutable
class PaymentId {
  const _value: String
}

@data @immutable
class Currency {
  const _code: String
}

@data @immutable
class Money {
  const _minor: Int
  const _currency: Currency

  @invariant(_minor >= 0)
}

@data @immutable
class ChargeRequest {
  const _id: RequestId
  const _amount: Money
  const _merchant: String
}

@data @immutable
class PaymentContext {
  const _idempotencyKey: String
  const _metadata: Record
}

@data @immutable
class ChargeReceipt {
  const _paymentId: PaymentId
  const _requestId: RequestId
  const _approved: Bool
}

@data @immutable
class RefundReceipt {
  const _paymentId: PaymentId
  const _amount: Money
}

@data @sealed
class PaymentError {
  @variant Declined(reason:)
  @variant Unavailable(retryAfter:)
  @variant Unknown(message:)
}
```

The immutable data classes are descriptive values. The payment service, generated adapters, mock state, and invocation handlers remain mutable workers.

---

# 3. The protocol is the source of truth

```
@protocol
class PaymentGateway<in Request, out Receipt> {
  @requires(request.amount.minor > 0)
  @ensures(result.requestId == request.id)
  charge(
    request: Request,
    context: PaymentContext
  ) -> Receipt

  @requires(amount.minor > 0)
  refund(
    paymentId: PaymentId,
    amount: Money
  ) -> Result<RefundReceipt, PaymentError>
}
```

`PaymentGateway<ChargeRequest, ChargeReceipt>` is an applied protocol type. Its reflected `charge` requirement has already substituted:

```
Request → ChargeRequest
Receipt → ChargeReceipt
```

Conceptually:

```
const gatewayType =
  PaymentGateway<ChargeRequest, ChargeReceipt>

gatewayType.origin
// PaymentGateway

gatewayType.arguments
// const (ChargeRequest, ChargeReceipt)

gatewayType.requirements
// charge(_:context:) -> ChargeReceipt
// refund(_:amount:)  -> Result<RefundReceipt, PaymentError>
```

The type annotations remain inert during ordinary program execution. `Mirror` explicitly chooses to inspect and apply them; it does not turn them into universal runtime checks. This maintains Phalcom’s reflective-but-dynamic typing model. Phalcom Hypothesis Implementation.txtTXT

---

# 4. Invocation representation

Every mock and adapter works with the same immutable invocation model.

```
@data @immutable
class Arguments {
  const _positional: Tuple
  const _labeled: Record

  all -> Tuple {
    return (*_positional, **_labeled)
  }
}

@data @immutable
class Invocation<T> {
  const _receiverType: Type
  const _requirement: MethodRequirement
  const _arguments: Arguments
  const _source: SourceLocation

  selector -> Selector {
    return _requirement.selector
  }

  returnType -> Type {
    return _requirement.returnType
  }
}
```

The two argument lanes are preserved rather than flattened into a string-keyed dictionary:

```
const arguments = Arguments.new(
  positional: (request),
  labeled: (context: context)
)

const reconstructed = (*arguments.positional, **arguments.labeled)
```

That distinction is essential. Selectors include positional structure and labels; `charge(_:context:)` cannot be treated as merely the text `"charge"`.

Records are used for open-ended descriptive metadata:

```
const metadata = {
  source: #checkout,
  traceId: "tr-104",
  customer: customer.id
}

const contextualMetadata = {
  **metadata,
  attempt: 2
}
```

A `*` expansion can preserve both tuple lanes, while `**` explicitly expands only the labeled lane.

---

# 5. Shared protocols, abstract base, and mixin

The generated classes need a small ordinary object hierarchy.

```
@protocol
class InvocationHandler {
  invoke<T>(invocation: Invocation<T>) -> T
}

@protocol
class HasInvocationState {
  invocationState -> InvocationState
}
```

The abstract base owns the mechanics common to mocks and adapters.

```
@abstract
class AbstractMirrorObject<T> {
  const _mirroredType: Type
  const _handler: InvocationHandler

  @constructor
  new(
    mirroredType: Type,
    handler: InvocationHandler
  ) {
    _mirroredType = mirroredType
    _handler = handler
  }

  mirroredType -> Type {
    return _mirroredType
  }

  dispatch<R>(
    requirement: MethodRequirement,
    positional: Tuple,
    labeled: Record
  ) -> R {
    const invocation = Invocation<R>.new(
      receiverType: _mirroredType,
      requirement: requirement,
      arguments: Arguments.new(
        positional: positional,
        labeled: labeled
      ),
      source: SourceLocation.caller
    )

    return _handler.invoke(invocation)
  }
}
```

The mixin provides reusable invocation history without pretending to define substitutability.

```
@mixin(HasInvocationState)
class RecordsInvocations {
  record(invocation: Invocation<Any>) {
    invocationState.record(invocation)
  }

  calls -> const List<Invocation<Any>> {
    return invocationState.calls
  }

  callsTo(selector: Selector) -> const List<Invocation<Any>> {
    return calls.select { call =>
      call.selector == selector
    }
  }
}
```

This division is clean:

- `InvocationHandler` and `HasInvocationState` describe required behavior.
- `AbstractMirrorObject<T>` supplies storage and the dispatch template.
- `RecordsInvocations` supplies reusable implementation.
- The synthesized concrete class supplies the protocol’s actual selectors.

---

# 6. What Mirror generates

Given:

```
const type =
  PaymentGateway<ChargeRequest, ChargeReceipt>
```

`Mirror` conceptually synthesizes:

```
@conforms(
  PaymentGateway<ChargeRequest, ChargeReceipt>,
  HasInvocationState
)
@with(RecordsInvocations)
class _PaymentGatewayMirror
  is AbstractMirrorObject<
    PaymentGateway<ChargeRequest, ChargeReceipt>
  > {

  charge(
    request: ChargeRequest,
    context: PaymentContext
  ) -> ChargeReceipt {
    return dispatch(
      requirement:
        mirroredType.requirement(#charge(_:context:)),
      positional: (request),
      labeled: (context: context)
    )
  }

  refund(
    paymentId: PaymentId,
    amount: Money
  ) -> Result<RefundReceipt, PaymentError> {
    return dispatch(
      requirement:
        mirroredType.requirement(#refund(_:amount:)),
      positional: (paymentId),
      labeled: (amount: amount)
    )
  }
}
```

These are ordinary methods. There is no `doesNotUnderstand` dependency and no second dispatch system.

The method builder copies:

- the exact selector;
- parameter names and labels;
- substituted parameter types;
- return type;
- retained requirement attributes;
- documentation and source provenance;
- explicit conformance metadata.

The generated class is cached under a key such as:

```
@data @immutable
class MirrorClassKey {
  const _type: Type
  const _kind: MirrorKind
  const _mixins: Tuple
  const _fingerprint: Bytes
}
```

The fingerprint changes when a protocol requirement or relevant annotation changes.

---

# 7. Protocol-driven mocks

The public API returns a handle rather than pretending one object should simultaneously have the static type `T` and expose mock-control methods.

```
@data
class MockHandle<T> {
  const _object: T
  const _state: MockState

  object -> T {
    return _object
  }

  when(selector: Selector) -> StubBuilder<T> {
    return StubBuilder.new(_state, selector)
  }

  verify(
    selector: Selector,
    times: Int,
    where predicate: [Invocation<Any>] -> Bool
  ) {
    _state.verify(
      selector,
      times: times,
      where: predicate
    )
  }
}
```

## Test setup

```
const payments =
  Mirror.mock(
    PaymentGateway<ChargeRequest, ChargeReceipt>,
    options: {
      strict: true,
      recordCalls: true
    }
  )
```

The stub is matched using the protocol selector and a labeled record of argument matchers:

```
payments
  .when(#charge(_:context:))
  .matching({
    0: Match.where { request =>
      request.amount.minor <= 100_00
    },
    context: Match.any<PaymentContext>
  })
  .then { invocation =>
    const request =
      invocation.arguments.positional.at(0)

    ChargeReceipt.new(
      paymentId: PaymentId.new("pay-test-1"),
      requestId: request.id,
      approved: true
    )
  }
```

A regular application class consumes only the protocol:

```
class CheckoutService {
  const _payments:
    PaymentGateway<ChargeRequest, ChargeReceipt>

  @constructor
  new(
    payments:
      PaymentGateway<ChargeRequest, ChargeReceipt>
  ) {
    _payments = payments
  }

  checkout(
    request: ChargeRequest,
    context: PaymentContext
  ) -> ChargeReceipt {
    return _payments.charge(
      request,
      context: context
    )
  }
}
```

The test:

```
const checkout =
  CheckoutService.new(payments.object)

const receipt =
  checkout.checkout(
    request,
    context: PaymentContext.new(
      idempotencyKey: "checkout-17",
      metadata: {
        customer: "customer-4"
      }
    )
  )

Assert.true(receipt.approved)

payments.verify(
  #charge(_:context:),
  times: 1,
  where: { invocation =>
    const calledRequest =
      invocation.arguments.positional.at(0)

    const calledContext =
      invocation.arguments.labeled.context

    calledRequest.id == request.id and
      calledContext.idempotencyKey == "checkout-17"
  }
)
```

An unstubbed strict invocation fails immediately:

```
UnexpectedInvocationError

PaymentGateway<ChargeRequest, ChargeReceipt>
received:

  refund(_:amount:)(
    PaymentId("pay-17"),
    amount: Money(500, USD)
  )

No stub matched this invocation.

Declared return type:
  Result<RefundReceipt, PaymentError>

Available stubs:
  charge(_:context:)
```

---

# 8. Protocol-driven adapters

Suppose the third-party library has a bad but unavoidable API:

```
class LegacyPaymentsClient {
  createPayment(
    cents: Int,
    currency: String,
    idempotencyKey: String
  ) -> Record {
    ...
  }

  reverse(
    paymentReference: String,
    cents: Int,
    currency: String
  ) -> Record {
    ...
  }
}
```

It does not conform to `PaymentGateway`. `Mirror.adapt` synthesizes a conforming façade.

```
const legacy = LegacyPaymentsClient.new(configuration)

const gateway =
  Mirror
    .adapt(
      legacy,
      to:
        PaymentGateway<
          ChargeRequest,
          ChargeReceipt
        >
    )
    .route(
      #charge(_:context:),
      to: #createPayment(_:_:idempotencyKey:),
      arguments: { invocation =>
        const request =
          invocation.arguments.positional.at(0)

        const context =
          invocation.arguments.labeled.context

        (
          request.amount.minor,
          request.amount.currency.code,
          idempotencyKey: context.idempotencyKey
        )
      },
      result: { raw =>
        ChargeReceipt.new(
          paymentId:
            PaymentId.new(raw.paymentReference),
          requestId:
            RequestId.new(raw.metadata.requestId),
          approved: raw.status == "accepted"
        )
      }
    )
    .route(
      #refund(_:amount:),
      to: #reverse(_:_:_:),
      arguments: { invocation =>
        const paymentId =
          invocation.arguments.positional.at(0)

        const amount =
          invocation.arguments.labeled.amount

        (
          paymentId.value,
          amount.minor,
          amount.currency.code
        )
      },
      result: { raw =>
        if raw.ok {
          return Result.Ok(
            RefundReceipt.new(
              paymentId:
                PaymentId.new(raw.paymentReference),
              amount: Money.new(
                minor: raw.refundedCents,
                currency:
                  Currency.new(raw.currency)
              )
            )
          )
        }

        return Result.Err(
          PaymentError.Declined(
            reason: raw.reason
          )
        )
      }
    )
    .build()
```

Internally, each generated adapter method performs:

```
const mapped =
  route.arguments.invoke(invocation)

const raw =
  source.send(route.target, *mapped)

return route.result.invoke(raw)
```

The route transformation returns a tuple whose positional and labeled lanes reconstruct the legacy call exactly.

Record spread is useful for route definitions and environment-specific overrides:

```
const commonRoute = {
  timeout: 2.seconds,
  retry: Retry.none,
  captureTrace: true
}

const productionRoute = {
  **commonRoute,
  timeout: 5.seconds,
  retry: Retry.exponential(maxAttempts: 3)
}
```

At `.build()`, the adapter validates completeness. Missing mappings are detected before the adapter is used:

```
AdapterIncompleteError

Cannot adapt LegacyPaymentsClient to
PaymentGateway<ChargeRequest, ChargeReceipt>.

Unimplemented requirement:
  refund(_:amount:)
    PaymentId
    amount: Money
    -> Result<RefundReceipt, PaymentError>
```

A direct compatible method can be forwarded automatically. Incompatible selectors always require an explicit route.

---

# 9. Contract testing from reflection

Mock generation proves that an object has the correct surface. Contract testing proves that an implementation behaves according to the protocol’s retained contracts.

```
@abstract
class Contract<T> {
  @abstract
  name -> Symbol

  @abstract
  check(
    subject: T,
    invocation: Invocation<Any>,
    result: Any
  ) -> None
}
```

The standard contract suite derives checks from reflected protocol metadata:

```
class ProtocolContractSuite<T> {
  const _mirror: ProtocolMirror<T>
  const _contracts: List<Contract<T>>

  check(
    subject: T,
    settings: Settings
  ) -> ContractReport {
    ...
  }
}
```

Usage:

```
const suite =
  Mirror.contracts(
    PaymentGateway<
      ChargeRequest,
      ChargeReceipt
    >
  )

const report = suite.check(
  gateway,
  settings:
    Settings.standard
      .maxExamples(500)
      .maxShrinks(2000)
)
```

The runner performs the following for each requirement:

1. Reflect the fully substituted parameter types.
2. Ask Hypothesis’s strategy registry for generators.
3. Generate positional and labeled argument lanes.
4. Evaluate reflected `@requires` metadata.
5. Skip generated calls outside the declared domain.
6. Invoke the implementation with `*positional` and `**labeled`.
7. Explicitly validate the result against the reflected return type.
8. Evaluate every reflected `@ensures`.
9. Shrink a failure to a minimal invocation.
10. report the exact requirement, arguments, result, and violated contract.

This is where Phalcom’s reflective type system becomes genuinely useful. The Hypothesis design already treats retained annotations as a basis for deriving strategies rather than implicit runtime enforcement. Phalcom Hypothesis Implementation.txtTXT Phalcom Hypothesis Implementation.txtTXT

A failure might become:

```
ContractViolation

Implementation:
  _LegacyPaymentsClientAsPaymentGateway

Protocol:
  PaymentGateway<ChargeRequest, ChargeReceipt>

Requirement:
  charge(_:context:) -> ChargeReceipt

Minimal invocation:
  request = ChargeRequest(
    id: RequestId(""),
    amount: Money(1, Currency("USD")),
    merchant: ""
  )

  context = PaymentContext(
    idempotencyKey: "",
    metadata: {}
  )

Result:
  ChargeReceipt(
    paymentId: PaymentId("0"),
    requestId: RequestId("legacy-generated-id"),
    approved: true
  )

Violated postcondition:
  result.requestId == request.id
```

That failure is far more useful than “adapter returned the wrong value.”

---

# 10. Cross-implementation behavioral laws

Preconditions and postconditions cover one invocation. Protocols often need multi-call laws.

```
class IdempotentChargeContract
  is Contract<
    PaymentGateway<
      ChargeRequest,
      ChargeReceipt
    >
  > {

  name -> Symbol {
    return #idempotentCharge
  }

  check(
    subject:
      PaymentGateway<
        ChargeRequest,
        ChargeReceipt
      >,
    invocation: Invocation<Any>,
    result: Any
  ) {
    if invocation.selector != #charge(_:context:) {
      return
    }

    const arguments = invocation.arguments

    const again = subject.charge(*arguments.all)

    Assert.equal(
      result.paymentId,
      again.paymentId,
      because: #sameIdempotencyKey
    )
  }
}
```

It can be added without changing the protocol:

```
const suite =
  Mirror
    .contracts(
      PaymentGateway<
        ChargeRequest,
        ChargeReceipt
      >
    )
    .including(
      IdempotentChargeContract.new()
    )
```

The same suite can run against:

```
const implementations = {
  reference:
    InMemoryPaymentGateway.new(),

  legacy:
    gateway,

  sandbox:
    SandboxPaymentGateway.new(configuration)
}

implementations.each { name, implementation =>
  suite.check(
    implementation,
    settings:
      Settings.standard.seed(42)
  )
}
```

The record makes the implementation name part of diagnostics, while record spread allows environment-specific additions:

```
const ciImplementations = {
  **implementations,
  remoteSandbox:
    RemoteSandboxGateway.new(secrets)
}
```

---

# 11. Reflection surface exercised

This demonstration gives a concrete consumer for nearly every major reflective object:

```
const mirror =
  Mirror.reflect(
    PaymentGateway<
      ChargeRequest,
      ChargeReceipt
    >
  )

mirror.type
mirror.type.origin
mirror.type.arguments
mirror.protocol
mirror.protocol.requirements

mirror.requirement(#charge(_:context:)).selector
mirror.requirement(#charge(_:context:)).parameters
mirror.requirement(#charge(_:context:)).returnType
mirror.requirement(#charge(_:context:)).attributes

mirror.requirement(#charge(_:context:))
  .parameter(#context)
  .label

mirror.requirement(#charge(_:context:))
  .attributesOfType(Requires)

mirror.requirement(#charge(_:context:))
  .attributesOfType(Ensures)
```

The protocol descriptor provides shape. Generic application provides substitution. Method reflection provides selectors and argument lanes. Attributes provide contracts. Mixins provide implementation reuse. Abstract classes provide lifecycle and storage. Regular classes remain the actual business objects.

---

# 12. Critical semantic boundaries

The library should enforce several hard rules.

**No implicit mocking through `doesNotUnderstand`.** Generated objects must have real methods for every protocol requirement.

**Mocks are strict by default.** Returning `None`, zero, or an arbitrary generated value for an unstubbed call hides defects.

**Adapters are complete at build time.** Every requirement must be directly compatible or explicitly routed.

**Return checks are library behavior.** Type annotations remain inert outside the contract runner.

**Contracts do not alter normal dispatch.** `@requires` and `@ensures` are deliberately evaluated by the contract engine.

**Generated methods preserve selectors exactly.** Labels, positional parameters, rest parameters, and return metadata cannot be normalized away.

**Protocol changes invalidate cached mirrors.** A structural fingerprint must include requirements, generic substitutions, and contract metadata.

**Mocks and adapters never manufacture nominal identity.** They structurally conform to the protocol; they do not become subclasses of a protocol descriptor.

---

# 13. What this proves about Phalcom

This is a serious metaprogramming demonstration rather than a decorative reflection example.

It proves that Phalcom can use its own object model to build developer tooling that normally requires compiler plugins or extensive language magic:

- A first-class protocol becomes an executable schema.
- Applied generic types become stable registry and cache keys.
- Abstract classes supply controlled implementation inheritance.
- Mixins supply limited, explicit code reuse without multiple class inheritance.
- Regular generated classes preserve ordinary dispatch.
- Tuples preserve positional and labeled argument structure.
- Records carry extensible configuration and metadata.
- `*` and `**` reconstruct calls and compose declarative specifications.
- Retained type information drives generation and diagnostics.
- Contract attributes become testable runtime metadata.
- Hypothesis supplies generated inputs and minimal counterexamples.
- Mocks, adapters, and real implementations share one conformance and contract engine.

The core recommendation is therefore:

```
Protocol descriptor
        │
        ▼
ProtocolMirror<T>
        │
        ├── MockClass<T>
        ├── AdapterClass<T>
        └── ContractSuite<T>
                │
                ▼
       Hypothesis-generated calls
```

That is the right showcase for Phalcom’s reflective model. It demonstrates that first-class types are not merely richer annotations: they enable libraries to synthesize real behavior, bridge incompatible systems, and mechanically test architectural promises.