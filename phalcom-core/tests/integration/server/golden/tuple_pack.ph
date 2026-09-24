// Task 28: heterogeneous tuple-shape rest capture is deliberately separate
// from record rows (`**R`) and sequence rests (`*T`).
identityTuple<S>(
    tuple: (String, ***S)
) -> (String, ***S) {
    tuple
}

runTuplePackProbe -> (
    String,
    OrderId,
    Money,
    Bool
) {
    let input = (
        "checkout",
        OrderId("ord-123"),
        Money(4250),
        true,
    )

    let output = identityTuple(input)

    expectType<(
        String,
        OrderId,
        Money,
        Bool
    )>(output)

    output
}
