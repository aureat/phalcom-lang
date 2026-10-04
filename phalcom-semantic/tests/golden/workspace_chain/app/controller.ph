// LAW CHAIN
// Controller observes inherited id and own name after cross-module publication.

import app.service.UserService

class Controller {
  @class
  run() {
    let mut user = UserService.current()
    let mut id = user.id()
    let mut name = user.name()
    (id, name)
  }
}

export Controller
