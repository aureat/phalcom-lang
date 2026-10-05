class IdentityToken {
  @class mut _next: Int = 0

  _id: Int

  @constructor
  private new(id uniqueId: Int) -> Self {
    _id = uniqueId
  }

  @class
  fresh -> Self {
    _next = _next + 1
    Self.new(_next)
  }

  ==(_ other: Dynamic) -> Bool { self === other }
  hash -> Int { _id.hash }
  toString -> String { "\(_id)" }
}

data Id<Tag> private(
  token: IdentityToken,
)

impl<Tag> Id<Tag> {
  @class
  fresh -> Self<Tag> {
    Id<Tag>(token: IdentityToken.fresh)
  }

  ==(_ other: Dynamic) -> Bool { token == other.token }
  hash -> Int { token.hash }
  toString -> String { "\(token)" }
}

export IdentityToken, Id
