//! Canonical VM-free member dispatch for semantic analysis.

use std::collections::{BTreeMap, BTreeSet};

use super::ids::{CallableId, ClassId, DispatchSide};
use super::surface::{ClassSurface, MemberSurface};

/// Receiver target used by semantic dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DispatchReceiver {
    /// An ordinary instance receiver.
    Instance(ClassId),
    /// An ordinary class-object receiver.
    ClassObject(ClassId),
    /// A `super` send whose lookup starts above the lexical class.
    Super { lexical_class: ClassId, side: DispatchSide },
}

/// One member resolved through inheritance and dispatch side.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedDispatch {
    /// Callable ID of the actual declaration.
    pub callable: CallableId,
    /// Class represented by the receiver expression.
    pub receiver_class: ClassId,
    /// Dispatch side used for lookup.
    #[cfg_attr(not(test), expect(dead_code, reason = "retained for side-aware dispatch regression assertions"))]
    pub side: DispatchSide,
}

/// Resolves members using the same side-aware inheritance walk for every
/// semantic consumer.
#[derive(Clone, Copy)]
pub(crate) struct DispatchResolver<'a> {
    classes: &'a BTreeMap<ClassId, ClassSurface>,
}

impl<'a> DispatchResolver<'a> {
    /// Creates a resolver over one coherent class-surface snapshot.
    pub(crate) fn new(classes: &'a BTreeMap<ClassId, ClassSurface>) -> Self {
        Self { classes }
    }

    pub(crate) fn member(&self, callable: &CallableId) -> Option<&'a MemberSurface> {
        self.classes
            .get(&callable.owner)
            .and_then(|class| class.members_by_side.get(&(callable.selector.clone(), callable.side)))
    }

    /// Reports whether a receiver class exists in this semantic surface.
    pub(crate) fn contains_class(&self, class: &ClassId) -> bool {
        self.classes.contains_key(class)
    }

    /// Resolves `selector` and returns the actual declaration owner.
    pub(crate) fn resolve(&self, receiver: &DispatchReceiver, selector: &str) -> Option<ResolvedDispatch> {
        let (receiver_class, side, start) = match receiver {
            DispatchReceiver::Instance(class) => (class.clone(), DispatchSide::Instance, Some(class.clone())),
            DispatchReceiver::ClassObject(class) => (class.clone(), DispatchSide::Class, Some(class.clone())),
            DispatchReceiver::Super { lexical_class, side } => {
                let start = self.superclass_of(lexical_class);
                (lexical_class.clone(), *side, start)
            }
        };

        let mut current = start;
        let mut visited = BTreeSet::new();
        while let Some(class) = current {
            if !visited.insert(class.clone()) {
                return None;
            }
            let surface = self.classes.get(&class)?;
            if let Some(member) = surface.members_by_side.get(&(selector.to_string(), side)) {
                return Some(ResolvedDispatch {
                    callable: member.callable.clone(),
                    receiver_class,
                    side,
                });
            }
            current = self.superclass_of(&class);
        }
        None
    }

    fn superclass_of(&self, class: &ClassId) -> Option<ClassId> {
        let surface = self.classes.get(class)?;
        surface
            .superclass
            .clone()
            .or_else(|| (class.name != "Object").then(|| ClassId::new(super::ids::ModuleId::new(super::ids::CORE_MODULE_URI), "Object")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::surface::build_module_surface;
    use phalcom_ast::parser::parse;

    fn surfaces(source: &str) -> BTreeMap<ClassId, ClassSurface> {
        let module = super::super::ids::ModuleId::new("file:///dispatch.ph");
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "unexpected parse errors: {:?}", parsed.errors);
        build_module_surface(module, &parsed.program).classes
    }

    #[test]
    fn resolves_inherited_member_to_declaring_owner() {
        let classes = surfaces("class Parent { value() { } }\nclass Child is Parent { }\n");
        let child = ClassId::new(super::super::ids::ModuleId::new("file:///dispatch.ph"), "Child");
        let resolver = DispatchResolver::new(&classes);

        let resolved = resolver.resolve(&DispatchReceiver::Instance(child), "value()").expect("inherited member");

        assert_eq!(resolved.callable.owner.name, "Parent");
        assert_eq!(resolved.side, DispatchSide::Instance);
    }

    #[test]
    fn preserves_instance_and_class_side_collisions() {
        let classes = surfaces("class Widget { make() { } @class make() { } }\n");
        let widget = ClassId::new(super::super::ids::ModuleId::new("file:///dispatch.ph"), "Widget");
        let resolver = DispatchResolver::new(&classes);

        let instance = resolver
            .resolve(&DispatchReceiver::Instance(widget.clone()), "make()")
            .expect("instance member");
        let class = resolver.resolve(&DispatchReceiver::ClassObject(widget), "make()").expect("class member");

        assert_eq!(instance.side, DispatchSide::Instance);
        assert_eq!(class.side, DispatchSide::Class);
        assert_ne!(instance.callable, class.callable);
    }

    #[test]
    fn super_starts_at_parent_and_preserves_side() {
        let classes = surfaces("class Parent { value() { } @class make() { } }\nclass Child is Parent { value() { } @class make() { } }\n");
        let child = ClassId::new(super::super::ids::ModuleId::new("file:///dispatch.ph"), "Child");
        let resolver = DispatchResolver::new(&classes);

        let instance = resolver
            .resolve(
                &DispatchReceiver::Super {
                    lexical_class: child.clone(),
                    side: DispatchSide::Instance,
                },
                "value()",
            )
            .expect("super instance member");
        let class = resolver
            .resolve(
                &DispatchReceiver::Super {
                    lexical_class: child,
                    side: DispatchSide::Class,
                },
                "make()",
            )
            .expect("super class member");

        assert_eq!(instance.callable.owner.name, "Parent");
        assert_eq!(class.callable.owner.name, "Parent");
        assert_eq!(instance.side, DispatchSide::Instance);
        assert_eq!(class.side, DispatchSide::Class);
    }
}
