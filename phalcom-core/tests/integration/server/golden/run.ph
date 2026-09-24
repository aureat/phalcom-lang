expectedGoldenTrace -> List<TraceEvent> {
    [
        RequestIdentified(RequestId("req-0001")),
        RouteDecoded(OrderId("ord-123")),
        Authenticated(UserId("usr-001")),
        BodyDecoded,
        DraftLoaded(OrderId("ord-123")),
        Submitted(OrderId("ord-123")),
        PaymentAuthorized(OrderId("ord-123")),
        Paid(OrderId("ord-123")),
        Saved(OrderId("ord-123")),
        Published(OrderId("ord-123")),
        Responded,
    ]
}

runGoldenProgram -> TestReport {
    // Keep the earliest vertical slice live as part of the final executable.
    match runTasks1Through10Smoke() {
        Err(message) => {
            return failedReport(
                "tasks 1-10 smoke failed: ${message}",
                []
            )
        }
        Ok(_) => ()
    }

    let fixtures = GoldenFixtureSet.canonical

    let executor = GoldenCommandExecutor {
        repository: fixtures.repository,
        events: fixtures.events,
    }

    let services = CheckoutServices<
        TestFx,
        TestEffect,
        GoldenCommandExecutor,
        FakePaymentGateway<Count(3)>
    > {
        effects: TestEffect {},
        executor,
        payment: fixtures.gateway,
        trace: fixtures.trace,
    }

    let result = processRequest(
        FixtureFactory.rawRequest,
        using: fixtures.ids,
        fixtures.authenticator,
        fixtures.decoder,
        services
    )

    expectType<
        TestFx<
            Result<
                Response<Ok, CheckoutView>,
                CheckoutError
            >
        >
    >(result)

    let response = match result.value {
        Err(error) => {
            return failedReport(
                "checkout failed: ${error}",
                fixtures.trace.events
            )
        }

        Ok(value) => value
    }

    expectType<Response<Ok, CheckoutView>>(response)
    verifyFinalTypeWitnesses(response, result)

    if response.body.orderId != OrderId("ord-123") {
        return failedReport(
            "unexpected response order id",
            fixtures.trace.events
        )
    }

    if response.body.state != "paid" {
        return failedReport(
            "unexpected response state",
            fixtures.trace.events
        )
    }

    if response.body.total != Money(4250) {
        return failedReport(
            "unexpected response total",
            fixtures.trace.events
        )
    }

    if fixtures.repository.calls != [
        FindDraft(OrderId("ord-123")),
        SavePaid(OrderId("ord-123")),
    ] {
        return failedReport(
            "unexpected repository call trace",
            fixtures.trace.events
        )
    }

    if fixtures.events.events != [
        OrderPaidEvent(
            OrderPaid {
                orderId: OrderId("ord-123"),
                owner: UserId("usr-001"),
                total: Money(4250),
            }
        ),
    ] {
        return failedReport(
            "unexpected event publication trace",
            fixtures.trace.events
        )
    }

    if fixtures.trace.events != expectedGoldenTrace() {
        return failedReport(
            "unexpected typed runtime trace",
            fixtures.trace.events
        )
    }

    let reflectedContext = match prepareCheckoutContext(
        FixtureFactory.rawRequest,
        using: fixtures.ids,
        fixtures.authenticator,
        fixtures.decoder
    ) {
        Err(error) => {
            return failedReport(
                "reflection context preparation failed: ${error}",
                fixtures.trace.events
            )
        }
        Ok(value) => value
    }

    let retry = RetryPolicy<Count(3)> {
        attempts: 3,
    }

    let reflection = verifyReflection(
        FixtureFactory.paidOrder,
        response,
        retry,
        reflectedContext
    )

    if not reflection.passed {
        return failedReport(
            "public reflection did not preserve specialized/inferred type identity",
            fixtures.trace.events
        )
    }

    let runtimeIdentity = verifyRuntimeTypeIdentity(
        FixtureFactory.paidOrder,
        response,
        retry
    )

    if not runtimeIdentity.passed {
        return failedReport(
            "runtime type identity did not preserve specialization",
            fixtures.trace.events
        )
    }

    if not verifyRuntimeRefinement() {
        return failedReport(
            "runtime `is` refinement checks failed",
            fixtures.trace.events
        )
    }

    // Tasks 28-29 are runtime-attached probes rather than isolated type files.
    let tupleProbe = runTuplePackProbe()
    expectType<(
        String,
        OrderId,
        Money,
        Bool
    )>(tupleProbe)

    let sequenceProbe = runSequencePackProbe()
    expectType<[
        OrderItem,
        OrderItem
    ]>(sequenceProbe)

    passedReport(
        RequestId("req-0001"),
        response,
        fixtures.trace.events
    )
}
