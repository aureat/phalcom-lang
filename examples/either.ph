enum Either<L, R> {

    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)

    isLeft -> Bool {
        match self {
            Left(_) => true
            Right(_) => false
        }
    }

    isRight -> Bool {
        match self {
            Left(_) => false
            Right(_) => true
        }
    }

    mapLeft<U>(_ f: (L) -> U) -> Either<U, R> {
        match self {
            Left(value) => Either::Left(f.call(value))
            Right(value) => Either::Right(value)
        }
    }

    mapRight<U>(_ f: (R) -> U) -> Either<L, U> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => Either::Right(f.call(value))
        }
    }

    bimap<A, B>(
        _ left: (L) -> A,
        _ right: (R) -> B
    ) -> Either<A, B> {
        match self {
            Left(value) => Either::Left(left.call(value))
            Right(value) => Either::Right(right.call(value))
        }
    }
}