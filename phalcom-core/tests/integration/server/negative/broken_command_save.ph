// Expected failure: SavePaid establishes R = Order<Paid>.
brokenSaveCase<R>(
    command: Command<R>
) -> TestFx<Result<R, InfrastructureError>> {
    match command {
        LoadDraft(_) => TestFx { value: Ok(None) }
        SavePaid(_) => {
            TestFx {
                value: Ok(FixtureFactory.draftOrder),
            }
        }
        PublishPaid(_) => TestFx { value: Ok(()) }
    }
}
