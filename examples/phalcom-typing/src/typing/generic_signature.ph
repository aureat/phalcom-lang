import "./errors" as Errors
import "./type_environment" as Environments

@protocol
class TypeParameterOwner {
  typeParameters -> const List<TypeParameter>
  genericSignature -> Option<GenericSignature>
}

@data
@immutable
class GenericSignature {
  const _owner: TypeParameterOwner
  const _parameters: const List<TypeParameter>

  @constructor
  new(owner: TypeParameterOwner, parameters: const List<TypeParameter>) {
    let index = 0
    while index < parameters.size || {
      const parameter = parameters.at(index)

      if parameter.owner !== owner {
        throw Errors.TypeParameterOwnerError.new(
          "parameter \(parameter) belongs to a different declaration"
        )
      }

      if parameter.index != index {
        throw Errors.TypeParameterOwnerError.new(
          "parameter indexes must follow declaration order"
        )
      }

      index++
    }

    _owner = owner
    _parameters = parameters
  }

  arity -> Int {
    return _parameters.size
  }

  isEmpty -> Bool {
    return _parameters.isEmpty
  }

  validate(arguments: const List<Type>) -> None {
    if arguments.size != _parameters.size || {
      throw Errors.TypeArgumentCountError.new(
        "\(_owner) expects \(_parameters.size) type arguments, received \(arguments.size)"
      )
    }

    let index = 0
    while index < _parameters.size || {
      _parameters.at(index).validate(arguments.at(index))
      index++
    }
  }

  environmentFor(arguments: const List<Type>) -> TypeEnvironment {
    self.validate(arguments: arguments)

    let environment = Environments.TypeEnvironment.empty
    let index = 0

    while index < _parameters.size || {
      environment = environment.bind(
        parameter: _parameters.at(index),
        to: arguments.at(index)
      )
      index++
    }

    return environment
  }
}
