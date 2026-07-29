// VM-authoritative operations are represented as ordinary standard-library
// methods. @native keeps the complete source surface inspectable while Rust
// supplies canonical interning, frame context, and reserved-selector safety.
class TypeRuntime {
  @class
  @native
  apply(origin: TypeConstructor, arguments: const List<Type>) -> AppliedType {
    const signature = origin.genericSignature.orElse {
      throw TypeApplicationError.new(
        "\(origin) is not a generic type constructor"
      )
    }

    signature.validate(arguments: arguments)
    const environment = signature.environmentFor(arguments: arguments)

    return AppliedType.new(
      origin: origin,
      arguments: arguments,
      environment: environment
    )
  }

  // Formal primitive method corresponding to source `Origin<A, B>`.
  // Parser lowering is conceptually: Origin.<...>(const [A, B]).
  @class
  @native
  <...>(origin: TypeConstructor, *arguments: Type) -> AppliedType {
    return self.apply(origin: origin, arguments: arguments.freeze)
  }

  @class
  @native
  currentApplication -> Option<AppliedType> {
    return None
  }

  @class
  @native
  forwardClassSide(
    application: AppliedType,
    selector: Selector,
    arguments: const List<Any>
  ) -> Any {
    // The native implementation pushes `application`, invokes the origin's
    // class-side selector with self = application.origin, and restores the
    // previous fiber-local context on every return or throw path.
    return application.origin.perform(
      selector,
      arguments: arguments
    )
  }

  @class
  @native
  isSubtype(candidate: Type, of: Type) -> Bool {
    // Phase 1 provides the primitive seam. Phase 2 extends this floor with
    // structural protocols, generic variance, Dynamic consistency, and the
    // complete special-type lattice.
    if candidate.equivalentTo(of) {
      return true
    }

    if candidate.isA(Class).not or of.isA(Class).not {
      return false
    }

    let current = candidate
    while current != None {
      if current === of {
        return true
      }
      current = current.superclass
    }

    return false
  }
}
