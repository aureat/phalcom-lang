# Primitive contracts

> Sources: native primitive metadata
> Raw: [native metadata source snapshot](../raw/native-meta/2026-09-07-phalcom-native-meta.md)
> Updated: 2026-09-08

A primitive contract binds a stable key to dispatch side, visibility, stability, anchor policy, parameter/return/callable types, raises, effects, return flow, termination, lifecycle, ABI, intrinsic identity, trust, documentation, and conceptual text.

## Contract vocabulary

Effects include purity, mutation, I/O, scheduling, reflection, nondeterminism, blocking, or unknown. Return flow can be a value, receiver, argument, never, or unknown. ABI and trust are explicit so a host implementation cannot accidentally imply a user-visible semantic guarantee.

The static type vocabulary mirrors symbolic notation. [Callable contracts](../type-system/callable-and-generic-contracts.md) owns generic/label structure; [semantic authority](../semantic/authority-and-identity.md) decides whether a formal fact is established.

## Status

Metadata is a declarative contract layer. It does not prove the runtime implementation exists; catalog and generated-surface gates provide separate evidence.
