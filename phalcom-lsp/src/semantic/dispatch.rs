//! Canonical VM-free member dispatch for semantic analysis.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::callable::CallableSummary;
use super::facts::CapturedMethodFamilyShape;
use super::ids::{CallableId, ClassId, DispatchSide};
use super::surface::{ClassSurface, MemberSurface, MemberVisibility};
use phalcom_common::selector::{SelectorKind, SelectorKindPattern, SelectorPattern};

pub(crate) trait ClassTable {
    fn class(&self, id: &ClassId) -> Option<&ClassSurface>;
    fn contains_class(&self, id: &ClassId) -> bool;
}

impl ClassTable for BTreeMap<ClassId, ClassSurface> {
    fn class(&self, id: &ClassId) -> Option<&ClassSurface> {
        self.get(id)
    }

    fn contains_class(&self, id: &ClassId) -> bool {
        self.contains_key(id)
    }
}

impl ClassTable for BTreeMap<ClassId, Arc<ClassSurface>> {
    fn class(&self, id: &ClassId) -> Option<&ClassSurface> {
        self.get(id).map(Arc::as_ref)
    }

    fn contains_class(&self, id: &ClassId) -> bool {
        self.contains_key(id)
    }
}

pub(crate) trait SummaryTable {
    fn summary(&self, id: &CallableId) -> Option<&CallableSummary>;
}

impl SummaryTable for BTreeMap<CallableId, CallableSummary> {
    fn summary(&self, id: &CallableId) -> Option<&CallableSummary> {
        self.get(id)
    }
}

impl SummaryTable for BTreeMap<CallableId, Arc<CallableSummary>> {
    fn summary(&self, id: &CallableId) -> Option<&CallableSummary> {
        self.get(id).map(Arc::as_ref)
    }
}

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
    #[allow(dead_code)]
    pub receiver_class: ClassId,
    /// Dispatch side used for lookup.
    #[allow(dead_code)]
    pub side: DispatchSide,
}

/// Resolves members using the same side-aware inheritance walk for every
/// semantic consumer.
#[derive(Clone, Copy)]
pub(crate) struct DispatchResolver<'a, T: ?Sized> {
    classes: &'a T,
}

impl<'a, T: ClassTable + ?Sized> DispatchResolver<'a, T> {
    /// Creates a resolver over one coherent class-surface snapshot.
    pub(crate) fn new(classes: &'a T) -> Self {
        Self { classes }
    }

    pub(crate) fn member(&self, callable: &CallableId) -> Option<&'a MemberSurface> {
        self.classes.class(&callable.owner).and_then(|class| class.member_by_id(callable))
    }

    /// Reports whether a receiver class exists in this semantic surface.
    pub(crate) fn contains_class(&self, class: &ClassId) -> bool {
        self.classes.contains_class(class)
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
            let surface = self.classes.class(&class)?;
            if let Some(member) = surface.member(selector, side) {
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

    /// Captures an effective immutable snapshot of a method family starting from `receiver`.
    pub(crate) fn capture_method_family(
        &self,
        receiver: &DispatchReceiver,
        pattern: &SelectorPattern,
    ) -> CapturedMethodFamilyShape {
        let (source_behavior, side, start) = match receiver {
            DispatchReceiver::Instance(class) => (class.clone(), DispatchSide::Instance, Some(class.clone())),
            DispatchReceiver::ClassObject(class) => (class.clone(), DispatchSide::Class, Some(class.clone())),
            DispatchReceiver::Super { lexical_class, side } => {
                let start = self.superclass_of(lexical_class);
                (lexical_class.clone(), *side, start)
            }
        };

        let mut exact = Vec::new();
        let mut rest = Vec::new();
        let mut seen_exact = BTreeSet::new();

        let mut current = start;
        let mut visited = BTreeSet::new();

        while let Some(class) = current {
            if !visited.insert(class.clone()) {
                break;
            }
            let Some(surface) = self.classes.class(&class) else {
                break;
            };

            // Direct exact members matching pattern
            for member in surface.members_on(side) {
                if member.rest.is_none() && pattern.matches(&member.selector) {
                    let key = member.selector.encode();
                    if seen_exact.insert(key)
                        && (member.visibility == MemberVisibility::Public || member.visibility == MemberVisibility::Internal)
                    {
                        exact.push((member.selector.clone(), member.callable.clone()));
                    }
                }
            }

            // Direct rest methods matching pattern base & kind
            for member in surface.members_on(side) {
                if let Some(ref rest_surface) = member.rest {
                    if pattern.base == member.selector.base
                        && pattern_kind_matches(&pattern.kind, member.selector.kind)
                        && (member.visibility == MemberVisibility::Public || member.visibility == MemberVisibility::Internal)
                    {
                        rest.push((member.callable.clone(), rest_surface.clone()));
                    }
                }
            }

            current = self.superclass_of(&class);
        }

        // Sort exact deterministically by canonical selector encoding
        exact.sort_by(|a, b| a.0.cmp(&b.0));

        CapturedMethodFamilyShape {
            source_behavior,
            pattern: pattern.clone(),
            exact: exact.into_boxed_slice(),
            rest: rest.into_boxed_slice(),
        }
    }

    fn superclass_of(&self, class: &ClassId) -> Option<ClassId> {
        let surface = self.classes.class(class)?;
        surface
            .superclass
            .clone()
            .or_else(|| (class.name != "Object").then(|| ClassId::new(super::ids::ModuleId::new(super::ids::CORE_MODULE_URI), "Object")))
    }
}

fn pattern_kind_matches(pattern_kind: &SelectorKindPattern, selector_kind: SelectorKind) -> bool {
    match pattern_kind {
        SelectorKindPattern::AnyNamed => matches!(selector_kind, SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method),
        SelectorKindPattern::Exact(kind) => *kind == selector_kind,
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

    #[test]
    fn capture_method_family_overrides_and_rest_hierarchy() {
        let src = r#"
class GrandParent {
  foo(_ x) { "grand" }
  foo(_ x, to) { "grand-to" }
  sum(***all) { "grand-sum" }
}
class Parent is GrandParent {
  foo(_ x) { "parent" }
  foo(_ x, duration) { "parent-dur" }
  @private
  secret(_ x) { "secret" }
}
class Child is Parent {
  foo(_ x) { "child" }
  sum(*numbers) { "child-sum" }
}
"#;
        let classes = surfaces(src);
        let child = ClassId::new(super::super::ids::ModuleId::new("file:///dispatch.ph"), "Child");
        let resolver = DispatchResolver::new(&classes);

        let pattern_foo = SelectorPattern::named_method("foo", [], [], true).unwrap();
        let family_foo = resolver.capture_method_family(&DispatchReceiver::Instance(child.clone()), &pattern_foo);

        assert_eq!(family_foo.exact.len(), 3);
        // child foo(_) shadows parent and grandparent
        assert_eq!(family_foo.exact[0].0.encode(), "foo(_)");
        assert_eq!(family_foo.exact[0].1.owner.name, "Child");
        // parent foo(_,duration)
        assert_eq!(family_foo.exact[1].0.encode(), "foo(_,duration)");
        assert_eq!(family_foo.exact[1].1.owner.name, "Parent");
        // grandparent foo(_,to)
        assert_eq!(family_foo.exact[2].0.encode(), "foo(_,to)");
        assert_eq!(family_foo.exact[2].1.owner.name, "GrandParent");

        let pattern_sum = SelectorPattern::named_method("sum", [], [], true).unwrap();
        let family_sum = resolver.capture_method_family(&DispatchReceiver::Instance(child), &pattern_sum);
        assert_eq!(family_sum.rest.len(), 2);
        // child sum(*) first
        assert_eq!(family_sum.rest[0].0.owner.name, "Child");
        // grandparent sum(***) second
        assert_eq!(family_sum.rest[1].0.owner.name, "GrandParent");
    }

    #[test]
    fn private_shadow_does_not_leak_superclass() {
        let src = r#"
class Parent {
  secret(_ x) { "parent-secret" }
}
class Child is Parent {
  @private
  secret(_ x) { "child-secret" }
}
"#;
        let classes = surfaces(src);
        let child = ClassId::new(super::super::ids::ModuleId::new("file:///dispatch.ph"), "Child");
        let resolver = DispatchResolver::new(&classes);

        let pattern = SelectorPattern::named_method("secret", [], [], true).unwrap();
        let family = resolver.capture_method_family(&DispatchReceiver::Instance(child), &pattern);

        // Child's private secret shadows parent's secret, so exact should NOT contain Parent::secret
        assert_eq!(family.exact.len(), 0);
    }
}
