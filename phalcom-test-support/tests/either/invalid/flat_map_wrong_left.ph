class Probe {
    @class
    run() {
        let source: Either<String, Int> = Either::Right(1)
        let bad = source.flatMap(|value| {
            let wrong: Either<Int, Bool> = Either::Right(true)
            wrong
        })
    }
}
