let runtimeLeft: Either<String, Int> = Either::Left("boom")
let runtimeRight: Either<String, Int> = Either::Right(41)

let runtimeLeftIsLeft = runtimeLeft.isLeft
let runtimeLeftIsRight = runtimeLeft.isRight
let runtimeRightIsLeft = runtimeRight.isLeft
let runtimeRightIsRight = runtimeRight.isRight

let mappedRight = runtimeRight.map(|value| { value + 1 })
let mappedRightValue = mappedRight.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mappedLeft = runtimeLeft.map(|value| { value + 1 })
let mappedLeftPreserved = mappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)

let mappedLeftSide = runtimeLeft.mapLeft(|error| { error == "boom" })
let mappedLeftSideValue = mappedLeftSide.fold(
    left: |value| { value },
    right: |value| { false }
)

let bimapRight = runtimeRight.bimap(
    left: |error| { error == "boom" },
    right: |value| { value + 1 }
)
let bimapRightValue = bimapRight.fold(
    left: |value| { -1 },
    right: |value| { value }
)

let flatMapped = runtimeRight.flatMap(|value| {
    let next: Either<String, Bool> = Either::Right(value == 41)
    next
})
let flatMappedValue = flatMapped.fold(
    left: |error| { false },
    right: |value| { value }
)

let flatMappedLeft = runtimeLeft.flatMap(|value| {
    let next: Either<String, Bool> = Either::Right(value == 41)
    next
})
let flatMappedLeftPreserved = flatMappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)

let swappedLeft = runtimeLeft.swap
let swappedLeftValue = swappedLeft.fold(
    left: |value| { false },
    right: |value| { value == "boom" }
)

let swappedRight = runtimeRight.swap
let swappedRightValue = swappedRight.fold(
    left: |value| { value == 41 },
    right: |value| { false }
)

let fallbackValue = runtimeLeft.getOrElse(99)
let preservedValue = runtimeRight.getOrElse(99)
let recoveredValue = runtimeLeft.recover(|error| { 77 })
let unrecoveredValue = runtimeRight.recover(|error| { 77 })

let replacement: Either<Bool, Int> = Either::Right(100)
let orElseLeft = runtimeLeft.orElse(replacement)
let orElseLeftValue = orElseLeft.fold(
    left: |value| { -1 },
    right: |value| { value }
)

let ignoredReplacement: Either<Bool, Int> = Either::Left(false)
let orElseRight = runtimeRight.orElse(ignoredReplacement)
let orElseRightValue = orElseRight.fold(
    left: |value| { -1 },
    right: |value| { value }
)

let zipOther: Either<String, Bool> = Either::Right(true)
let zipped = runtimeRight.zip(zipOther)
let zipValue = zipped.fold(
    left: |error| { -1 },
    right: |pair| {
        let (number, flag) = pair
        if flag { number } else { -1 }
    }
)

let runtimeAll = (
    runtimeLeftIsLeft
    and not runtimeLeftIsRight
    and not runtimeRightIsLeft
    and runtimeRightIsRight
    and mappedRightValue == 42
    and mappedLeftPreserved
    and mappedLeftSideValue
    and bimapRightValue == 42
    and flatMappedValue
    and flatMappedLeftPreserved
    and swappedLeftValue
    and swappedRightValue
    and fallbackValue == 99
    and preservedValue == 41
    and recoveredValue == 77
    and unrecoveredValue == 41
    and orElseLeftValue == 100
    and orElseRightValue == 41
    and zipValue == 41
)
