class FixtureFactory

impl FixtureFactory {
    class.user -> User {
        User {
            id: UserId("usr-001"),
            name: "Golden User",
        }
    }

    class.draftOrder -> Order<Draft> {
        let item = OrderItem {
            sku: "golden-sku",
            quantity: 1,
            unitPrice: Money(4250),
        }

        Order<Draft> {
            id: OrderId("ord-123"),
            owner: UserId("usr-001"),
            items: [item],
            total: Money(4250),
        }
    }

    class.checkoutRequest -> CheckoutRequest {
        CheckoutRequest {
            paymentToken: PaymentToken("tok_golden"),
        }
    }

    class.paymentRequest -> PaymentRequest {
        PaymentRequest {
            orderId: OrderId("ord-123"),
            amount: Money(4250),
            token: PaymentToken("tok_golden"),
        }
    }

    class.orderPaid -> OrderPaid {
        OrderPaid {
            orderId: OrderId("ord-123"),
            owner: UserId("usr-001"),
            total: Money(4250),
        }
    }

    class.paidOrder -> Order<Paid> {
        let item = OrderItem {
            sku: "golden-sku",
            quantity: 1,
            unitPrice: Money(4250),
        }

        Order<Paid> {
            id: OrderId("ord-123"),
            owner: UserId("usr-001"),
            items: [item],
            total: Money(4250),
        }
    }


    class.rawRequest -> RawRequest {
        RawRequest {
            method: .Post,
            path: "/orders/ord-123/checkout",
            headers: Headers {
                authorization: .Some("Bearer golden-user"),
            },
            rawBody: Bytes.empty,
        }
    }
}
