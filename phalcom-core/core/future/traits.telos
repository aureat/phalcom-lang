trait Equatable {
    ==(_ other: Self) -> Bool
}

trait Comparable {
    compare(_ other: Self) -> Ordering
}

trait Hashable {
    ...
}

trait From<Source: Type> {
    from(_ source: Source) -> Self
}

trait TryFrom<Source: Type> {
    type Error: Type

    tryFrom(_ source: Source) -> Result<Self, Error>
}
