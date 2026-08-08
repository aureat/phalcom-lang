import "./type_descriptor" as Descriptors
import "./applied_member" as Members
import "./type_runtime" as Runtime

@data
@immutable
class AppliedType is Descriptors.TypeDescriptor {
  const _origin: TypeConstructor
  const _arguments: const List<Type>
  const _environment: TypeEnvironment

  @constructor
  new(
    origin: TypeConstructor,
    arguments: const List<Type>,
    environment: TypeEnvironment
  ) {
    _origin = origin
    _arguments = arguments
    _environment = environment
  }

  displayName -> String {
    const rendered = _arguments
      .map |argument| { argument.displayName }
      .joined(", ")

    return "\(_origin.displayName)<\(rendered)>"
  }

  origin -> TypeConstructor {
    return _origin
  }

  arguments -> const List<Type> {
    return _arguments
  }

  environment -> TypeEnvironment {
    return _environment
  }

  typeParameters -> const List<TypeParameter> {
    return const []
  }

  freeParameters -> const List<TypeParameter> {
    let result = const []

    _arguments.each |argument| {
      argument.freeParameters.each |parameter| {
        if result.includes(parameter).not {
          result = result.appending(parameter).freeze
        }
      }
    }

    return result
  }

  isApplied -> Bool {
    return true
  }

  isGeneric -> Bool {
    return self.freeParameters.isEmpty.not
  }

  substitute(using: TypeEnvironment) -> Type {
    const substituted = _arguments.map |argument| {
      argument.substitute(using: using)
    }.freeze

    if substituted == _arguments {
      return self
    }

    return Runtime.TypeRuntime.apply(
      origin: _origin,
      arguments: substituted
    )
  }

  equivalentTo(other: Type) -> Bool {
    if other.isA(AppliedType).not {
      return false
    }

    if _origin !== other.origin {
      return false
    }

    if _arguments.size != other.arguments.size {
      return false
    }

    let index = 0
    while index < _arguments.size {
      if _arguments.at(index).equivalentTo(other.arguments.at(index)).not {
        return false
      }
      index++
    }

    return true
  }

  methodFor(selector: Selector) -> Option<AppliedMethod> {
    return _origin.methodFor(selector).map |method| {
      Members.AppliedMethod.new(
        method: method,
        environment: _environment
      )
    }
  }

  fields -> const List<AppliedField> {
    return _origin.fields.map |field| {
      Members.AppliedField.new(
        field: field,
        environment: _environment
      )
    }.freeze
  }

  // Explicit representation of the forwarding primitive used after AppliedType
  // has failed its own selector lookup. All origin class-side selectors qualify.
  forward(selector: Selector, arguments: const List<Any>) -> Any {
    return Runtime.TypeRuntime.forwardClassSide(
      application: self,
      selector: selector,
      arguments: arguments
    )
  }

  doesNotUnderstand(message: Message) -> Any {
    return self.forward(
      selector: message.selector,
      arguments: message.arguments.freeze
    )
  }
}
