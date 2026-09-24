// Task 31: public reflection over specialized and inferred types.
//
// This file intentionally exercises the proposed reified type surface rather
// than emulating reflection in user code. `value.type` yields the runtime type
// value for a value, and structural type values expose their ordered `fields`.

data ReflectionReport {
    passed: Bool
    orderPaidIdentity: Bool
    responseIdentity: Bool
    retryIdentity: Bool
    contextFields: List<String>
}

reflectContextFields<T>(value: T) -> List<String> {
    value.type.fields.map |field| field.name
}

verifyReflection(
    paid: Order<Paid>,
    response: Response<Ok, CheckoutView>,
    retry: RetryPolicy<Count(3)>,
    context: {
        requestId: RequestId,
        user: User,
        params: {
            orderId: OrderId
        },
        body: CheckoutRequest,
        method: Method,
        path: String,
        headers: Headers,
        rawBody: Bytes
    }
) -> ReflectionReport {
    let orderPaidIdentity = paid.type == Order<Paid>
    let responseIdentity = response.type == Response<Ok, CheckoutView>
    let retryIdentity = retry.type == RetryPolicy<Count(3)>
    let contextFields = reflectContextFields(context)

    let expectedFields = [
        "requestId",
        "user",
        "params",
        "body",
        "method",
        "path",
        "headers",
        "rawBody",
    ]

    ReflectionReport {
        passed:
            orderPaidIdentity and
            responseIdentity and
            retryIdentity and
            contextFields == expectedFields,
        orderPaidIdentity,
        responseIdentity,
        retryIdentity,
        contextFields,
    }
}
