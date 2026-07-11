# Open Questions

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

Undecided design points. Each must be resolved here before its dependent
implementation begins.

1. **`let` vs `var`.** Recommendation: `let` = immutable binding, `var` = mutable
   binding, `var x` with no initializer is `None`. Not yet ratified. (The lexer
   currently has only `let`.)
2. **`Number`.** One numeric type, or `Int` / `Float` split? Affects the VM value
   representation and every arithmetic opcode. Decide before the inliner.
3. **External vs internal parameter names.** Swift allows `move(to target:)` —
   external label `to`, internal binding `target`. Phalcom currently binds the
   label name directly. Worth it?
4. **Class hierarchy mutability.** Is `Test.superclass = Test` legal at runtime
   (Smalltalk: yes) or sealed after definition (Wren: no)? Affects whether slot
   layouts and inline caches can assume stability.
5. **String interpolation syntax.** `"{name}"` is assumed. `"${name}"` and
   `"\(name)"` are alternatives.
6. **Set literal.** Currently `Set(...)`. `#{1, 2, 3}` remains available if the
   ceremony becomes annoying.
7. **Destructuring.** Tuples exist, so `let (a, b) = point` and
   `let [first, *rest] = list` are natural. Not yet specified.
8. **Modules / imports.** The `import` token exists; semantics are unspecified.
9. **Error handling.** `throw` / `try` / `catch`, or `Result` as a sibling of
   `Option`? Interacts with non-local return.
10. **Traits / mixins / multiple inheritance.** Unspecified. Single inheritance is
    the current invariant ([Object Model](object-model.md)).
11. **`Behavior` in the kernel.** The [Object Model](object-model.md) introduces
    `Behavior` as the shared superclass of `Class`/`Metaclass`. Ratify or drop.
</content>
