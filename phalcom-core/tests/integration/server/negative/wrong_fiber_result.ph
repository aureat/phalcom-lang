// Expected failure: Fiber block result is String, so the specialization is
// Fiber<String>, not Fiber<TestReport>.
invalidFiberResult() {
    let fiber = Fiber.new {
        "wrong-result"
    }

    expectType<Fiber<TestReport>>(fiber)
}
