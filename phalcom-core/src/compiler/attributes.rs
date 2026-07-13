use std::collections::HashMap;
use phalcom_ast::ast::{
    AssignmentExpr, Attribute, BinaryExpr, BinaryOp, ClassDef, ClassMember, ConstructDef, Expr, FieldDef, GetPropertyExpr, GetterDef, MethodDef,
    ParameterDef, ReturnStatement, SetterDef, SuperclassRef, VariantDef,
};
use crate::compiler::lib::CompilerError;
use crate::interner::Symbol;
use crate::method::{encode_selector, SignatureKind};

/// The active contract-stripping mode (U-ANNOT-CONTRACTS plan §3.6,
/// `annotations-contract-semantics.md`'s stripping table).
///
/// Selected on the CLI (`--release`/`--unchecked`, default `Debug`) and
/// threaded through `Compiler` (`compiler::lib`) into every
/// [`ExpandCtx`] built for a class's attribute expansion. Guard weaving
/// (`@requires`/`@ensures`/`@invariant`'s runtime checks) and reflectable
/// metadata (`MethodObject::contracts`) are stripped independently along
/// two separate axes — see [`ExpandCtx::strip_metadata`] for the second
/// axis. The verbatim table:
///
/// | Mode | `@requires` guard | `@ensures` guard | `@invariant` guard | Metadata (default) |
/// |------|------|------|------|------|
/// | `Debug` (default) | woven | woven | woven | retained |
/// | `Release` | woven | stripped | stripped | retained (opt out `--strip-contract-metadata`) |
/// | `Unchecked` | stripped | stripped | stripped | stripped by default |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// Every contract guard is woven; metadata is retained. The default.
    Debug,
    /// `@ensures`/`@invariant` guards are stripped (no-op weave); `@requires`
    /// stays woven. Metadata is retained by default (opt out with
    /// `--strip-contract-metadata`).
    Release,
    /// All three guards are stripped. Metadata is stripped by default.
    Unchecked,
}

/// Shared state threaded through one class's attribute expansion
/// ([`expand_class_attributes`]).
pub struct ExpandCtx<'a> {
    /// The compiler's symbol interner, needed to intern synthesized names
    /// (e.g. `__old_0`) and reflectable-metadata selector symbols.
    pub interner: &'a mut crate::interner::Interner,
    /// The active [`CompileMode`] — governs whether each expander's guard
    /// weave is a no-op (§3.6's first axis).
    pub compile_mode: CompileMode,
    /// Whether reflectable contract metadata
    /// (`MethodObject::contracts`, plan §3.5) should be skipped for this
    /// compile (§3.6's second, independent axis). Derived from
    /// [`Self::compile_mode`] plus the `--strip-contract-metadata` CLI
    /// override — see `Compiler::new`'s call site — never coupled to guard
    /// stripping directly.
    pub strip_metadata: bool,
    /// The compile-time superclass-edge map
    /// ([`crate::vm::VM::class_parents`]), read-only here — used to walk an
    /// attribute name's `extends` chain up to the `Attribute` root (M-ATTR-ROOT,
    /// `attribute-classes.md` §"Decision") so an unrecognized attribute name
    /// that *does* name a user `Attribute` subclass is retained silently
    /// instead of raising `attr.unknown` (see `resolves_to_attribute_class`
    /// in this module).
    pub class_parents: &'a HashMap<Symbol, Symbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    Class,
    Method,
    Getter,
    Setter,
    Construct,
    /// A declared [`phalcom_ast::ast::ClassMember::Field`] (U-ANNOT-LAYOUT
    /// §3.1) — the legal target for the `@get`/`@set` derive tier (§3.2).
    Field,
    /// A declared [`phalcom_ast::ast::ClassMember::Variant`] (U-ANNOT-LAYOUT
    /// §3.4, `annotations-data.md` §"`@variant`") — the sole legal target for
    /// `@variant`. Distinguishes a variant arm from an ordinary `Method`, so
    /// `@variant` on a plain method is `attr.illegal_target`
    /// (`annotations-legality-grammar.md`'s legality table: "`@variant` |
    /// Class-nested variant decl | plain method").
    Variant,
}

pub trait AttributeExpander {
    fn legal_targets(&self) -> &'static [Target];
    fn expand(
        &self,
        ctx: &mut ExpandCtx,
        member: &mut ClassMember,
        args: &[Expr],
    ) -> Result<(), CompilerError>;
}

use phalcom_ast::ast::{Argument, MethodCallExpr, Statement, BlockExpr, LetBinding, BindingKind, Pattern};
use phalcom_common::range::SourceRange;

fn is_pure_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Number { .. } | Expr::String { .. } | Expr::Boolean { .. } | Expr::Var { .. } | Expr::Field { .. } | Expr::SelfVar { .. } | Expr::SuperVar { .. } => true,
        Expr::Assignment(_) | Expr::SetProperty(_) | Expr::SetIndex(_) => false,
        Expr::Unary(u) => is_pure_expr(&u.expr),
        Expr::Binary(b) => is_pure_expr(&b.left) && is_pure_expr(&b.right),
        Expr::MethodCall(m) => {
            // impure list: mutable sends like add, remove, put, or setter name=
            let impure_names = ["add", "remove", "put"];
            if impure_names.contains(&m.method.as_str()) || m.method.ends_with('=') {
                return false;
            }
            is_pure_expr(&m.object) && m.args.iter().all(|a| is_pure_expr(&a.expr))
        }
        Expr::GetProperty(g) => is_pure_expr(&g.object),
        Expr::Index(i) => is_pure_expr(&i.object) && is_pure_expr(&i.index),
        Expr::Block(b) => b.body.iter().all(|s| {
            match s {
                Statement::Expr { expr, .. } => is_pure_expr(expr),
                Statement::Let(l) => l.value.as_ref().map_or(true, |v| is_pure_expr(v)),
                Statement::Return(r) => r.value.as_ref().map_or(true, |v| is_pure_expr(v)),
                _ => false,
            }
        }),
        _ => true,
    }
}

/// Recognizes the `old(sub)` pseudo-selector call shape. `old` is a reserved
/// name meaningful only inside `@ensures` (annotations-contracts.md); it is
/// not an ordinary method — the parser has no bare-call grammar (calls
/// always need an explicit receiver), so `old(sub)` parses as an *invocation
/// of the variable* `old` — `Expr::MethodCall{ object: Var("old"), method:
/// "call", args: [sub] }` (`parse_call`'s bare-`(args)` postfix, mirroring
/// how a `Function`/`Block` value stored in a var is called). Matching on
/// this exact shape, not a method literally named `old`, is what lets
/// `old(...)` compile at all — `old` is never a real binding.
fn as_old_call(m: &MethodCallExpr) -> bool {
    m.method == "call" && m.args.len() == 1 && matches!(&m.object, Expr::Var { value, .. } if value == "old")
}

fn contains_old_call(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if as_old_call(m) {
                true
            } else {
                contains_old_call(&m.object) || m.args.iter().any(|a| contains_old_call(&a.expr))
            }
        }
        Expr::Unary(u) => contains_old_call(&u.expr),
        Expr::Binary(b) => contains_old_call(&b.left) || contains_old_call(&b.right),
        Expr::Index(i) => contains_old_call(&i.object) || contains_old_call(&i.index),
        Expr::GetProperty(g) => contains_old_call(&g.object),
        _ => false,
    }
}

fn validate_purity(args: &[Expr]) -> Result<(), CompilerError> {
    for arg in args {
        if !is_pure_expr(arg) {
            return Err(CompilerError::Message("contract.impure_predicate: predicate contains mutating or side-effecting operations".to_string()));
        }
    }
    Ok(())
}

pub struct RequiresExpander;
impl AttributeExpander for RequiresExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Method, Target::Getter, Target::Setter]
    }

    fn expand(
        &self,
        ctx: &mut ExpandCtx,
        member: &mut ClassMember,
        args: &[Expr],
    ) -> Result<(), CompilerError> {
        validate_purity(args)?;

        // §3.6 axis 1 (guard stripping): `@requires` is woven in `Debug`
        // and `Release` (row 1/2 of the table) — only `Unchecked` strips it.
        // Purity validation above still runs regardless of mode: it is a
        // compile-time soundness floor, not a runtime guard, so stripping
        // the *guard* must not also silently skip catching an impure
        // predicate (implementer judgment call, flagged in the return
        // contract — the plan does not state this explicitly).
        if ctx.compile_mode == CompileMode::Unchecked {
            for arg in args {
                if contains_old_call(arg) {
                    return Err(CompilerError::Message("contract.old_in_precondition: requires cannot contain old() expressions".to_string()));
                }
            }
            return Ok(());
        }

        let method_name = match member {
            ClassMember::Method(m) => m.name.clone(),
            ClassMember::Getter(g) => g.name.clone(),
            ClassMember::Setter(s) => s.name.clone(),
            _ => unreachable!(),
        };

        // requires checks go directly into the prologue
        let body = match member {
            ClassMember::Method(m) => &mut m.body,
            ClassMember::Getter(g) => &mut g.body,
            ClassMember::Setter(s) => &mut s.body,
            _ => unreachable!(),
        };

        // Weave requires in declaration order
        let mut new_prologue = Vec::new();
        for arg in args {
            if contains_old_call(arg) {
                return Err(CompilerError::Message("contract.old_in_precondition: requires cannot contain old() expressions".to_string()));
            }

            let range = arg.range();
            let err_msg = format!("Precondition failed for method `{}`: {}", method_name, arg.range().start);
            new_prologue.push(build_check_stmt(arg.clone(), "PreconditionError", err_msg, range));
        }

        new_prologue.append(body);
        *body = new_prologue;

        Ok(())
    }
}

pub struct EnsuresExpander;
impl AttributeExpander for EnsuresExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Method, Target::Getter, Target::Setter]
    }

    fn expand(
        &self,
        ctx: &mut ExpandCtx,
        member: &mut ClassMember,
        args: &[Expr],
    ) -> Result<(), CompilerError> {
        validate_purity(args)?;

        // §3.6 axis 1 (guard stripping): `@ensures` is woven only in
        // `Debug` (row 1) — both `Release` and `Unchecked` strip it
        // (rows 2/3). Purity validation still runs unconditionally, same
        // rationale as `RequiresExpander`.
        if ctx.compile_mode != CompileMode::Debug {
            return Ok(());
        }

        let method_name = match member {
            ClassMember::Method(m) => m.name.clone(),
            ClassMember::Getter(g) => g.name.clone(),
            ClassMember::Setter(s) => s.name.clone(),
            _ => unreachable!(),
        };

        let body = match member {
            ClassMember::Method(m) => &mut m.body,
            ClassMember::Getter(g) => &mut g.body,
            ClassMember::Setter(s) => &mut s.body,
            _ => unreachable!(),
        };

        // 1. Hoist old(...) calls to lets
        let mut old_lets = Vec::new();
        let mut new_args = Vec::new();
        
        for arg in args {
            let mut rewritten_arg = arg.clone();
            rewrite_old_calls(&mut rewritten_arg, &mut old_lets, &mut new_args, ctx)?;
            new_args.push(rewritten_arg);
        }

        // Prepend old lets to the body
        let mut new_body = old_lets;

        // If body has early returns, we rewrite them or we can wrap the whole body in a block call
        // Wait, ensures must verify on *all* exit paths. Wrapping in a block or rewriting Statement::Return.
        // Let's rewrite returns recursively in the body.
        rewrite_returns(body, &new_args, &method_name);

        new_body.append(body);

        // Append ensures checks at the end if the body doesn't end with an explicit Return
        let last_is_return = body.last().map_or(false, |s| matches!(s, Statement::Return(_)));
        if !last_is_return {
            // Bind last expression to __result if it's Statement::Expr
            let mut result_stmt = None;
            if let Some(Statement::Expr { expr, range }) = new_body.last().cloned() {
                new_body.pop();
                let range = expr.range();
                new_body.push(Statement::Let(LetBinding {
                    kind: BindingKind::Let,
                    pattern: Pattern::Name { name: "__result".to_string(), range },
                    value: Some(expr),
                    range,
                }));
                result_stmt = Some(Statement::Expr { expr: Expr::Var { value: "__result".to_string(), range }, range });
            }

            for arg in &new_args {
                let range = arg.range();
                let err_msg = format!("Postcondition failed for method `{}`: {}", method_name, arg.range().start);
                new_body.push(build_check_stmt(arg.clone(), "PostconditionError", err_msg, range));
            }

            if let Some(stmt) = result_stmt {
                new_body.push(stmt);
            }
        }

        *body = new_body;
        Ok(())
    }
}

fn rewrite_old_calls(
    expr: &mut Expr,
    old_lets: &mut Vec<Statement>,
    new_args: &mut Vec<Expr>,
    ctx: &mut ExpandCtx,
) -> Result<(), CompilerError> {
    match expr {
        Expr::MethodCall(m) if as_old_call(m) => {
            let mut inner = m.args[0].expr.clone();
            rewrite_old_calls(&mut inner, old_lets, new_args, ctx)?;

            // contract.old_on_mutable: capturing the whole receiver aliases
            // the live, mutable object — `old(self)`/`old(super)` can never
            // observe pre-mutation state, since the snapshot is the same
            // reference the method goes on to mutate (annotations-contracts.md
            // "old(...) ⊗ mutable aliasing"). Anything else (a field read, a
            // getter call, an arithmetic expression) is accepted: Phalcom is
            // dynamically typed with no flow analysis (the same floor-not-proof
            // limitation as the truthiness ban, ADR-0021), so whether a given
            // sub-expression's *runtime value* is itself a mutable heap
            // reference can't be checked here — only the unambiguous
            // whole-receiver case is.
            match &inner {
                Expr::SelfVar { .. } | Expr::SuperVar { .. } => {
                    return Err(CompilerError::Message("contract.old_on_mutable: old() operand must not be the whole receiver (aliases the live, mutable object)".to_string()));
                }
                _ => {}
            }

            let var_name = format!("__old_{}", old_lets.len());
            let range = m.range;
            old_lets.push(Statement::Let(LetBinding {
                kind: BindingKind::Let,
                pattern: Pattern::Name { name: var_name.clone(), range },
                value: Some(inner),
                range,
            }));

            *expr = Expr::Var { value: var_name, range };
        }
        Expr::MethodCall(m) => {
            rewrite_old_calls(&mut m.object, old_lets, new_args, ctx)?;
            for arg in &mut m.args {
                rewrite_old_calls(&mut arg.expr, old_lets, new_args, ctx)?;
            }
        }
        Expr::Unary(u) => rewrite_old_calls(&mut u.expr, old_lets, new_args, ctx)?,
        Expr::Binary(b) => {
            rewrite_old_calls(&mut b.left, old_lets, new_args, ctx)?;
            rewrite_old_calls(&mut b.right, old_lets, new_args, ctx)?;
        }
        Expr::Index(i) => {
            rewrite_old_calls(&mut i.object, old_lets, new_args, ctx)?;
            rewrite_old_calls(&mut i.index, old_lets, new_args, ctx)?;
        }
        Expr::GetProperty(g) => rewrite_old_calls(&mut g.object, old_lets, new_args, ctx)?,
        _ => {}
    }
    Ok(())
}

fn rewrite_returns(body: &mut Vec<Statement>, ensures_args: &[Expr], method_name: &str) {
    let mut rewritten = Vec::new();
    for stmt in std::mem::take(body) {
        match stmt {
            Statement::Return(ret) => {
                let range = ret.range;
                let value_expr = ret.value.unwrap_or(Expr::Var { value: "None".to_string(), range });
                let mut local_block = Vec::new();
                
                // let __result = value_expr
                local_block.push(Statement::Let(LetBinding {
                    kind: BindingKind::Let,
                    pattern: Pattern::Name { name: "__result".to_string(), range },
                    value: Some(value_expr),
                    range,
                }));

                for arg in ensures_args {
                    let arg_range = arg.range();
                    let err_msg = format!("Postcondition failed for method `{}`: {}", method_name, arg.range().start);
                    local_block.push(build_check_stmt(arg.clone(), "PostconditionError", err_msg, arg_range));
                }

                // return __result
                local_block.push(Statement::Return(phalcom_ast::ast::ReturnStatement {
                    value: Some(Expr::Var { value: "__result".to_string(), range }),
                    range,
                }));

                rewritten.push(Statement::Expr {
                    expr: Expr::Block(Box::new(BlockExpr {
                        params: Vec::new(),
                        body: local_block,
                        expr_body: false,
                        range,
                    })),
                    range,
                });
            }
            Statement::Expr { expr: Expr::Block(mut b), range } => {
                rewrite_returns(&mut b.body, ensures_args, method_name);
                rewritten.push(Statement::Expr { expr: Expr::Block(b), range });
            }
            _ => rewritten.push(stmt),
        }
    }
    *body = rewritten;
}

/// Registry entry for `@invariant` as a class-target attribute.
///
/// This impl's own [`AttributeExpander::expand`] is a deliberate no-op: the
/// registry/legality-check machinery in [`expand_class_attributes`] requires
/// every registered name to have an `AttributeExpander` row so `attr.unknown`
/// fires correctly, but `@invariant`'s actual weave — folding
/// [`ClassDef::invariants`] into every public non-static, non-constructor
/// member's prologue/epilogue (ADR-0052 Fix 1) — needs the *whole class*
/// (every member, not just the one this attribute happened to be attached
/// to), so it runs once from [`expand_class_attributes`] itself via
/// `weave_invariant_checks`, not per-attribute through this trait method.
pub struct InvariantExpander;
impl AttributeExpander for InvariantExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Class] // Note: standalone invariant is parsed directly to class.invariants, but @invariant class-decorator target is legal too
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@construct` as a class-target attribute (U-ANNOT-LAYOUT
/// §3.3, `annotations-construct.md`).
///
/// Like [`InvariantExpander`], this impl's own [`AttributeExpander::expand`]
/// is a deliberate no-op: `@construct` needs the *whole class* (every
/// declared [`phalcom_ast::ast::ClassMember::Field`], read in declaration
/// order, not just the one member this attribute happened to be attached to
/// — and `@construct` attaches to the class header, not any one member) so
/// its derive runs once from [`expand_class_attributes`] itself via
/// `derive_construct`, not per-attribute through this trait method. The
/// registry row still exists so the ordinary `attr.unknown`/
/// `attr.illegal_target` legality checks fire correctly for `@construct`
/// like any other registered name.
pub struct ConstructExpander;
impl AttributeExpander for ConstructExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Class]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@get` (U-ANNOT-LAYOUT §3.2, `annotations-construct.md`
/// §"@get/@set"). Same deliberate-no-op shape as [`ConstructExpander`]: the
/// derive appends a sibling [`ClassMember::Getter`] next to the `Field` this
/// attribute is attached to, which `AttributeExpander::expand`'s
/// mutate-in-place signature cannot do — the real derive runs once from
/// [`expand_class_attributes`] via `derive_accessors`. This row only exists
/// so `attr.unknown`/`attr.illegal_target` fire correctly for `@get`.
pub struct GetExpander;
impl AttributeExpander for GetExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Field]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@set` — see [`GetExpander`].
pub struct SetExpander;
impl AttributeExpander for SetExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Field]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@data` as a class-target attribute (U-ANNOT-LAYOUT
/// §3.4, `annotations-data.md` §"`@data`"). Same deliberate-no-op shape as
/// [`ConstructExpander`]: `@data`'s real derive (constructor reuse/generation
/// plus `==`/`hash`/`toString`/`with(...)`) needs the whole class's declared
/// fields and existing members, so it runs once from
/// [`expand_class_attributes`] via `derive_data`, not per-attribute through
/// this trait method. The registry row still exists so `attr.unknown`/
/// `attr.illegal_target` fire correctly for `@data`.
pub struct DataExpander;
impl AttributeExpander for DataExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Class]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@sealed` as a class-target attribute (U-ANNOT-LAYOUT
/// §3.4, `annotations-data.md` §"`@sealed`"). Same deliberate-no-op shape as
/// [`DataExpander`]: `@sealed`'s real work — recording the class as sealed in
/// [`crate::vm::VM::sealed_classes`] and rejecting a same-selector cross-unit
/// subclass at *that subclass's* compile time — happens in
/// `compiler::lib::class_decl::Compiler::compile_class`, not here (it needs
/// the compiling `Compiler`'s own module handle, which `AttributeExpander`'s
/// signature has no access to). The registry row still exists so
/// `attr.unknown`/`attr.illegal_target` fire correctly for `@sealed`.
pub struct SealedExpander;
impl AttributeExpander for SealedExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Class]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@variant` (U-ANNOT-LAYOUT §3.4, `annotations-data.md`
/// §"`@variant`"). Same deliberate-no-op shape as [`GetExpander`]: `@variant`'s
/// real derive — stripping the [`phalcom_ast::ast::ClassMember::Variant`] arm
/// and generating a sibling top-level class plus the enclosing class's
/// `match(...)` visitor — needs the whole class's `@variant` arm set and
/// returns *sibling statements*, a shape [`AttributeExpander::expand`] cannot
/// produce, so it runs once from [`expand_class_attributes`] via
/// `expand_variants`. This row only exists so `attr.unknown`/
/// `attr.illegal_target` fire correctly for `@variant` (e.g. `@variant` on an
/// ordinary method, per the legality table's "plain method" illegal-target
/// row).
pub struct VariantExpander;
impl AttributeExpander for VariantExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Variant]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

/// Registry entry for `@On` — the builtin attribute carrying legality + tier
/// for a user `Attribute` subclass's own header (M-ATTR-ROOT,
/// `attribute-classes.md` §"`@On`"). Like [`InvariantExpander`]/
/// [`ConstructExpander`], this impl's own [`AttributeExpander::expand`] is a
/// deliberate no-op — `@On`'s actual work (tier-vs-hook validation, forced to
/// positional args by the parser's arg-list grammar, see
/// `docs/forge/DEFERRED.md`) needs the *whole class* (its declared members,
/// to check for a matching reserved hook selector), so it runs once from
/// [`expand_class_attributes`] itself via `validate_attribute_class`, not
/// per-attribute through this trait method. The registry row still exists so
/// `@On` on any non-`Class` target raises the ordinary `attr.illegal_target`.
pub struct OnExpander;
impl AttributeExpander for OnExpander {
    fn legal_targets(&self) -> &'static [Target] {
        &[Target::Class]
    }

    fn expand(
        &self,
        _ctx: &mut ExpandCtx,
        _member: &mut ClassMember,
        _args: &[Expr],
    ) -> Result<(), CompilerError> {
        Ok(())
    }
}

pub struct AttributeRegistry {
    expanders: HashMap<String, Box<dyn AttributeExpander + Send + Sync>>,
}

impl AttributeRegistry {
    pub fn new() -> Self {
        let mut expanders: HashMap<String, Box<dyn AttributeExpander + Send + Sync>> = HashMap::new();
        expanders.insert("requires".to_string(), Box::new(RequiresExpander));
        expanders.insert("ensures".to_string(), Box::new(EnsuresExpander));
        expanders.insert("invariant".to_string(), Box::new(InvariantExpander));
        expanders.insert("construct".to_string(), Box::new(ConstructExpander));
        expanders.insert("get".to_string(), Box::new(GetExpander));
        expanders.insert("set".to_string(), Box::new(SetExpander));
        expanders.insert("data".to_string(), Box::new(DataExpander));
        expanders.insert("sealed".to_string(), Box::new(SealedExpander));
        expanders.insert("variant".to_string(), Box::new(VariantExpander));
        expanders.insert("On".to_string(), Box::new(OnExpander));
        Self { expanders }
    }

    pub fn get(&self, name: &str) -> Option<&(dyn AttributeExpander + Send + Sync)> {
        self.expanders.get(name).map(|boxed| &**boxed)
    }
}

/// Strips a single leading underscore from a declared field's name
/// (`"_x"` → `"x"`), the naming convention `@construct`'s generated
/// parameter/label uses (`annotations-construct.md`'s worked example). A
/// field name with no leading underscore is returned unchanged.
fn strip_leading_underscore(name: &str) -> String {
    name.strip_prefix('_').unwrap_or(name).to_string()
}

/// Builds `_field = value` — an ordinary field assignment statement, the
/// shape every `@construct`-derived constructor body statement takes.
fn field_assign_stmt(field_name: &str, value: Expr, range: SourceRange) -> Statement {
    Statement::Expr {
        expr: Expr::Assignment(Box::new(AssignmentExpr {
            name: Box::new(Expr::Field { value: field_name.to_string(), range }),
            value,
            range,
        })),
        range,
    }
}

/// Derives a `construct new(...)` member from `class`'s own declared
/// [`FieldDef`]s, in declaration order, and appends it to `class.members`
/// (U-ANNOT-LAYOUT §3.3, `annotations-construct.md` "The derive").
///
/// **Own-fields-only case**: this does not chain a superclass constructor
/// (the inheritance-aware F-fix, `annotations-construct-inheritance.md`, is a
/// separate follow-on build-order step, not implemented here) — a `@construct`
/// class that `extends` a superclass with its own constructor gets only its
/// own fields' assignments, never a `super.new(...)` call.
///
/// A field carrying a `default` expression is omitted from the generated
/// parameter list (`annotations-construct-inheritance.md`: "supply-and-
/// default is mutually exclusive per field"); every defaulted field's default
/// expression is assigned first, in declaration order, *before* the
/// labeled-parameter assignments — so a later field's default (if any) still
/// observes prior defaults already applied.
///
/// Emits a real [`ConstructDef`]/[`ClassMember::Construct`] — not a
/// `MethodDef` (`annotations-construct.md`'s "Prerequisite 2 correction":
/// `MethodDef` has no `is_constructor` field; emitting a plain method named
/// `new` would silently produce a non-constructor). The emitted node is
/// compiled through the exact same path a hand-written `construct` member is
/// (`compiler::lib::class_decl`'s per-member loop), so it gets the same
/// selector encoding, `constructor_aliases` call-site registration, and
/// `has_new_construct` bare-allocator-guard interaction a hand-written
/// constructor would.
///
/// # Errors
///
/// Returns `attr.accessor_collision` if `class` already carries a
/// hand-written `construct` of the exact same derived selector (ADR-0012:
/// selector is the sole dispatch key, no last-wins) — a differently-
/// selectored hand-written `construct` (e.g. `construct anonymous()`)
/// coexists unaffected, since the collision check is selector-keyed, not
/// "any hand-written construct present".
fn derive_construct(class: &mut ClassDef, ctx: &mut ExpandCtx, attr_range: SourceRange) -> Result<(), CompilerError> {
    let fields: Vec<FieldDef> = class
        .members
        .iter()
        .filter_map(|m| match m {
            ClassMember::Field(f) => Some(f.clone()),
            _ => None,
        })
        .collect();

    let param_fields: Vec<&FieldDef> = fields.iter().filter(|f| f.default.is_none()).collect();
    let labels: Vec<Option<String>> = param_fields.iter().map(|f| Some(strip_leading_underscore(&f.name))).collect();
    let arity = param_fields.len() as u8;

    let derived_selector = encode_selector("new", &labels, SignatureKind::Initializer(arity));
    let derived_sym = ctx.interner.intern(&derived_selector);

    for m in &class.members {
        if let ClassMember::Construct(c) = m {
            let c_labels: Vec<Option<String>> = c.params.iter().map(|p| p.label.clone()).collect();
            let c_selector = encode_selector(&c.name, &c_labels, SignatureKind::Initializer(c.params.len() as u8));
            if ctx.interner.intern(&c_selector) == derived_sym {
                return Err(CompilerError::Message(format!(
                    "attr.accessor_collision: `@construct` on class `{}` collides with a hand-written `construct {}(...)` of the same selector",
                    class.name, c.name
                )));
            }
        }
    }

    let params: Vec<ParameterDef> = param_fields
        .iter()
        .map(|f| {
            let pname = strip_leading_underscore(&f.name);
            ParameterDef { name: pname.clone(), label: Some(pname), is_rest: false, range: f.range }
        })
        .collect();

    let mut body = Vec::new();
    for f in &fields {
        if let Some(default_expr) = &f.default {
            body.push(field_assign_stmt(&f.name, default_expr.clone(), f.range));
        }
    }
    for f in &param_fields {
        let pname = strip_leading_underscore(&f.name);
        body.push(field_assign_stmt(&f.name, Expr::Var { value: pname, range: f.range }, f.range));
    }

    class.members.push(ClassMember::Construct(ConstructDef {
        name: "new".to_string(),
        params,
        body,
        range: attr_range,
    }));
    Ok(())
}

/// Returns the derived selector `m` would occupy if `m` is a
/// [`ClassMember::Method`]/[`ClassMember::Getter`]/[`ClassMember::Setter`],
/// or `None` for a member kind with no ordinary dispatch selector of its own
/// ([`ClassMember::Field`]/[`ClassMember::Construct`]/[`ClassMember::Variant`]).
/// Shared by [`check_selector_collision`]/[`class_has_selector`] — the one
/// place this module computes "what selector does this existing member
/// occupy" generically across member kinds.
fn member_selector(m: &ClassMember) -> Option<String> {
    match m {
        ClassMember::Method(md) => {
            let labels: Vec<Option<String>> = md.params.iter().map(|p| p.label.clone()).collect();
            Some(encode_selector(&md.name, &labels, SignatureKind::Method(md.params.len() as u8)))
        }
        ClassMember::Getter(g) => Some(encode_selector(&g.name, &[], SignatureKind::Getter)),
        ClassMember::Setter(s) => Some(encode_selector(&s.name, &[], SignatureKind::Setter)),
        ClassMember::Field(_) | ClassMember::Construct(_) | ClassMember::Variant(_) => None,
    }
}

/// Returns `attr.accessor_collision` if `class` already carries a
/// hand-written [`ClassMember::Method`]/[`ClassMember::Getter`]/
/// [`ClassMember::Setter`] whose own derived selector
/// ([`member_selector`]) exactly matches `derived_selector` (ADR-0012:
/// selector is the sole dispatch key, no last-wins). Shared by every
/// layout-derive attribute that can collide with a hand-written member
/// (`@get`/`@set`, `@data`'s `==`/`hash`/`toString`/`with(...)`) — plan §3.2
/// asks for one helper here, not N inline copies (`derive_construct`'s own
/// inline construct-vs-construct check is a different member kind and stays
/// as-is).
fn check_selector_collision(class: &ClassDef, ctx: &mut ExpandCtx, derived_selector: &str, attr_name: &str) -> Result<(), CompilerError> {
    if class_has_selector(class, ctx, derived_selector) {
        return Err(CompilerError::Message(format!(
            "attr.accessor_collision: `@{}` on class `{}` collides with a hand-written member of the same selector",
            attr_name, class.name
        )));
    }
    Ok(())
}

/// Returns `attr.accessor_collision` if `class` already carries a
/// hand-written accessor of the given `base_name`/`kind`
/// ([`check_selector_collision`] specialized to `@get`/`@set`'s
/// getter/setter shape).
fn check_accessor_collision(class: &ClassDef, ctx: &mut ExpandCtx, base_name: &str, kind: SignatureKind, attr_name: &str) -> Result<(), CompilerError> {
    let derived_selector = encode_selector(base_name, &[], kind);
    check_selector_collision(class, ctx, &derived_selector, attr_name)
}

/// Returns whether `class` already carries a member (of any kind
/// [`member_selector`] recognizes) occupying `selector` exactly — a
/// non-erroring existence probe, used where a missing member should be
/// filled in silently rather than reported as a collision (`@data`'s
/// no-op-if-already-present derives).
fn class_has_selector(class: &ClassDef, ctx: &mut ExpandCtx, selector: &str) -> bool {
    let sym = ctx.interner.intern(selector);
    for m in &class.members {
        if let Some(existing) = member_selector(m) {
            if ctx.interner.intern(&existing) == sym {
                return true;
            }
        }
    }
    false
}

/// Derives `Getter`/`Setter` members for every declared [`FieldDef`]
/// carrying `@get`/`@set` and appends them to `class.members` (U-ANNOT-LAYOUT
/// §3.2, `annotations-construct.md` §"@get/@set").
///
/// Must run before [`expand_class_attributes`]'s member-level loop consumes
/// each `Field`'s attributes — same signature gap as [`derive_construct`]:
/// `AttributeExpander::expand` only mutates the one member it's attached to,
/// and this derive needs to append sibling members instead.
///
/// Field reads/writes in this AST always use [`Expr::Field`] (never
/// [`Expr::Var`]) — the getter body reads `_label` via `Expr::Field`, and the
/// setter body assigns it via [`field_assign_stmt`] reading its `value`
/// parameter via `Expr::Var` (an ordinary parameter binding, not a field).
///
/// `@get(priv)`'s bare argument is advisory naming only (§3.2) — parsed but
/// not inspected here; nothing gates on it.
fn derive_accessors(class: &mut ClassDef, ctx: &mut ExpandCtx) -> Result<(), CompilerError> {
    let fields: Vec<FieldDef> = class
        .members
        .iter()
        .filter_map(|m| match m {
            ClassMember::Field(f) => Some(f.clone()),
            _ => None,
        })
        .collect();

    for field in &fields {
        let base_name = strip_leading_underscore(&field.name);

        for attr in &field.attributes {
            if attr.name == "get" {
                check_accessor_collision(class, ctx, &base_name, SignatureKind::Getter, "get")?;
                class.members.push(ClassMember::Getter(GetterDef {
                    name: base_name.clone(),
                    body: vec![Statement::Expr {
                        expr: Expr::Field { value: field.name.clone(), range: attr.range },
                        range: attr.range,
                    }],
                    is_static: false,
                    attributes: Vec::new(),
                    range: attr.range,
                }));
            } else if attr.name == "set" {
                check_accessor_collision(class, ctx, &base_name, SignatureKind::Setter, "set")?;
                class.members.push(ClassMember::Setter(SetterDef {
                    name: base_name.clone(),
                    param: "value".to_string(),
                    body: vec![field_assign_stmt(
                        &field.name,
                        Expr::Var { value: "value".to_string(), range: attr.range },
                        attr.range,
                    )],
                    is_static: false,
                    attributes: Vec::new(),
                    range: attr.range,
                }));
            }
        }
    }
    Ok(())
}

/// Builds the field-by-field `and`-folded structural equality expression for
/// `@data`'s derived `==(other)` ([`derive_data`]): `self._f1 == other.f1 and
/// self._f2 == other.f2 and ...`, using the AST's own [`BinaryOp::Equal`]/
/// [`BinaryOp::And`] (`annotations-data.md`'s pseudocode names `and(_)` as a
/// method send, but the compiler already lowers a native `BinaryOp::And` to
/// the identical short-circuit jump sequence — `compiler::lib::expr` — so
/// this reuses that node directly rather than hand-building a `MethodCall`).
/// `other.<name>` reads through an ordinary [`GetPropertyExpr`] getter send,
/// never a direct field peek — [`Expr::Field`] is always implicit-`self`, so
/// there is no other way to read a sibling instance's declared field
/// (`derive_data`'s own doc explains why every field gets an auto-derived
/// getter when `==`/`hash` are generated). A field-less class (`@data` with
/// no declared `FieldDef`s) returns a vacuous `true`.
fn build_data_eq(fields: &[FieldDef], range: SourceRange) -> Expr {
    let mut acc: Option<Expr> = None;
    for f in fields {
        let base_name = strip_leading_underscore(&f.name);
        let field_read = Expr::Field { value: f.name.clone(), range };
        let other_read = Expr::GetProperty(Box::new(GetPropertyExpr { object: Expr::Var { value: "other".to_string(), range }, property: base_name, range }));
        let eq = Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::Equal, left: field_read, right: other_read, range }));
        acc = Some(match acc {
            None => eq,
            Some(left) => Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::And, left, right: eq, range })),
        });
    }
    acc.unwrap_or(Expr::Boolean { value: true, range })
}

/// Builds `@data`'s derived `hash` getter body ([`derive_data`]): a
/// left-folded polynomial hash-combine over every field's own `.hash`
/// (`acc = acc * 31 + field.hash`), matching the equality/hash contract's
/// consistency requirement (`a == b ⇒ a.hash == b.hash`,
/// `docs/spec/v0.2/experimental/equality-and-hash.md`) since it reads the
/// exact same fields `==` compares, in the exact same order.
///
/// `annotations-data.md`'s own pseudocode chains `.combine(...)` — no such
/// primitive or `core.ph` method exists on HEAD, so this builds the
/// equivalent arithmetic fold by hand instead (the Rubric's own fallback,
/// same rationale as [`build_data_to_string`]'s `+`-chain over `StringInterp`).
/// A field-less class returns a constant `0`.
fn build_data_hash(fields: &[FieldDef], range: SourceRange) -> Expr {
    let mut acc: Option<Expr> = None;
    for f in fields {
        let hash_read = Expr::GetProperty(Box::new(GetPropertyExpr { object: Expr::Field { value: f.name.clone(), range }, property: "hash".to_string(), range }));
        acc = Some(match acc {
            None => hash_read,
            Some(left) => {
                let scaled = Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::Multiply, left, right: Expr::Number { value: 31.0, range }, range }));
                Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::Add, left: scaled, right: hash_read, range }))
            }
        });
    }
    acc.unwrap_or(Expr::Number { value: 0.0, range })
}

/// Builds `@data`'s derived `toString` getter body ([`derive_data`]):
/// `"ClassName(" + String.new(_f1) + ", " + String.new(_f2) + ... + ")"`.
///
/// Reuses the exact `String.new(expr)`-per-segment, `BinaryOp::Add`-joined
/// shape `crate::parser::Parser::desugar_string_interp` (phalcom-ast) already
/// produces for `\(expr)` string interpolation, hand-built rather than
/// constructed through the parser's interpolation desugar directly — there is
/// no `Expr::StringInterp` AST node to synthesize (interpolation desugars
/// entirely at *parse* time into this same `+`-chain, so by the time this
/// module ever sees an AST there is nothing left to reuse but the shape
/// itself). This is the Rubric's own flagged fallback ("build the equivalent
/// `+`-chain manually is safer/less coupled").
fn build_data_to_string(class_name: &str, fields: &[FieldDef], range: SourceRange) -> Expr {
    let mut acc = Expr::String { value: format!("{}(", class_name), range };
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            acc = Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::Add, left: acc, right: Expr::String { value: ", ".to_string(), range }, range }));
        }
        let stringified = Expr::MethodCall(Box::new(MethodCallExpr {
            object: Expr::Var { value: "String".to_string(), range },
            method: "new".to_string(),
            args: vec![Argument { label: None, expr: Expr::Field { value: f.name.clone(), range }, range }],
            range,
        }));
        acc = Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::Add, left: acc, right: stringified, range }));
    }
    Expr::Binary(Box::new(BinaryExpr { op: BinaryOp::Add, left: acc, right: Expr::String { value: ")".to_string(), range }, range }))
}

/// Builds `@data`'s derived `with(...)` method body ([`derive_data`]): one
/// labeled parameter per non-defaulted field, each resolved via an
/// `(param == None).ifTrue({ self.<field> }, ifFalse: { param })` conditional
/// (see this function's body comment for why this diverges from
/// `annotations-data.md`'s literal `<param>.orElse { self.<field> }`
/// pseudocode) — a caller passes `None` for every field left unchanged, an
/// ordinary raw value for every field being replaced; every label is still
/// required at the call site, since keyword-argument omission is an ordinary
/// different-selector dispatch miss under this language's selector-identity
/// model, ADR-0012 — there is no partial-application sugar here. Allocates
/// one new instance via the same derived `new` selector `@construct`/`@data`'s
/// own constructor-derive produces. Shallow by construction: every field
/// value (`self.<field>`/`<param>`) is copied by reference, never cloned
/// (`annotations-data.md`'s explicit "standard functional-update semantics,
/// not a deep clone").
fn build_data_with(class_name: &str, param_fields: &[&FieldDef], range: SourceRange) -> Expr {
    let args: Vec<Argument> = param_fields
        .iter()
        .map(|f| {
            let pname = strip_leading_underscore(&f.name);
            // `(param == None).ifTrue({ self.<field> }, ifFalse: { param })` —
            // *not* `param.orElse { self.<field> }`, which
            // `annotations-data.md`'s own pseudocode literally shows. That
            // shape only compiles/works when `param` is already an `Option`
            // (`Option#orElse` is defined nowhere else, `core.ph`) — but
            // `with(...)`'s whole point is to accept a *raw* replacement
            // value (`money.with(cents: 700, ...)`, not `Some(700)`), which
            // has no `orElse` to send. This is the Rubric's own
            // "build the equivalent manually" fallback (same rationale as
            // `hash`/`toString` above): a plain identity comparison against
            // the `None` singleton, dispatched through the sacred two-armed
            // `ifTrue(_, ifFalse:_)` conditional every other derived/hand-
            // written control-flow send in this codebase already uses.
            let is_none = Expr::Binary(Box::new(BinaryExpr {
                op: BinaryOp::Equal,
                left: Expr::Var { value: pname.clone(), range },
                right: Expr::Var { value: "None".to_string(), range },
                range,
            }));
            let fallback_block = Expr::Block(Box::new(BlockExpr {
                params: Vec::new(),
                body: vec![Statement::Expr { expr: Expr::Field { value: f.name.clone(), range }, range }],
                expr_body: true,
                range,
            }));
            let keep_block = Expr::Block(Box::new(BlockExpr {
                params: Vec::new(),
                body: vec![Statement::Expr { expr: Expr::Var { value: pname.clone(), range }, range }],
                expr_body: true,
                range,
            }));
            let resolved = Expr::MethodCall(Box::new(MethodCallExpr {
                object: is_none,
                method: "ifTrue".to_string(),
                args: vec![Argument { label: None, expr: fallback_block, range }, Argument { label: Some("ifFalse".to_string()), expr: keep_block, range }],
                range,
            }));
            Argument { label: Some(pname), expr: resolved, range }
        })
        .collect();
    Expr::MethodCall(Box::new(MethodCallExpr {
        object: Expr::Var { value: class_name.to_string(), range },
        method: "new".to_string(),
        args,
        range,
    }))
}

/// Derives `@data`'s generate-phase additions to `class` (U-ANNOT-LAYOUT
/// §3.4, `annotations-data.md` §"`@data`"): the field-to-constructor derive
/// (reused unchanged from [`derive_construct`], own-fields-only shape — the
/// inheritance-aware F-fix, §3.3's separate build-order step, is
/// deliberately not layered on here), then `==(other)`, `hash`, `toString`,
/// and `with(...)`, in that order, appended to `class.members`.
///
/// # Constructor reuse
///
/// If `class.members` already carries a `Construct` member named `"new"` —
/// whether hand-written or already derived earlier in this same expansion
/// pass by a preceding `@construct` (attribute processing order in source is
/// not fixed either way) — this step is skipped entirely rather than
/// re-derived, avoiding a spurious self-collision (§3.4: "detect via 'a
/// `ConstructDef` named `new` was already emitted earlier in this same
/// expansion pass'").
///
/// # `==`/`hash` togetherness
///
/// `==`/`hash` are derived **together or not at all**
/// (`docs/spec/v0.2/experimental/equality-and-hash.md`'s hash-consistency
/// invariant: `a == b ⇒ a.hash == b.hash`, which a lone hand-written `==`
/// paired with a derived `hash` — or vice versa — could silently violate). A
/// class hand-writing exactly one of the two is `attr.accessor_collision`; a
/// class hand-writing **both** is a silent no-op (hand-written wins, same
/// "opt-in derive, hand-written wins on collision" precedent `@construct`
/// already follows); a class hand-writing **neither** gets both derived.
/// Deriving `==` also backfills a plain getter accessor for any field that
/// doesn't already have one — `other.<name>` (`build_data_eq`) can only ever
/// read a sibling instance's field through a real getter send, since
/// [`Expr::Field`] is always implicit-`self` and there is no other
/// field-read mechanism in this AST; a hand-written accessor of the same
/// selector is reused as-is, never treated as a collision here (only
/// `==`/`hash` carry the strict togetherness rule — this backfill is an
/// implementation necessity, not a user-facing derived member of its own).
///
/// # `toString`/`with(...)`
///
/// Both are independently no-op-if-already-present (silently skipped, no
/// error) — `annotations-data.md` states no explicit collision rule for
/// these two, unlike `==`/`hash`, so the same "hand-written wins" precedent
/// applies without an error. `with(...)` is omitted entirely for a
/// field-less (or all-defaulted-fields) class, since a zero-parameter
/// `with()` would carry no useful functional-update surface.
///
/// # Errors
///
/// Propagates [`derive_construct`]'s own `attr.accessor_collision` (a
/// hand-written `construct` of a different selector than the derived `new`
/// does **not** error — see that function's own doc), or returns
/// `attr.accessor_collision` directly if `class` hand-writes exactly one of
/// `==`/`hash`.
fn derive_data(class: &mut ClassDef, ctx: &mut ExpandCtx, attr_range: SourceRange) -> Result<(), CompilerError> {
    let fields: Vec<FieldDef> = class
        .members
        .iter()
        .filter_map(|m| match m {
            ClassMember::Field(f) => Some(f.clone()),
            _ => None,
        })
        .collect();

    let has_new_construct = class.members.iter().any(|m| matches!(m, ClassMember::Construct(c) if c.name == "new"));
    if !has_new_construct {
        derive_construct(class, ctx, attr_range)?;
    }

    let eq_selector = encode_selector("==", &[None], SignatureKind::Method(1));
    let hash_selector = make_signature_getter("hash");
    let has_eq = class_has_selector(class, ctx, &eq_selector);
    let has_hash = class_has_selector(class, ctx, &hash_selector);
    if has_eq != has_hash {
        return Err(CompilerError::Message(format!(
            "attr.accessor_collision: `@data` on class `{}` requires `==`/`hash` to be derived together — a hand-written `{}` without the other is a collision",
            class.name,
            if has_eq { "==" } else { "hash" }
        )));
    }
    if !has_eq {
        for f in &fields {
            let base_name = strip_leading_underscore(&f.name);
            let getter_selector = make_signature_getter(&base_name);
            if !class_has_selector(class, ctx, &getter_selector) {
                class.members.push(ClassMember::Getter(GetterDef {
                    name: base_name,
                    body: vec![Statement::Expr { expr: Expr::Field { value: f.name.clone(), range: attr_range }, range: attr_range }],
                    is_static: false,
                    attributes: Vec::new(),
                    range: attr_range,
                }));
            }
        }

        let eq_body = build_data_eq(&fields, attr_range);
        class.members.push(ClassMember::Method(MethodDef {
            name: "==".to_string(),
            params: vec![ParameterDef { name: "other".to_string(), label: None, is_rest: false, range: attr_range }],
            body: vec![Statement::Return(ReturnStatement { value: Some(eq_body), range: attr_range })],
            is_static: false,
            attributes: Vec::new(),
            range: attr_range,
        }));

        let hash_body = build_data_hash(&fields, attr_range);
        class.members.push(ClassMember::Getter(GetterDef {
            name: "hash".to_string(),
            body: vec![Statement::Return(ReturnStatement { value: Some(hash_body), range: attr_range })],
            is_static: false,
            attributes: Vec::new(),
            range: attr_range,
        }));
    }

    let to_string_selector = make_signature_getter("toString");
    if !class_has_selector(class, ctx, &to_string_selector) {
        let ts_body = build_data_to_string(&class.name, &fields, attr_range);
        class.members.push(ClassMember::Getter(GetterDef {
            name: "toString".to_string(),
            body: vec![Statement::Return(ReturnStatement { value: Some(ts_body), range: attr_range })],
            is_static: false,
            attributes: Vec::new(),
            range: attr_range,
        }));
    }

    let param_fields: Vec<&FieldDef> = fields.iter().filter(|f| f.default.is_none()).collect();
    if !param_fields.is_empty() {
        let with_labels: Vec<Option<String>> = param_fields.iter().map(|f| Some(strip_leading_underscore(&f.name))).collect();
        let with_selector = encode_selector("with", &with_labels, SignatureKind::Method(param_fields.len() as u8));
        if !class_has_selector(class, ctx, &with_selector) {
            let with_body = build_data_with(&class.name, &param_fields, attr_range);
            let with_params: Vec<ParameterDef> = param_fields
                .iter()
                .map(|f| {
                    let pname = strip_leading_underscore(&f.name);
                    ParameterDef { name: pname.clone(), label: Some(pname), is_rest: false, range: f.range }
                })
                .collect();
            class.members.push(ClassMember::Method(MethodDef {
                name: "with".to_string(),
                params: with_params,
                body: vec![Statement::Return(ReturnStatement { value: Some(with_body), range: attr_range })],
                is_static: false,
                attributes: Vec::new(),
                range: attr_range,
            }));
        }
    }

    Ok(())
}

/// `encode_selector(base, &[], SignatureKind::Getter)` — a tiny naming
/// shorthand used repeatedly by [`derive_data`] (a bare getter selector is
/// just its own name; kept as a named helper purely for readability at each
/// call site, not because the encoding is nontrivial).
fn make_signature_getter(base: &str) -> String {
    encode_selector(base, &[], SignatureKind::Getter)
}

/// Lower-cases the first character of `name` (e.g. `"Circle"` → `"circle"`),
/// leaving the rest unchanged — the variant-name-to-keyword-label convention
/// [`expand_variants`]'s generated `match(...)` visitor and `__matchArm`
/// overrides use (`annotations-data.md`'s worked `Circle`/`Rect` example:
/// `match(circle:, rect:)`). Byte-based (not full Unicode case-folding) —
/// consistent with every other identifier-shaping helper in this module
/// ([`strip_leading_underscore`]).
fn lower_first(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Expands every `@variant Name(labels...)` arm declared in `class`'s body
/// (U-ANNOT-LAYOUT §3.4, `annotations-data.md` §"`@variant`") into a sibling
/// top-level `Statement::Class`, `extends class.name`, itself carrying
/// `@data` and one `FieldDef` per label — then, once every arm is known,
/// appends the enclosing class's generated `match(...)` visitor
/// (`self.__matchArm(...)`, double-dispatched to each variant's own
/// `__matchArm` override).
///
/// Returns the generated sibling [`Statement::Class`] nodes for the caller
/// (`compiler::lib::class_decl::Compiler::compile_class`, DEC-ANNOT-G) to
/// compile immediately after the enclosing class — this is the one place
/// this tier's generate phase produces something other than a member of the
/// class it's attached to, the load-bearing reason
/// [`expand_class_attributes`]'s return type widened to include a
/// `Vec<Statement>` alongside the (still member-only) `ClassDef` it returns.
///
/// Returns an empty `Vec` (no-op) if `class` declares no `@variant` arms at
/// all.
///
/// # Errors
///
/// Returns a compile error if `class` declares one or more `@variant` arms
/// but `has_sealed` is `false` — `@variant` is only meaningful inside a
/// `@sealed` class body (`annotations-data.md`'s own worked example always
/// pairs the two; `@sealed` is what lets the generated visitor be checked
/// exhaustive at all, per that doc's `@sealed` section).
fn expand_variants(class: &mut ClassDef, has_sealed: bool) -> Result<Vec<Statement>, CompilerError> {
    let variants: Vec<VariantDef> = class
        .members
        .iter()
        .filter_map(|m| match m {
            ClassMember::Variant(v) => Some(v.clone()),
            _ => None,
        })
        .collect();
    if variants.is_empty() {
        return Ok(Vec::new());
    }
    if !has_sealed {
        return Err(CompilerError::Message(format!(
            "attr.illegal_target: `@variant` requires its enclosing class `{}` to also carry `@sealed`",
            class.name
        )));
    }

    class.members.retain(|m| !matches!(m, ClassMember::Variant(_)));

    let variant_kw_names: Vec<String> = variants.iter().map(|v| lower_first(&v.name)).collect();
    let mut siblings = Vec::with_capacity(variants.len());

    for v in &variants {
        let mut members = Vec::with_capacity(v.labels.len() + 1);
        for label in &v.labels {
            members.push(ClassMember::Field(FieldDef {
                name: format!("_{}", label),
                mutable: true,
                default: None,
                attributes: Vec::new(),
                range: v.range,
            }));
        }

        // `__matchArm(k1, k2, ...) { return <ownKeyword>.call(self) }` — every
        // variant implements the identical selector (same keyword-name list,
        // positional, in `@variant` declaration order across the *whole*
        // sealed family), overriding only which positional block it calls —
        // the double-dispatch step the enclosing class's `match(...)` defers
        // to.
        let own_kw = lower_first(&v.name);
        let params: Vec<ParameterDef> = variant_kw_names
            .iter()
            .map(|n| ParameterDef { name: n.clone(), label: None, is_rest: false, range: v.range })
            .collect();
        let call_expr = Expr::MethodCall(Box::new(MethodCallExpr {
            object: Expr::Var { value: own_kw, range: v.range },
            method: "call".to_string(),
            args: vec![Argument { label: None, expr: Expr::SelfVar { range: v.range }, range: v.range }],
            range: v.range,
        }));
        members.push(ClassMember::Method(MethodDef {
            name: "__matchArm".to_string(),
            params,
            body: vec![Statement::Return(ReturnStatement { value: Some(call_expr), range: v.range })],
            is_static: false,
            attributes: Vec::new(),
            range: v.range,
        }));

        siblings.push(Statement::Class(ClassDef {
            name: v.name.clone(),
            superclass: Some(SuperclassRef { name: class.name.clone(), range: v.range }),
            members,
            attributes: vec![Attribute { name: "data".to_string(), args: Vec::new(), range: v.range }],
            invariants: Vec::new(),
            range: v.range,
        }));
    }

    // `match(k1:, k2:, ...) { return self.__matchArm(k1, k2, ...) }` on the
    // enclosing sealed class — the keyword-argument list is exactly the
    // declared `@variant` names, in declaration order (`annotations-data.md`:
    // "exhaustiveness for free" — a call site omitting or misnaming an arm
    // is an ordinary missing-keyword-argument dispatch miss, no new
    // diagnostic needed).
    let match_range = variants.first().expect("checked non-empty above").range;
    let match_params: Vec<ParameterDef> = variant_kw_names
        .iter()
        .map(|n| ParameterDef { name: n.clone(), label: Some(n.clone()), is_rest: false, range: match_range })
        .collect();
    let arm_args: Vec<Argument> = variant_kw_names
        .iter()
        .map(|n| Argument { label: None, expr: Expr::Var { value: n.clone(), range: match_range }, range: match_range })
        .collect();
    let arm_call = Expr::MethodCall(Box::new(MethodCallExpr {
        object: Expr::SelfVar { range: match_range },
        method: "__matchArm".to_string(),
        args: arm_args,
        range: match_range,
    }));
    class.members.push(ClassMember::Method(MethodDef {
        name: "match".to_string(),
        params: match_params,
        body: vec![Statement::Return(ReturnStatement { value: Some(arm_call), range: match_range })],
        is_static: false,
        attributes: Vec::new(),
        range: match_range,
    }));

    Ok(siblings)
}

/// Walks `name`'s compile-time `extends` chain
/// ([`ExpandCtx::class_parents`]) up to the `Attribute` root, returning
/// whether `name` names a (possibly transitive) `Attribute` subclass.
///
/// Used by [`expand_class_attributes`] to decide whether an attribute name
/// unrecognized by the [`AttributeRegistry`] (`@requires`/`@ensures`/
/// `@invariant`/`@construct`/`@On` are the only registered names) is a real
/// user `Attribute` subclass — retained silently — or genuinely unknown —
/// `attr.unknown` (M-ATTR-ROOT, `attribute-classes.md` §"Decision"). The walk
/// terminates at the implicit `Object` root (no entry in `class_parents`) or
/// on revisiting an already-seen symbol (a reopen-redefinition back-edge, the
/// same guard [`crate::compiler::lib::Compiler::inherits_new_construct`]
/// uses).
fn resolves_to_attribute_class(class_parents: &HashMap<Symbol, Symbol>, interner: &mut crate::interner::Interner, name: &str) -> bool {
    let attribute_sym = interner.intern("Attribute");
    let mut sym = interner.intern(name);
    let mut visited = std::collections::HashSet::new();
    while visited.insert(sym) {
        if sym == attribute_sym {
            return true;
        }
        match class_parents.get(&sym) {
            Some(&parent) => sym = parent,
            None => return false,
        }
    }
    false
}

/// The hook selectors reserved for an `Attribute` subclass's tier
/// declaration ([`validate_attribute_class`]), each paired with the tier
/// name (as matched against a bare `@On(...)` argument `Var`) it belongs to.
const RESERVED_HOOKS: &[(&str, &str)] =
    &[("expand", "Compile"), ("finalizeLayout", "Layout"), ("wrap", "Install"), ("resolveMissing", "Dispatch"), ("aroundSend", "Runtime")];

/// The five tier names recognized in a bare `@On(...)` argument position —
/// matched by `Var` name, not resolved to the runtime [`Tier`] singleton
/// object it names, since attribute-arg lists are positional-only (the
/// parser cannot express `tier: Install`, see `docs/forge/DEFERRED.md`) and
/// this check runs at compile time, before any singleton could be evaluated.
const TIER_NAMES: &[&str] = &["Compile", "Layout", "Install", "Dispatch", "Runtime"];

/// Validates a would-be `Attribute` subclass's own `@On(...)` tier
/// declaration against its declared members (M-ATTR-ROOT,
/// `attribute-classes.md` §"A-1"/§"The `Attribute` root and the hook
/// protocol"). Called from [`expand_class_attributes`] only when
/// `is_attribute_class` is set (the class directly `extends Attribute`).
///
/// # Errors
///
/// - `attr.compile_tier_reserved` — a declared tier of `Compile` or `Layout`
///   (compiler-native only, A-3); no non-builtin class may occupy these.
/// - `attr.missing_hook` — a declared `Install`/`Dispatch`/`Runtime` tier
///   with no matching hook selector (`wrap`/`resolveMissing`/`aroundSend`)
///   implemented among the class's members.
/// - `attr.undeclared_hook` — a reserved hook selector implemented without a
///   matching declared tier (no `@On` tier at all, or a tier declared but a
///   *different* reserved selector is also implemented) — these names are
///   reserved on `Attribute` subclasses specifically so an unrelated
///   same-named method can't silently be drafted into a tier.
fn validate_attribute_class(class: &ClassDef, class_attrs: &[phalcom_ast::ast::Attribute]) -> Result<(), CompilerError> {
    let on_attr = class_attrs.iter().find(|a| a.name == "On");
    let declared_tier: Option<&str> = on_attr.and_then(|attr| {
        attr.args.iter().find_map(|arg| match arg {
            Expr::Var { value, .. } if TIER_NAMES.contains(&value.as_str()) => Some(value.as_str()),
            _ => None,
        })
    });

    if let Some(tier) = declared_tier
        && (tier == "Compile" || tier == "Layout")
    {
        return Err(CompilerError::Message(format!(
            "attr.compile_tier_reserved: class `{}` declares `@On(..., {})` — Compile/Layout are compiler-native tiers, not available to user `Attribute` subclasses",
            class.name, tier
        )));
    }

    let expected_hook = declared_tier.map(|tier| RESERVED_HOOKS.iter().find(|(_, t)| *t == tier).map(|(h, _)| *h).unwrap());

    let implemented: Vec<&str> = class
        .members
        .iter()
        .filter_map(|m| match m {
            ClassMember::Method(md) if RESERVED_HOOKS.iter().any(|(h, _)| *h == md.name) => Some(md.name.as_str()),
            _ => None,
        })
        .collect();

    match expected_hook {
        Some(hook) => {
            if !implemented.contains(&hook) {
                return Err(CompilerError::Message(format!(
                    "attr.missing_hook: class `{}` declares `@On(..., {})` but implements no `{}(_)` hook method",
                    class.name,
                    declared_tier.unwrap(),
                    hook
                )));
            }
            if let Some(&extra) = implemented.iter().find(|h| **h != hook) {
                return Err(CompilerError::Message(format!(
                    "attr.undeclared_hook: class `{}` implements reserved hook method `{}(_)` without a matching declared `@On` tier",
                    class.name, extra
                )));
            }
        }
        None => {
            if let Some(&extra) = implemented.first() {
                return Err(CompilerError::Message(format!(
                    "attr.undeclared_hook: class `{}` implements reserved hook method `{}(_)` without a matching declared `@On` tier",
                    class.name, extra
                )));
            }
        }
    }

    Ok(())
}

/// Expands every `@name(args…)` attribute attached to `class` or one of its
/// members (U-ANNOT-CONTRACTS's core pass, grown by U-ANNOT-LAYOUT's `@get`/
/// `@set`/`@construct`/`@data`/`@sealed`/`@variant` rows) into ordinary AST,
/// returning the rewritten class alongside any sibling top-level statements
/// its expansion produced.
///
/// # Return shape (DEC-ANNOT-G)
///
/// Returns `(ClassDef, Vec<Statement>)`, not a bare `ClassDef` — the one
/// non-additive change U-ANNOT-LAYOUT makes to U-ANNOT-CONTRACTS's own
/// landed shape (§3.4 "the one place this plan's stated 'strict dependency,
/// not reshape' assumption needs a caveat"). The `Vec<Statement>` is always
/// `Statement::Class` nodes generated by `expand_variants` (a private helper
/// in this module) — every `@variant
/// Name(labels...)` arm inside `class`'s body becomes a sibling top-level
/// class, `extends class.name`, which this function cannot append to
/// `class.members` itself (a variant is a sibling *global* class, not a
/// member of the class that declares it). The caller
/// (`compiler::lib::class_decl::Compiler::compile_class`) compiles each
/// returned sibling immediately after `class` itself, via a recursive
/// `compile_class` call — this keeps every generate-phase decision owned by
/// this module, per DEC-ANNOT-G's recommendation (a), rather than leaking
/// `@variant`-specific knowledge into `compiler::lib`. The returned `Vec` is
/// empty for every class that declares no `@variant` arms (the overwhelming
/// majority) — no behavioral change to any pre-existing caller's assumptions
/// beyond destructuring the new tuple.
///
/// # Errors
///
/// Returns `attr.unknown`/`attr.illegal_target` for a misplaced or
/// unrecognized attribute, `attr.accessor_collision` for a hand-written
/// member colliding with a derived one (`@get`/`@set`/`@construct`/`@data`),
/// `contract.impure_predicate` for an impure `@invariant` predicate, or any
/// error a specific expander/derive raises (`derive_construct`,
/// `derive_data`, `expand_variants`, `validate_attribute_class`).
pub fn expand_class_attributes(
    mut class: ClassDef,
    ctx: &mut ExpandCtx,
    registry: &AttributeRegistry,
    is_attribute_class: bool,
) -> Result<(ClassDef, Vec<Statement>), CompilerError> {
    // 1. Expand class-level attributes
    let class_attrs = std::mem::take(&mut class.attributes);
    let mut class_invariants = std::mem::take(&mut class.invariants);
    // U-ANNOT-LAYOUT §3.4: whether `class` itself carries `@sealed` — read
    // before `class_attrs` is moved back into `class.attributes` below, and
    // threaded into `expand_variants` (a `@variant` arm with no enclosing
    // `@sealed` is a compile error, not a silent open hierarchy).
    let has_sealed = class_attrs.iter().any(|a| a.name == "sealed");
    for attr in &class_attrs {
        if let Some(expander) = registry.get(&attr.name) {
            let legal = expander.legal_targets().contains(&Target::Class);
            if !legal {
                return Err(CompilerError::Message(format!(
                    "attr.illegal_target: attribute `@{}` is not legal on class targets",
                    attr.name
                )));
            }
            if attr.name == "invariant" {
                validate_purity(&attr.args)?;
                for arg in &attr.args {
                    class_invariants.push((arg.clone(), attr.range));
                }
            } else if attr.name == "construct" {
                derive_construct(&mut class, ctx, attr.range)?;
            } else if attr.name == "data" {
                derive_data(&mut class, ctx, attr.range)?;
            }
            // `@sealed`'s own bookkeeping (recording `class` + the compiling
            // module in `VM::sealed_classes`) needs the compiling
            // `Compiler`'s module handle, which this function has no access
            // to — it runs from `compile_class` itself, right after this
            // call returns (see that function's doc).
        } else if resolves_to_attribute_class(ctx.class_parents, ctx.interner, &attr.name) {
            // M-ATTR-ROOT: an unrecognized name that resolves to a user
            // `Attribute` subclass is retained silently — its runtime
            // instantiate+attach codegen is emitted separately by
            // `compiler::lib::class_decl::compile_class`, not here.
        } else {
            return Err(CompilerError::Message(format!(
                "attr.unknown: unknown attribute `@{}`",
                attr.name
            )));
        }
    }

    if is_attribute_class {
        validate_attribute_class(&class, &class_attrs)?;
    }

    class.attributes = class_attrs;

    // Validate standalone invariants for purity too
    for (inv_expr, _) in &class_invariants {
        if !is_pure_expr(inv_expr) {
            return Err(CompilerError::Message("contract.impure_predicate: predicate contains mutating or side-effecting operations".to_string()));
        }
    }

    // 1.5. Derive @get/@set accessors from declared fields, before the
    // member-level loop below consumes each Field's attributes.
    derive_accessors(&mut class, ctx)?;

    // 1.6. Expand @variant arms into sibling top-level classes + the
    // enclosing class's generated `match(...)` visitor — also before the
    // member-level loop below, so a stripped `Variant` member never reaches
    // it (U-ANNOT-LAYOUT §3.4, DEC-ANNOT-G).
    let sibling_statements = expand_variants(&mut class, has_sealed)?;

    // 2. Expand member-level attributes
    for member in &mut class.members {
        let member_target = match member {
            ClassMember::Method(_) => Target::Method,
            ClassMember::Getter(_) => Target::Getter,
            ClassMember::Setter(_) => Target::Setter,
            ClassMember::Construct(_) => Target::Construct,
            ClassMember::Field(_) => Target::Field,
            // Unreachable in practice — `expand_variants` above (1.6) always
            // strips every `Variant` member before this loop runs. Kept for
            // match exhaustiveness over `ClassMember`'s full variant set.
            ClassMember::Variant(_) => Target::Variant,
        };

        let attrs = match member {
            ClassMember::Method(m) => std::mem::take(&mut m.attributes),
            ClassMember::Getter(g) => std::mem::take(&mut g.attributes),
            ClassMember::Setter(s) => std::mem::take(&mut s.attributes),
            ClassMember::Construct(_) => Vec::new(),
            ClassMember::Field(f) => std::mem::take(&mut f.attributes),
            ClassMember::Variant(v) => std::mem::take(&mut v.attributes),
        };

        for attr in &attrs {
            if let Some(expander) = registry.get(&attr.name) {
                if !expander.legal_targets().contains(&member_target) {
                    return Err(CompilerError::Message(format!(
                        "attr.illegal_target: attribute `@{}` is not legal on this target",
                        attr.name
                    )));
                }
                expander.expand(ctx, member, &attr.args)?;
            } else if resolves_to_attribute_class(ctx.class_parents, ctx.interner, &attr.name) {
                // Retained silently — see the class-level branch above.
            } else {
                return Err(CompilerError::Message(format!(
                    "attr.unknown: unknown attribute `@{}`",
                    attr.name
                )));
            }
        }

        match member {
            ClassMember::Method(m) => m.attributes = attrs,
            ClassMember::Getter(g) => g.attributes = attrs,
            ClassMember::Setter(s) => s.attributes = attrs,
            ClassMember::Construct(_) => {}
            ClassMember::Field(f) => f.attributes = attrs,
            ClassMember::Variant(v) => v.attributes = attrs,
        }
    }

    // 3. Weave class invariants into methods, getters, setters, and
    // constructors. §3.6 axis 1: `@invariant`'s guard is woven only in
    // `Debug` (table row 1) — `Release`/`Unchecked` strip it (rows 2/3),
    // same no-op-weave rule as `EnsuresExpander`. Purity validation above
    // (`is_pure_expr` over `class_invariants`) still ran unconditionally.
    if !class_invariants.is_empty() && ctx.compile_mode == CompileMode::Debug {
        let class_name = class.name.clone();
        for member in &mut class.members {
            match member {
                ClassMember::Method(m) if !m.is_static => {
                    weave_invariant_checks(&mut m.body, &class_invariants, &class_name, false);
                }
                ClassMember::Getter(g) if !g.is_static => {
                    weave_invariant_checks(&mut g.body, &class_invariants, &class_name, false);
                }
                ClassMember::Setter(s) if !s.is_static => {
                    weave_invariant_checks(&mut s.body, &class_invariants, &class_name, false);
                }
                ClassMember::Construct(c) => {
                    // Entry check skipped — invariant cannot hold pre-construction.
                    weave_invariant_checks(&mut c.body, &class_invariants, &class_name, true);
                }
                _ => {}
            }
        }
    }

    class.invariants = class_invariants;
    Ok((class, sibling_statements))
}

/// Builds `predicate.ifFalse { <error_class>.raise("<msg>") }` — the shared
/// check-or-raise shape used by `@requires`/`@ensures`/`@invariant` alike.
fn build_check_stmt(predicate: Expr, error_class: &str, err_msg: String, range: SourceRange) -> Statement {
    // `<ErrorClass>.new(msg).raise()` — `Error#raise()` (floor-census.md
    // §2.15) unwinds with the *instance* `self`; it takes no message
    // argument, so the message must be baked into a constructed instance
    // first, not passed to `raise` itself.
    let new_instance = Expr::MethodCall(Box::new(MethodCallExpr {
        object: Expr::Var { value: error_class.to_string(), range },
        method: "new".to_string(),
        args: vec![Argument { label: None, expr: Expr::String { value: err_msg, range }, range }],
        range,
    }));
    let error_expr = Expr::MethodCall(Box::new(MethodCallExpr {
        object: new_instance,
        method: "raise".to_string(),
        args: Vec::new(),
        range,
    }));
    let block_expr = Expr::Block(Box::new(BlockExpr {
        params: Vec::new(),
        body: vec![Statement::Expr { expr: error_expr, range }],
        expr_body: true,
        range,
    }));
    let check_call = Expr::MethodCall(Box::new(MethodCallExpr {
        object: predicate,
        method: "ifFalse".to_string(),
        args: vec![Argument { label: None, expr: block_expr, range }],
        range,
    }));
    Statement::Expr { expr: check_call, range }
}

/// Builds `self.<method>()` — a zero-arg send on the implicit receiver, used
/// for the `__invariantEnter`/`__invariantExit` re-entrancy-guard primitives.
fn self_send0(method: &str, range: SourceRange) -> Expr {
    Expr::MethodCall(Box::new(MethodCallExpr {
        object: Expr::SelfVar { range },
        method: method.to_string(),
        args: Vec::new(),
        range,
    }))
}

/// Returns `stmt`'s source span.
///
/// [`Statement`] has no inherent `range()` method (unlike [`Expr`]/[`Pattern`]),
/// so the invariant weave — the only place in this module that needs a
/// statement's span rather than an expression's — reads it out of whichever
/// variant carries one.
fn statement_range(stmt: &Statement) -> SourceRange {
    match stmt {
        Statement::Class(c) => c.range,
        Statement::Let(l) => l.range,
        Statement::Return(r) => r.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(f) => f.range,
        Statement::Break { range } | Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Import(i) => i.range,
    }
}

/// Weaves the receiver-scoped `@invariant` re-entrancy guard
/// ([ADR-0052](../../../docs/adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
/// Fix 1) around `body`, replacing it in place:
///
/// ```text
/// let __invariant_owner = self.__invariantEnter()
/// __invariant_owner.ifTrue { <entry checks> }   // skipped for constructors
/// return Block.new { <original body> }.ensure {
///     __invariant_owner.ifTrue {
///         <exit checks>
///         self.__invariantExit()
///     }
/// }
/// ```
///
/// The original `body` is moved verbatim into the wrapping block literal —
/// no rewriting of its `return` statements is needed. `return`/`throw`/a
/// fiber-abort inside it are already a single unwind primitive that
/// `Block#ensure(_)` is guaranteed to observe, so the exit check and the
/// `checking`-set removal fire on every exit path (ADR-0052 Fix 1). Ownership
/// is a captured local, never a re-check of the guard set, so only the
/// outermost call on a given receiver inserts/removes/checks (ADR-0052 Bug 1).
///
/// `is_construct` skips the entry check: a constructor's invariant cannot
/// hold before the object is built, so only its exit (post-construction)
/// state is checked — matching Eiffel's own-object nesting rule.
fn weave_invariant_checks(body: &mut Vec<Statement>, invariants: &[(Expr, SourceRange)], class_name: &str, is_construct: bool) {
    let range = body.first().map(statement_range).unwrap_or_default();

    let owner_var = Expr::Var { value: "__invariant_owner".to_string(), range };
    let owner_let = Statement::Let(LetBinding {
        kind: BindingKind::Let,
        pattern: Pattern::Name { name: "__invariant_owner".to_string(), range },
        value: Some(self_send0("__invariantEnter", range)),
        range,
    });

    let check_stmts = |_for_entry: bool| -> Vec<Statement> {
        invariants
            .iter()
            .map(|(inv_expr, inv_range)| {
                let err_msg = format!("Invariant failed for class `{}`: {}", class_name, inv_expr.range().start);
                build_check_stmt(inv_expr.clone(), "InvariantError", err_msg, *inv_range)
            })
            .collect()
    };

    let mut new_body = vec![owner_let];
    if !is_construct {
        new_body.push(Statement::Expr {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: owner_var.clone(),
                method: "ifTrue".to_string(),
                args: vec![Argument {
                    label: None,
                    expr: Expr::Block(Box::new(BlockExpr { params: Vec::new(), body: check_stmts(true), expr_body: false, range })),
                    range,
                }],
                range,
            })),
            range,
        });
    }

    let original_body = std::mem::take(body);
    let body_block = Expr::Block(Box::new(BlockExpr { params: Vec::new(), body: original_body, expr_body: true, range }));

    let mut cleanup_body = check_stmts(false);
    cleanup_body.push(Statement::Expr { expr: self_send0("__invariantExit", range), range });
    let cleanup_guard = Statement::Expr {
        expr: Expr::MethodCall(Box::new(MethodCallExpr {
            object: owner_var,
            method: "ifTrue".to_string(),
            args: vec![Argument {
                label: None,
                expr: Expr::Block(Box::new(BlockExpr { params: Vec::new(), body: cleanup_body, expr_body: false, range })),
                range,
            }],
            range,
        })),
        range,
    };
    let cleanup_block = Expr::Block(Box::new(BlockExpr { params: Vec::new(), body: vec![cleanup_guard], expr_body: false, range }));

    let ensure_call = Expr::MethodCall(Box::new(MethodCallExpr {
        object: body_block,
        method: "ensure".to_string(),
        args: vec![Argument { label: None, expr: cleanup_block, range }],
        range,
    }));

    // A constructor's initializer implicitly returns `self` and cannot
    // `return` a value — run the wrapped block for effect only there;
    // every other member kind returns the block's (and thus the original
    // body's) implicit value, unchanged from the unwoven behavior.
    if is_construct {
        new_body.push(Statement::Expr { expr: ensure_call, range });
    } else {
        new_body.push(Statement::Return(phalcom_ast::ast::ReturnStatement { value: Some(ensure_call), range }));
    }
    *body = new_body;
}
