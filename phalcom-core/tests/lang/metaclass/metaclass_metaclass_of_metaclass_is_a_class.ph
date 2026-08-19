// area: metaclass
// spec: object-model.md §5 (metaclass tower)
// status: PASS
// One rung further up the metaclass tower than `parallel_rule_user_class`:
// `Point.class` is Point's metaclass; `Point.class.class` is that
// metaclass's OWN class (the metaclass-of-the-metaclass). Pinned as observed:
// the doubly-lifted object reports `isA(Class)` true, while the metaclass
// itself (`Point.class`) reports `isA(Class)` false — the tower's `Class`
// membership is not uniform across metaclass rungs in the current
// implementation.

class Point {
}
System.print(Point.class.class.is(Class))
System.print(Point.class.is(Class))
