import "./type_runtime" as Runtime

// The common behavioral surface of every type-expression object.
// Runtime Class and Protocol objects conform without changing inheritance.
@protocol
class Type {
  displayName -> String
  origin -> Type
  arguments -> const List<Type>
  typeParameters -> const List<TypeParameter>
  freeParameters -> const List<TypeParameter>

  isGeneric -> Bool
  isApplied -> Bool

  substitute(using: TypeEnvironment) -> Type
  equivalentTo(other: Type) -> Bool

  // This is a method on the first-class Protocol descriptor named Type.
  // It is not an instance requirement imposed on conforming type objects.
  @class
  currentApplication -> Option<AppliedType> {
    return Runtime.TypeRuntime.currentApplication
  }
}
