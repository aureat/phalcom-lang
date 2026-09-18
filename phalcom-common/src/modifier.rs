//! Shared declaration modifier and visibility categories.

/// Source-level and semantic visibility of declarations and members.
///
/// Absence of an explicit visibility modifier in source defaults to [`Visibility::Public`].
/// Accessibility is computed by comparing caller context against declaration provenance.
/// Note that visibility does NOT form a simple total order (`Protected` is class-hierarchy
/// oriented while `Module`, `Package`, `Internal` are provenance-oriented).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Protected,
    Module,
    Package,
    Internal,
}

impl Visibility {
    pub const fn is_public(self) -> bool {
        matches!(self, Visibility::Public)
    }

    pub const fn is_private(self) -> bool {
        matches!(self, Visibility::Private)
    }

    pub const fn is_protected(self) -> bool {
        matches!(self, Visibility::Protected)
    }

    pub const fn is_module(self) -> bool {
        matches!(self, Visibility::Module)
    }

    pub const fn is_package(self) -> bool {
        matches!(self, Visibility::Package)
    }

    pub const fn is_internal(self) -> bool {
        matches!(self, Visibility::Internal)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Protected => "protected",
            Visibility::Module => "module",
            Visibility::Package => "package",
            Visibility::Internal => "internal",
        }
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Implementation origin of a declaration.
///
/// Absence of an explicit modifier in source defaults to [`ImplementationKind::Phalcom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImplementationKind {
    #[default]
    Phalcom,
    Native,
    Foreign,
}

impl ImplementationKind {
    pub const fn is_phalcom(self) -> bool {
        matches!(self, ImplementationKind::Phalcom)
    }

    pub const fn is_native(self) -> bool {
        matches!(self, ImplementationKind::Native)
    }

    pub const fn is_foreign(self) -> bool {
        matches!(self, ImplementationKind::Foreign)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            ImplementationKind::Phalcom => "phalcom",
            ImplementationKind::Native => "native",
            ImplementationKind::Foreign => "foreign",
        }
    }
}

impl std::fmt::Display for ImplementationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
