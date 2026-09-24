trait EventPublisher<F>
where F: Type -> Type
{
    type Error: Type

    publish<E: DomainEvent>(
        event: E
    ) -> F<
        Result<
            (),
            Self::Error
        >
    >
}
