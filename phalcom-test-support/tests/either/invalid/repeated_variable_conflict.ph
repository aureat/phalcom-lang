class Probe {
    @class
    merge<T>(_ value: Either<T, T>) -> T {
        value.fold(
            left: |v| { v },
            right: |v| { v }
        )
    }

    @class
    run(_ source: Either<Int, String>) {
        let bad = Probe.merge(source)
    }
}
