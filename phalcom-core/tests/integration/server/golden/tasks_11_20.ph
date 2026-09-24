data Tasks11Through20Result {
    requestId: RequestId
    orderId: OrderId
    response: Response<Ok, CheckoutView>
    repositoryCalls: List<RepositoryCall>
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
