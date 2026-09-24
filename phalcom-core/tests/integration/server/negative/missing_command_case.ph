// Expected failure: non-exhaustive match; PublishPaid is missing.
brokenCommandMatch<R>(
    command: Command<R>
) -> TestFx<Result<R, InfrastructureError>> {
    match command {
        LoadDraft(_) => {
            TestFx { value: Ok(None) }
        }
        SavePaid(order) => {
            TestFx { value: Ok(order) }
        }
    }
}
