// Expected failure: one map closure cannot infer a single B from OrderId/String.
brokenClosureResult() {
    let effect = TestFx<Bool> {
        value: true,
    }

    TestEffect {}
        .map(effect) |flag| {
            if flag {
                OrderId("ord-123")
            } else {
                "not-an-order-id"
            }
        }
}
