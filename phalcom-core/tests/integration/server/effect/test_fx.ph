data TestFx<T> {
    value: T
}

class TestEffect

impl Effect<TestFx> for TestEffect {
    pure<T>(
        value: T
    ) -> TestFx<T> {
        TestFx<T> {
            value,
        }
    }

    map<A, B>(
        effect: TestFx<A>,
        transform: (A) -> B
    ) -> TestFx<B> {
        TestFx<B> {
            value: transform(effect.value),
        }
    }

    flatMap<A, B>(
        effect: TestFx<A>,
        transform: (A) -> TestFx<B>
    ) -> TestFx<B> {
        transform(effect.value)
    }
}
