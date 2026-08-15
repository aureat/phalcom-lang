# Hoare Logic and Contracts

## Hoare triple

```text
{P} C {Q}
```

means: if precondition `P` holds and command `C` terminates normally, postcondition `Q` holds afterward (partial correctness). Total correctness additionally proves termination.

## Assignment axiom

For simple variable assignment:

```text
{Q[e/x]} x := e {Q}
```

This substitution intuition underlies weakest-precondition generation.

## Sequence

```text
{P} C1 {R}
{R} C2 {Q}
----------------
{P} C1; C2 {Q}
```

## Conditional

Prove both paths:

```text
{P ∧ b} C1 {Q}
{P ∧ ¬b} C2 {Q}
--------------------
{P} if b then C1 else C2 {Q}
```

## Loop invariant

For:

```text
while b { C }
```

find invariant `I` such that:

1. precondition establishes `I`;
2. `I ∧ b` and body preserve `I`;
3. `I ∧ ¬b` implies desired postcondition.

Termination needs a decreasing variant/well-founded measure if total correctness is required.

## Phalcom contracts

Conceptual mapping:

```text
@requires P
@ensures Q
method m(...) { body }
```

Verification:

```text
Assume P at entry
Verify body normal exits imply Q
```

Class `@invariant I` can be checked at defined stable boundaries, commonly constructor/public-method boundaries, rather than literally after every instruction. Ratify exact semantics.

## Exceptions

A single `{P} C {Q}` only describes normal completion. Add exceptional postconditions or explicit outcome logic:

```text
Q_normal(result, state)
Q_throw(error, state)
```

## Frame conditions

Postconditions need to know which heap locations may change. A contract can include/modularly derive `modifies`/effect sets even if user syntax stays implicit initially.

## Contracts as documentation and proof interface

Contracts are module/callable summaries. They improve error locality and permit modular verification without inlining every callee.

---

## Deep treatment: contract semantics for a dynamic object language

### State model

For Phalcom, a Hoare state is richer than a map from local names to integers. A useful abstract state is:

```text
σ = (ρ, H, M, K, E)
```

where:

- `ρ` maps SSA/local semantic IDs to values;
- `H` is heap/object state;
- `M` represents module/global state relevant to the proof;
- `K` is control context such as home frames/handlers;
- `E` is any explicit effect/scheduler/world state needed by the verified fragment.

A Hoare triple `{P} C {Q}` is meaningful only relative to a defined transition semantics for `C`. The prover may abstract the state, but its abstraction must conservatively cover actual executions.

### Partial correctness with multiple outcomes

For a dynamic language, prefer a multi-postcondition view:

```text
{P} C {
  normal: Qn
  return: Qr
  throw: Qt
  nonlocal: Qnl
  suspend: Qs
}
```

A surface `@ensures` may intentionally constrain only normal method completion. If so, the verifier must prove it on every normal return/fall-through path but must not incorrectly demand it on throws. Class invariants may have a different policy: for example, they may need restoration before externally observable normal exits and perhaps before certain exceptional exits. That exact policy must be ratified, not assumed.

### Call rule

For a callee with contract:

```text
requires Pre_c
ensures  Post_c
modifies W_c
throws   T_c
```

a modular normal call rule is conceptually:

```text
prove Pre_c(actuals, H)
execute/havoc locations in W_c
assume Post_c(result, old(H), H') on normal successor
branch according to T_c on exceptional successors
```

The caller may use only the guaranteed contract, not arbitrary facts from one implementation body unless target/body identity is itself soundly fixed.

### Method verification

For method `m` with pre/post:

```text
@requires P
@ensures Q
m(args) { body }
```

verify:

```text
assume P at entry
snapshot old-state terms required by Q
execute body symbolically / generate wp
for each normal exit v,H': prove Q[v/result, H_entry/old, H']
```

Do not assume `Q` at entry. For recursion, recursive *calls* can use the candidate contract as induction hypothesis while the current invocation must still prove the contract from its body.

### `old` semantics

`old(e)` is not “evaluate `e` later using an old receiver.” It denotes the value of a well-defined pre-state expression. For a pure local expression:

```text
⟦old(e)⟧σ_entry = ⟦e⟧σ_entry
```

If `e` reads heap state, the value is based on `H_entry`. If `e` itself can invoke user code, mutate, throw, or depend on nondeterminism, the language must either reject it in `old`, define snapshot evaluation at entry, or require purity evidence. Current Phalcom runtime weaving has only a conservative purity floor; a static prover should not confuse that syntactic restriction with a theorem of purity.

### Class invariants

A class invariant `I(self)` needs explicit stable boundaries. Common choices:

```text
constructor successful exit establishes I
public method entry may assume I
public method normal exit must re-establish I
private helper may temporarily break I
```

But dynamic callbacks complicate this: if a method calls user code while the receiver invariant is broken, that callback may observe the invalid state or re-enter the object. Therefore a sound invariant discipline must define **visible states**. A stronger rule may require invariants before any call-out that can re-enter/observe the object.

This is a major interaction between object invariants, dynamic dispatch, callbacks, and FFI.

### Frame rule intuition

A simple frame principle is:

```text
{P} C {Q}
C does not modify resources described by R
------------------------------------------
{P ∧ R} C {Q ∧ R}
```

In a conventional heap model, “does not modify” is justified by effect/frame summaries and alias analysis. Without that premise, carrying `R` across a call is unsound.

### Example with aliasing

```phalcom
@ensures(other.value == old(other.value))
update(other) {
  self.helper(other)
}
```

If `helper` may mutate `other`, the postcondition cannot be proved by observing that `update` itself has no direct assignment to `other.value`. Effects are transitive through calls and aliases.

### Contract inheritance

If Phalcom chooses contracts on overridable members, dynamic dispatch needs a rule ensuring callers verified against a base contract remain safe under overrides. A conventional behavioral rule weakens preconditions and strengthens postconditions in overrides, with effects no broader than allowed by the public contract. Whether Phalcom adopts exact inheritance, explicit contract refinement, protocols, or another scheme is a language-design decision. The prover must enforce the ratified rule rather than infer one from syntax.

### Contracts versus runtime checks

A runtime check establishes a property only on executions that reach and pass it. A static proof establishes a universally quantified property over the modeled legal executions. The same surface contract can support both modes, but the evidence is different:

```text
runtime weave: execute predicate and raise on failure
static proof: turn predicate into assumptions/obligations according to role
```

For `@requires`, the callee may assume it during verification; callers must prove it. For `@ensures`, the callee must prove it; verified callers may assume it after normal return.

### Review questions

- Which outcomes does each contract constrain?
- At which boundaries may a class invariant be assumed or temporarily broken?
- Can a contract expression invoke user code?
- What does `old(e)` mean for mutable references and method sends?
- How do override contracts remain safe under dynamic dispatch?
- What frame/effect summary justifies preserving unrelated heap facts?
