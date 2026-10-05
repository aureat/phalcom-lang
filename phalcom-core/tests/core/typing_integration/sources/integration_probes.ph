class UnifiedTypingProbe {
    @class
    run(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>
    ) {
        let mut mapped = source.map(|value| value > 0)

        let mut bound = MonadAlgorithms.bind(
            monad,
            source,
            |value| {
                let mut next: Either<String, Bool> = Either::Right(value > 0)
                next
            }
        )
    }
}
