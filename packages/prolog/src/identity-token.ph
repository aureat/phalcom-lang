// Still unsure about class method and field syntax. possible alternatives:
//
// Self.id -> Int
// Self.id=(_ value: Int) -> ()
// Self._id: Int
// I'm gravitating more towards `Self`
//
// class.id -> Int
// class.id=(_ value: Int) -> ()
// class._id: Int
//
// - `self` means the current instance — either the current instance object or the class object
// - `class` means the class of the current instance — either the class of the current instance object or
// the class of the current class object
// - `Self` always means the current class object, even if called from an instance method
// I have no idea whether these rules are coherent
//
class IdentityToken {
  mut Self._next: Int = 0
  private mut Self.next via _next

  _id: Int
  private id via _id

  private Self(id uniqueId: Int) -> Self {
    Self { _id = uniqueId }
  }

  Self.fresh -> Self {
    Self(++next) // technically, self(++next) would work too, but this is more explicit
  }

  ==(_ other: IdentityToken) -> Bool {
    self === other
  }

  hash -> Int {
    id.hash
  }

  toString -> String {
    "\(id)"
  }
}

data Id<Tag> private(
  token: IdentityToken,
)

impl<Tag> Id<Tag> {
  Self.fresh -> Id<Tag> {
    Id(IdentityToken.fresh)
  }

  ==(_ other: Id<Tag>) -> Bool {
    token == other.token
  }

  hash -> Int {
    token.hash
  }

  toString -> String {
    "\(token)"
  }
}

export (
  IdentityToken,
  Id
)