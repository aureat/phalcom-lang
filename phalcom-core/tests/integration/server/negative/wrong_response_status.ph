// EXPECT: Response<Created, OrderId> does not satisfy Response<Ok, OrderId>.
requiresOk(
    _ response: Response<Ok, OrderId>
) -> () {
    ()
}

negativeWrongResponseStatus(
    id: OrderId
) {
    let response = Response<Created, OrderId> {
        body: id,
        headers: Headers { authorization: .None },
    }

    requiresOk(response)
}
