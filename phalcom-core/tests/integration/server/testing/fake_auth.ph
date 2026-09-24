class FakeAuthenticator {
    expected: AuthToken
    user: User
}

impl Authenticator for FakeAuthenticator {
    type Error = Unauthorized

    authenticate(
        token: AuthToken
    ) -> Result<User, Unauthorized> {
        if token == expected {
            Ok(user)
        } else {
            Err(Unauthorized {})
        }
    }
}
