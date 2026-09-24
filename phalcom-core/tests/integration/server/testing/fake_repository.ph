data RepositoryCall {
    FindDraft(OrderId)
    SaveDraft(OrderId)
    SaveSubmitted(OrderId)
    SavePaid(OrderId)
    SaveCancelled(OrderId)
}

class FakeOrderRepository {
    draft: Option<Order<Draft>>
    calls: List<RepositoryCall>
}

impl OrderRepository<TestFx> for FakeOrderRepository {
    type Id = OrderId
    type Error = RepositoryError

    findDraft(
        id: OrderId
    ) -> TestFx<
        Result<
            Option<Order<Draft>>,
            RepositoryError
        >
    > {
        calls.append(.FindDraft(id))

        TestFx {
            value: .Ok(draft),
        }
    }

    save<S: OrderState>(
        order: Order<S>
    ) -> TestFx<
        Result<
            Order<S>,
            RepositoryError
        >
    > {
        if order is Order<Draft> {
            calls.append(.SaveDraft(order.id))
        } else if order is Order<Submitted> {
            calls.append(.SaveSubmitted(order.id))
        } else if order is Order<Paid> {
            calls.append(.SavePaid(order.id))
        } else if order is Order<Cancelled> {
            calls.append(.SaveCancelled(order.id))
        }

        TestFx {
            value: .Ok(order),
        }
    }
}
