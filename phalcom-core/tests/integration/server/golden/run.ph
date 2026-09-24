const expectedGoldenTrace: List<TraceEvent> = {
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

runTasks11Through20 -> Result<Tasks11Through20Result, String> {
    let ids = FixedRequestIdFactory {
        value: RequestId("req-0001"),
    }

    let authenticator = FakeAuthenticator {
        expected: AuthToken("Bearer golden-user"),
        user: FixtureFactory.user,
    }

    let decoder = CheckoutRequestDecoder {
        value: FixtureFactory.checkoutRequest,
    }

    let repository = FakeOrderRepository {
        draft: .Some(FixtureFactory.draftOrder),
        calls: [],
    }

    let events = RecordingEventPublisher {
        events: [],
    }

    let executor = GoldenCommandExecutor {
        repository,
        events,
    }

    let gateway = FakePaymentGateway<Count(3)> {
        authorization: PaymentAuthorization {
            approved: true,
        },
    }

    let services = CheckoutServices<
        TestFx,
        TestEffect,
        GoldenCommandExecutor,
        FakePaymentGateway<Count(3)>
    > {
        effects: TestEffect {},
        executor,
        payment: gateway,
        trace: TraceRecorder { events: [] },
    }

    let context = match prepareCheckoutContext(
        FixtureFactory.rawRequest,
        using: ids,
        authenticator,
        decoder
    ) {
        .Err(error) => {
            return .Err("request preparation failed: ${error}")
        }
        .Ok(value) => value
    }

    expectType<RequestId>(context.requestId)
    expectType<User>(context.user)
    expectType<OrderId>(context.params.orderId)
    expectType<CheckoutRequest>(context.body)

    let checkout = checkoutHandler(
        context,
        using: services
    )

    expectType<
        TestFx<
            Result<
                Response<Ok, CheckoutView>,
                CheckoutError
            >
        >
    >(checkout)

    match checkout.value {
        Err(error) => {
            Err("checkout failed: ${error}")
        }

        Ok(response) => {
            Ok(
                Tasks11Through20Result {
                    requestId: context.requestId,
                    orderId: response.body.orderId,
                    response,
                    repositoryCalls: repository.calls,
                }
            )
        }
    }
}

let identityTuple =
  <S>|tuple: (String, ***S)| -> (String, ***S) {
    tuple
  }

data Runner() {
    requestId: RequestId
    orderId: OrderId
    response: Response<Ok, CheckoutView>
    repositoryCalls: List<RepositoryCall>
}

impl Runner {
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

    runGoldenProgram -> TestReport {
        // Keep the earliest vertical slice live as part of the final executable.
        match runTasks1Through10Smoke() {
            Err(message) => {
                return TestReport.failed(
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
                return TestReport.failed(
                    "checkout failed: ${error}",
                    fixtures.trace.events
                )
            }

            Ok(value) => value
        }

        expectType<Response<Ok, CheckoutView>>(response)
        verifyFinalTypeWitnesses(response, result)

        if response.body.orderId != OrderId("ord-123") {
            return TestReport.failed(
                "unexpected response order id",
                fixtures.trace.events
            )
        }

        if response.body.state != "paid" {
            return TestReport.failed(
                "unexpected response state",
                fixtures.trace.events
            )
        }

        if response.body.total != Money(4250) {
            return TestReport.failed(
                "unexpected response total",
                fixtures.trace.events
            )
        }

        if fixtures.repository.calls != [
            FindDraft(OrderId("ord-123")),
            SavePaid(OrderId("ord-123")),
        ] {
            return TestReport.failed(
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
            return TestReport.failed(
                "unexpected event publication trace",
                fixtures.trace.events
            )
        }

        if fixtures.trace.events != expectedGoldenTrace() {
            return TestReport.failed(
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
                return TestReport.failed(
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
            return TestReport.failed(
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
            return TestReport.failed(
                "runtime type identity did not preserve specialization",
                fixtures.trace.events
            )
        }

        if not verifyRuntimeRefinement() {
            return TestReport.failed(
                "runtime `is` refinement checks failed",
                fixtures.trace.events
            )
        }

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


        TestReport.passed(
            RequestId("req-0001"),
            response,
            fixtures.trace.events
        )
    }
}
