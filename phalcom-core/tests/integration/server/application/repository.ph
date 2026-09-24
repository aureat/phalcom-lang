trait OrderRepository<F>
where F: Type -> Type
{
    type Id: Type
    type Error: Type

    findDraft(
        id: Self::Id
    ) -> F<
        Result<
            Option<Order<Draft>>,
            Self::Error
        >
    >

    save<S: OrderState>(
        order: Order<S>
    ) -> F<
        Result<
            Order<S>,
            Self::Error
        >
    >
}
