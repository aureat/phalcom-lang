data PaymentToken(_ value: String)

data PaymentAuthorization {
    approved: Bool
}

data CheckoutRequest {
    paymentToken: PaymentToken
}

data PaymentRequest {
    orderId: OrderId
    amount: Money
    token: PaymentToken
}

type data Retry {
    Never
    Count(Int)
}

data RetryPolicy<R: Retry> {
    attempts: Int
}
