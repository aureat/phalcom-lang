type data OrderState {
    Draft
    Submitted
    Paid
    Cancelled
}

data OrderItem {
    sku: String
    quantity: Int
    unitPrice: Money
}

data Order<S: OrderState> {
    id: OrderId
    owner: UserId
    items: List<OrderItem>
    total: Money
}

data SubmitError {
    EmptyOrder
}

data PaymentError {
    Declined
}

submit(
    order: Order<Draft>
) -> Result<Order<Submitted>, SubmitError> {
    if order.items.isEmpty {
        return Err(EmptyOrder)
    }

    Ok(
        Order<Submitted> {
            id: order.id,
            owner: order.owner,
            items: order.items,
            total: order.total,
        }
    )
}

pay(
    order: Order<Submitted>,
    authorization: PaymentAuthorization
) -> Result<Order<Paid>, PaymentError> {
    if not authorization.approved {
        return Err(Declined)
    }

    Ok(
        Order<Paid> {
            id: order.id,
            owner: order.owner,
            items: order.items,
            total: order.total,
        }
    )
}
