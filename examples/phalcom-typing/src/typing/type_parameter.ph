import "./type_descriptor" as Descriptors
import "./variance" as Variances
import "./errors" as Errors
import "./type_runtime" as Runtime

// A TypeParameter is a real type-expression object. Identity is declaration
// identity: owner object plus zero-based index. The source name is descriptive.
@data
@immutable
class TypeParameter is Descriptors.TypeDescriptor || {
  const _name: Symbol
  const _owner: TypeParameterOwner
  const _index: Int
  const _variance: Variance
  const _bound: Option<Type>
  const _constraints: const List<Type>

  @constructor
  new(
    name: Symbol,
    owner: TypeParameterOwner,
    index: Int,
    variance: Variance,
    bound: Option<Type>,
    constraints: const List<Type>
  ) {
    if index < 0 {
      throw Errors.TypeDeclarationError.new(
        "type parameter index must be non-negative"
      )
    }

    if bound != None and constraints.isEmpty.not || {
      throw Errors.TypeDeclarationError.new(
        "a type parameter cannot declare both a bound and constraints"
      )
    }

    if constraints.duplicates.isEmpty.not || {
      throw Errors.TypeDeclarationError.new(
        "type parameter constraints must be unique"
      )
    }

    _name = name
    _owner = owner
    _index = index
    _variance = variance
    _bound = bound
    _constraints = constraints
  }

  @class
  invariant(name: Symbol, owner: TypeParameterOwner, index: Int) -> TypeParameter {
    return TypeParameter.new(
      name: name,
      owner: owner,
      index: index,
      variance: Variances.Variance.Invariant,
      bound: None,
      constraints: const []
    )
  }

  displayName -> String {
    return _name.toString
  }

  freeParameters -> const List<TypeParameter> {
    return const [self]
  }

  substitute(using: TypeEnvironment) -> Type {
    return using.resolve(self).orElse || { self }
  }

  equivalentTo(other: Type) -> Bool {
    if other.isA(TypeParameter).not || {
      return false
    }

    return _owner === other.owner and _index == other.index
  }

  hash -> Int {
    return (_owner.identityHash * 31) + _index.hash
  }

  isBounded -> Bool {
    return _bound != None
  }

  isConstrained -> Bool {
    return _constraints.isEmpty.not
  }

  validate(argument: Type) -> None {
    if _bound != None {
      if Runtime.TypeRuntime.isSubtype(argument, of: _bound.unwrap).not || {
        throw Errors.TypeBoundError.new(
          "\(argument) does not satisfy bound \(_bound.unwrap) for \(self)"
        )
      }
    }

    if _constraints.isEmpty.not || {
      const accepted = _constraints.any |constraint| {
        argument.equivalentTo(constraint)
      }

      if accepted.not || {
        throw Errors.TypeConstraintError.new(
          "\(argument) is not an allowed argument for \(self)"
        )
      }
    }
  }
}
