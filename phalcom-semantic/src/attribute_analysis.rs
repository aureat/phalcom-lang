//! Stable identity and retained semantic products for executable attribute
//! arguments.

use crate::checker::CallableAnalysis;
use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId};
use phalcom_ast::ast::{Attribute, ClassMember, Expr, Program, Statement};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Canonical source owner of an attribute site.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AttributeOwnerId {
    Module(ModuleId),
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
}

/// Context in which the compiler evaluates an attribute argument.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AttributeAnalysisContext {
    /// Attribute construction runs as part of module initialization.
    ModuleInitialization,
    /// A method contract is retained as a receiver-shaped closure.
    RequiresContract,
    /// A method contract is retained as a receiver-shaped closure.
    EnsuresContract,
    /// A class invariant is woven into one receiver callable in Debug mode.
    ClassInvariant,
}

/// Runtime mode used when projecting semantic evidence into executable phase
/// requirements. Semantic checking itself remains mode-independent.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AttributeExecutionMode {
    /// All runtime contracts and invariants are emitted.
    #[default]
    Debug,
    /// Requirements for stripped ensures contracts and invariants are omitted.
    Release,
    /// Requirements for all stripped contract guards are omitted.
    Unchecked,
}

impl AttributeAnalysisContext {
    /// Returns whether this site's expression is executed in the given mode.
    pub const fn is_emitted_in(self, mode: AttributeExecutionMode) -> bool {
        match self {
            Self::ModuleInitialization => true,
            Self::RequiresContract => !matches!(mode, AttributeExecutionMode::Unchecked),
            Self::EnsuresContract | Self::ClassInvariant => matches!(mode, AttributeExecutionMode::Debug),
        }
    }
}

/// Stable identity for one argument of one attribute occurrence.
///
/// The range is deliberately excluded: it is source metadata and can change
/// when preceding text is edited. `occurrence` distinguishes repeated
/// attributes with the same name on one canonical owner.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AttributeArgumentId {
    pub owner: AttributeOwnerId,
    pub attribute_name: Box<str>,
    pub occurrence: u32,
    pub argument: u32,
    pub context: AttributeAnalysisContext,
    /// Callable receiving a class invariant. Invariants are shared source
    /// expressions woven into multiple callables, so this is part of identity.
    pub consumer_callable: Option<CallableId>,
}

impl AttributeArgumentId {
    pub fn module(&self) -> &ModuleId {
        match &self.owner {
            AttributeOwnerId::Module(module) => module,
            AttributeOwnerId::Declaration(declaration) => &declaration.module,
            AttributeOwnerId::Callable(callable) => callable.module(),
            AttributeOwnerId::Field(field) => &field.owner.module,
        }
    }
}

/// Canonical checker evidence captured for one executable attribute argument.
#[derive(Clone, Debug)]
pub struct AttributeArgumentAnalysis {
    pub id: AttributeArgumentId,
    pub analysis: Arc<CallableAnalysis>,
}

/// One source argument whose execution context is currently modeled by the
/// semantic attribute query.
#[derive(Clone, Debug)]
pub struct AttributeArgumentSite {
    pub id: AttributeArgumentId,
    pub module: ModuleId,
    pub expression: Expr,
    /// Present for receiver-shaped method contract closures.
    pub callable_context: Option<CallableId>,
}

/// Enumerates class and class-member attribute arguments with canonical owners.
/// Other declaration forms remain on the conservative phase-evidence path
/// until their execution context is explicitly modeled here.
pub fn collect_supported_attribute_argument_sites(module: &ModuleId, program: &Program) -> Vec<AttributeArgumentSite> {
    let mut sites = Vec::new();
    let mut invariant_sites = Vec::new();
    let mut occurrences = BTreeMap::<(AttributeOwnerId, Box<str>, AttributeAnalysisContext), u32>::new();

    let mut append = |owner: AttributeOwnerId, attribute: &Attribute, context: AttributeAnalysisContext, callable_context: Option<CallableId>| {
        let occurrence_key = (owner.clone(), attribute.name.clone().into_boxed_str(), context);
        let occurrence = occurrences.entry(occurrence_key).or_default();
        for (argument, expression) in attribute.args.iter().enumerate() {
            sites.push(AttributeArgumentSite {
                id: AttributeArgumentId {
                    owner: owner.clone(),
                    attribute_name: attribute.name.clone().into_boxed_str(),
                    occurrence: *occurrence,
                    argument: argument as u32,
                    context,
                    consumer_callable: (context == AttributeAnalysisContext::ClassInvariant)
                        .then(|| callable_context.clone())
                        .flatten(),
                },
                module: module.clone(),
                expression: expression.clone(),
                callable_context: callable_context.clone(),
            });
        }
        *occurrence = occurrence.saturating_add(1);
    };

    for statement in &program.statements {
        let Statement::Class(class) = statement else { continue };
        let declaration = DeclarationId::new(module.clone(), class.name.clone().into());
        for attribute in &class.attributes {
            // Class invariants are retained separately and woven into
            // instance method bodies by core; they are not evaluated as
            // module-initialization attributes.
            if attribute.name == "invariant" {
                continue;
            }
            append(
                AttributeOwnerId::Declaration(declaration.clone()),
                attribute,
                AttributeAnalysisContext::ModuleInitialization,
                None,
            );
        }
        for member in &class.members {
            let (owner, callable) = match member {
                ClassMember::Field(field) => (
                    AttributeOwnerId::Field(FieldId::new(
                        declaration.clone(),
                        field.name.clone(),
                        if field.is_static {
                            crate::identity::DispatchSide::Class
                        } else {
                            crate::identity::DispatchSide::Instance
                        },
                    )),
                    None,
                ),
                ClassMember::Method(_) | ClassMember::Getter(_) | ClassMember::Setter(_) | ClassMember::Index(_) => {
                    let Some(callable) = crate::checker::declaration_signature::callable_id_for_member(&declaration, member) else {
                        continue;
                    };
                    (AttributeOwnerId::Callable(callable.clone()), Some(callable))
                }
                ClassMember::Delegation(_) | ClassMember::TypeMember(_) => continue,
            };
            for attribute in member.attributes() {
                let contract_context = match (callable.as_ref(), attribute.name.as_str()) {
                    (Some(_), "requires") => Some(AttributeAnalysisContext::RequiresContract),
                    (Some(_), "ensures") => Some(AttributeAnalysisContext::EnsuresContract),
                    _ => None,
                };
                if let Some(context) = contract_context {
                    append(owner.clone(), attribute, context, callable.clone());
                } else {
                    append(owner.clone(), attribute, AttributeAnalysisContext::ModuleInitialization, None);
                }
            }
        }

        // Core weaves each standalone invariant into every non-static method,
        // getter, and setter that has a body. Keep one retained semantic query
        // per consumer callable so receiver and generic context stay canonical.
        for (occurrence, (expression, _range)) in class.invariants.iter().enumerate() {
            for member in &class.members {
                let callable = match member {
                    ClassMember::Method(method) if !method.is_static && method.body.statements().is_some() => {
                        crate::checker::declaration_signature::callable_id_for_member(&declaration, member)
                    }
                    ClassMember::Getter(getter) if !getter.is_static && getter.body.statements().is_some() => {
                        crate::checker::declaration_signature::callable_id_for_member(&declaration, member)
                    }
                    ClassMember::Setter(setter) if !setter.is_static && setter.body.statements().is_some() => {
                        crate::checker::declaration_signature::callable_id_for_member(&declaration, member)
                    }
                    _ => None,
                };
                let Some(callable) = callable else { continue };
                invariant_sites.push(AttributeArgumentSite {
                    id: AttributeArgumentId {
                        owner: AttributeOwnerId::Declaration(declaration.clone()),
                        attribute_name: "invariant".into(),
                        occurrence: occurrence as u32,
                        argument: 0,
                        context: AttributeAnalysisContext::ClassInvariant,
                        consumer_callable: Some(callable.clone()),
                    },
                    module: module.clone(),
                    expression: expression.clone(),
                    callable_context: Some(callable),
                });
            }
        }
    }
    sites.extend(invariant_sites);
    sites
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{ModuleComponent, ModulePath, ResolvedProjectId};

    fn module() -> ModuleId {
        ModuleId::resolved(
            ResolvedProjectId::from_raw(71),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("attributes").expect("valid module name")]),
        )
    }

    fn sites(source: &str) -> Vec<AttributeArgumentSite> {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "fixture parse errors: {:#?}", parsed.errors);
        collect_supported_attribute_argument_sites(&module(), &parsed.program)
    }

    #[test]
    fn class_field_and_contract_arguments_get_distinct_semantic_contexts() {
        let sites = sites("@custom(1)\nclass Box {\n  @requires(self.value) read() {}\n  @ensures(self.value) read2() {}\n  @custom(2) _value: Int\n}\n");

        assert_eq!(sites.len(), 4);
        assert_eq!(sites[0].id.context, AttributeAnalysisContext::ModuleInitialization);
        assert!(matches!(sites[0].id.owner, AttributeOwnerId::Declaration(_)));
        assert_eq!(sites[1].id.context, AttributeAnalysisContext::RequiresContract);
        assert!(matches!(sites[1].id.owner, AttributeOwnerId::Callable(_)));
        assert!(sites[1].callable_context.is_some());
        assert_eq!(sites[2].id.context, AttributeAnalysisContext::EnsuresContract);
        assert!(matches!(sites[2].id.owner, AttributeOwnerId::Callable(_)));
        assert_eq!(sites[3].id.context, AttributeAnalysisContext::ModuleInitialization);
        assert!(matches!(sites[3].id.owner, AttributeOwnerId::Field(_)));
    }

    #[test]
    fn argument_identity_does_not_depend_on_source_ranges() {
        let compact = sites("@custom(1)\nclass Box {}\n");
        let shifted = sites("\n\n@custom(1)\nclass Box {}\n");

        assert_eq!(compact[0].id, shifted[0].id);
        assert_ne!(compact[0].expression.range(), shifted[0].expression.range());
    }

    #[test]
    fn repeated_attributes_have_distinct_occurrence_identity() {
        let sites = sites("@custom(1)\n@custom(2)\nclass Box {}\n");

        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].id.attribute_name, sites[1].id.attribute_name);
        assert_ne!(sites[0].id.occurrence, sites[1].id.occurrence);
    }

    #[test]
    fn class_invariants_are_not_module_initialization_attributes() {
        let sites = sites("@invariant(self.ready)\n@custom(1)\nclass Box {}\n");

        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].id.attribute_name.as_ref(), "custom");
        assert_eq!(sites[0].id.context, AttributeAnalysisContext::ModuleInitialization);
    }

    #[test]
    fn class_invariants_get_distinct_receiver_callable_sites() {
        let sites = sites("class Box {\n  @invariant(self.ready)\n  read() {}\n  other() {}\n}\n");

        let invariants = sites
            .iter()
            .filter(|site| site.id.context == AttributeAnalysisContext::ClassInvariant)
            .collect::<Vec<_>>();
        assert_eq!(invariants.len(), 2);
        assert!(invariants.iter().all(|site| site.callable_context.is_some()));
        assert_ne!(invariants[0].id.consumer_callable, invariants[1].id.consumer_callable);
        assert_eq!(invariants[0].id.occurrence, 0);
        assert_eq!(invariants[1].id.occurrence, 0);
    }

    #[test]
    fn runtime_guard_contexts_match_compile_mode_stripping() {
        use AttributeAnalysisContext::{ClassInvariant, EnsuresContract, ModuleInitialization, RequiresContract};
        use AttributeExecutionMode::{Debug, Release, Unchecked};

        assert!(ModuleInitialization.is_emitted_in(Debug));
        assert!(ModuleInitialization.is_emitted_in(Release));
        assert!(ModuleInitialization.is_emitted_in(Unchecked));
        assert!(RequiresContract.is_emitted_in(Debug));
        assert!(RequiresContract.is_emitted_in(Release));
        assert!(!RequiresContract.is_emitted_in(Unchecked));
        assert!(EnsuresContract.is_emitted_in(Debug));
        assert!(!EnsuresContract.is_emitted_in(Release));
        assert!(!EnsuresContract.is_emitted_in(Unchecked));
        assert!(ClassInvariant.is_emitted_in(Debug));
        assert!(!ClassInvariant.is_emitted_in(Release));
        assert!(!ClassInvariant.is_emitted_in(Unchecked));
    }
}
