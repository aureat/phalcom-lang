data SmokeResult {
    orderId: OrderId
    finalOrder: Order<Paid>
    response: Response<Ok, OrderId>
}

runTasks1Through10Smoke -> Result<SmokeResult, String> {
    let draft = FixtureFactory.draftOrder

    let submitted = match submit(draft) {
        .Err(_) => {
            return .Err("submit failed")
        }

        .Ok(value) => value
    }

    expectType<Order<Submitted>>(submitted)

    let authorization = PaymentAuthorization {
        approved: true,
    }

    let paid = match pay(submitted, authorization) {
        .Err(_) => {
            return .Err("payment failed")
        }

        .Ok(value) => value
    }

    expectType<Order<Paid>>(paid)

    let effects = TestEffect {}
    let wrapped = effects.pure(paid)
    expectType<TestFx<Order<Paid>>>(wrapped)

    let mapped = effects.map(wrapped) |order| order.id
    expectType<TestFx<OrderId>>(mapped)

    let chained = effects.flatMap(wrapped) |order| {
        TestFx<OrderId> {
            value: order.id,
        }
    }
    expectType<TestFx<OrderId>>(chained)

    let response = Response<Ok, OrderId> {
        body: paid.id,
        headers: Headers {
            authorization: .None,
        },
    }
    expectType<Response<Ok, OrderId>>(response)

    // Parameterized static identity is instantiated here even though the
    // concrete payment capability is exercised in the later application task.
    let retry = RetryPolicy<Count(3)> {
        attempts: 3,
    }
    expectType<RetryPolicy<Count(3)>>(retry)

    .Ok(
        SmokeResult {
            orderId: paid.id,
            finalOrder: paid,
            response,
        }
    )
}
