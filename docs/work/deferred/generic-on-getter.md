> Status: historical design note. Generic getters are implemented and current;
> this document records the former parser/signature gap and is no longer a
> statement of supported behavior. The authoritative rules and current
> evidence are in [SC-4.8](../../impl/semantic/semantic-completeness/sc-4.8/phalcom-type-system-completion-technical-spec.md)
> and the [implementation state](../../impl/semantic/typing-integration/typing-integration-implementation-state.md).

# Parser

There is no deep type-theoretic reason getters must be non-generic. The current prohibition is mainly because Phalcom’s getter model has not defined how getter-local type parameters are instantiated.

And in the current implementation, the restriction goes deeper than that parser check. `GetterDef` does not even carry `generic_parameters` or a `where_clause`; it only has the name, return annotation, body, etc.  The semantic signature builder then explicitly gives every getter `generics: None` and resolves its return type using only the enclosing declaration’s type parameters.

So deleting this:

```rust
if !generic_parameters.is_empty() {
    return Err(SyntaxError {
        kind: SyntaxErrorKind::Message(
            "generic parameters not permitted on getters".to_string()
        ),
        range: start..self.prev_end,
    });
}
```

would not actually implement generic getters. It would merely let generic syntax reach an AST node incapable of representing it.

The design question is more interesting.

A normal generic method has an obvious instantiation event:

```phalcom
convert<U>(_ value: T) -> U
```

and a call:

```phalcom
obj.convert(...)
```

provides argument types from which `U` might be inferred. If explicit generic application exists, there is another obvious source:

```phalcom
obj.convert<Int>(...)
```

A getter has no value arguments:

```phalcom
value<T> -> T
```

and is consumed simply as:

```phalcom
obj.value
```

So what is `T`?

There are several possible answers.

The first is contextual inference:

```phalcom
class Factory {
    empty<T> -> List<T> {
        List()
    }
}

const ints: List<Int> = factory.empty
```

Here the expected type gives:

```text
factory.empty : List<T>
expected      : List<Int>

therefore T = Int
```

That is perfectly legitimate bidirectional typing.

It also enables genuinely useful APIs:

```phalcom
identity<T> -> (T) -> T {
    |value| value
}

const f: (Int) -> Int = object.identity
```

or:

```phalcom
none<T> -> Option<T> {
    None
}

const missing: Option<User> = factory.none
```

Nothing is inherently unsound about these.

The problem appears when there is no contextual information:

```phalcom
const x = factory.empty
```

What is `x`?

You have three possible semantics.

One possibility is to reject the access because the getter-local type parameter cannot be inferred:

```text
cannot infer type parameter `T` of `Factory.empty<T>`

  factory.empty
          ^^^^^

`T` appears only in the result type and no expected type is available
```

That is a completely reasonable rule.

Another possibility is explicit type application:

```phalcom
factory.empty<Int>
```

or whatever generic-member application syntax Phalcom eventually establishes.

Then:

```text
factory.empty<Int> : List<Int>
```

This is also clean, but Phalcom must explicitly define the syntax and distinguish it from ordinary expressions involving `<` and `>`.

The third possibility is much more powerful: leave the value polymorphic.

Then:

```phalcom
const x = factory.empty
```

could conceptually have:

```text
x : forall T. List<T>
```

But that's a significantly different feature. Now getter evaluation is yielding something with a universally quantified type. That enters first-class polymorphism, value restriction questions, generic-value instantiation, reification, and so forth. I would not accidentally introduce that just to support `<T>` on getters.

So the conservative rule is:

> Generic getters are allowed, but every getter-local type parameter must be inferable from the expected type or explicitly supplied at the access site.

That fits Phalcom quite well.

For example:

```phalcom
class Collections {

    empty<T> -> List<T> {
        List()
    }
}
```

These should work:

```phalcom
const users: List<User> = collections.empty
const numbers: List<Int> = collections.empty
```

because the getter is instantiated separately:

```text
access #1:
    empty<T> -> List<T>
    expected List<User>
    T := User

access #2:
    empty<T> -> List<T>
    expected List<Int>
    T := Int
```

Whereas:

```phalcom
const x = collections.empty
```

should produce an inference diagnostic unless explicit type application or first-class polymorphism resolves it.

This becomes particularly valuable with constraints:

```phalcom
zero<T> -> T
    where T: Numeric
{
    T.zero
}
```

assuming the final bound syntax permits that.

Then:

```phalcom
const i: Int = numbers.zero
const f: Float = numbers.zero
```

could instantiate the same getter differently.

There is another useful pattern relevant to `Result`/`Option`:

```phalcom
defaultValue<T> -> Result<T, Error>
    where T: Default
{
    Result::Ok(T.default)
}
```

Again, the expected result type can determine `T`.

So I don't think Phalcom should permanently prohibit generic getters.

However, there is an important semantic distinction from methods: generic getter parameters usually cannot be inferred from input parameters because getters have none. Therefore result-directed inference becomes essential rather than optional.

That means supporting this well is coupled to Phalcom's bidirectional inference model.

The implementation should also treat a getter as a callable for generic-signature purposes. The current semantic code already conceptually groups methods/getters/setters/indexers under `CallableSyntaxRef`, but then special-cases getters:

```rust
CallableSyntaxRef::Getter(getter) => (
    None,
    Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
    annotation_fact(
        ctx,
        &declaration_resolver,
        getter.return_annotation.as_ref(),
        UnknownReason::UnannotatedDeclaration,
    ),
),
```



That `None` is where generic getter support is currently structurally shut off.

I would eventually change `GetterDef` from roughly:

```rust
pub struct GetterDef {
    pub name: String,
    pub return_annotation: Option<TypeAnnotation>,
    pub body: MemberBody,
    ...
}
```

to:

```rust
pub struct GetterDef {
    pub name: String,

    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,

    pub return_annotation: Option<TypeAnnotation>,
    pub body: MemberBody,

    ...
}
```

Then semantic construction becomes analogous to methods:

```rust
CallableSyntaxRef::Getter(getter) => {
    let generic_signature =
        if getter.generic_parameters.is_empty() {
            None
        } else {
            let mut diagnostics = Vec::new();

            let signature = resolve_generic_signature(
                ctx.store,
                ctx.declarations,
                &declaration_resolver,
                &ctx.current_module,
                TypeParameterOwner::Callable(callable.clone()),
                &getter.generic_parameters,
                getter.where_clause.as_ref(),
                &mut diagnostics,
            );

            ctx.publish_diagnostics(diagnostics);

            Some(signature)
        };

    let getter_type_parameters = generic_signature
        .as_ref()
        .map(...)
        .unwrap_or_default();

    let getter_resolver = ScopedTypeResolver {
        parent: &declaration_resolver,
        type_parameters: getter_type_parameters,
    };

    (
        generic_signature,
        Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
        annotation_fact(
            ctx,
            &getter_resolver,
            getter.return_annotation.as_ref(),
            UnknownReason::UnannotatedDeclaration,
        ),
    )
}
```

Then:

```phalcom
empty<T> -> List<T>
```

really has the semantic signature:

```text
empty : <T>() -> List<T>
```

The selector remains just the getter selector:

```text
#empty
```

because generic arguments are not part of runtime dispatch identity. They specialize the selected callable's type signature; they should not produce separate selectors.

That is important. You don't want:

```text
#empty<Int>
#empty<String>
```

to become different runtime methods. They are applications of the same generic declaration.

There is also a strong symmetry argument for allowing them. Phalcom currently regards methods, getters, setters, and indexers as callable semantic entities.  If generic parameters are fundamentally callable-owned type parameters, artificially saying “methods can own them but getters cannot” needs a semantic reason. The absence of value parameters isn't enough by itself.

I would still prohibit them on setters for now unless there's a compelling use case.

A setter:

```phalcom
value<T>=(put value: T)
```

would actually have an input from which `T` can be inferred, so it is technically easier than a getter. But generic setters create odd-looking property semantics and probably don't earn their complexity yet.

Generic subscript accessors are more defensible:

```phalcom
get<T>[_ key: Key] -> T
```

depending on Phalcom's indexing design.

So I would distinguish “possible” from “worth exposing”:

| Callable | Generic parameters | Recommendation |
|---|---:|---|
| methods | yes | definitely |
| getters | yes | yes |
| setters | possible | defer |
| index getters | yes | likely |
| index setters | possible | evaluate separately |
| constructors | yes | definitely |

For getters specifically, I would remove the language-level prohibition. But don't just remove that parser `if`. Implement it as an actual semantic feature.

The rule I would specify is:

> A getter may declare callable-local generic parameters and constraints. Getter type arguments are instantiated at each access. Type arguments may be inferred from the expected type of the getter expression or supplied explicitly where generic-member application syntax permits. If a required type parameter cannot be determined, the access is ill-typed.

That gives a useful and coherent feature without requiring first-class `forall` values.

And it leads to a nice property for Phalcom:

```phalcom
empty<T> -> List<T>
```

can behave naturally in contextual positions:

```phalcom
consumeUsers(collections.empty)
//                         ^ T inferred as User

const xs: List<Int> = collections.empty
//                                  ^ T inferred as Int
```

That's exactly the kind of result-directed generic inference a strong semantic analyzer should be able to handle.

So the short answer is: **we currently don't allow them because the AST and semantic signature implementation were designed with getter generics hardcoded out, probably as an early simplification. I don't see a compelling language-design reason to keep that restriction.** For Phalcom's intended type system, generic getters are worth supporting—provided their instantiation and inference rules are made explicit.

# Inference

The clean implementation is to make a getter participate in the ordinary generic callable-application pipeline as a zero-argument callable, and pass the contextual expected type into that pipeline.

Right now, Phalcom already has the infrastructure for this. `analyze_expression` receives an `ExpectedType`, `check_typed_expr` checks the resulting expression against it, and generic inference explicitly treats expected-result context as selection information rather than evidence.  The canonical callable application function also already accepts `expected: &ExpectedType`.

The missing pieces are:

1. getters cannot represent callable-local generics in the AST;
2. getter semantic signatures force `generics = None`;
3. `Expr::GetProperty` currently discards the expected type;
4. getter access needs to go through `apply_resolved_callable(...)` with zero arguments.

The resulting flow should be:

```text
const xs: List<Int> = factory.empty
                              │
                              │ expected = List<Int>
                              ▼
                    resolve getter `empty`
                              │
                              ▼
                    signature:
                    <T>() -> List<T>
                              │
                              ▼
                    instantiate T := ?T
                              │
                              ▼
                    return = List<?T>
                              │
                    contextual constraint
                              ▼
                    List<?T> <: List<Int>
                              │
                              ▼
                         ?T := Int
                              │
                              ▼
                    result = List<Int>
```

The important point is that the annotation `List<Int>` does not become proof that the getter returns `List<Int>`. It only selects `T = Int`. The declaration `<T>() -> List<T>` remains the authority for the result type. That distinction already matches Phalcom's generic-inference architecture: expected context is control/selection information, not value evidence.

First, extend `GetterDef`.

Currently `GetterDef` does not have generic parameters or a `where_clause`.  Change it conceptually from:

```rust
pub struct GetterDef {
    pub name: String,
    pub return_annotation: Option<TypeAnnotation>,
    pub body: MemberBody,
    pub is_static: bool,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    pub name_range: SourceRange,
}
```

to:

```rust
pub struct GetterDef {
    pub name: String,

    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,

    pub return_annotation: Option<TypeAnnotation>,
    pub body: MemberBody,
    pub is_static: bool,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    pub name_range: SourceRange,
}
```

Then remove the parser restriction:

```rust
if !generic_parameters.is_empty() {
    return Err(SyntaxError {
        kind: SyntaxErrorKind::Message(
            "generic parameters not permitted on getters".to_string()
        ),
        range: start..self.prev_end,
    });
}
```

The existing parser structure actually makes this easy. It already parses:

```text
name
generic parameters?
parameter list?
return annotation?
where clause?
body
```

Then it decides:

```text
parameter list present     -> Method
parameter list absent      -> Getter
```

So this:

```phalcom
empty<T> -> List<T> {
    List()
}
```

should simply produce:

```rust
GetterDef {
    name: "empty",
    generic_parameters: [T],
    where_clause: None,
    return_annotation: List<T>,
    ...
}
```

Do this in both `parse_class_member()` and `parse_enum_behavior_member()`.

Second, make getter semantic signatures genuinely generic.

The current semantic-signature builder treats methods correctly by resolving their callable-owned type parameters:

```rust
resolve_generic_signature(
    ...,
    TypeParameterOwner::Callable(callable.clone()),
    &method.generic_parameters,
    method.where_clause.as_ref(),
    ...
)
```

but getters are hardcoded to:

```rust
CallableSyntaxRef::Getter(getter) => (
    None,
    Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
    annotation_fact(
        ctx,
        &declaration_resolver,
        getter.return_annotation.as_ref(),
        UnknownReason::UnannotatedDeclaration,
    ),
),
```



That needs to become essentially the same generic setup used for a method, except the parameter array remains empty.

I would actually extract the duplicated generic setup into a helper:

```rust
fn callable_local_generic_environment(
    ctx: &mut CheckingContext<'_>,
    callable: &CallableId,
    declaration_resolver: &dyn TypeResolver,
    generic_parameters: &[GenericParameterSyntax],
    where_clause: Option<&WhereClauseSyntax>,
) -> (
    Option<GenericSignature>,
    ScopedTypeResolver<'_>,
) {
    ...
}
```

Conceptually, the getter branch becomes:

```rust
CallableSyntaxRef::Getter(getter) => {
    let generic_signature =
        if getter.generic_parameters.is_empty() {
            None
        } else {
            let mut diagnostics = Vec::new();

            let signature = resolve_generic_signature(
                ctx.store,
                ctx.declarations,
                &declaration_resolver,
                &ctx.current_module,
                TypeParameterOwner::Callable(callable.clone()),
                &getter.generic_parameters,
                getter.where_clause.as_ref(),
                &mut diagnostics,
            );

            ctx.publish_diagnostics(diagnostics);

            Some(signature)
        };

    let getter_type_parameters = generic_signature
        .as_ref()
        .map(|signature| {
            signature.parameters
                .iter()
                .map(|&parameter_id| {
                    let name =
                        ctx.store.type_parameter(parameter_id)
                            .name
                            .to_string();

                    let form =
                        ctx.store.parameter_form(parameter_id);

                    (name, form)
                })
                .collect()
        })
        .unwrap_or_default();

    let getter_resolver = ScopedTypeResolver {
        parent: &declaration_resolver,
        type_parameters: getter_type_parameters,
    };

    (
        generic_signature,

        Vec::<CallableParameterSemantic>::new()
            .into_boxed_slice(),

        annotation_fact(
            ctx,
            &getter_resolver,
            getter.return_annotation.as_ref(),
            UnknownReason::UnannotatedDeclaration,
        ),
    )
}
```

After that, this getter:

```phalcom
empty<T> -> List<T>
```

really has a semantic callable signature equivalent to:

```text
empty : <T>() -> List<T>
```

The selector remains:

```text
#empty
```

Generics must not alter runtime selector identity.

Third, and this is the critical inference change, pass `ExpectedType` through property access.

Currently the expression dispatcher does:

```rust
Expr::GetProperty(get) => synthesize_get_property(ctx, get),
```

even though `analyze_expression_inner` itself already receives:

```rust
expected: &ExpectedType
```

So the contextual information dies exactly there.

Change it to:

```rust
Expr::GetProperty(get) =>
    synthesize_get_property(ctx, get, expected),
```

and change the function:

```rust
fn synthesize_get_property(
    ctx: &mut CheckingContext<'_>,
    get: &GetPropertyExpr,
    expected: &ExpectedType,
) -> TypedExpression
```

The receiver should still be synthesized without context:

```rust
let recv_typed =
    analyze_expression(
        ctx,
        &get.object,
        &ExpectedType::None,
    );
```

because:

```phalcom
factory.empty
```

having an expected `List<Int>` tells us something about `empty`, not about `factory`.

Then resolve the getter normally.

The major architectural change is that once you obtain its resolved callable, don't directly take its declared return type. Turn it into an ordinary callable application:

```rust
let target =
    CallableApplicationTarget::from_dispatch(resolved);

apply_resolved_callable(
    ctx,
    &target,
    premise,
    &[],          // getter has zero arguments
    expected,     // critical
    get.range,
)
```

The repository already has examples of zero-argument getters being fed through `apply_resolved_callable` with an empty argument list.

This is the right abstraction:

```text
getter access

factory.empty

is semantically:

resolve #empty
+
apply callable with zero arguments
```

It is not a separate kind of type inference.

That will also eliminate the architectural problem already identified in the semantic-correctness work: `synthesize_get_property` currently bypasses the canonical call engine.

Fourth, let the generic application engine use the expected result to solve the getter variables.

Suppose the declaration is:

```phalcom
empty<T> -> List<T>
```

Generic application creates:

```text
T -> inference variable α
```

and transforms the formal return:

```text
List<T>
```

into:

```text
List<α>
```

There are no arguments, so there are no argument constraints.

Without context:

```phalcom
const x = factory.empty
```

you have:

```text
α unconstrained
result = List<α>
```

and inference must fail as underconstrained:

```text
cannot infer type parameter `T` for `empty`
```

Do not substitute `Dynamic`.

Do not default `T`.

Do not invent a fresh existential user-visible type.

But with:

```phalcom
const x: List<Int> = factory.empty
```

the expected type gives:

```text
formal specialized return:
    List<α>

expected:
    List<Int>
```

The generic solver gets a contextual selection relation:

```text
List<α> <: List<Int>
```

The exact decomposition depends on the variance of `List`, but eventually this should select:

```text
α = Int
```

and therefore:

```text
empty<Int> -> List<Int>
```

Phalcom's generic-inference specification already explicitly describes this stage:

```text
solve value + declared-generic constraints
        ↓
optionally add expected-result selection constraints
        ↓
solve contextual selection
        ↓
materialize justified result
```

and explicitly states that expected context is selection, not value evidence.

This is especially important for getters because they may have no argument-derived constraints at all.

There is one solver behavior you need to verify: an initially underconstrained solution cannot be treated as terminal before contextual-result selection gets a chance.

For a getter:

```text
<T>() -> List<T>
```

the first solver phase naturally says:

```text
T underconstrained
```

That is not yet a failure if an expected result exists.

The sequence must be:

```text
instantiate T as α

apply declaration constraints

analyze arguments
    -- none

base state:
    α unresolved

if expected exists:
    constrain formal return against expected
    solve again

only now classify:
    solved / underconstrained / conflicting
```

If the implementation currently returns immediately on `Underconstrained`, change that control flow.

Fifth, apply `where` constraints after contextual selection in the same generic session.

For:

```phalcom
default<T> -> T
    where T <: Serializable
{
    ...
}
```

and:

```phalcom
const x: User = factory.default
```

the process should be:

```text
return:
    α

context:
    User

select:
    α := User

declared constraint:
    α <: Serializable

validate:
    User <: Serializable
```

If `User` does not satisfy the constraint, inference fails.

The context must never be allowed to bypass the getter's declared bounds.

A particularly good example is:

```phalcom
zero<T> -> T
    where T <: Number
```

Then:

```phalcom
const x: Int = values.zero
```

works if:

```text
Int <: Number
```

while:

```phalcom
const x: String = values.zero
```

must fail at the generic constraint.

Sixth, distinguish context-selected specialization from proof authority.

This subtlety matters.

For:

```phalcom
empty<T> -> List<T>
```

there are no value arguments proving `T = Int`.

But that does not mean:

```phalcom
const xs: List<Int> = factory.empty
```

must become `Assumed`.

The declaration itself means:

```text
∀T. empty<T>() -> List<T>
```

The expected type only selects which legal member of that universally quantified family is being requested:

```text
T := Int
```

The authority for:

```text
List<Int>
```

comes from specializing the established declared signature.

So:

```text
expected List<Int>
        ↓ selection only
T := Int
        ↓
declared ∀T. List<T>
        ↓ specialization
established List<Int>
```

The expected type should not be attached as generic inference support.

This is exactly why the current generic-inference work separates:

```text
InferenceOutcome
InferenceProofState
TypeKnowledge
```

rather than equating “solver found a substitution” with “we have evidence for the expression.”

Seventh, make contextual inference work everywhere `ExpectedType` already flows.

Once property access actually consumes `expected`, all of these can work naturally:

```phalcom
const xs: List<Int> = factory.empty
```

```phalcom
consumeUsers(factory.empty)
```

if:

```phalcom
consumeUsers(_ users: List<User>)
```

and:

```phalcom
users() -> List<User> {
    factory.empty
}
```

assuming return-expression checking supplies the declared return type as the expected type.

You should not implement separate “binding getter inference”, “argument getter inference”, and “return getter inference”.

The architecture should be:

```text
context establishes ExpectedType
             ↓
analyze_expression(..., expected)
             ↓
property access preserves expected
             ↓
canonical callable application
             ↓
generic inference
```

That is the whole benefit of bidirectional typing.

Finally, I would test at least these cases:

```phalcom
class Factory {
    empty<T> -> List<T> {
        List()
    }
}
```

Success from binding context:

```phalcom
const xs: List<Int> = factory.empty
```

Expected:

```text
factory.empty : List<Int>
T = Int
```

Success from argument context:

```phalcom
consume(_ xs: List<String>) {}

consume(factory.empty)
```

Expected:

```text
T = String
```

Success from return context:

```phalcom
make() -> List<User> {
    factory.empty
}
```

Expected:

```text
T = User
```

No context:

```phalcom
const xs = factory.empty
```

Expected:

```text
generic inference underconstrained:
cannot infer `T`
```

Bound success:

```phalcom
default<T> -> T
    where T <: Number

const x: Int = factory.default
```

Bound failure:

```phalcom
const x: String = factory.default
```

Expected:

```text
String does not satisfy T <: Number
```

And importantly:

```phalcom
constant<T> -> Int {
    1
}

const x = factory.constant
```

should probably still be underconstrained, even though the result type is known. `T` may affect the implementation, reflection, reification, or constraints despite not occurring in the return type. Unless Phalcom explicitly adopts erasable irrelevant generic parameters, every callable-local generic parameter should be instantiated.

So the central implementation change is surprisingly small conceptually:

```rust
// Before
Expr::GetProperty(get) =>
    synthesize_get_property(ctx, get)

// After
Expr::GetProperty(get) =>
    synthesize_get_property(ctx, get, expected)
```

and inside getter resolution:

```rust
apply_resolved_callable(
    ctx,
    &target,
    premise,
    &[],
    expected,
    get.range,
)
```

Everything else is making `GetterDef` and `CallableSemanticSignature` accurately represent `<T>` and `where`.

That is the design I would use. Generic getter inference then stops being a special feature: it becomes the zero-argument case of Phalcom's existing bidirectional generic callable inference.
