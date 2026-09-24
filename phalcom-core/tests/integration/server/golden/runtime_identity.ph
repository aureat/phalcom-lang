// Task 32: connect public reified type identity to ordinary runtime values.
// No compile-time type-test branching is used here.

data RuntimeTypeIdentityReport {
    passed: Bool
    paid: Bool
    response: Bool
    retry: Bool
}

verifyRuntimeTypeIdentity(
    paid: Order<Paid>,
    response: Response<Ok, CheckoutView>,
    retry: RetryPolicy<Count(3)>
) -> RuntimeTypeIdentityReport {
    let paidIdentity = paid.type == Order<Paid>
    let responseIdentity = response.type == Response<Ok, CheckoutView>
    let retryIdentity = retry.type == RetryPolicy<Count(3)>

    RuntimeTypeIdentityReport {
        passed:
            paidIdentity and
            responseIdentity and
            retryIdentity,
        paid: paidIdentity,
        response: responseIdentity,
        retry: retryIdentity,
    }
}
