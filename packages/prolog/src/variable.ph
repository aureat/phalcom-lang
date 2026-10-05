from .id import (
  IdentityToken,
  Id
)

data Variable<T> private(
  id: VariableId,
  name: Symbol,
)

impl<T> Variable<T> {
  @class
  named(name: Symbol) -> Variable<T> {
    Variable(
      VariableId.fresh,
      name
    )
  }
}

export Variable