// Task 29: sequence-rest coverage. This uses the ratified named `*T` rest
// spelling and intentionally keeps it distinct from `**R` and `***S`.
identitySequence<T>(
    items: [
        OrderItem,
        *T
    ]
) -> [
    OrderItem,
    *T
] {
    items
}

runSequencePackProbe -> [
    OrderItem,
    OrderItem
] {
    let first = OrderItem {
        sku: "golden-sku-1",
        quantity: 1,
        unitPrice: Money(2100),
    }

    let second = OrderItem {
        sku: "golden-sku-2",
        quantity: 1,
        unitPrice: Money(2150),
    }

    let output = identitySequence([
        first,
        second,
    ])

    expectType<[
        OrderItem,
        OrderItem
    ]>(output)

    output
}
