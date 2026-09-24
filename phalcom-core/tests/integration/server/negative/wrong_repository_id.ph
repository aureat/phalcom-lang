// EXPECT: associated type mismatch when a consumer requires OrderId.
class WrongIdRepository

impl OrderRepository<TestFx> for WrongIdRepository {
    type Id = UserId
    type Error = RepositoryError

    findDraft(
        _ id: UserId
    ) -> TestFx<Result<Option<Order<Draft>>, RepositoryError>>

    save<S: OrderState>(
        order: Order<S>
    ) -> TestFx<Result<Order<S>, RepositoryError>>
}
