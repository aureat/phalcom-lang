use crate::identity::DeclarationId;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath};
use phalcom_native_meta::UniverseKey;

pub(crate) fn universe_module(components: &[&str]) -> ModuleId {
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
    pub char_: DeclarationId,
    pub symbol: DeclarationId,
    pub number: DeclarationId,
    pub list: DeclarationId,
    pub array: DeclarationId,
    pub vector: DeclarationId,
    pub slice: DeclarationId,
    pub buffer: DeclarationId,
    pub set: DeclarationId,
    pub mutable_bag: DeclarationId,
    pub indexed_dict: DeclarationId,
    pub ordered_set: DeclarationId,
    pub bag: DeclarationId,
    pub bijection: DeclarationId,
    pub map: DeclarationId,
    pub range: DeclarationId,
    pub bytes: DeclarationId,
    pub tuple: DeclarationId,
    pub pack: DeclarationId,
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
            char_: universe_declaration(UniverseKey::Char),
            symbol: universe_declaration(UniverseKey::Symbol),
            number: universe_declaration(UniverseKey::Number),
            list: universe_declaration(UniverseKey::List),
            array: universe_declaration(UniverseKey::Array),
            vector: universe_declaration(UniverseKey::Vector),
            slice: universe_declaration(UniverseKey::Slice),
            buffer: universe_declaration(UniverseKey::Buffer),
            set: universe_declaration(UniverseKey::Set),
            mutable_bag: universe_declaration(UniverseKey::MutableBag),
            indexed_dict: universe_declaration(UniverseKey::IndexedDict),
            ordered_set: universe_declaration(UniverseKey::OrderedSet),
            bag: universe_declaration(UniverseKey::Bag),
            bijection: universe_declaration(UniverseKey::Bijection),
            map: universe_declaration(UniverseKey::Map),
            range: universe_declaration(UniverseKey::Range),
            bytes: universe_declaration(UniverseKey::Bytes),
            tuple: universe_declaration(UniverseKey::Tuple),
            pack: universe_declaration(UniverseKey::Pack),
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

/// Canonical declaration IDs for normative structural capabilities and reflection forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralUniverseIds {
    pub product: DeclarationId,
    pub ordered_product: DeclarationId,
    pub unordered_product: DeclarationId,
    pub sum: DeclarationId,

    pub product_kind: DeclarationId,
    pub product_shape: DeclarationId,
    pub ordered_tail: DeclarationId,
    pub ordered_shape: DeclarationId,

    pub presence: DeclarationId,
    pub field_info: DeclarationId,
    pub product_field: DeclarationId,
    pub unordered_tail: DeclarationId,
    pub unordered_shape: DeclarationId,
    pub field_lookup: DeclarationId,
    pub unordered_shape_view: DeclarationId,

    pub variant: DeclarationId,
}

impl Default for StructuralUniverseIds {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuralUniverseIds {
    pub fn new() -> Self {
        let structural = universe_module(&["reflection", "structural"]);
        Self {
            product: DeclarationId::new(structural.clone(), "Product".into()),
            ordered_product: DeclarationId::new(structural.clone(), "OrderedProduct".into()),
            unordered_product: DeclarationId::new(structural.clone(), "UnorderedProduct".into()),
            sum: DeclarationId::new(structural.clone(), "Sum".into()),

            product_kind: DeclarationId::new(structural.clone(), "ProductKind".into()),
            product_shape: DeclarationId::new(structural.clone(), "ProductShape".into()),
            ordered_tail: DeclarationId::new(structural.clone(), "OrderedTail".into()),
            ordered_shape: DeclarationId::new(structural.clone(), "OrderedShape".into()),

            presence: DeclarationId::new(structural.clone(), "Presence".into()),
            field_info: DeclarationId::new(structural.clone(), "FieldInfo".into()),
            product_field: DeclarationId::new(structural.clone(), "ProductField".into()),
            unordered_tail: DeclarationId::new(structural.clone(), "UnorderedTail".into()),
            unordered_shape: DeclarationId::new(structural.clone(), "UnorderedShape".into()),
            field_lookup: DeclarationId::new(structural.clone(), "FieldLookup".into()),
            unordered_shape_view: DeclarationId::new(structural.clone(), "UnorderedShapeView".into()),

            variant: DeclarationId::new(structural, "Variant".into()),
        }
    }

    pub fn is_structural_capability(&self, decl: &DeclarationId) -> bool {
        decl == &self.product || decl == &self.ordered_product || decl == &self.unordered_product || decl == &self.sum
    }

    pub fn is_canonical_structural_capability(decl: &DeclarationId) -> bool {
        static IDS: std::sync::OnceLock<StructuralUniverseIds> = std::sync::OnceLock::new();
        IDS.get_or_init(StructuralUniverseIds::new).is_structural_capability(decl)
    }
}
