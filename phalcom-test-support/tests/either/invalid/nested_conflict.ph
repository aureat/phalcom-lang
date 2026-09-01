class Probe {
    @class
    flatten<L, R>(_ value: Either<L, Either<L, R>>) -> Either<L, R> {
        value.flatMap(|inner| { inner })
    }

    @class
    run(_ outer: Either<String, Either<Bool, Int>>) {
        let bad = Probe.flatten(outer)
    }
}
