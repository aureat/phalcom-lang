data Unauthorized()

data DecodeError {
    InvalidCheckoutBody
}

data RouteDecodeError {
    InvalidCheckoutPath
}

data RepositoryError {
    Unavailable
}

data InfrastructureError {
    Repository(RepositoryError)
    EventPublish
}

data CheckoutError {
    Unauthorized
    Forbidden
    OrderNotFound
    Route(RouteDecodeError)
    Decode(DecodeError)
    Submit(SubmitError)
    Payment(PaymentError)
    Infrastructure(InfrastructureError)
}
