trait Effect<F>
where F: Type -> Type
{
    pure<T>(
        value: T
    ) -> F<T>

    map<A, B>(
        effect: F<A>,
        transform: (A) -> B
    ) -> F<B>

    flatMap<A, B>(
        effect: F<A>,
        transform: (A) -> F<B>
    ) -> F<B>
}
