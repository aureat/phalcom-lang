class Probe {
    @class
    merge<T>(_ value: Either<T, T>) -> T {
        value.fold(
            left: |v| { v },
            right: |v| { v }
        )
    }

    @class
    run() {
        let source: Either<Int, String> = Either::Right("hello")
        let bad = Probe.merge(source)
    }
}
