# Reject oversized message sends

## Problem

`Bytecode::Invoke` and `Bytecode::SuperSend` encode argument count as `u8`.
Compiler lowering currently casts source argument counts with `as u8`. A send
with more than 255 arguments therefore truncates its encoded count. The VM
then finds receiver from wrong stack offset and selector arity no longer
matches source shape. This can corrupt VM stack state.

The compiler must reject any source construct whose emitted send needs more
than 255 arguments. Preserve existing bytecode format; this task does not
widen opcode operands.

## Scope

Add a typed compiler diagnostic for an argument count above `u8::MAX`.

Create one shared checked conversion from `usize` to send arity. Use it before
constructing selectors, compiling send arguments, or emitting bytecode.

Apply it to every source path that emits a variable-arity send:

- ordinary method sends
- `super` sends
- bracket/index reads
- bracket/index writes, whose implicit trailing `put:` value counts as one
  argument

Reject oversized pinned selector references too, since their selector arity
also currently narrows to `u8`.

Reject oversized declared method, constructor, and subscript arities. Without
this companion guard, declaration selector identity can alias after narrowing
even if ordinary calls are protected.

## Error model and user-visible result

This is a compile failure, not a language-level exception. Add a structured
`CompilerError` variant carrying:

```rust
ArityLimit {
    subject: &'static str,
    found: usize,
    limit: u8,
    span: SourceRange,
}
```

`subject` identifies the rejected syntax (`message send`, `super send`,
`subscript read`, `subscript write`, `pinned selector`, or the relevant
declaration). `found` must retain the original `usize` count; never report the
already-narrowed value. `span` is the whole offending call/reference/
declaration. The current compile-error renderer is span-less, but retaining the
span makes the error object testable now and ready for source excerpts later.

Required display text:

```text
message send has 256 arguments; bytecode supports at most 255
```

The CLI result for source compilation is exactly a compile diagnostic:

```text
error: message send has 256 arguments; bytecode supports at most 255
```

No bytecode unit is returned or executed. Consequently there is no VM
traceback, no `RuntimeError`, no `RuntimeError::Raise`, and no catchable
language `Error` object. The outer error is
`PhError::Compile(CompilerError::ArityLimit { .. })`.

Do not map this to the existing `RuntimeError::Arity`: that error means a
well-formed call reached a callable with the wrong signature. This failure is
an unrepresentable call-site shape and must stop before stack effects or method
lookup.

## Out of scope

- Widening `Invoke` or `SuperSend` operands past `u8`
- New variadic calling convention
- Runtime handling for malformed externally loaded bytecode

Bytecode is compiler-produced today. A future bytecode loader needs a verifier,
but a runtime check cannot recover an original count after it has already been
truncated to `u8`.

## Likely change points

- `phalcom-core/src/compiler/lib/error.rs`: structured oversized-arity error
- `phalcom-core/src/compiler/lib/expr.rs`: ordinary, super, index, set-index,
  pinned-selector lowering
- `phalcom-core/src/compiler/lib/class_decl.rs`: declared signature arities
- `phalcom-core/src/compiler/attributes.rs`: generated declared signatures

Do not leave raw `as u8` conversions where an AST-sourced parameter or argument
count reaches `SignatureKind`, `Invoke`, or `SuperSend`.

## Acceptance tests

- A 255-argument ordinary send compiles and executes normally.
- A 256-argument ordinary send returns the new compile error, before bytecode
  for that send is emitted.
- The compiler API returns `PhError::Compile(CompilerError::ArityLimit {`
  `subject: "message send", found: 256, limit: 255, .. })` for that fixture;
  CLI output is the single required `error:` line and contains no `Traceback`.
- A 256-argument `super` send returns the same compile error.
- A bracket read accepts 255 arguments and rejects 256.
- A bracket write accepts 254 explicit index arguments plus its implicit
  `put:` value; it rejects 255 explicit index arguments.
- A pinned selector reference and each declaration form reject arity above 255.
- Existing compiler and VM test suites remain green.

## Invariant

Every compiler-emitted `Invoke` and `SuperSend` operand represents exactly the
number of argument values pushed after its receiver. No source `usize` count
may be narrowed to `u8` without checked validation.
