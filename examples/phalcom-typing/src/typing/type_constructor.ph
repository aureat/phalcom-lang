import "./type_runtime" as Runtime

// Internal capability implemented by Class, Protocol, and future generic alias
// objects. The angle selector is real and reflectable, but reserved from user
// declaration and replacement.
@protocol
class TypeConstructor {
  typeParameters -> const List<TypeParameter>
  genericSignature -> Option<GenericSignature>

  @native
  <...>(*arguments: Type) -> AppliedType {
    return Runtime.TypeRuntime.apply(
      origin: self,
      arguments: arguments.freeze
    )
  }
}
