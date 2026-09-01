//! Durable package metadata, requirement, and artifact identity descriptors.

use crate::manifest::ValidatedProjectManifest;
use std::fmt;

/// Structured authorship metadata.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PackageAuthorDescriptor {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

impl PackageAuthorDescriptor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: None,
            url: None,
        }
    }

    /// Parses an author string formatted optionally as `"Name <email> (url)"`.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        let mut name = s;
        let mut email = None;
        let mut url = None;

        // Check for URL in parentheses: (url)
        if let Some(open_paren) = name.rfind('(') {
            if let Some(close_paren) = name[open_paren..].find(')') {
                let actual_close = open_paren + close_paren;
                let potential_url = &name[open_paren + 1..actual_close].trim();
                if !potential_url.is_empty() {
                    url = Some(potential_url.to_string());
                    name = name[..open_paren].trim();
                }
            }
        }

        // Check for email in angle brackets: <email>
        if let Some(open_angle) = name.rfind('<') {
            if let Some(close_angle) = name[open_angle..].find('>') {
                let actual_close = open_angle + close_angle;
                let potential_email = &name[open_angle + 1..actual_close].trim();
                if !potential_email.is_empty() {
                    email = Some(potential_email.to_string());
                    name = name[..open_angle].trim();
                }
            }
        }

        Self {
            name: name.trim().to_string(),
            email,
            url,
        }
    }
}

impl fmt::Display for PackageAuthorDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(email) = &self.email {
            write!(f, " <{}>", email)?;
        }
        if let Some(url) = &self.url {
            write!(f, " ({})", url)?;
        }
        Ok(())
    }
}

/// A durable, unresolved package requirement embedded in package metadata.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PackageRequirementDescriptor {
    /// Local import root used by this package's source.
    pub alias: Box<str>,
    /// Registry/distribution package name.
    pub package: String,
    /// Version constraint.
    pub version_requirement: String,
    /// Optionality flag.
    pub optional: bool,
}

/// Opaque durable package artifact identity.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub enum PackageArtifactIdentity {
    Builtin(Box<str>),
    Resolved { name: String, version: Option<String> },
    Standalone(Box<str>),
    Synthetic(Box<str>),
}

impl PackageArtifactIdentity {
    pub fn canonical_uri(&self) -> String {
        match self {
            Self::Builtin(name) => format!("pkg:{name}"),
            Self::Resolved { name, version } => {
                if let Some(ver) = version {
                    format!("pkg:{name}@{ver}")
                } else {
                    format!("pkg:{name}")
                }
            }
            Self::Standalone(name) => format!("pkg:{name}"),
            Self::Synthetic(name) => format!("pkg:{name}"),
        }
    }
}

impl fmt::Display for PackageArtifactIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_uri())
    }
}

/// Origin / provenance of a loaded package in a development or runtime environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PackageOrigin {
    Builtin,
    Workspace,
    Path,
    Registry,
    Vendored,
    Embedded,
}

impl fmt::Display for PackageOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin => write!(f, "builtin"),
            Self::Workspace => write!(f, "workspace"),
            Self::Path => write!(f, "path"),
            Self::Registry => write!(f, "registry"),
            Self::Vendored => write!(f, "vendored"),
            Self::Embedded => write!(f, "embedded"),
        }
    }
}

/// Immutable descriptive information associated with a root package artifact.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PackageInfoDescriptor {
    /// Distribution/package artifact name.
    pub name: String,
    /// Root language namespace.
    pub namespace: Box<str>,
    /// Package version where meaningful.
    pub version: Option<String>,
    /// Structured authorship metadata.
    pub authors: Vec<PackageAuthorDescriptor>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    /// Durable dependency requirements.
    pub requirements: Vec<PackageRequirementDescriptor>,
    /// Optional durable default executable entry.
    pub default_entry: Option<String>,
    /// Opaque package artifact identity.
    pub identity: PackageArtifactIdentity,
}

impl PackageInfoDescriptor {
    /// Constructs a `PackageInfoDescriptor` from a validated project manifest.
    pub fn from_manifest(manifest: &ValidatedProjectManifest) -> Self {
        let requirements = manifest
            .dependencies
            .iter()
            .map(|(comp, (orig_alias, spec))| match spec {
                crate::manifest::DependencySpec::Package { package, version } => PackageRequirementDescriptor {
                    alias: comp.as_str().to_string().into_boxed_str(),
                    package: package.clone(),
                    version_requirement: version.clone(),
                    optional: false,
                },
                crate::manifest::DependencySpec::Path { .. } => {
                    // For dev projects, normalize path dependencies if they specify alias
                    PackageRequirementDescriptor {
                        alias: comp.as_str().to_string().into_boxed_str(),
                        package: orig_alias.clone(),
                        version_requirement: "*".to_string(),
                        optional: false,
                    }
                }
            })
            .collect();

        let authors = manifest.authors.iter().map(|a| PackageAuthorDescriptor::parse(a)).collect();

        Self {
            name: manifest.name.clone(),
            namespace: manifest.namespace.as_str().to_string().into_boxed_str(),
            version: manifest.version.clone(),
            authors,
            description: manifest.description.clone(),
            license: manifest.license.clone(),
            homepage: manifest.homepage.clone(),
            repository: manifest.repository.clone(),
            requirements,
            default_entry: manifest.default_entry.clone().or_else(|| manifest.entry.clone()),
            identity: PackageArtifactIdentity::Resolved {
                name: manifest.name.clone(),
                version: manifest.version.clone(),
            },
        }
    }

    /// Constructs the canonical `PackageInfoDescriptor` for the builtin `universe` package.
    pub fn builtin_universe(version: Option<&str>) -> Self {
        Self {
            name: "universe".to_string(),
            namespace: "universe".to_string().into_boxed_str(),
            version: version.map(|v| v.to_string()).or_else(|| Some(env!("CARGO_PKG_VERSION").to_string())),
            authors: vec![],
            description: Some("Primordial language and object universe for Phalcom.".to_string()),
            license: Some("Apache-2.0 OR MIT".to_string()),
            homepage: None,
            repository: Some("https://github.com/aureat/phalcom-lang".to_string()),
            requirements: vec![],
            default_entry: None,
            identity: PackageArtifactIdentity::Builtin("universe".to_string().into_boxed_str()),
        }
    }

    /// Constructs a minimal `PackageInfoDescriptor` for a standalone package without a manifest.
    pub fn standalone(namespace: &str) -> Self {
        Self {
            name: namespace.to_string(),
            namespace: namespace.to_string().into_boxed_str(),
            version: None,
            authors: vec![],
            description: None,
            license: None,
            homepage: None,
            repository: None,
            requirements: vec![],
            default_entry: None,
            identity: PackageArtifactIdentity::Standalone(namespace.to_string().into_boxed_str()),
        }
    }
}

/// A dependency as resolved inside an active development project.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResolvedProjectDependencyDescriptor {
    pub alias: Box<str>,
    pub requirement: Option<PackageRequirementDescriptor>,
    pub package_info: PackageInfoDescriptor,
    pub origin: PackageOrigin,
}
