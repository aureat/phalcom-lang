# Design Best Practices

> Generic wisdom layer of the `language-design` skill — principles with real exemplars/violations, not tutorials. Phalcom's committed choices: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** making a judgment call about consistency, ergonomics, surface syntax, evolution/compat, or "is this a good design".

## Contents
- Principle of least surprise / consistency
- Orthogonality & composability
- One kind of thing
- Small core, powerful library
- Worse-is-better vs the right thing
- Grow the language, don't complete it
- Make illegal states unrepresentable
- Explicit over implicit
- Errors are UX
- Evolution & backward compatibility
- One obvious way (TOOWTDI vs TIMTOWTDI)
- Spec-first & mechanized semantics
- Sequence: correctness → clarity → speed, safe by construction

## Principle of least surprise / consistency
A regular rule the user can extrapolate beats a fast path riddled with exceptions.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: everything-is-an-object, one dispatch rule | Smalltalk | any receiver learnable from one mental model; no carve-outs |
| violation: `typeof null === "object"` | JS | absence lies about its type; every guard needs a special case |
| violation: `==` coercion lattice | JS/PHP | `"" == 0`, `[] == ![]`; comparison is unlearnable, `===` retrofitted |

**Apply.** Before adding a rule, ask "can the user derive the next case from it?" — if it needs a footnote, the design leaks. → overlay: single label-encoded dispatch rule covers `foo`/`foo()`/`foo(_)` uniformly.

## Orthogonality & composability
Independent features should combine without inventing per-pair rules.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: closures × lists × recursion compose freely | Scheme | tiny spec, unbounded combinations, no feature-interaction table |
| violation: `const` × references × templates × overloading | C++ | combinatorial interaction blowup; nobody knows the whole language |
| violation: `async` colors every function it touches | JS/Rust | sync/async split forces duplicated APIs (function-color problem) |

**Apply.** Cost of a feature = the pairs it forces you to specify against every other feature; prefer one that drops out of what exists. → overlay: control flow is block-sends, so laziness/short-circuit fall out of the object model instead of new grammar.

## One kind of thing
Uniform substrate (one object model, one syntax) removes whole categories of rule.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: code is data (homoiconic) | Lisp | macros = ordinary list transforms; no separate macro grammar |
| exemplar: message-send is the only verb | Smalltalk | control flow, arithmetic, iteration all one mechanism |
| violation: `int` vs `Integer`, primitive vs object | Java | boxing rules, `null` unboxing NPEs, generics can't hold primitives |

**Apply.** Every "kind" you add doubles the rules that must cross-reference it; collapse value/reference and primitive/object splits if you can. → overlay: `Bool`/`Number`/absence are all real objects; no primitive tier beside the object model.

## Small core, powerful library
Put power in the library, not the grammar; a growable core outlives a kitchen-sink one.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: `ifTrue:`/`whileTrue:` are library messages | Smalltalk | control flow is user-extensible; core stays tiny |
| exemplar: ~21 keywords, semantics via tables/metatables | Lua | embeddable, one-person-comprehensible, easily retargeted |
| violation: syntax for every feature | C++/Perl | grammar unparseable without semantic feedback; no two impls agree |

**Apply.** Ask "can this be a method/library instead of syntax?" — reserve grammar for what genuinely can't be expressed in the object model. → overlay: `if`/`while`/`for` are keyword *sugar* over block sends, not primitive AST nodes.

## Worse-is-better vs the right thing
Gabriel: a simple thing that ships and spreads beats a perfect one that doesn't.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: simple, portable, "good enough" | C / Unix | spread everywhere; interface simplicity < implementation simplicity |
| violation: the maximally correct system | Lisp machines | out-competed; correctness didn't survive contact with distribution |

**Apply.** When correctness and shippability conflict, prefer a smaller design that lands and can grow; don't gold-plate a feature nobody can yet use. → overlay: optimizations (inline-cache population, NaN-boxing) deferred *behind* committed APIs — ship the shape, tune later.

## Grow the language, don't complete it
Steele: design the extension mechanisms so users add features without touching the core.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: hygienic macros, MOP, operator defn | Scheme/CLOS | users add control forms & class semantics without a compiler patch |
| exemplar: metaobject protocol is programmable | CLOS (AMOP) | dispatch itself is user-tunable; the language grows in userland |
| violation: fixed operator set, no reification | Go (early) | every abstraction gap waits on a language-team release |

**Apply.** Reserve the seams now (reflection, `perform`, message reification) even if unused, so extension later isn't a breaking redesign. → overlay: failed sends reify as `Message` + `doesNotUnderstand(_)`, giving proxies/DSLs/`respondsTo` for free.

## Make illegal states unrepresentable
Encode invariants in types/ADTs so bad states can't be built, rather than validated at runtime.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: sum types + no null; `Option`/`Result` | Rust/ML | "absent" and "error" are values the compiler forces you to handle |
| violation: null inhabits every reference type | Java/C/C# | Hoare's "billion-dollar mistake"; NPE anywhere, checkable nowhere |
| violation: stringly-typed enums | many | typos are runtime failures, not compile errors |

**Apply.** Prefer a constructor set that admits only valid values over a validator that rejects invalid ones after construction. → overlay: no surface `nil`; absence is `Option` `Some`/`None`, and truthiness on it is a *compile* error, not a runtime coercion.

## Explicit over implicit
Implicit coercions/conversions read as convenience and detonate as footguns.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: no implicit numeric/bool coercion | Python 3 | `"1" + 1` raises; the surprise happens at write time, loudly |
| violation: `+` overloaded across string/number | JS | `[] + {}`, `1 + "1"` → data-dependent nonsense |
| violation: `0 == "0" == false` chains non-transitively | PHP | equality is not an equivalence relation |

**Apply.** Make conversions a named call the reader can see; reserve overloading for operations that are genuinely the same across types. → overlay: conditions must be `Bool` (no truthiness); string interpolation desugars to an explicit `toString` + concat.

## Errors are UX
Diagnostic quality is a first-class language feature, not a postscript.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: spans + "did you mean" + fix-its | Rust/Elm | errors teach; adoption cites the compiler as a feature |
| violation: template instantiation error walls | C++ | one mistake → pages of noise; users flee generic code |
| violation: `undefined is not a function` | JS | no span, no cause; debugging is archaeology |

**Apply.** Budget for spans, multiple-error recovery, and suggestions from day one — a parser that dies on the first error is a worse product than one that recovers. → overlay: hand-written parser with panic-mode recovery emits *multiple* diagnostics per run; newlines are real tokens for precise spans.

## Evolution & backward compatibility
Plan versioning and deprecation up front; never break the world silently.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: editions, same compiler, opt-in per crate | Rust | breaking syntax changes without splitting the ecosystem |
| exemplar: `__future__`, long deprecation windows | Python (later) | opt-in new semantics before they become default |
| violation: incompatible 2→3 with no bridge era | Python | decade-long split; libraries stranded, users stuck on EOL runtime |

**Apply.** Decide the compatibility mechanism *before* the first breaking need arises; a silent semantics change is worse than a loud, versioned one. → overlay: absence/dispatch/selector spelling are locked in ADRs so later changes are amendments with a paper trail, not silent drift.

## One obvious way (TOOWTDI vs TIMTOWTDI)
Fewer redundant ways → more consistent ecosystems; more ways → dialect fragmentation.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: "one obvious way to do it" | Python (Zen) | readable across teams; idioms converge |
| violation: "more than one way to do it" | Perl | every codebase a private dialect; onboarding cost compounds |
| tradeoff: many equivalent forms | Ruby/Scala | expressive for authors, higher cognitive load for readers |

**Apply.** Redundant surface syntax has an ecosystem cost paid by every future reader; add a second spelling only when it buys real expressiveness. → overlay: one canonical **comma** selector form (`move(_,to,duration)`) shared by compiler and every runtime builder — no competing spellings.

## Spec-first & mechanized semantics
A written spec plus conformance tests prevents implementation-defined drift.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: formal spec + reference test suite | WebAssembly | independent impls agree by construction; portable semantics |
| exemplar: fully formalized static + dynamic semantics | Standard ML | proofs about the language, not just about programs |
| violation: "the implementation is the spec" | early PHP/Perl | behavior = whatever the C code did; bugs became load-bearing |

**Apply.** Write the semantics down before optimizing the implementation; treat the spec + golden corpus as the oracle the VM must match. → overlay: `docs/spec/*` + ADRs are the source of truth; `verify_invariants()` and a golden `.ph` corpus enforce them against the runtime.

## Sequence: correctness → clarity → speed, safe by construction
Order the work correctness-first; build safety in rather than bolting it on — but don't foreclose optimization.
| Exemplar / Violation | Lang | Consequence |
|---|---|---|
| exemplar: safe by default, `unsafe` opt-in & audited | Rust | memory safety is the floor; escape hatches are localized and greppable |
| violation: performance-led, safety retrofitted | C | UB everywhere; decades of CVEs from the original speed-first bias |
| principle: shape hot paths early, tune later | many VMs | leaving room for inline caches/JIT beats premature micro-tuning |

**Apply.** Get it correct and clear, keep the representation optimizable, and make the safe path the default one; see [performance.md](performance.md) and [security.md](security.md) for the axis-level detail. → overlay: handle/arena heap removes the borrow-panic surface *by construction* (no `Rc<RefCell>`), while inline-cache/NaN-box optimizations sit deferred behind stable APIs.

## Implementation hygiene (for a Rust bytecode VM)
- Full rustdoc on public items; `cargo doc` clean — see project `rust-documentation-guidelines`.
- Idiom & ownership discipline — invoke the `rust-best-practices` skill; borrow-model soundness over cloning.
- Test substrate — golden `.ph` corpus + snapshot + property/fuzz + miri lanes; invoke `rust-testing`, `rust-sanitizers-miri`, `fuzzing-*` skills. Do NOT duplicate their content — point to them.
