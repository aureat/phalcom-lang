// Expected failure: LoadDraft establishes R = Option<Order<Draft>>.
brokenLoadCase<R>(command: Command<R>) -> TestFx<Result<R, InfrastructureError>> {
    match command {
        LoadDraft(_) => {
            TestFx {
                value: Ok(FixtureFactory.draftOrder),
            }
        }
        SavePaid(order) => TestFx { value: Ok(order) }
        PublishPaid(_) => TestFx { value: Ok(()) }
    }
}
