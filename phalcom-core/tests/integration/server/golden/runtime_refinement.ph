// Task 33: flow-sensitive runtime refinement uses `is` only.

describeDomainEvent(
    event: DomainEvent
) -> String {
    if event is OrderPaid {
        return "paid:${event.orderId.value}"
    }

    if event is DiagnosticEvent {
        return "diagnostic:${event.label}"
    }

    "unknown"
}

orderPaidIdAfterNegativeGuard(
    event: DomainEvent
) -> Option<OrderId> {
    if not (event is OrderPaid) {
        return .None
    }

    // The terminating negative branch leaves the fallthrough path refined.
    .Some(event.orderId)
}

isGoldenPaidEvent(
    event: DomainEvent
) -> Bool {
    if event is OrderPaid and event.total == Money(4250) {
        return true
    }

    false
}

verifyRuntimeRefinement -> Bool {
    let paid: DomainEvent = FixtureFactory.orderPaid
    let diagnostic: DomainEvent = DiagnosticEvent {
        label: "golden",
    }

    describeDomainEvent(paid) == "paid:ord-123" and
    describeDomainEvent(diagnostic) == "diagnostic:golden" and
    orderPaidIdAfterNegativeGuard(paid) == .Some(OrderId("ord-123")) and
    orderPaidIdAfterNegativeGuard(diagnostic) == .None and
    isGoldenPaidEvent(paid)
}
