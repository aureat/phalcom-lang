// Expected failure: PublishPaid establishes R = ().
brokenPublishCase<R>(
    command: Command<R>
) -> TestFx<Result<R, InfrastructureError>> {
    match command {
        LoadDraft(_) => TestFx { value: Ok(None) }
        SavePaid(order) => TestFx { value: Ok(order) }
        PublishPaid(_) => TestFx { value: Ok(true) }
    }
}
