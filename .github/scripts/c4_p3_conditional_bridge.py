from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} anchor count: {count}")
    file.write_text(text.replace(old, new, 1))


# Normalize runtime probes around the known applied type-form runtime defect.
replace_once(
    "phalcom-core/tests/core/language/traits.rs",
    "let caller = Caller.new()\nlet stringResult = caller.stringTag(Value<String>.new())\nlet intResult = caller.intTag(Value<Int>.new())",
    "let caller = Caller.new()\nlet stringValue: Value<String> = Value.new()\nlet intValue: Value<Int> = Value.new()\nlet stringResult = caller.stringTag(stringValue)\nlet intResult = caller.intTag(intValue)",
    "specialized fixture",
)
replace_once(
    "phalcom-core/tests/core/language/traits.rs",
    'trait Tagged { tag -> String { "default" } }\nclass Number {}\nclass Value<T> { @constructor new() {} }\nimpl<T> Value<T> where T <: Number { tag -> String { "number" } }\nimpl<T> Tagged for Value<T> {}\nclass Caller { run(_ value: Value<Number>) -> String { value.tag } }\nlet result = Caller.new().run(Value<Number>.new())',
    'trait Tagged { tag -> String { "default" } }\nclass NumericBase {}\nclass Numeric is NumericBase {}\nclass Value<T> { @constructor new() {} }\nimpl<T> Value<T> where T <: NumericBase { tag -> String { "number" } }\nimpl<T> Tagged for Value<T> {}\nclass Caller { run(_ value: Value<Numeric>) -> String { value.tag } }\nlet value: Value<Numeric> = Value.new()\nlet result = Caller.new().run(value)',
    "conditional fixture",
)

# Publish conditional inherent method handles for trait execution without
# installing them into target class dictionaries.
replace_once(
    "phalcom-core/src/compiler/lib/impl_decl.rs",
    """        Err(CompilerError::ImplCallableMismatch(SourceRange::new(0, 0)))
    }

    /// Installs all semantically accepted inherent `impl` members for nominal `target`""",
    """        Err(CompilerError::ImplCallableMismatch(SourceRange::new(0, 0)))
    }

    /// Ensures every conditional inherent callable retained by one trait
    /// invocation has an executable method handle available to the VM. The
    /// semantic proof has already selected these targets; this only publishes
    /// the existing C2-compiled method object by canonical callable identity.
    pub(crate) fn prepare_trait_invocation_methods(
        &mut self,
        spec: &crate::modules::semantic_lowering::TraitInvocationSpec,
    ) -> Result<(), CompilerError> {
        self.prepare_trait_selection_method(&spec.selection.selection)?;
        for selection in spec.selection.requirement_targets.values() {
            self.prepare_trait_selection_method(selection)?;
        }
        Ok(())
    }

    fn prepare_trait_selection_method(
        &mut self,
        selection: &phalcom_semantic::impls::RequirementSelectionTemplate,
    ) -> Result<(), CompilerError> {
        use phalcom_semantic::impls::RequirementSelectionTemplate;

        let callable = match selection {
            RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => Some(candidate.callable.clone()),
            RequirementSelectionTemplate::InherentCallable {
                callable,
                conditional_impl: Some(_),
                ..
            } => Some(callable.clone()),
            RequirementSelectionTemplate::InherentCallable { .. }
            | RequirementSelectionTemplate::ConformanceCallable { .. }
            | RequirementSelectionTemplate::DataComponent { .. }
            | RequirementSelectionTemplate::TraitDefault { .. } => None,
        };
        let Some(callable) = callable else {
            return Ok(());
        };
        let method = self.get_or_compile_conditional_method(&callable)?;
        self.vm.detached_method_objects.insert(callable, method);
        Ok(())
    }

    /// Installs all semantically accepted inherent `impl` members for nominal `target`""",
    "conditional witness helper",
)
replace_once(
    "phalcom-core/src/compiler/lib/scope.rs",
    """        let Some((_, spec)) = lowering
            .trait_invocations
            .iter()
            .find(|(site, _)| site.range == range && site.kind == crate::modules::semantic_lowering::LoweringSiteKind::TraitInvoke)
        else {
            return Ok(false);
        };
        let spec = spec.clone();
        let index = self""",
    """        let Some((_, spec)) = lowering
            .trait_invocations
            .iter()
            .find(|(site, _)| site.range == range && site.kind == crate::modules::semantic_lowering::LoweringSiteKind::TraitInvoke)
        else {
            return Ok(false);
        };
        let spec = spec.clone();
        self.prepare_trait_invocation_methods(&spec)?;
        let index = self""",
    "ordinary trait invoke preparation",
)
replace_once(
    "phalcom-core/src/compiler/lib/associated.rs",
    """            CallableReferenceLoweringSpec::MakeTraitBoundMethod { invocation } => {
                let phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. } = &expr.target else {""",
    """            CallableReferenceLoweringSpec::MakeTraitBoundMethod { invocation } => {
                self.prepare_trait_invocation_methods(&invocation)?;
                let phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. } = &expr.target else {""",
    "trait bound reference preparation",
)
replace_once(
    "phalcom-core/src/vm/mod.rs",
    """    /// Detached conformance witnesses and trait defaults, keyed by semantic
    /// callable identity and never installed in target class dictionaries.
    pub(crate) detached_method_objects: HashMap<CallableId, ObjRef>,""",
    """    /// Executable semantic methods that are intentionally absent from target
    /// class dictionaries: conformance witnesses, trait defaults, and proven
    /// conditional inherent members. Keyed by canonical callable identity and
    /// rooted by the VM for the lifetime of the executable program.
    pub(crate) detached_method_objects: HashMap<CallableId, ObjRef>,""",
    "semantic method registry comment",
)
