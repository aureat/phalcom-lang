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

    @class
    algorithmBind(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>,
        _ next: (Int) -> Either<String, Bool>
    ) {
        let bound = MonadAlgorithms.bind(monad, source, next)
    }

    @class
    nestedSequenceEvidence(
        _ monad: StringEitherMonad,
        _ values: List<Either<String, Int>>,
        _ initial: Either<String, List<Int>>
    ) {
        let sequenced = MonadAlgorithms.sequenceSeed(monad, values, initial)
    }

    @class
    kleisliEvidence(
        _ monad: StringEitherMonad,
        _ first: (String) -> Either<String, Int>,
        _ second: (Int) -> Either<String, Bool>
    ) {
        let composed = MonadAlgorithms.kleisli(monad, first, second)
    }

    @class
    traverseEvidence(
        _ monad: StringEitherMonad,
        _ values: List<Int>,
        _ transform: (Int) -> Either<String, Bool>
    ) {
        let traversed = MonadAlgorithms.traverse(monad, values, transform)
    }
}
