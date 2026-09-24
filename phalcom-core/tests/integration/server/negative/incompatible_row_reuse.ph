data RowProbeUser {
    user: User
}

data RowProbeOrder {
    user: OrderId
}

forceSameRow<R>(
    left: { **R },
    right: { **R }
) -> () {
    ()
}

invalidIncompatibleRowReuse() {
    forceSameRow(
        RowProbeUser {
            user: FixtureFactory.user,
        },
        RowProbeOrder {
            user: OrderId("ord-123"),
        }
    )
}
