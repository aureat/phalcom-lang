use crate::error::PhResult;
use crate::heap::Object;
use crate::value::repr::ValueTag;
use crate::vm::VM;
use std::fmt::{self, Debug, Display};

use super::Value;

impl Value {
    /// Renders this value the way `System.print` and `toString` present it.
    ///
    /// Strings render as their raw content (no quotes); numbers, booleans and the
    /// `nil` sentinel render as their literals; a symbol renders as `Symbol("…")`;
    /// a `List` renders as `[e1, e2, …]` (each element recursively via this same
    /// renderer); immediate `None` renders as `"None"` and an immediate `Some`
    /// value as `"Some(v)"` (`v` rendered recursively) — kept in agreement
    /// with the `.ph`-message `Option#toString` (U-CORE-4, R-INV-4.1). Any other
    /// heap object renders via its debug form. Returns an owned [`String`]
    /// rather than allocating a heap object.
    pub fn to_string(&self, vm: &VM) -> String {
        if self.is_some() {
            return render_option(*self, vm, false);
        }
        if self.is_nil() {
            return "nil".to_string();
        }
        if self.is_unit() {
            return "()".to_string();
        }
        if let Some(b) = self.as_bool() {
            return bool_literal(b).to_string();
        }
        if let Some(n) = self.as_int() {
            return n.to_string();
        }
        if let Some(n) = self.as_float() {
            return render_float(n);
        }
        if let Some(s) = self.symbol_value() {
            return s.to_string(vm);
        }
        if self.is_none() {
            return "None".to_string();
        }
        if let Some(id) = self.as_obj() {
            return match vm.heap.get(id) {
                Object::LargeInt(bigint) => bigint.to_string(),
                Object::Str(string) => string.value(),
                Object::List(list) => {
                    let parts: Vec<String> = list.elements().iter().map(|v| v.to_string(vm)).collect();
                    format!("[{}]", parts.join(", "))
                }
                // The `.ph` `Bytes#toString` (bytes.md §4) is the surface
                // spelling; this is the same debug form for the raw-render
                // path (echo of a receiver with no user override yet).
                Object::Bytes(bytes) => format!("Bytes({})", bytes.len()),
                Object::Map(map) => {
                    let parts: Vec<String> = map.entries().map(|(k, v)| format!("{}: {}", k.to_string(vm), v.to_string(vm))).collect();
                    format!("{{{}}}", parts.join(", "))
                }
                Object::Set(set) => {
                    let parts: Vec<String> = set.entries().map(|(k, _)| k.to_string(vm)).collect();
                    format!("Set({})", parts.join(", "))
                }
                Object::Tuple(tuple) => {
                    let parts: Vec<String> = tuple.values().iter().map(|v| v.to_string(vm)).collect();
                    format!("({})", parts.join(", "))
                }
                Object::Range(range) => {
                    let lower = range.lower().map_or_else(String::new, |value| value.to_string(vm));
                    let upper = range.upper().map_or_else(String::new, |value| value.to_string(vm));
                    let sep = if range.upper_inclusive() { "..=" } else { ".." };
                    format!("{lower}{sep}{upper}")
                }
                Object::Record(_) => "<record>".to_string(),
                _ => self.to_debug(vm),
            };
        }
        "<invalid value>".to_string()
    }

    /// Renders this value for display (`System.print`) by sending it a
    /// `toString` message, unconditionally.
    pub fn to_display_string(&self, vm: &mut VM) -> PhResult<String> {
        if self.is_int() {
            if vm.universe.int_tostring_pristine {
                return Ok(self.to_string(vm));
            }
        } else if self.is_float() {
            if vm.universe.float_tostring_pristine {
                return Ok(self.to_string(vm));
            }
        } else if self.is_symbol() {
            if vm.universe.symbol_tostring_pristine {
                return Ok(self.to_string(vm));
            }
        } else if let Some(id) = self.as_obj() {
            if vm.universe.str_tostring_pristine && matches!(vm.heap.get(id), Object::Str(_)) {
                return Ok(self.to_string(vm));
            }
            if vm.universe.int_tostring_pristine && matches!(vm.heap.get(id), Object::LargeInt(_)) {
                return Ok(self.to_string(vm));
            }
        }
        let selector = vm.get_or_intern("toString");
        let rendered = vm.send_dynamic(*self, selector, &[])?;
        Ok(rendered.to_string(vm))
    }

    /// Renders this value's debug form (used by error messages and diagnostics).
    ///
    /// Identical to [`Value::to_string`] except that a symbol renders as
    /// `<symbol N>`.
    pub fn to_debug(&self, vm: &VM) -> String {
        if self.is_some() {
            return render_option(*self, vm, true);
        }
        if self.is_nil() {
            return "nil".to_string();
        }
        if self.is_unit() {
            return "()".to_string();
        }
        if let Some(b) = self.as_bool() {
            return bool_literal(b).to_string();
        }
        if let Some(n) = self.as_int() {
            return n.to_string();
        }
        if let Some(n) = self.as_float() {
            return render_float(n);
        }
        if let Some(s) = self.symbol_value() {
            return s.to_debug();
        }
        if self.is_none() {
            return "None".to_string();
        }
        if let Some(id) = self.as_obj() {
            return match vm.heap.get(id) {
                Object::LargeInt(bigint) => bigint.to_string(),
                Object::Str(string) => string.value(),
                Object::Instance(instance) => instance.to_debug(&vm.heap),
                Object::Class(class) => class.to_debug(),
                Object::Method(method) => method.to_debug(vm),
                Object::Module(module) => module.to_debug(),
                Object::Closure(_) => "<closure>".to_string(),
                Object::Block(_) => "<closure>".to_string(),
                Object::BoundMethod(_) => "<bound method>".to_string(),
                Object::List(_) => "<list>".to_string(),
                Object::Bytes(_) => "<bytes>".to_string(),
                Object::Fiber(_) => "<fiber>".to_string(),
                Object::Map(_) => "<map>".to_string(),
                Object::Set(_) => "<set>".to_string(),
                Object::Tuple(_) => "<tuple>".to_string(),
                Object::Record(_) => "<record>".to_string(),
                Object::Range(_) => "<range>".to_string(),
                Object::Family(_) => "<family>".to_string(),
                Object::Selector(sel) => format!("Selector(#{})", sel.selector.encode()),
                Object::SelectorPattern(pat) => format!("SelectorPattern(#{})", pat.pattern.encode()),
                Object::MethodFamily(_) => "<method family>".to_string(),
                Object::BoundMethodFamily(_) => "<bound method family>".to_string(),
                Object::Upvalue(_) => "<upvalue>".to_string(),
                Object::PackBuilder(_) => "<internal pack builder>".to_string(),
                Object::RecordLiteralBuilder(_) => "<internal Record literal builder>".to_string(),
                Object::Project(proj) => format!("<Project {}>", proj.name),
                Object::ProjectManifest(m) => format!("<ProjectManifest {}>", m.name),
                Object::PackageInfo(info) => format!("<PackageInfo {}>", info.name),
                Object::PackageAuthor(a) => format!("<PackageAuthor {}>", a.name),
                Object::PackageRequirement(r) => format!("<PackageRequirement {}>", r.package),
                Object::ResolvedProjectDependency(d) => format!("<ResolvedProjectDependency {}>", vm.interner.lookup(d.alias)),
                Object::ModuleDependency(_) => "<ModuleDependency>".to_string(),
                Object::ExportTable(_) => "<ExportTable>".to_string(),
                Object::Export(e) => format!("<Export {}>", vm.interner.lookup(e.name)),
                Object::ChildModuleTable(_) => "<ChildModuleTable>".to_string(),
                Object::ModuleIdentity(id) => format!("<ModuleIdentity {}>", id.id_str),
                Object::PackageIdentity(id) => format!("<PackageIdentity {}>", id.identity_str),
                Object::ProjectIdentity(id) => format!("<ProjectIdentity {}>", id.identity_str),
                Object::Uri(u) => format!("<Uri {}>", u.uri_str),
            };
        }
        "<invalid value>".to_string()
    }
}

/// Returns the surface literal (`"true"` / `"false"`) for a boolean.
fn bool_literal(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

fn fmt_base_value(f: &mut fmt::Formatter<'_>, base: Value) -> fmt::Result {
    match base.tag() {
        ValueTag::Nil => write!(f, "nil"),
        ValueTag::Unit => write!(f, "()"),
        ValueTag::Bool => write!(f, "{}", base.as_bool().unwrap_or(false)),
        ValueTag::Int => write!(f, "{}", base.as_int().unwrap_or(0)),
        ValueTag::Float => write!(f, "{}", render_float(base.as_float().unwrap_or(0.0))),
        ValueTag::Symbol => write!(f, "Symbol({})", base.symbol_value().map(|s| s.0).unwrap_or(0)),
        ValueTag::Obj => {
            if let Some(id) = base.as_obj() {
                write!(f, "<obj {id:?}>")
            } else {
                write!(f, "<obj invalid>")
            }
        }
        ValueTag::None => write!(f, "None"),
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let depth = self.option_depth();
        for _ in 0..depth {
            write!(f, "Some(")?;
        }
        let base = self.without_some_wrappers();
        fmt_base_value(f, base)?;
        for _ in 0..depth {
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let depth = self.option_depth();
        for _ in 0..depth {
            write!(f, "Some(")?;
        }
        let base = self.without_some_wrappers();
        fmt_base_value(f, base)?;
        for _ in 0..depth {
            write!(f, ")")?;
        }
        Ok(())
    }
}

fn render_option(value: Value, vm: &VM, debug: bool) -> String {
    let depth = value.option_depth();
    let base = value.without_some_wrappers();
    let inner = if debug { base.to_debug(vm) } else { base.to_string(vm) };

    let depth_usize = usize::try_from(depth).unwrap_or(usize::MAX);
    let extra = depth_usize.saturating_mul(6);
    let mut out = String::with_capacity(inner.len().saturating_add(extra));

    for _ in 0..depth {
        out.push_str("Some(");
    }
    out.push_str(&inner);
    for _ in 0..depth {
        out.push(')');
    }
    out
}

/// Canonical shortest-roundtrip rendering for an `f64` value.
///
/// Matches the FLOAT-TEXT grammar from the spec:
/// - `NaN` for any IEEE 754 NaN.
/// - `Infinity` / `-Infinity` for positive/negative infinity.
/// - Shortest decimal that round-trips for all finite values (via `ryu`).
pub(crate) fn render_float(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n.is_infinite() {
        if n.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        }
    } else {
        let mut buf = ryu::Buffer::new();
        buf.format(n).to_string()
    }
}
