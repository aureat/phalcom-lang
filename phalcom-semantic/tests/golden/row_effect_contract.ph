// LAW CHAIN
// 1. User is nominal; Normalizer exposes its data as an open structural record row.
// 2. Repository.load has an exceptional path, so Pipeline joins value flow with a throw edge.
// 3. Contract annotations describe pre/post obligations independently from return and effect facts.
// 4. Presenter consumes the known row fields while Probe owns one deliberate record-to-String refutation.
//
// OBSERVATIONS
// 01 User constructor establishes nominal User and field facts.
// 02 User.score is constrained by the Normalizer precondition.
// 03 Normalizer's return uses `#{name: String, score: Int, | R}` row syntax.
// 04 the record row tail remains distinct from nominal User identity.
// 05 Repository<User> specializes inherited load output to User.
// 06 Repository.load contributes an exceptional/opaque effect path through throw.
// 07 Pipeline joins fallback construction with repository flow.
// 08 Normalizer publishes the structural record through a contract-bearing call.
// 09 Presenter reads required row fields without assuming a closed record.
// 10 bad: String owns one local refutation; record facts remain available.
// 11 independent constructor/member call remains Int after the contradiction.
// 12 Pipeline -> Normalizer -> Presenter -> Probe retains nominal, row, contract, and effect dependencies.
//
// The current source surface has no standalone effect-row annotation. The explicit
// throw path keeps effect/exit analysis observable without collapsing it into type facts.

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
    throw "repository unavailable"
  }
}

class UserRepository is Repository<User> {}

class Normalizer {
  @class
  @requires(user.score() >= 0)
  @ensures(result.name == user.name())
  normalize(_ user: User) -> #{name: String, score: Int, | R} {
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
  present(_ record: #{name: String, score: Int, | R}) -> (String, Int) {
    (record.name, record.score)
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
