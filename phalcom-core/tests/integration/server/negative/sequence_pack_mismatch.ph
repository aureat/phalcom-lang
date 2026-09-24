// Expected failure: the sequence probe expects two OrderItem entries; the
// captured `*T` tail must not silently accept String as an OrderItem.
invalidSequencePack -> [
    OrderItem,
    OrderItem
] {
    let item = OrderItem {
        sku: "golden-sku",
        quantity: 1,
        unitPrice: Money(4250),
    }

    identitySequence([
        item,
        "not-an-order-item",
    ])
}
