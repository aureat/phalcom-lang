class UnifiedTypingProbe {
    @class
    run(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>
    ) {
        let mapped = source.map(|value| value > 0)

        let bound = MonadAlgorithms.bind(
            monad,
            source,
            |value| {
                let next: Either<String, Bool> = Either::Right(value > 0)
                next
            }
        )
    }
}
