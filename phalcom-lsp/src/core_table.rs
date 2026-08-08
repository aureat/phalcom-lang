//! Builtin core-class selector table, for receiver-aware completion.
//!
//! Stage 3 (`docs/forge/units/U-LSP/plan.md` "Stage 3", ADR-0056 §4). When a
//! completion's receiver resolves to a **builtin** class (`Bool`, `String`,
//! `List`, …) rather than a user-defined one, its member list is not in the
//! [`WorkspaceIndex`](crate::index::WorkspaceIndex) — the index only walks
//! user `.ph` source. It comes from this table instead, the same
//! `core-table.json` the VS Code extension's `completions.ts` consumed
//! (U-VSPHALCOM `gen-core-table`).
//!
//! The JSON is **embedded at build time** via [`include_str!`] from
//! `tools/vsphalcom/src/generated/core-table.json`, so the running server
//! needs no filesystem lookup and cannot ship without its builtin table. This
//! crate stays VM-free (ADR-0056 §2): the table is plain data, not a link to
//! `phalcom-core`.
//!
//! Every selector here is already in ADR-0012 comma-form (the `gen-core-table`
//! gate enforces it), so it drops straight into the same rendering path
//! `crate::completion` uses for user-class selectors from `crate::selectors`.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// The dispatch kind of a class member: how it is spelled and rendered.
///
/// Mirrors the `"kind"` field of each `core-table.json` entry
/// (`"getter"`/`"setter"`/`"method"`) and is also produced from user-class
/// AST members by [`crate::index`], so builtin and user completions render
/// through one shared path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberKind {
    /// A bare-name getter (`size`) — no parens, no argument slots.
    Getter,
    /// A `name=(put)` setter — a single-argument write, rendered `name = value`.
    Setter,
    /// A method (`move(_,to,duration)`) — parenthesized, zero or more slots.
    Method,
    /// A static (metaclass) method — a class-side message. Rendered like a
    /// [`Method`](Self::Method); carried as its own variant so a future pass
    /// can filter static members out of *instance*-receiver completion
    /// (`core-table.json`'s `"static-method"` kind).
    #[serde(rename = "static-method")]
    StaticMethod,
    /// A `@constructor new(...)` factory — a class-side constructor. Rendered
    /// like a [`Method`](Self::Method) (`core-table.json`'s `"construct"`
    /// kind).
    Construct,
}

impl MemberKind {
    /// Whether this kind renders with a parenthesized, argument-slot snippet
    /// (methods, static methods, constructs) rather than bare-name / setter
    /// forms.
    pub fn is_method_like(self) -> bool {
        matches!(self, MemberKind::Method | MemberKind::StaticMethod | MemberKind::Construct)
    }
}

/// One builtin class member: its ADR-0012 comma-form selector and dispatch
/// kind.
///
/// The `core-table.json` `"source"` field (`"native"`/`"core"`) is not
/// consulted by completion and is dropped on deserialization (serde ignores
/// unknown fields by default).
#[derive(Debug, Clone, Deserialize)]
pub struct CoreMember {
    /// The member's ADR-0012 comma-form selector (e.g. `at(_,put)`).
    pub selector: String,
    /// Whether the member is a getter, setter, or method.
    pub kind: MemberKind,
    /// Whether the receiver is the class object rather than an instance.
    #[serde(default, rename = "classSide")]
    pub class_side: bool,
    /// Source/runtime visibility carried by generated metadata.
    #[serde(default)]
    pub visibility: CoreVisibility,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Visibility encoded in generated core metadata.
pub enum CoreVisibility {
    /// Accessible from ordinary source.
    #[default]
    Public,
    /// Accessible only in defining lexical class.
    Private,
    /// Accessible in defining class and subclasses.
    Protected,
    /// Accessible only to privileged core/runtime source.
    Internal,
}

/// The parsed builtin core-class table: `class name -> member list`.
///
/// The `core-table.json` top-level `"keywords"` array is not consulted by
/// completion and is dropped on deserialization.
#[derive(Debug, Clone, Deserialize)]
pub struct CoreTable {
    /// Each builtin class's own declared members, keyed by class name.
    ///
    /// The list is the class's *own* surface, as harvested by
    /// `gen-core-table`; builtin superclass chains are not encoded in the
    /// JSON, so this table does not resolve inherited builtin members (see
    /// [`crate::completion`] for how inheritance is handled where it can be).
    #[serde(default)]
    pub classes: HashMap<String, Vec<CoreMember>>,
}

/// The raw `core-table.json` bytes, embedded at build time.
const CORE_TABLE_JSON: &str = include_str!("../../tools/vsphalcom/src/generated/core-table.json");

impl CoreTable {
    /// Returns the process-wide, lazily-parsed builtin table embedded from
    /// `core-table.json`.
    ///
    /// Parsed once on first call and cached for the server's lifetime — the
    /// table is immutable build-time data, so no reload path exists.
    ///
    /// # Panics
    ///
    /// Panics if the embedded `core-table.json` is not valid JSON of the
    /// expected shape. This is a build-time invariant (the file is generated
    /// and checked in), so a failure here is a broken build, not a runtime
    /// condition a client can trigger.
    pub fn bundled() -> &'static CoreTable {
        static TABLE: OnceLock<CoreTable> = OnceLock::new();
        TABLE.get_or_init(|| serde_json::from_str(CORE_TABLE_JSON).expect("embedded core-table.json must be valid CoreTable JSON"))
    }

    /// The member list of the builtin class named `class`, or `None` if no
    /// such builtin class exists.
    pub fn class_members(&self, class: &str) -> Option<&[CoreMember]> {
        self.classes.get(class).map(Vec::as_slice)
    }

    /// Every builtin member across every class, de-duplicated by selector
    /// (first-seen kind wins) and sorted by selector for a deterministic
    /// completion order.
    ///
    /// This is the "receiver type unknown" fallback set — the full builtin
    /// surface, matching the pre-LSP `completions.ts` behavior so Stage 3 is
    /// never worse than the status quo when it cannot resolve a receiver.
    pub fn all_members(&self) -> Vec<CoreMember> {
        let mut merged: std::collections::BTreeMap<String, CoreMember> = std::collections::BTreeMap::new();
        for members in self.classes.values() {
            for member in members {
                merged
                    .entry(member.selector.clone())
                    .and_modify(|existing| {
                        if existing.visibility != CoreVisibility::Public && member.visibility == CoreVisibility::Public {
                            *existing = member.clone();
                        }
                    })
                    .or_insert_with(|| member.clone());
            }
        }
        merged.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_parses_and_has_known_classes() {
        let table = CoreTable::bundled();
        // `Bool` is present in the generated table (read directly 2026-07-13).
        assert!(table.classes.contains_key("Bool"));
        assert!(table.classes.contains_key("Behavior"));
    }

    #[test]
    fn member_kinds_deserialize_from_lowercase_strings() {
        let table = CoreTable::bundled();
        let behavior = table.class_members("Behavior").expect("Behavior present");
        // `name` is a getter, `superclass=(put)` is a setter (per the fixture).
        assert!(behavior.iter().any(|m| m.selector == "name" && m.kind == MemberKind::Getter));
        assert!(behavior.iter().any(|m| m.selector == "superclass=(put)" && m.kind == MemberKind::Setter));
    }

    #[test]
    fn all_members_is_deduped_and_sorted() {
        let all = CoreTable::bundled().all_members();
        // Sorted, strictly ascending (dedup guarantees no equal neighbours).
        for pair in all.windows(2) {
            assert!(pair[0].selector < pair[1].selector, "not sorted/deduped");
        }
        assert!(!all.is_empty());
    }

    #[test]
    fn unknown_class_has_no_members() {
        assert!(CoreTable::bundled().class_members("NoSuchClass").is_none());
    }
}
