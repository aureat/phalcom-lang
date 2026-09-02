class MonadSemanticProbe {
    @class
    inheritedMap(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>
    ) {
        let mapped = monad.map(
            source,
            |value| { value > 0 }
        )
    }

    @class
    inheritedPure(_ monad: StringEitherMonad) {
        let lifted = monad.pure(42)
    }

    @class
    inheritedFlatMap(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>,
        _ next: (Int) -> Either<String, Bool>
    ) {
        let chained = monad.flatMap(source, next)
    }
}
