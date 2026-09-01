use crate::identity::DeclarationId;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath};
use phalcom_native_meta::UniverseKey;

fn universe_module(components: &[&str]) -> ModuleId {
    let components = components
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
        .collect::<Vec<_>>();
    ModuleId::universe(ModulePath::from_components(components))
}

/// Returns declaration identity owned by canonical Universe source module.
pub fn universe_declaration(key: UniverseKey) -> DeclarationId {
    DeclarationId::new(universe_module(key.source_path()), key.name().into())
}

/// Canonical declaration IDs for core types and forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreDeclarationIds {
    pub object: DeclarationId,
    pub bool_: DeclarationId,
    pub int: DeclarationId,
    pub float: DeclarationId,
    pub string: DeclarationId,
    pub symbol: DeclarationId,
    pub number: DeclarationId,
    pub list: DeclarationId,
    pub set: DeclarationId,
    pub map: DeclarationId,
    pub function: DeclarationId,
    pub closure: DeclarationId,
    pub option: DeclarationId,
    pub result: DeclarationId,
    pub ordering: DeclarationId,
}

impl Default for CoreDeclarationIds {
    fn default() -> Self {
        Self {
            object: universe_declaration(UniverseKey::Object),
            bool_: universe_declaration(UniverseKey::Bool),
            int: universe_declaration(UniverseKey::Int),
            float: universe_declaration(UniverseKey::Float),
            string: universe_declaration(UniverseKey::String),
            symbol: universe_declaration(UniverseKey::Symbol),
            number: universe_declaration(UniverseKey::Number),
            list: universe_declaration(UniverseKey::List),
            set: universe_declaration(UniverseKey::Set),
            map: universe_declaration(UniverseKey::Map),
            function: universe_declaration(UniverseKey::Function),
            closure: universe_declaration(UniverseKey::Closure),
            option: universe_declaration(UniverseKey::Option),
            result: universe_declaration(UniverseKey::Result),
            ordering: universe_declaration(UniverseKey::Ordering),
        }
    }
}

impl CoreDeclarationIds {
    pub fn is_object(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.object
    }

    pub fn is_callable_supertype(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.function || declaration == &self.closure || declaration == &self.object
    }

    pub fn is_option(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.option
    }

    pub fn is_result(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.result
    }

    pub fn is_ordering(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.ordering
    }

    pub fn is_core_adt(&self, declaration: &DeclarationId) -> bool {
        self.is_option(declaration) || self.is_result(declaration) || self.is_ordering(declaration)
    }
}
