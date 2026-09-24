// Expected failure: `Ok` belongs to Status, not OrderState.
wrongStaticCaseDomain(
    order: Order<Ok>
) -> Order<Ok> {
    order
}
