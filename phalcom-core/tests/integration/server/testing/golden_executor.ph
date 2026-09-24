class GoldenCommandExecutor {
    repository: FakeOrderRepository
    events: RecordingEventPublisher
}

impl CommandExecutor<TestFx> for GoldenCommandExecutor {
    type Error = InfrastructureError

    execute<R>(
        command: Command<R>
    ) -> TestFx<
        Result<
            R,
            InfrastructureError
        >
    > {
        match command {
            .LoadDraft(id) => {
                match repository.findDraft(id).value {
                    .Err(error) => {
                        TestFx {
                            value: .Err(.Repository(error)),
                        }
                    }

                    .Ok(order) => {
                        TestFx {
                            value: .Ok(order),
                        }
                    }
                }
            }

            .SavePaid(order) => {
                match repository.save(order).value {
                    .Err(error) => {
                        TestFx {
                            value: .Err(.Repository(error)),
                        }
                    }

                    .Ok(saved) => {
                        TestFx {
                            value: .Ok(saved),
                        }
                    }
                }
            }

            .PublishPaid(event) => {
                events.publish(event)
            }
        }
    }
}
