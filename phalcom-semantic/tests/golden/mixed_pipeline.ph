// LAW CHAIN
// 1. UserRepository inherits Repository<User> and publishes User.
// 2. Pipeline joins constructor User with repository User.
// 3. Normalizer publishes a structural {name, score} record.
// 4. Record -> String is one owned refutation; Presenter and score remain valid.

class User {
  _name: String
  _score: Int

  @constructor
  new(_ name: String, _ score: Int) {
    _name = name
    _score = score
  }

  name() -> String { _name }
  score() -> Int { _score }
}

class Repository<T> {
  load() -> T {
    mystery()
  }
}

class UserRepository is Repository<User> {}

class Normalizer {
  @class
  normalize(_ user: User) -> #{name: String, score: Int} {
    #{name: user.name(), score: user.score()}
  }
}

class Pipeline {
  @class
  fetch(_ repo: UserRepository, _ fallback: Bool) {
    let user = if fallback {
      User.new("fallback", 0)
    } else {
      repo.load()
    }

    let record = Normalizer.normalize(user)
    record
  }
}

class Presenter {
  @class
  present(_ record: #{name: String, score: Int}) -> (String, Int) {
    ("user", 1)
  }
}

class Probe {
  @class
  run(_ repo: UserRepository, _ fallback: Bool) {
    let record = Pipeline.fetch(repo, fallback)

    let bad: String = record
    let presented = Presenter.present(record)
    let (label, count) = presented

    let independent = User.new("ok", 5).score()
    (label, count, independent)
  }
}
