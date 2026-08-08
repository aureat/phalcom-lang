import "./errors" as Errors

@data
@immutable
class TypeBinding {
  const _parameter: TypeParameter
  const _type: Type
}

// An immutable declaration-identity substitution map. A compact native map may
// replace the linear representation without changing this public surface.
@data
@immutable
class TypeEnvironment {
  const _bindings: const List<TypeBinding>

  @class
  empty -> TypeEnvironment {
    return TypeEnvironment.new(bindings: const [])
  }

  @constructor
  new(bindings: const List<TypeBinding>) {
    _bindings = bindings
  }

  size -> Int {
    return _bindings.size
  }

  isEmpty -> Bool {
    return _bindings.isEmpty
  }

  resolve(parameter: TypeParameter) -> Option<Type> {
    let index = _bindings.size - 1

    while index >= 0 {
      const binding = _bindings.at(index)
      if binding.parameter.equivalentTo(parameter) {
        return Some.new(binding.type)
      }
      index--
    }

    return None
  }

  bind(parameter: TypeParameter, to: Type) -> TypeEnvironment {
    const existing = self.resolve(parameter)

    if existing != None {
      if existing.unwrap.equivalentTo(to) {
        return self
      }

      throw Errors.TypeEnvironmentConflictError.new(
        "conflicting substitution for \(parameter): \(existing.unwrap) and \(to)"
      )
    }

    const binding = TypeBinding.new(parameter: parameter, type: to)
    return TypeEnvironment.new(
      bindings: _bindings.appending(binding).freeze
    )
  }

  merge(other: TypeEnvironment) -> TypeEnvironment {
    let result = self
    let index = 0

    while index < other.bindings.size || {
      const binding = other.bindings.at(index)
      result = result.bind(
        parameter: binding.parameter,
        to: binding.type
      )
      index++
    }

    return result
  }

  substitute(type: Type) -> Type {
    return type.substitute(using: self)
  }
}
