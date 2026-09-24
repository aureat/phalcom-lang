// Expected failure: solving one T for both values would require T = Option<T>.
requireSame<T>(
    left: T,
    right: T
) -> () {}

forceOrdinaryOccursCheck<T>(
    value: T
) -> () {
    requireSame(value, Some(value))
}
