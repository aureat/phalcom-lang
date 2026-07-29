// Applied members are substituted reflective views. They never copy bytecode,
// alter selectors, or create specialized dispatch entries.
@data
@immutable
class AppliedParameter {
  const _parameter: Parameter
  const _environment: TypeEnvironment

  name -> Symbol {
    return _parameter.name
  }

  label -> Option<Symbol> {
    return _parameter.label
  }

  position -> Int {
    return _parameter.position
  }

  type -> Option<Type> {
    return _parameter.type.map { annotation =>
      annotation.substitute(using: _environment)
    }
  }

  attributes -> const List<Attribute> {
    return _parameter.attributes
  }
}

@data
@immutable
class AppliedMethod {
  const _method: Method
  const _environment: TypeEnvironment

  selector -> Selector {
    return _method.selector
  }

  owner -> Class {
    return _method.owner
  }

  executable -> Executable {
    return _method.executable
  }

  parameters -> const List<AppliedParameter> {
    return _method.parameters.map { parameter =>
      AppliedParameter.new(
        parameter: parameter,
        environment: _environment
      )
    }.freeze
  }

  returnType -> Option<Type> {
    return _method.returnType.map { annotation =>
      annotation.substitute(using: _environment)
    }
  }

  typeParameters -> const List<TypeParameter> {
    return _method.typeParameters
  }

  invokeOn(receiver: Any, arguments: const List<Any>) -> Any {
    return _method.invokeOn(receiver, arguments: arguments)
  }
}

@data
@immutable
class AppliedField {
  const _field: Field
  const _environment: TypeEnvironment

  name -> Symbol {
    return _field.name
  }

  type -> Option<Type> {
    return _field.type.map { annotation =>
      annotation.substitute(using: _environment)
    }
  }

  isMutable -> Bool {
    return _field.isMutable
  }
}
