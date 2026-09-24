// Expected failure: ordinary type inference and row inference form a cycle.
requireValueRow<T, R>(
    value: T,
    row: { value: T, **R }
) -> () {}

forceCrossDomainCycle<R>( row: { **R } ) -> () {
    requireValueRow( row, { value: row, **row })
}
