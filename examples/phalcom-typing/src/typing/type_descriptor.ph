// Shared implementation for synthetic type expressions. Existing Class and
// Protocol objects conform to Type directly and do not inherit this class.
@abstract
class TypeDescriptor {
  origin -> Type {
    return self
  }

  arguments -> const List<Type> {
    return const []
  }

  typeParameters -> const List<TypeParameter> {
    return const []
  }

  freeParameters -> const List<TypeParameter> {
    return const []
  }

  isGeneric -> Bool {
    return false
  }

  isApplied -> Bool {
    return false
  }

  substitute(using: TypeEnvironment) -> Type {
    return self
  }

  equivalentTo(other: Type) -> Bool {
    return self === other
  }

  toString -> String {
    return self.displayName
  }
}
