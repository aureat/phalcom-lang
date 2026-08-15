# Recovery and Incomplete Programs

## 1. Editor semantics is analysis over imperfect syntax, not a different language

A live editor routinely sees source that cannot be executed: missing delimiters, half-written selectors, incomplete imports, duplicate temporary declarations, absent arguments, or unfinished type syntax. Semantic analysis should preserve useful unaffected facts without silently interpreting invalid complete programs as valid Phalcom.

Keep three layers distinct:

```text
source recovery fact     parser/semantic structure is synthetic or incomplete
semantic uncertainty     meaning cannot be determined from available program
language error           complete source violates a normative rule
```

A missing token inserted by recovery is not a runtime value and not `Dynamic`.

## 2. Recovery principle: local damage, bounded uncertainty

When a malformed region appears, invalidate only semantic facts whose meaning depends on it. Example:

```phalcom
class User {
  name() { "Ada" }
  age(        // incomplete method
}

let u = User()
u.name()
```

If parser recovery preserves the `User` class and complete `name` member surface, completion/navigation for `u.name()` should remain available. The broken `age` declaration may be recovery-only/invalid without poisoning every class fact.

## 3. Recovery-aware source model

Downstream code needs explicit markers:

```rust
struct SourceOrigin {
    range: SourceRange,
    recovery: RecoveryStatus,
}

enum RecoveryStatus {
    Authored,
    MissingToken,
    SyntheticNode,
    SkippedRegion,
}
```

Or equivalent AST metadata. Semantic facts should know when a declaration/selector was reconstructed heuristically. Do not infer certainty merely because the parser produced an AST node.

## 4. Partial declarations

A half-written declaration may have enough information to create a temporary editor surface but not a stable language declaration. Use separate states:

```text
CompleteDeclaration(id)
RecoverableDeclaration(temp/snapshot-local id, known prefix)
InvalidDeclaration(reason)
```

Recovery identities should be snapshot-local unless a stable remapping policy is explicitly needed. They should never leak into persisted package metadata or correctness proofs.

## 5. Incomplete calls and selectors

Completion often occurs at exactly the point where a selector is incomplete:

```phalcom
user.upd|
```

The semantic engine should be able to resolve the receiver independently, then expose candidate members/families using the known selector prefix. That is a completion query, not normative dispatch resolution.

Similarly, an incomplete labeled call can expose signature candidates without claiming that the final canonical selector is known. Keep “prefix/member search” separate from “resolved call target.”

## 6. Missing imports/dependencies

A source file may be valid while a dependency is not yet downloaded/indexed. Represent:

```text
Blocked(MissingModule(package/module identity))
```

rather than `Unknown` or `UnresolvedName`. This enables:

- precise diagnostic wording;
- automatic recomputation when dependency appears;
- avoiding false “no such member” conclusions;
- provenance showing why analysis is incomplete.

## 7. Duplicate declarations during editing

Typing a replacement can transiently create duplicates. If Phalcom forbids duplicates, the semantic engine should preserve the language error while still offering navigation where unambiguous.

Possible resolution result:

```text
Ambiguous { candidates: [id1, id2], cause: DuplicateDeclarations }
```

Do not arbitrarily pick “first in map order”; that yields unstable editor behavior and can misdirect rename.

## 8. Poison containment

An invalid expression may produce a recovery fact that propagates only where its value is needed. Avoid global poison:

```text
let a = valid()
let b = broken(
use(a)
```

Facts for `a` and its uses can remain valid. `b` may be blocked by parse recovery. This resembles error types in compilers but should remain a separate recovery domain from the formal type system.

## 9. Batch versus editor policy

The same core can support two policies:

```text
Editor policy:
    publish coherent partial facts + recovery metadata
    suppress cascaded diagnostics where root syntax error dominates

Batch/check policy:
    report syntax/recovery errors as invalid program
    may still run semantics for additional diagnostics if safe
    never treat recovery-synthetic constructs as accepted semantics
```

This avoids maintaining two resolvers.

## 10. Incremental recovery

Recovery structures change frequently during typing. Identity/invalidation must tolerate transitions:

```text
complete -> incomplete -> complete
```

After the final complete edit, incremental analysis must equal a clean parse/analyze of the final source. Recovery-only identities/evidence must be removed; do not join them forever into facts.

## 11. Diagnostics

Avoid cascades by attaching diagnostics to causes. If selector cannot be resolved because receiver expression is syntactically missing, do not emit both “missing expression” and dozens of “unknown member” errors. Preserve blocked cause:

```text
MemberResolution::Blocked(ReceiverRecoveryError)
```

The checker/LSP diagnostic adapter can suppress derivative messages while hover/completion may still show partial information.

## 12. Fuzzing and robustness

Semantic analysis over editor input should be fuzzed with parser outputs and random edit sequences. Properties:

- no panic/invalid memory behavior for any parser-produced tree;
- deterministic facts for the same recovered tree;
- source ranges remain within file/recovery conventions;
- recovery node cannot become a persisted/stable declaration accidentally;
- after repairs, stale recovery diagnostics/facts disappear;
- unrelated declarations retain identity/facts when malformed region changes.

## 13. Review questions

1. Is this uncertainty caused by invalid source, unavailable dependency, or genuinely dynamic semantics?
2. Which declarations remain trustworthy outside the malformed region?
3. Are recovery-generated IDs snapshot-local?
4. Can completion work from a selector prefix without claiming dispatch resolution?
5. Are duplicate declarations represented as ambiguity rather than arbitrary choice?
6. Will a repaired edit retract all recovery-only evidence?
7. Does batch checking reject recovery-synthetic semantics even if editor analysis used them?
