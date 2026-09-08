from universe.concurrency.fiber import Future

// concurrency_order_fulfillment.ph
//
// Integrated concurrency regression / behavior test for Phalcom.
//
// Scenario:
//   A small fulfillment service accepts an order, concurrently reserves stock
//   and obtains a shipping quote, computes the payable amount through a Future
//   continuation, authorizes payment, drives a warehouse station through a
//   bidirectional Fiber protocol, and verifies fiber failure isolation.
//
// The final section intentionally probes a known architectural seam:
// Future.async currently treats the first yield of a suspending action as
// terminal completion. Under the intended semantics, that assertion should
// pass; on the current implementation it is expected to expose the bug.

// -----------------------------------------------------------------------------
// Tiny self-checking harness
// -----------------------------------------------------------------------------

class Assert {
  @class
  equal(_ actual, _ expected, _ message) {
    if not (actual == expected) {
      return Error.new(
        "ASSERTION FAILED: \(message); expected=\(expected), actual=\(actual)"
      ).raise()
    }
    ()
  }

  @class
  truth(_ condition, _ message) {
    if not condition {
      return Error.new("ASSERTION FAILED: \(message)").raise()
    }
    ()
  }

  @class
  falsehood(_ condition, _ message) {
    if condition {
      return Error.new("ASSERTION FAILED: \(message)").raise()
    }
    ()
  }
}

// -----------------------------------------------------------------------------
// Domain model
// -----------------------------------------------------------------------------

class Order {
  const _id = None
  const _sku = None
  const _units = 0
  const _subtotalCents = 0

  @constructor
  new(_ id, _ sku, _ units, _ subtotalCents) {
    _id = id
    _sku = sku
    _units = units
    _subtotalCents = subtotalCents
  }

  id { _id }
  sku { _sku }
  units { _units }
  subtotalCents { _subtotalCents }
}

class Reservation {
  const _orderId = None
  const _sku = None
  const _units = 0

  @constructor
  new(_ orderId, _ sku, _ units) {
    _orderId = orderId
    _sku = sku
    _units = units
  }

  orderId { _orderId }
  sku { _sku }
  units { _units }
}

class ShippingQuote {
  const _carrier = None
  const _cents = 0

  @constructor
  new(_ carrier, _ cents) {
    _carrier = carrier
    _cents = cents
  }

  carrier { _carrier }
  cents { _cents }
}

class Authorization {
  const _orderId = None
  const _code = None
  const _amountCents = 0

  @constructor
  new(_ orderId, _ code, _ amountCents) {
    _orderId = orderId
    _code = code
    _amountCents = amountCents
  }

  orderId { _orderId }
  code { _code }
  amountCents { _amountCents }
}

// -----------------------------------------------------------------------------
// Async service facades
//
// These are deliberately backed by Future.new + System.schedule rather than
// Future.async. Each service represents an external operation that completes
// on a later scheduler turn.
// -----------------------------------------------------------------------------

class InventoryService {
  @class
  reserve(_ order) {
    const future = Future.new()

    System.schedule {
      System.print("[inventory] reserving \(order.units) x \(order.sku)")
      future.settleValue(
        Reservation.new(order.id, order.sku, order.units)
      )
    }

    future
  }
}

class ShippingService {
  @class
  quote(_ order) {
    const future = Future.new()

    System.schedule {
      System.print("[shipping] quoting order \(order.id)")
      future.settleValue(
        ShippingQuote.new("FalconExpress", 875)
      )
    }

    future
  }
}

class PaymentGateway {
  @class
  authorize(_ order, _ amountCents) {
    const future = Future.new()

    System.schedule {
      System.print(
        "[payment] authorizing \(amountCents) cents for \(order.id)"
      )
      future.settleValue(
        Authorization.new(order.id, "AUTH-7F3A", amountCents)
      )
    }

    future
  }
}

// -----------------------------------------------------------------------------
// Test 1 — scheduler is deferred and root await drives progress
// -----------------------------------------------------------------------------

System.print("== scheduler / root-await ==")

let scheduledRan = false
const scheduledProbe = Future.new()

System.schedule {
  scheduledRan = true
  scheduledProbe.settleValue("scheduler-turn-ran")
}

Assert.falsehood(
  scheduledRan,
  "System.schedule must enqueue work rather than execute it synchronously"
)

const scheduledValue = scheduledProbe.await

Assert.equal(
  scheduledValue,
  "scheduler-turn-ran",
  "root Future.await must drive ready work until its Future settles"
)

Assert.truth(
  scheduledRan,
  "scheduled work must have executed before await returns"
)

// -----------------------------------------------------------------------------
// Test 2 — realistic fulfillment flow with overlapping Futures
// -----------------------------------------------------------------------------

System.print("== fulfillment workflow ==")

const order = Order.new(
  "ORD-1042",
  "PHALCOM-BOOK",
  2,
  12500
)

// Start independent operations before waiting for either.
const reservationFuture: Future<Reservation> = InventoryService.reserve(order)
const shippingFuture: Future<ShippingQuote> = ShippingService.quote(order)

// Register a continuation while shippingFuture is still pending.
// This exercises the pending Future.map path and its waiter/continuation drain.
const payableFuture = shippingFuture.map |quote| {
  order.subtotalCents + quote.cents
}

// The first await should run only as much scheduled work as necessary to make
// reservationFuture ready. The shipping job remains available for later work.
const reservation = reservationFuture.await

Assert.equal(
  reservation.orderId,
  order.id,
  "inventory reservation must belong to the requested order"
)

Assert.equal(
  reservation.units,
  2,
  "inventory must reserve the requested unit count"
)

// Waiting for the mapped Future requires:
//   1. the shipping job to settle shippingFuture;
//   2. shippingFuture.drain to schedule the map continuation;
//   3. the continuation to settle payableFuture.
const payableCents = payableFuture.await

Assert.equal(
  payableCents,
  13375,
  "payable total must include the asynchronous shipping quote"
)

const authorizationFuture = PaymentGateway.authorize(
  order,
  payableCents
)

const authorization = authorizationFuture.await

Assert.equal(
  authorization.orderId,
  order.id,
  "payment authorization must retain order identity"
)

Assert.equal(
  authorization.amountCents,
  payableCents,
  "payment must authorize the computed payable amount"
)

Assert.equal(
  authorization.code,
  "AUTH-7F3A",
  "payment gateway must return the expected authorization code"
)

// -----------------------------------------------------------------------------
// Test 3 — a real coroutine protocol: warehouse station handshake
//
// This is not placeholder yielding. The Fiber represents a warehouse station
// that cannot advance from one physical step until the controller acknowledges
// completion of the previous step.
//
// Fiber.call(value) must become the return value of the suspended Fiber.yield.
// -----------------------------------------------------------------------------

System.print("== warehouse fiber handshake ==")

const station = Fiber.new {
  const pickAck = Fiber.yield(
    "PICK \(order.units) x \(order.sku)"
  )

  Assert.equal(
    pickAck,
    "PICKED",
    "resume value must be delivered back into the suspended yield expression"
  )

  const packAck = Fiber.yield(
    "PACK order \(order.id)"
  )

  Assert.equal(
    packAck,
    "PACKED",
    "second resume value must reach the correct suspension point"
  )

  "READY:\(order.id)"
}

Assert.equal(
  station.call(),
  "PICK 2 x PHALCOM-BOOK",
  "first station turn must request picking"
)

Assert.falsehood(
  station.isDone,
  "a fiber that yielded must remain resumable"
)

Assert.equal(
  station.call("PICKED"),
  "PACK order ORD-1042",
  "second station turn must request packing"
)

Assert.equal(
  station.call("PACKED"),
  "READY:ORD-1042",
  "terminal fiber return must be delivered to the resumer"
)

Assert.truth(
  station.isDone,
  "fiber must become terminal after its entry returns"
)

Assert.truth(
  station.error.isNone,
  "successfully completed fiber must not expose a failure"
)

// -----------------------------------------------------------------------------
// Test 4 — try() contains an operational failure
//
// A delivery-label worker fails independently. The caller wants to inspect the
// failure and continue processing the order rather than let that fiber failure
// escape.
// -----------------------------------------------------------------------------

System.print("== fiber failure containment ==")

const labelWorker = Fiber.new {
  Error.new(
    "carrier API refused label for \(order.id)"
  ).raise()
}

const labelError = labelWorker.try()

Assert.truth(
  labelWorker.isDone,
  "failed fiber must be terminal"
)

Assert.truth(
  labelWorker.error.isSome,
  "failed fiber must retain its captured Error"
)

Assert.equal(
  labelError.message,
  "carrier API refused label for ORD-1042",
  "Fiber.try must deliver terminal failure as a value"
)

// -----------------------------------------------------------------------------
// Test 5 — call() failure cascades until a try() boundary
//
// This models a fulfillment sub-pipeline where an inner carrier session is
// linked to an outer dispatch worker. The outer worker uses call(), so an
// uncaught child failure is not handled locally. The root uses try() around
// the outer worker and acts as the containment boundary.
// -----------------------------------------------------------------------------

System.print("== nested call/try failure boundary ==")

const carrierSession = Fiber.new {
  Error.new("carrier connection dropped").raise()
}

const dispatchWorker = Fiber.new {
  carrierSession.call()

  // Current semantics should never execute this after the child fails through
  // a call-mode boundary.
  Error.new(
    "dispatch worker continued after linked child failure"
  ).raise()
}

const dispatchError = dispatchWorker.try()

Assert.equal(
  dispatchError.message,
  "carrier connection dropped",
  "the original child Error must reach the first try boundary"
)

Assert.truth(
  carrierSession.isDone and dispatchWorker.isDone,
  "call-mode failure must terminate both the child and linked resumer"
)

Assert.truth(
  carrierSession.error.isSome and dispatchWorker.error.isSome,
  "both fibers in the call-mode failure chain must record failure"
)

// -----------------------------------------------------------------------------
// Test 6 — current blocker probe: Future.async + nested await
//
// Intended semantics:
//
//   Future.async starts work on a fresh Fiber.
//   If that action awaits a pending Future, the outer Future MUST remain
//   pending. The action merely parked; it did not complete.
//
// Current implementation bug:
//
//   Future.async calls fib.try() once and mistakes the first Fiber.yield(None)
//   performed by await for terminal completion. The outer Future therefore
//   settles with None too early.
//
// This assertion is intentionally placed last so every preceding concurrency
// behavior is checked before the blocker is exposed.
// -----------------------------------------------------------------------------

System.print("== Future.async suspension probe ==")

const databaseReply = Future.new()

const receiptFuture = Future.async {
  const receiptNumber = databaseReply.await
  "RECEIPT-\(receiptNumber)"
}

// Run the driver. It should start the action, observe that it parked on
// databaseReply, and leave receiptFuture pending.
System.runScheduled()

Assert.falsehood(
  receiptFuture.isReady,
  "Future.async must not settle merely because its action suspended in await"
)

// Once the dependency becomes ready, the action should resume and terminal
// completion should settle receiptFuture with the real result.
databaseReply.settleValue(9001)
System.runScheduled()

Assert.equal(
  receiptFuture.await,
  "RECEIPT-9001",
  "Future.async must settle with the action's terminal result after resumption"
)

System.print("ALL CONCURRENCY CHECKS PASSED")
