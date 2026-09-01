class Probe {
    @class
    flatten<L, R>(_ value: Either<L, Either<L, R>>) -> Either<L, R> {
        value.flatMap(|inner| { inner })
    }

    @class
    run() {
        let inner: Either<Bool, Int> = Either::Right(1)
        let outer: Either<String, Either<Bool, Int>> = Either::Right(inner)
        let bad = Probe.flatten(outer)
    }
}
