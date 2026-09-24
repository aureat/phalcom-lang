data Command<R> {
    LoadDraft(OrderId) -> Command<Option<Order<Draft>>>
    SavePaid(Order<Paid>) -> Command<Order<Paid>>
    PublishPaid(OrderPaid) -> Command<()>
}

trait CommandExecutor<F>
where F: Type -> Type
{
    type Error: Type

    execute<R>(
        command: Command<R>
    ) -> F<
        Result<
            R,
            Self::Error
        >
    >
}
