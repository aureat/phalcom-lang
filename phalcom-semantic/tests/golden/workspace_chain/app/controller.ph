// LAW CHAIN
// Controller observes inherited id and own name after cross-module publication.

import app.service.UserService

class Controller {
  @class
  run() {
    let user = UserService.current()
    let id = user.id()
    let name = user.name()
    (id, name)
  }
}

export Controller
