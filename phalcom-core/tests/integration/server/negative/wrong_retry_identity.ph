// EXPECT: RetryPolicy<Count(4)> does not satisfy RetryPolicy<Count(3)>.
requiresThreeRetries(
    _ policy: RetryPolicy<Count(3)>
) -> () {
    ()
}

negativeWrongRetryIdentity() {
    requiresThreeRetries(
        RetryPolicy<Count(4)> {
            attempts: 4,
        }
    )
}
