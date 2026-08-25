// LAW CHAIN
// UserService composes Repository -> User and republishes the imported result.

import app.model.User
import app.repository.UserRepository

class UserService {
  @class
  current() -> User {
    UserRepository.load()
  }
}

export UserService
