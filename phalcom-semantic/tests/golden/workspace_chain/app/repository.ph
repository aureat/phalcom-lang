// LAW CHAIN
// Repository imports and publishes User through a class-side constructor call.

import app.model.User

class UserRepository {
  @class
  load() -> User {
    User.new()
  }
}

export UserRepository
