use super::support::{Fixture, monads_source};
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::store::TypeData;

/// MON-INHERIT-01/02: each generic superclass template passes through the same constructor-kinded parameter rather than replacing it with a proper type.
#[test]
fn generic_hierarchy_templates_preserve_constructor_parameter() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let applicative = f.info("Applicative").supertype_template.as_ref().expect("Applicative superclass template");
    let applicative_args = f.assert_applied(applicative.supertype, "Functor", 1);
    assert_eq!(applicative_args[0], f.type_parameter_form("Applicative", 0));

    let monad = f.info("Monad").supertype_template.as_ref().expect("Monad superclass template");
    let monad_args = f.assert_applied(monad.supertype, "Applicative", 1);
    assert_eq!(monad_args[0], f.type_parameter_form("Monad", 0));
}

/// MON-INHERIT-03/04: a concrete Either specialization projects through Monad -> Applicative -> Functor with one coherent constructor substitution.
#[test]
fn either_monad_projects_constructor_through_full_generic_hierarchy() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let string = f.ty("String");
    let (store, specialization) = f.specialize_receiver("EitherMonad", &[string], "Functor");
    let owners = specialization.path.iter().map(|step| step.owner.name.as_ref()).collect::<Vec<_>>();
    assert_eq!(owners, ["EitherMonad", "Monad", "Applicative", "Functor"]);

    let functor_f = store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(f.decl("Functor")), 0)
        .expect("Functor.F parameter");
    let constructor = specialization.environment.get_param(functor_f).expect("specialized Functor.F");
    assert!(matches!(store.get(constructor), TypeData::Lambda(_)), "Functor.F must remain a constructor lambda");
}

/// MON-INHERIT-05: substitution composes correctly through an additional non-generic concrete subclass hop.
#[test]
fn concrete_subclass_hop_preserves_higher_kinded_specialization() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let (store, specialization) = f.specialize_receiver("StringEitherMonad", &[], "Functor");
    let owners = specialization.path.iter().map(|step| step.owner.name.as_ref()).collect::<Vec<_>>();
    assert_eq!(owners, ["StringEitherMonad", "EitherMonad", "Monad", "Applicative", "Functor"]);

    let functor_f = store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(f.decl("Functor")), 0)
        .expect("Functor.F parameter");
    let constructor = specialization.environment.get_param(functor_f).expect("specialized Functor.F");
    let TypeData::Lambda(lambda_id) = store.get(constructor) else {
        panic!("expected a type-lambda specialization")
    };
    let lambda = store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")), "concrete subclass projection must preserve E = String: {free:#?}");
}
