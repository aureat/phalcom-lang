let mut runtimeMonad = StringEitherMonad.new()
let mut runtimeRight: Either<String, Int> = Either::Right(41)
let mut runtimeLeft: Either<String, Int> = Either::Left("boom")

let mut monadMappedRight = runtimeMonad.map(runtimeRight, |value| { value + 1 })
let mut monadMappedRightValue = monadMappedRight.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mut mapLeftTransformCalls = 0
let mut monadMappedLeft = runtimeMonad.map(runtimeLeft, |value| {
    mapLeftTransformCalls = mapLeftTransformCalls + 1
    value + 1
})
let mut monadMappedLeftPreserved = monadMappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)
let mut monadMapLeftShortCircuited = mapLeftTransformCalls == 0

let mut monadPure = runtimeMonad.pure(7)
let mut monadPureValue = monadPure.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mut map2Right: Either<String, Int> = Either::Right(1)
let mut map2Other: Either<String, Int> = Either::Right(2)
let mut monadMap2 = runtimeMonad.map2(map2Right, map2Other, |a, b| { a + b })
let mut monadMap2Value = monadMap2.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mut map2LeftFailureCalls = 0
let mut map2Failure: Either<String, Int> = Either::Left("map2-fail")
let mut monadMap2Failure = runtimeMonad.map2(map2Failure, map2Other, |a, b| {
    map2LeftFailureCalls = map2LeftFailureCalls + 1
    a + b
})
let mut monadMap2FailurePreserved = monadMap2Failure.fold(
    left: |error| { error == "map2-fail" },
    right: |value| { false }
)
let mut monadMap2LeftShortCircuited = map2LeftFailureCalls == 0

let mut map2RightFailureCalls = 0
let mut map2RightFailure: Either<String, Int> = Either::Left("map2-right-fail")
let mut monadMap2RightFailure = runtimeMonad.map2(map2Right, map2RightFailure, |a, b| {
    map2RightFailureCalls = map2RightFailureCalls + 1
    a + b
})
let mut monadMap2RightFailurePreserved = monadMap2RightFailure.fold(
    left: |error| { error == "map2-right-fail" },
    right: |value| { false }
)
let mut monadMap2RightShortCircuited = map2RightFailureCalls == 0

let mut flatMapNext: (Int) -> Either<String, Bool> = |value| {
    let mut next: Either<String, Bool> = Either::Right(value == 41)
    next
}
let mut monadFlatMap = runtimeMonad.flatMap(runtimeRight, flatMapNext)
let mut monadFlatMapValue = monadFlatMap.fold(
    left: |error| { false },
    right: |value| { value }
)

let mut flatMapFailureNextCalls = 0
let mut flatMapFailureNext: (Int) -> Either<String, Bool> = |value| {
    flatMapFailureNextCalls = flatMapFailureNextCalls + 1
    let mut next: Either<String, Bool> = Either::Right(value == 41)
    next
}
let mut monadFlatMapFailure = runtimeMonad.flatMap(runtimeLeft, flatMapFailureNext)
let mut monadFlatMapFailurePreserved = monadFlatMapFailure.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)
let mut monadFlatMapShortCircuited = flatMapFailureNextCalls == 0

let mut kleisliFirst: (Int) -> Either<String, Int> = |value| {
    let mut next: Either<String, Int> = Either::Right(value + 1)
    next
}
let mut kleisliSecond: (Int) -> Either<String, Bool> = |value| {
    let mut next: Either<String, Bool> = Either::Right(value == 42)
    next
}
let mut runtimeKleisli = MonadAlgorithms.kleisli(runtimeMonad, kleisliFirst, kleisliSecond)
let mut runtimeKleisliValue = runtimeKleisli.call(41).fold(
    left: |error| { false },
    right: |value| { value }
)

let mut traverseSuccessTransform: (Int) -> Either<String, Int> = |value| {
    let mut next: Either<String, Int> = Either::Right(value + 10)
    next
}
let mut runtimeTraverseSuccess = MonadAlgorithms.traverse(runtimeMonad, [1, 2, 3], traverseSuccessTransform)
let mut runtimeTraverseSuccessValue = runtimeTraverseSuccess.fold(
    left: |error| { false },
    right: |values| {
        values.size == 3 and values[0] == 11 and values[1] == 12 and values[2] == 13
    }
)

let mut traverseFailureTransformCalls = 0
let mut traverseFailureTransform: (Int) -> Either<String, Int> = |value| {
    traverseFailureTransformCalls = traverseFailureTransformCalls + 1
    if (value == 2) {
        let mut failed: Either<String, Int> = Either::Left("traverse-fail")
        failed
    } else {
        let mut success: Either<String, Int> = Either::Right(value)
        success
    }
}
let mut runtimeTraverseFailure = MonadAlgorithms.traverse(runtimeMonad, [1, 2, 3], traverseFailureTransform)
let mut runtimeTraverseFailurePreserved = runtimeTraverseFailure.fold(
    left: |error| { error == "traverse-fail" },
    right: |values| { false }
)
let mut runtimeTraverseShortCircuited = traverseFailureTransformCalls == 2

let mut runtimeAll = monadMappedRightValue == 42 and monadMappedLeftPreserved and monadMapLeftShortCircuited and monadPureValue == 7 and monadMap2Value == 3 and monadMap2FailurePreserved and monadMap2LeftShortCircuited and monadMap2RightFailurePreserved and monadMap2RightShortCircuited and monadFlatMapValue and monadFlatMapFailurePreserved and monadFlatMapShortCircuited and runtimeKleisliValue and runtimeTraverseSuccessValue and runtimeTraverseFailurePreserved and runtimeTraverseShortCircuited
