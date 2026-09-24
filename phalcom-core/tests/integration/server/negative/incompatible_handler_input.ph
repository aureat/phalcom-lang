data Admin {
    id: UserId
}

useUserHandler(
    handler: ({
        user: User,
        ...
    }) -> String
) -> () {}

adminOnly(
    context: {
        user: Admin,
        ...
    }
) -> String {
    context.user.id.value
}

// Expected failure: a handler requiring Admin cannot stand in for one that
// must accept an arbitrary context containing User.
brokenHandlerCompatibility() {
    useUserHandler(adminOnly)
}
