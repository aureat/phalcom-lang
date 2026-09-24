data AuthToken(_ value: String)

trait Authenticator {
    type Error: Type

    authenticate(
        token: AuthToken
    ) -> Result<User, Self::Error>
}
