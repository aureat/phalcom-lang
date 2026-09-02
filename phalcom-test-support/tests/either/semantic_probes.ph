class EitherGenericProbe {
    @class
    preserve<L, R>(_ value: Either<L, R>) -> Either<L, R> {
        value
    }

    @class
    lift<L, A, B>(
        _ value: Either<L, A>,
        _ transform: (A) -> B
    ) -> Either<L, B> {
        value.map(transform)
    }

    @class
    flatten<L, R>(
        _ value: Either<L, Either<L, R>>
    ) -> Either<L, R> {
        value.flatMap(|inner| {
            inner
        })
    }

    @class
    merge<T>(_ value: Either<T, T>) -> T {
        value.fold(
            left: |v| { v },
            right: |v| { v }
        )
    }
}

class EitherInferenceProbe {
    @class
    contextualLeft() {
        let contextualLeft: Either<String, Int> = Either::Left("failure")
    }

    @class
    contextualRight() {
        let contextualRight: Either<String, Int> = Either::Right(42)
    }

    @class
    canonicalPaths() {
        let fromLeft: Either<String, Int> = Either::Left("left")
        let fromRight: Either<String, Int> = Either::Right(42)
    }

    @class
    mapInference() {
        let source: Either<String, Int> = Either::Right(42)
        let mapped = source.map(|value| {
            value > 0
        })
    }

    @class
    mapLeftInference() {
        let source: Either<String, Int> = Either::Right(42)
        let mappedLeft = source.mapLeft(|value| {
            value == "failure"
        })
    }

    @class
    bimapInference() {
        let source: Either<String, Int> = Either::Right(42)
        let bimapped = source.bimap(
            left: |value| { value == "failure" },
            right: |value| { value > 0 }
        )
    }

    @class
    swapInference() {
        let source: Either<String, Int> = Either::Right(42)
        let swapped = source.swap
    }

    @class
    orElseInference() {
        let source: Either<String, Int> = Either::Left("failure")
        let replacement: Either<Bool, Int> = Either::Right(100)
        let recovered = source.orElse(replacement)
    }

    @class
    chainedInference() {
        let initial: Either<String, Int> = Either::Right(41)
        let mapped = initial.map(|value| { value == 41 })
        let leftMapped = mapped.mapLeft(|value| { 100 })
        let swapped = leftMapped.swap
    }
}

class EitherHigherOrderProbe {
    @class
    inferAcrossArguments() {
        let source: Either<String, Int> = Either::Right(41)
        let lifted = EitherGenericProbe.lift(
            source,
            |value| { value == 41 }
        )
    }

    @class
    freshCalls() {
        let firstInput: Either<String, Int> = Either::Right(41)
        let first = EitherGenericProbe.lift(
            firstInput,
            |value| { value == 41 }
        )

        let secondInput: Either<Int, Bool> = Either::Right(true)
        let second = EitherGenericProbe.lift(
            secondInput,
            |value| { "second" }
        )
    }
}

class EitherNestedProbe {
    @class
    flattenNested() {
        let inner: Either<String, Int> = Either::Right(73)
        let outer: Either<String, Either<String, Int>> = Either::Right(inner)
        let flattened = EitherGenericProbe.flatten(outer)
    }

    @class
    repeatedVariable() {
        let left: Either<Int, Int> = Either::Left(10)
        let right: Either<Int, Int> = Either::Right(20)
        let leftValue = EitherGenericProbe.merge(left)
        let rightValue = EitherGenericProbe.merge(right)
    }
}
