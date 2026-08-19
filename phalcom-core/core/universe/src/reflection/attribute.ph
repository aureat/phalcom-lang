// `Attribute`/`On`/`Tier` (M-ATTR-ROOT, attribute-classes.md §"Decision"/
// §"`@On`"/§"Bootstrap"): the reified-descriptor root every attribute class
// extends, the `@On` builtin attribute carrying legality + declared tier, and
// the tier marker classes. `@Name(args)` desugars, at the enclosing class's
// definition time, to `Name.new(args)` + `artifact.__attach(_a)`
// (`compiler::attributes`/`compiler::lib::class_decl`) — the constructed
// instance is retained on the decorated artifact's native `_attributes` store
// (`ClassObject`/`MethodObject`/`ModuleObject`, `primitive/attribute.rs`),
// reflectable via `Behavior#attributes`/`Method#attributes` below.
//
// **Forced deviation (positional-only args, filed to `docs/forge/DEFERRED.md`):**
// attribute-arg lists are positional-only — `parser.rs`'s
// `parse_attribute_arg_list` has no label grammar — so `On`'s own
// constructors are positional (`On.new(target)` / `On.new(target, tier)`),
// not the spec's labeled `tier:`/`inherited:` form. `inherited:` is dropped
// entirely for the same reason (v0.3 follow-on once labeled attribute args
// exist). A single `target` (not a list) is stored, since the parser also has
// no list-literal syntax yet (`core.ph` L306) to build a multi-target list at
// a use site — multi-target `@On` is deferred alongside labeled args.

// Root. Every attribute extends this — usage (retention, `resolves_to_
// attribute_class`'s `extends` chain walk) is fixed in
// `compiler::attributes` at this root.
class Attribute {}

// Builtin attribute carrying legality + declared tier (A-1) — recursion
// bottoms out here: `On` is itself an `Attribute` subclass, so `@On(...)` on
// an `Attribute` subclass's own header is retained/reflectable like any other
// attribute. `tier` is `None` for passive metadata (no hook selector may be
// implemented — `attr.undeclared_hook`) or one of the `Tier` marker classes
// below (`Install`/`Dispatch`/`Runtime` — `Compile`/`Layout` are reserved for
// compiler-native hooks only, `attr.compile_tier_reserved`).
class On is Attribute {
  _targets
  _tier

  @constructor
  new(_ target) { _targets = target; _tier = None }
  @constructor
  new(_ target, _ tier) { _targets = target; _tier = tier }

  targets { _targets }
  tier { _tier }
}

// The tier marker classes (attribute-classes.md: "same pattern Phalcom
// already uses for `Bool`'s `True`/`False`" — real singleton objects, not
// symbols; a bare class, used purely by identity, is the same pattern
// `True`/`False` already establish). `Compile`/`Layout` are reserved for
// compiler-native hooks only; `Install`/`Dispatch`/`Runtime` are the
// user-facing tiers (M-INSTALL/M-DISPATCH/M-RUNTIME, PLAN-DECORATORS.md).
class Tier {}
class Compile is Tier {}
class Layout is Tier {}
class Install is Tier {}
class Dispatch is Tier {}
class Runtime is Tier {}
