from .id import VariableId

data Variable<T> private(
  id: VariableId,
  name: Symbol,
)

impl<T> Variable<T> {
  @class
  named(name: Symbol) -> Self {
    Variable<T>(
      id: VariableId.fresh,
      name: name
    )
  }
}

export Variable
