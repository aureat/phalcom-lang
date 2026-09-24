// Expected failure: the refinement is scoped to the true branch only.
invalidRefinementEscape(
    event: DomainEvent
) -> OrderId {
    if event is OrderPaid {
        let observed = event.orderId
    }

    event.orderId
}
