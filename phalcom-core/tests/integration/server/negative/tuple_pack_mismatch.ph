// Expected failure: heterogeneous tuple tail cannot change its final Bool
// element to String while preserving the same `***S` shape.
invalidTuplePack -> (
    String,
    OrderId,
    Money,
    Bool
) {
    identityTuple((
        "checkout",
        OrderId("ord-123"),
        Money(4250),
        "not-a-bool",
    ))
}
