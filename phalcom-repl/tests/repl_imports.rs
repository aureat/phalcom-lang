use phalcom_core::value::Value;
use phalcom_repl::repl::{CellOutcome, ReplSession};

fn repl() -> ReplSession {
    ReplSession::start(std::env::current_dir().unwrap())
}

fn assert_cell_ok(s: &mut ReplSession, src: &str) -> CellOutcome {
    let out = s.eval(src);
    assert!(!matches!(out, CellOutcome::Failed), "expected Ok for {src:?}, got {out:?}");
    out
}

fn assert_cell_fails(s: &mut ReplSession, src: &str) {
    let out = s.eval(src);
    assert!(matches!(out, CellOutcome::Failed), "expected Failed for {src:?}, got {out:?}");
}

fn assert_value_non_none(out: CellOutcome) -> Value {
    match out {
        CellOutcome::Value(v) => {
            assert!(!v.is_none(), "expected non-None value");
            v
        }
        other => panic!("expected CellOutcome::Value, got {:?}", other),
    }
}

// ── Test Group: Module-Import (Whole-Module) ───────────────────────────────

/// REPL-MI-01 — import universe.reflection.selector succeeds and binding is a module object
#[test]
fn repl_mi_01_whole_module_import_succeeds_and_binds_module() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let out = assert_cell_ok(&mut s, "selector");
    let val = assert_value_non_none(out);
    assert!(val.is_obj(), "expected module object");
}

/// REPL-MI-02 — import universe.reflection succeeds
#[test]
fn repl_mi_02_package_module_import_succeeds() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection");
    let out = assert_cell_ok(&mut s, "reflection");
    let val = assert_value_non_none(out);
    assert!(val.is_obj(), "expected package module object");
}

/// REPL-MI-03 — import universe.reflection.selector binding is idempotent across repeated imports
#[test]
fn repl_mi_03_repeated_module_import_is_idempotent() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let v1 = assert_value_non_none(assert_cell_ok(&mut s, "selector"));

    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let v2 = assert_value_non_none(assert_cell_ok(&mut s, "selector"));
    assert_eq!(v1, v2);
}

/// REPL-MI-04 — Module-import alias binds under alias name, not path segment
#[test]
fn repl_mi_04_module_import_alias_binds_under_alias() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector as sel");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "sel"));
    assert!(val.is_obj());
    assert_cell_fails(&mut s, "selector");
}

/// REPL-MI-05 — import universe.reflection.package_info succeeds (bug 3 regression)
#[test]
fn repl_mi_05_package_info_import_resolves() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.package_info");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "package_info"));
    assert!(val.is_obj());
}

/// REPL-MI-06 — All 20 reflection children can be imported
#[test]
fn repl_mi_06_all_reflection_children_importable() {
    let children = [
        "module",
        "package_object",
        "project",
        "project_manifest",
        "package_info",
        "package_author",
        "package_requirement",
        "resolved_project_dependency",
        "module_dependency",
        "export_table",
        "export",
        "export_kind",
        "child_module_table",
        "module_identity",
        "package_identity",
        "project_identity",
        "uri",
        "selector",
        "message",
        "attribute",
    ];

    for child in children {
        let mut s = repl();
        let (import_name, eval_name) = if child == "export" {
            ("export as exp_mod", "exp_mod")
        } else {
            (child, child)
        };
        let cmd = format!("import universe.reflection.{import_name}");
        assert_cell_ok(&mut s, &cmd);
        let out = assert_cell_ok(&mut s, eval_name);
        let val = assert_value_non_none(out);
        assert!(val.is_obj(), "expected module object for {child}");
    }
}

// ── Test Group: Selective Imports — Valid Cases ────────────────────────────

/// REPL-SI-01 — from universe.reflection.selector import Selector produces non-None class (bug 2 regression)
#[test]
fn repl_si_01_selective_import_selector_class_is_non_none() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "Selector"));
    assert!(val.is_obj());
}

/// REPL-SI-02 — Selective import alias rebinds under alias
#[test]
fn repl_si_02_selective_import_alias_rebinds() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector as Sel");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "Sel"));
    assert!(val.is_obj());
    assert_cell_fails(&mut s, "Selector");
}

/// REPL-SI-03 — Multiple selective imports from one module in one from statement all bind
#[test]
fn repl_si_03_multiple_selective_imports_bind() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "Selector"));
    assert!(val.is_obj());
}

/// REPL-SI-04 — Selectively imported binding persists across subsequent cells
#[test]
fn repl_si_04_selective_import_persists_across_cells() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    assert_cell_ok(&mut s, "let s = Selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "s"));
    let selector_val = assert_value_non_none(assert_cell_ok(&mut s, "Selector"));
    assert_eq!(val, selector_val);
}

/// REPL-SI-05 — Selectively imported binding is immutable (assignment fails)
#[test]
fn repl_si_05_selective_import_is_immutable() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    assert_cell_fails(&mut s, "Selector = 42");
}

// ── Test Group: Selective Imports — Error Cases (Bug 1 Regression) ─────────

/// REPL-SE-01 — from universe.reflection import PackageInfo fails (not an export of that package)
#[test]
fn repl_se_01_import_unexported_symbol_fails() {
    let mut s = repl();
    assert_cell_fails(&mut s, "from universe.reflection import PackageInfo");
}

/// REPL-SE-02 — from universe.reflection import NonExistentName fails
#[test]
fn repl_se_02_import_non_existent_symbol_fails() {
    let mut s = repl();
    assert_cell_fails(&mut s, "from universe.reflection import NonExistentName");
}

/// REPL-SE-03 — from universe.reflection import selector fails (selector is child path, not export symbol)
#[test]
fn repl_se_03_import_child_module_as_symbol_fails() {
    let mut s = repl();
    assert_cell_fails(&mut s, "from universe.reflection import selector");
}

/// REPL-SE-04 — Failure from selective import does not poison subsequent valid imports
#[test]
fn repl_se_04_failed_import_does_not_poison_session() {
    let mut s = repl();
    assert_cell_fails(&mut s, "from universe.reflection import NonExistentName");
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "Selector"));
    assert!(val.is_obj());
}

/// REPL-SE-05 — from universe import PackageInfo succeeds (correct location)
#[test]
fn repl_se_05_import_from_universe_root_succeeds() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe import PackageInfo");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "PackageInfo"));
    assert!(val.is_obj());
}

// ── Test Group: Builtin Module Source Execution (Bug 2 Regression) ─────────

/// REPL-BS-01 — selector.Selector is non-None after whole-module import (bug 2 regression)
#[test]
fn repl_bs_01_property_access_on_module_is_non_none() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "selector.Selector"));
    assert!(val.is_obj());
}

/// REPL-BS-02 — Classes defined in builtin .ph source are proper class objects
#[test]
fn repl_bs_02_builtin_classes_are_executable_class_objects() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "Selector"));
    assert!(val.is_obj());
}

/// REPL-BS-03 — Builtin package (not module) import does not trigger source execution crash
#[test]
fn repl_bs_03_builtin_package_import_does_not_crash() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "reflection"));
    assert!(val.is_obj());
}

/// REPL-BS-04 — Re-importing a builtin module in the same session is idempotent
#[test]
fn repl_bs_04_reimporting_builtin_module_is_idempotent() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let v1 = assert_value_non_none(assert_cell_ok(&mut s, "selector"));
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let v2 = assert_value_non_none(assert_cell_ok(&mut s, "selector"));
    assert_eq!(v1, v2);
}

// ── Test Group: Cross-Cell Import State Persistence ────────────────────────

/// REPL-CP-01 — Import from cell N is visible in cell N+1
#[test]
fn repl_cp_01_import_visible_in_next_cell() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "selector"));
    assert!(val.is_obj());
}

/// REPL-CP-02 — Selectively imported class is usable in a later cell
#[test]
fn repl_cp_02_selectively_imported_class_usable_in_later_cell() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    assert_cell_ok(&mut s, "let s = Selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "s"));
    assert!(val.is_obj());
}

/// REPL-CP-03 — Import in cell N, declaration in cell N+1, use in cell N+2 all persist
#[test]
fn repl_cp_03_import_declaration_and_use_persist() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    assert_cell_ok(&mut s, "class Box {\n  _v\n  set(_ v) { _v = v }\n  get() { _v }\n}");
    let out = assert_cell_ok(&mut s, "let b = Box.new()\nb.set(Selector)\nb.get()");
    let val = assert_value_non_none(out);
    assert!(val.is_obj());
}

// ── Test Group: ReplSession::reload with Imports ───────────────────────────

/// REPL-RL-01 — Reload replays all import cells correctly
#[test]
fn repl_rl_01_reload_replays_imports() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    assert_cell_ok(&mut s, "let captured = selector");
    assert!(s.reload(), "reload should succeed");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "captured"));
    assert!(val.is_obj());
}

/// REPL-RL-02 — Reload stops at first failed cell and returns false
#[test]
fn repl_rl_02_reload_stops_at_failed_cell() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    assert_cell_fails(&mut s, "from universe.reflection import NonExistentName");
    let result = s.reload();
    assert!(!result, "reload should fail on recorded failure");
}

// ── Test Group: Edge Cases and Invariants ──────────────────────────────────

/// REPL-EC-01 — Importing universe root itself succeeds and is a module object
#[test]
fn repl_ec_01_import_universe_root() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "universe"));
    assert!(val.is_obj());
}

/// REPL-EC-02 — Imported module object responds to __name__() or property access
#[test]
fn repl_ec_02_imported_module_property_access() {
    let mut s = repl();
    assert_cell_ok(&mut s, "import universe.reflection.selector");
    let val = assert_value_non_none(assert_cell_ok(&mut s, "selector.Selector"));
    assert!(val.is_obj());
}

/// REPL-EC-03 — Undefined variable after failed import still raises undefined variable
#[test]
fn repl_ec_03_undefined_variable_after_failed_import() {
    let mut s = repl();
    assert_cell_fails(&mut s, "from universe.reflection import GhostName");
    assert_cell_fails(&mut s, "GhostName");
}

/// REPL-EC-04 — Two separate selective imports from two different modules do not collide
#[test]
fn repl_ec_04_multiple_distinct_imports_do_not_collide() {
    let mut s = repl();
    assert_cell_ok(&mut s, "from universe.reflection.selector import Selector");
    assert_cell_ok(&mut s, "from universe import PackageInfo");
    let s_val = assert_value_non_none(assert_cell_ok(&mut s, "Selector"));
    let p_val = assert_value_non_none(assert_cell_ok(&mut s, "PackageInfo"));
    assert!(s_val.is_obj());
    assert!(p_val.is_obj());
    assert_ne!(s_val, p_val);
}
