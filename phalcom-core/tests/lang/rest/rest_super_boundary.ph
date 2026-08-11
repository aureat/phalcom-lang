// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md super fallback
// status: PASS

class GrandRest {
  choose(*items) { return 300 + items.size }
  fallback(*items) { return 400 + items.size }
}

class ParentRestBoundary is GrandRest {
  choose(_ left, _ right) { return 500 }
  fallback(_ fixed, **extra) { return 600 + extra.size }
}

class ChildRestBoundary is ParentRestBoundary {
  choose(*items) { return 700 + items.size }
  fallback(*items) { return 800 + items.size }

  exactSuper() { return super.choose(1, 2) }
  exactSuperDynamic() {
    const args = (1, 2)
    return super.choose(*args)
  }

  fallbackSuper() { return super.fallback(1, 2) }
  fallbackSuperDynamic() {
    const args = (1, 2)
    return super.fallback(*args)
  }
}

const child = ChildRestBoundary.new()
System.print(child.exactSuper())
System.print(child.exactSuperDynamic())
System.print(child.fallbackSuper())
System.print(child.fallbackSuperDynamic())
