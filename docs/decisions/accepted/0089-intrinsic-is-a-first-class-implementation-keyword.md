# TDR-0089 — `intrinsic` Is a First-Class Implementation-Kind Keyword and `@intrinsic` Is Retired

- Ratified: 2026-09-22
- Status: Accepted
- Date: 2026-09-22
- Supersedes: none
- Amends: [TDR-0082](./0082-declaration-modifiers-and-identity-based-access-domains.md) for `@intrinsic` source attribute status; [TDR-0088](./0088-native-type-operations-use-closed-compiler-normalization.md) for source declaration syntax and native-type-member phrasing
- Depends on: [TDR-0082](./0082-declaration-modifiers-and-identity-based-access-domains.md), [TDR-0083](./0083-type-level-domains-and-typed-compile-time-parameters.md), [TDR-0085](./0085-parameterized-type-members-and-symbolic-projections.md), [TDR-0088](./0088-native-type-operations-use-closed-compiler-normalization.md)
- Related: LANG007.C1; `docs/specs/source-declarations.md`; `docs/specs/type-members.md`; `docs/specs/universe.md`
- Repository evidence revision: `aureat/phalcom-lang@33ee08f2`

## Context

TDR-0082 established declaration modifiers (`native`, `foreign`, `private`, `protected`, `module`, `package`, `internal`) and classified `@primitive` and `@intrinsic` as metadata-only attributes. Subsequently, TDR-0088 specified that type-level operations normalize entirely in the compiler/semantic layer without runtime VM execution, using `@intrinsic` and `native type` examples in historical text.

However, treating compiler-intrinsic declarations as `@intrinsic` attributes or conflating them with `native` declarations creates confusion between declaration implementation provenance and compiler/runtime internal optimization metadata. A declaration whose implementation is provided directly by compiler/semantic calculus is not a host-bound `native` declaration, nor should its implementation category be governed by an attribute.

## Decision

1. `intrinsic` is a first-class lexical source keyword and declaration modifier.
2. Lexing and parsing normalize the modifier into `ImplementationKind::Intrinsic`. The four canonical implementation categories are:
   - `Phalcom`: ordinary source-language implementation body;
   - `Native`: host/native runtime binding;
   - `Foreign`: bounded foreign interface binding;
   - `Intrinsic`: implementation provided directly by canonical compiler/semantic intrinsic semantics.
3. The source attribute `@intrinsic` is retired and removed as a builtin attribute. It receives no builtin compiler privileges, attribute expanders, or semantic recognition.
4. `@primitive` is unaffected by this decision and remains an independent metadata attribute where specified for Universe native declarations.
5. `ImplementationKind::Intrinsic` is orthogonal to visibility and access authority (`Visibility::{Public, Private, Protected, Module, Package, Internal}`). An `intrinsic` declaration obeys the same access rules as any other declaration.
6. Target legality of `intrinsic` is defined by the owning declaration category's grammar and semantics (e.g., `intrinsic type ...`, `intrinsic class ...`, `intrinsic data ...`). `intrinsic` being a keyword does not imply that every declaration category admits intrinsic implementation without explicit specification.
7. LANG007 intrinsic type members (such as Tuple `Positionals`, `Labeled`, `At`, and structural `Map`) are implemented purely via semantic/type-system normalization. They do not generate runtime methods, VM bytecodes, message sends, or runtime native callables.
8. `native` and `intrinsic` are distinct implementation kinds. `native` implies runtime host execution; `intrinsic` implies compiler-implemented behavior.
9. Internal `NativeIntrinsicId` metadata (used by compiler/runtime layers for recognized primitive operations such as `BoolNot`, `BoolAnd`, `BoolOr`) remains an internal semantic/optimization identifier for native surfaces. It is separate from the source implementation kind and does not reintroduce `@intrinsic` source attribute syntax.
10. Semantic behavior and normalization are keyed by canonical declaration/member identity and domain structure, not by surface keyword spelling.
11. Source-like presentation and reflection tooling must render valid current language syntax (`intrinsic ...`) and must not synthesize retired `@intrinsic(...)` attribute syntax.

## Relationship to Earlier Decisions

- **Amends TDR-0082:** Replaces the rule that `@intrinsic` is a live metadata-only source attribute. `intrinsic` is now a first-class declaration modifier keyword mapped to `ImplementationKind::Intrinsic`.
- **Amends TDR-0088:** Updates the source-level syntax and phrasing for compiler-normalized type operations from `native type` / `@intrinsic` to `intrinsic type`.
- **Preserves TDR-0082:** Preserves identity-based access control, visibility semantics, native/foreign distinctions, and the invariant that names never imply implementation authority.
- **Preserves TDR-0088:** Preserves closed compiler-side type normalization, side-effect-free evaluation, and zero-VM type-member execution.

## Explicit Non-Decisions and Rejections

- **No attribute alias:** `@intrinsic` is not retained as a backwards-compatibility alias.
- **No visibility conflation:** `intrinsic` is an implementation kind, never an access domain or visibility modifier.
- **No native synonymy:** `intrinsic` and `native` are not interchangeable.
- **No metadata-driven kind inference:** `ImplementationKind::Intrinsic` is not derived from internal `NativeIntrinsicId` metadata.
- **No VM type-member dispatch:** Intrinsic type members do not materialize runtime objects or execute in VM frames.
