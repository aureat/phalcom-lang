class MonadSemanticProbe {
    @class
    inheritedMap(
        _ monad: StringContractEitherMonad,
        _ source: Either<String, Int>
    ) {
        let mut mapped = monad.map(
            source,
            |value| { value > 0 }
        )
    }

    @class
    inheritedPure(_ monad: StringContractEitherMonad) {
        let mut lifted = monad.pure(42)
    }

    @class
    inheritedFlatMap(
        _ monad: StringContractEitherMonad,
        _ source: Either<String, Int>,
        _ next: (Int) -> Either<String, Bool>
    ) {
        let mut chained = monad.flatMap(source, next)
    }

    @class
    algorithmBind(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>,
        _ next: (Int) -> Either<String, Bool>
    ) {
        let mut bound = MonadAlgorithms.bind(monad, source, next)
    }

    @class
    constructorAgreement(
        _ left: Either<String, Int>,
        _ right: Either<String, Bool>
    ) {
        let mut agreed = MonadAlgorithms.sameConstructor(left, right)
    }

    @class
    nestedSequenceEvidence(
        _ monad: StringEitherMonad,
        _ values: List<Either<String, Int>>,
        _ initial: Either<String, List<Int>>
    ) {
        let mut sequenced = MonadAlgorithms.sequenceSeed(monad, values, initial)
    }

    @class
    sequenceEvidence(
        _ monad: StringEitherMonad,
        _ values: List<Either<String, Int>>
    ) {
        let mut sequenced = MonadAlgorithms.sequence(monad, values)
    }

    @class
    kleisliEvidence(
        _ monad: StringEitherMonad,
        _ first: (String) -> Either<String, Int>,
        _ second: (Int) -> Either<String, Bool>
    ) {
        let mut composed = MonadAlgorithms.kleisli(monad, first, second)
    }

    @class
    traverseEvidence(
        _ monad: StringEitherMonad,
        _ values: List<Int>,
        _ transform: (Int) -> Either<String, Bool>
    ) {
        let mut traversed = MonadAlgorithms.traverse(monad, values, transform)
    }
}
