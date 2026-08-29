use crate::identity::DeclarationId;
use phalcom_modules::identity::ModuleId;

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
}

impl Default for CoreDeclarationIds {
    fn default() -> Self {
        let module = ModuleId::core();
        Self {
            object: DeclarationId::new(module.clone(), "Object".into()),
            bool_: DeclarationId::new(module.clone(), "Bool".into()),
            int: DeclarationId::new(module.clone(), "Int".into()),
            float: DeclarationId::new(module.clone(), "Float".into()),
            string: DeclarationId::new(module.clone(), "String".into()),
            symbol: DeclarationId::new(module.clone(), "Symbol".into()),
            number: DeclarationId::new(module.clone(), "Number".into()),
            list: DeclarationId::new(module.clone(), "List".into()),
            set: DeclarationId::new(module.clone(), "Set".into()),
            map: DeclarationId::new(module.clone(), "Map".into()),
            function: DeclarationId::new(module.clone(), "Function".into()),
            closure: DeclarationId::new(module, "Closure".into()),
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
}
