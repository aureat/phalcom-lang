//! Presentation IR for canonical core classes and methods.

use super::merge::{MergedClassSurface, SurfaceMergeOutcome};
use crate::identity::{DeclarationId, DispatchSide};
use phalcom_native_meta::{EffectSpec, ImplementationKind, NativeIntrinsicId, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_native_surface::NativeSurfaceId;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodPresentation {
    pub selector: String,
    pub side: DispatchSide,
    pub display_signature: String,
    pub implementation_kind: ImplementationKind,
    pub native_id: Option<NativeSurfaceId>,
    pub intrinsic: Option<NativeIntrinsicId>,
    pub effects: Option<String>,
    pub raises: RaisesSpec,
    pub flow: ReturnFlowSpec,
    pub lifecycle: NativeLifecycleSpec,
    pub documentation: Option<String>,
    pub conceptual: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassPresentation {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub superclass: Option<String>,
    pub members: Vec<MethodPresentation>,
    pub documentation: Option<String>,
}

impl ClassPresentation {
    pub fn from_merged(merged: &MergedClassSurface<'_>) -> Self {
        let mut members = Vec::new();

        for ((side, sel), outcome) in &merged.members {
            match outcome {
                SurfaceMergeOutcome::NativeOnly(n) => {
                    let effects_str = match n.effects() {
                        EffectSpec::Pure => Some("pure".to_string()),
                        EffectSpec::Known(k) => Some(format!("{:?}", k)),
                        EffectSpec::Unknown => None,
                    };
                    members.push(MethodPresentation {
                        selector: sel.clone(),
                        side: *side,
                        display_signature: sel.to_string(),
                        implementation_kind: ImplementationKind::NativePrimitive,
                        native_id: Some(n.id()),
                        intrinsic: n.intrinsic(),
                        effects: effects_str,
                        raises: n.raises(),
                        flow: n.flow(),
                        lifecycle: n.lifecycle(),
                        documentation: n.docs().map(|s| s.to_string()),
                        conceptual: n.conceptual().map(|s| s.to_string()),
                    });
                }
                SurfaceMergeOutcome::Generated(n) => {
                    let effects_str = match n.effects() {
                        EffectSpec::Pure => Some("pure".to_string()),
                        EffectSpec::Known(k) => Some(format!("{:?}", k)),
                        EffectSpec::Unknown => None,
                    };
                    members.push(MethodPresentation {
                        selector: sel.clone(),
                        side: *side,
                        display_signature: sel.to_string(),
                        implementation_kind: ImplementationKind::Generated,
                        native_id: Some(n.id()),
                        intrinsic: n.intrinsic(),
                        effects: effects_str,
                        raises: n.raises(),
                        flow: n.flow(),
                        lifecycle: n.lifecycle(),
                        documentation: n.docs().map(|s| s.to_string()),
                        conceptual: n.conceptual().map(|s| s.to_string()),
                    });
                }
                SurfaceMergeOutcome::SourceOnly(s) => {
                    members.push(MethodPresentation {
                        selector: sel.clone(),
                        side: *side,
                        display_signature: sel.to_string(),
                        implementation_kind: ImplementationKind::Source,
                        native_id: None,
                        intrinsic: None,
                        effects: None,
                        raises: RaisesSpec::Unknown,
                        flow: ReturnFlowSpec::Unknown,
                        lifecycle: NativeLifecycleSpec::UNKNOWN,
                        documentation: s.doc_comment.clone(),
                        conceptual: None,
                    });
                }
                SurfaceMergeOutcome::SourceDeclarationNativeImplementation { source, native } => {
                    let doc = source.doc_comment.clone().or_else(|| native.docs().map(|s| s.to_string()));
                    members.push(MethodPresentation {
                        selector: sel.clone(),
                        side: *side,
                        display_signature: sel.to_string(),
                        implementation_kind: ImplementationKind::NativePrimitive,
                        native_id: Some(native.id()),
                        intrinsic: native.intrinsic(),
                        effects: if native.effects() == EffectSpec::Pure {
                            Some("pure".to_string())
                        } else {
                            None
                        },
                        raises: native.raises(),
                        flow: native.flow(),
                        lifecycle: native.lifecycle(),
                        documentation: doc,
                        conceptual: native.conceptual().map(|s| s.to_string()),
                    });
                }
                SurfaceMergeOutcome::SourceWrapperOverNative { source, native } => {
                    members.push(MethodPresentation {
                        selector: sel.clone(),
                        side: *side,
                        display_signature: sel.to_string(),
                        implementation_kind: ImplementationKind::Source,
                        native_id: Some(native.id()),
                        intrinsic: native.intrinsic(),
                        effects: None,
                        raises: native.raises(),
                        flow: native.flow(),
                        lifecycle: native.lifecycle(),
                        documentation: source.doc_comment.clone().or_else(|| native.docs().map(str::to_owned)),
                        conceptual: native.conceptual().map(str::to_owned),
                    });
                }
                SurfaceMergeOutcome::Conflict { source, native, .. } => {
                    members.push(MethodPresentation {
                        selector: sel.clone(),
                        side: *side,
                        display_signature: sel.to_string(),
                        implementation_kind: ImplementationKind::Source,
                        native_id: Some(native.id()),
                        intrinsic: None,
                        effects: None,
                        raises: RaisesSpec::Unknown,
                        flow: ReturnFlowSpec::Unknown,
                        lifecycle: NativeLifecycleSpec::UNKNOWN,
                        documentation: source.doc_comment.clone(),
                        conceptual: None,
                    });
                }
            }
        }

        members.sort_by(|a, b| (a.side, &a.selector).cmp(&(b.side, &b.selector)));

        ClassPresentation {
            declaration_id: merged.declaration_id.clone(),
            name: merged.name.clone(),
            superclass: merged.superclass.clone(),
            members,
            documentation: merged.doc_comment().map(str::to_owned),
        }
    }

    /// Renders concise markdown overview.
    pub fn render_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("## class {}\n\n", self.name));
        if let Some(ref sup) = self.superclass {
            out.push_str(&format!("*Extends `{sup}`*\n\n"));
        }
        if let Some(ref doc) = self.documentation {
            out.push_str(&format!("{}\n\n", doc));
        }
        out.push_str("### Methods\n\n");
        for m in &self.members {
            let side_badge = match m.side {
                DispatchSide::Instance => "",
                DispatchSide::Class => "*(static)* ",
            };
            let mut badges = Vec::new();
            if m.implementation_kind == ImplementationKind::NativePrimitive {
                badges.push("native");
            }
            if m.implementation_kind == ImplementationKind::Generated {
                badges.push("generated");
            }
            if let Some(_intrin) = m.intrinsic {
                badges.push("intrinsic");
            }
            if let Some(ref eff) = m.effects {
                badges.push(eff.as_str());
            }
            let badge_str = if badges.is_empty() {
                String::new()
            } else {
                format!(" · `{}`", badges.join(" · "))
            };

            out.push_str(&format!("- {}{}{}\n", side_badge, m.selector, badge_str));
            if let Some(ref doc) = m.documentation {
                out.push_str(&format!("  > {}\n", doc.trim()));
            }
        }
        out
    }

    /// Renders virtual Phalcom source.
    pub fn render_virtual_source(&self) -> String {
        let mut out = String::new();
        out.push_str("// Generated Canonical Core Surface — Read Only\n");
        let sup = self.superclass.as_deref().map(|s| format!(" is {s}")).unwrap_or_default();
        out.push_str(&format!("class {}{} {{\n", self.name, sup));
        for m in &self.members {
            if m.implementation_kind == ImplementationKind::NativePrimitive {
                out.push_str("  @native\n");
            }
            if let Some(intrin) = m.intrinsic {
                out.push_str(&format!("  @intrinsic({:?})\n", intrin));
            }
            if let Some(ref eff) = m.effects {
                out.push_str(&format!("  @{}\n", eff));
            }
            let static_kw = if m.side == DispatchSide::Class { "static " } else { "" };
            out.push_str(&format!("  {}{}\n\n", static_kw, m.selector));
        }
        out.push_str("}\n");
        out
    }
}

/// Renders the stable read-only source document used to present canonical
/// builtin declaration provenance. This text is a presentation product only:
/// it is never linked, type checked, or executed.
pub fn render_canonical_core_source() -> Arc<str> {
    let mut bindings = phalcom_native_meta::universe::UNIVERSE_BINDINGS.iter().collect::<Vec<_>>();
    bindings.sort_by(|left, right| left.name.cmp(right.name));

    let mut out = String::from("// Generated Canonical Core Surface — Read Only\n");
    out.push_str("// Semantic identities and runtime behavior remain compiler-owned.\n\n");
    for binding in bindings {
        out.push_str("class ");
        out.push_str(binding.name);
        out.push_str(" {}\n\n");
    }
    Arc::from(out)
}
