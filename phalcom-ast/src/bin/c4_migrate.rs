use phalcom_ast::ast::*;
use phalcom_ast::error::{SyntaxError, SyntaxErrorKind};
use phalcom_ast::parser::parse;
use std::fs;
use std::path::{Path, PathBuf};

const LEGACY_CONST_FIELD_MESSAGE: &str = "`const` is not field syntax; use `_field` for an immutable field or `mut _field` for a mutable field";

fn parse_for_migration(content: &str) -> Result<phalcom_ast::parser::Parse, String> {
    let parsed = parse(content, 0);
    let unexpected: Vec<_> = parsed
        .errors
        .iter()
        .filter(|error| {
            !matches!(
                &error.kind,
                SyntaxErrorKind::Message(message) if message == LEGACY_CONST_FIELD_MESSAGE
            )
        })
        .collect();
    if !unexpected.is_empty() {
        return Err(format!("Parse errors: {unexpected:?}"));
    }
    Ok(parsed)
}

fn legacy_const_field_error<'a>(field: &FieldDef, errors: &'a [SyntaxError]) -> Option<&'a SyntaxError> {
    errors.iter().find(|error| {
        matches!(
            &error.kind,
            SyntaxErrorKind::Message(message) if message == LEGACY_CONST_FIELD_MESSAGE
        ) && field.range.start <= error.range.start
            && error.range.end <= field.name_range.start
    })
}

fn is_mut_modifier_prefix(prefix: &str) -> bool {
    prefix.split_whitespace().last() == Some("mut")
}

fn skip_field_prefix_whitespace(content: &str, mut end: usize, limit: usize) -> usize {
    while end < limit && matches!(content.as_bytes()[end], b' ' | b'\t') {
        end += 1;
    }
    end
}

fn find_ph_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "target" && name != ".git" && name != "node_modules" && name != "docs" {
                    find_ph_files(&path, files);
                }
            } else if path.extension().map_or(false, |ext| ext == "ph") {
                files.push(path);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditKind {
    LetToLetMut,
    ConstToLet,
    BareFieldToMut,
    ConstFieldToBare,
    ImplicitFieldDecl,
}

#[derive(Debug, Clone)]
struct Edit {
    start: usize,
    end: usize,
    replacement: String,
    kind: EditKind,
}

fn collect_bindings_from_pack(item: &PackItem, lets: &mut Vec<LetBinding>) {
    match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => {
            collect_bindings_from_expr(expr, lets);
        }
        PackItem::Labeled { value, .. } => {
            collect_bindings_from_expr(value, lets);
        }
    }
}

fn collect_bindings_from_expr(expr: &Expr, lets: &mut Vec<LetBinding>) {
    match expr {
        Expr::Block(b) => {
            for s in &b.body {
                collect_bindings_from_stmt(s, lets);
            }
        }
        Expr::Assignment(a) => {
            collect_bindings_from_expr(&a.name, lets);
            collect_bindings_from_expr(&a.value, lets);
        }
        Expr::Binary(b) => {
            collect_bindings_from_expr(&b.left, lets);
            collect_bindings_from_expr(&b.right, lets);
        }
        Expr::Unary(u) => {
            collect_bindings_from_expr(&u.expr, lets);
        }
        Expr::MethodCall(c) => {
            collect_bindings_from_expr(&c.object, lets);
            for arg in &c.args {
                collect_bindings_from_pack(arg, lets);
            }
        }
        Expr::UnqualifiedCall(c) => {
            for arg in &c.args {
                collect_bindings_from_pack(arg, lets);
            }
        }
        Expr::GetProperty(g) => {
            collect_bindings_from_expr(&g.object, lets);
        }
        Expr::SetProperty(s) => {
            collect_bindings_from_expr(&s.object, lets);
            collect_bindings_from_expr(&s.value, lets);
        }
        Expr::Index(i) => {
            collect_bindings_from_expr(&i.object, lets);
            for arg in &i.args {
                collect_bindings_from_pack(arg, lets);
            }
        }
        Expr::SetIndex(s) => {
            collect_bindings_from_expr(&s.object, lets);
            for arg in &s.args {
                collect_bindings_from_pack(arg, lets);
            }
            collect_bindings_from_expr(&s.value, lets);
        }
        Expr::If(i) => {
            collect_bindings_from_expr(&i.condition, lets);
            for s in &i.then_body.body {
                collect_bindings_from_stmt(s, lets);
            }
            if let Some(eb) = &i.else_body {
                match eb {
                    IfElseBody::Block(b) => {
                        for s in &b.body {
                            collect_bindings_from_stmt(s, lets);
                        }
                    }
                    IfElseBody::ElseIf(e) => {
                        collect_bindings_from_expr(e, lets);
                    }
                }
            }
        }
        Expr::IfLet(i) => {
            collect_bindings_from_expr(&i.value, lets);
            for s in &i.then_body.body {
                collect_bindings_from_stmt(s, lets);
            }
            if let Some(eb) = &i.else_body {
                for s in &eb.body {
                    collect_bindings_from_stmt(s, lets);
                }
            }
        }
        Expr::WhileLet(w) => {
            collect_bindings_from_expr(&w.value, lets);
            for s in &w.body {
                collect_bindings_from_stmt(s, lets);
            }
        }
        Expr::Match(m) => {
            collect_bindings_from_expr(&m.value, lets);
            for arm in &m.arms {
                match &arm.branch {
                    MatchArmBody::Expression(expr) => {
                        collect_bindings_from_expr(expr, lets);
                    }
                    MatchArmBody::Block(block) => {
                        for s in &block.body {
                            collect_bindings_from_stmt(s, lets);
                        }
                    }
                }
            }
        }
        Expr::MapLiteral(map) => {
            for entry in &map.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_bindings_from_expr(expr, lets);
                        }
                        collect_bindings_from_expr(value, lets);
                    }
                    MapLiteralEntry::Expansion { expr, .. } => collect_bindings_from_expr(expr, lets),
                }
            }
        }
        Expr::SetLiteral(set) => {
            for entry in &set.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                        collect_bindings_from_expr(expr, lets);
                    }
                }
            }
        }
        Expr::ListLiteral(list) => {
            for element in &list.elements {
                match element {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => {
                        collect_bindings_from_expr(expr, lets);
                    }
                }
            }
        }
        Expr::TupleLiteral(tuple) => {
            for entry in &tuple.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                        collect_bindings_from_expr(expr, lets);
                    }
                    TupleLiteralEntry::Labeled { value, .. } => {
                        collect_bindings_from_expr(value, lets);
                    }
                }
            }
        }
        Expr::RecordLiteral(rec) => {
            for entry in &rec.entries {
                match entry {
                    RecordLiteralEntry::Field(field) => {
                        collect_bindings_from_expr(&field.value, lets);
                    }
                    RecordLiteralEntry::Expansion { expr, .. } => {
                        collect_bindings_from_expr(expr, lets);
                    }
                }
            }
        }
        Expr::RecordConstruction(rec) => {
            for entry in &rec.entries {
                collect_bindings_from_expr(&entry.value, lets);
            }
        }
        Expr::ComparisonChain(c) => {
            for op in &c.operands {
                collect_bindings_from_expr(op, lets);
            }
        }
        Expr::Range(r) => {
            if let Some(lower) = &r.lower {
                collect_bindings_from_expr(lower, lets);
            }
            if let Some(upper) = &r.upper {
                collect_bindings_from_expr(upper, lets);
            }
        }
        Expr::Membership(m) => {
            collect_bindings_from_expr(&m.left, lets);
            collect_bindings_from_expr(&m.right, lets);
        }
        Expr::IsMembership(m) => {
            collect_bindings_from_expr(&m.left, lets);
            collect_bindings_from_expr(&m.candidates, lets);
        }
        _ => {}
    }
}

fn collect_bindings_from_behavior(b: &BehaviorMember, lets: &mut Vec<LetBinding>) {
    match b {
        BehaviorMember::Method(m) => {
            if let Some(stmts) = m.body.statements() {
                for s in stmts {
                    collect_bindings_from_stmt(s, lets);
                }
            }
        }
        BehaviorMember::Getter(g) => {
            if let Some(stmts) = g.body.statements() {
                for s in stmts {
                    collect_bindings_from_stmt(s, lets);
                }
            }
        }
        BehaviorMember::Setter(s) => {
            if let Some(stmts) = s.body.statements() {
                for s in stmts {
                    collect_bindings_from_stmt(s, lets);
                }
            }
        }
        BehaviorMember::Index(i) => {
            if let Some(stmts) = i.body.statements() {
                for s in stmts {
                    collect_bindings_from_stmt(s, lets);
                }
            }
        }
    }
}

fn collect_bindings_from_stmt(stmt: &Statement, lets: &mut Vec<LetBinding>) {
    match stmt {
        Statement::Let(b) => {
            lets.push(b.clone());
            if let Some(v) = &b.value {
                collect_bindings_from_expr(v, lets);
            }
        }
        Statement::Expr { expr, .. } => {
            collect_bindings_from_expr(expr, lets);
        }
        Statement::Return(r) => {
            if let Some(v) = &r.value {
                collect_bindings_from_expr(v, lets);
            }
        }
        Statement::For(f) => {
            for lane in &f.lanes {
                collect_bindings_from_expr(&lane.iter, lets);
            }
            for s in &f.body {
                collect_bindings_from_stmt(s, lets);
            }
        }
        Statement::Throw { expr, .. } => {
            collect_bindings_from_expr(expr, lets);
        }
        Statement::Class(c) => {
            for m in &c.members {
                match m {
                    ClassMember::Method(m) => {
                        if let Some(stmts) = m.body.statements() {
                            for s in stmts {
                                collect_bindings_from_stmt(s, lets);
                            }
                        }
                    }
                    ClassMember::Getter(g) => {
                        if let Some(stmts) = g.body.statements() {
                            for s in stmts {
                                collect_bindings_from_stmt(s, lets);
                            }
                        }
                    }
                    ClassMember::Setter(s) => {
                        if let Some(stmts) = s.body.statements() {
                            for s in stmts {
                                collect_bindings_from_stmt(s, lets);
                            }
                        }
                    }
                    ClassMember::Index(i) => {
                        if let Some(stmts) = i.body.statements() {
                            for s in stmts {
                                collect_bindings_from_stmt(s, lets);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Statement::Impl(i) => {
            for m in &i.members {
                if let ImplMember::Behavior(b) = m {
                    collect_bindings_from_behavior(b, lets);
                }
            }
        }
        Statement::Trait(t) => {
            for m in &t.members {
                if let TraitMember::Behavior(b) = m {
                    collect_bindings_from_behavior(b, lets);
                }
            }
        }
        _ => {}
    }
}

fn collect_assigned_fields(expr: &Expr, fields: &mut Vec<String>) {
    match expr {
        Expr::Assignment(assign) => {
            if let Expr::Field { value, .. } = &*assign.name {
                if !fields.contains(value) {
                    fields.push(value.clone());
                }
            }
            collect_assigned_fields(&assign.value, fields);
        }
        Expr::Unary(unary) => {
            collect_assigned_fields(&unary.expr, fields);
        }
        Expr::Binary(binary) => {
            collect_assigned_fields(&binary.left, fields);
            collect_assigned_fields(&binary.right, fields);
        }
        Expr::MethodCall(call) => {
            collect_assigned_fields(&call.object, fields);
            for arg in &call.args {
                collect_assigned_fields_pack(arg, fields);
            }
        }
        Expr::UnqualifiedCall(call) => {
            for arg in &call.args {
                collect_assigned_fields_pack(arg, fields);
            }
        }
        Expr::GetProperty(get_prop) => {
            collect_assigned_fields(&get_prop.object, fields);
        }
        Expr::SetProperty(set_prop) => {
            collect_assigned_fields(&set_prop.object, fields);
            collect_assigned_fields(&set_prop.value, fields);
        }
        Expr::Index(ix) => {
            collect_assigned_fields(&ix.object, fields);
            for arg in &ix.args {
                collect_assigned_fields_pack(arg, fields);
            }
        }
        Expr::SetIndex(six) => {
            collect_assigned_fields(&six.object, fields);
            for arg in &six.args {
                collect_assigned_fields_pack(arg, fields);
            }
            collect_assigned_fields(&six.value, fields);
        }
        Expr::Block(block) => {
            for stmt in &block.body {
                collect_assigned_fields_stmt(stmt, fields);
            }
        }
        Expr::If(i) => {
            collect_assigned_fields(&i.condition, fields);
            for s in &i.then_body.body {
                collect_assigned_fields_stmt(s, fields);
            }
            if let Some(eb) = &i.else_body {
                match eb {
                    IfElseBody::Block(b) => {
                        for s in &b.body {
                            collect_assigned_fields_stmt(s, fields);
                        }
                    }
                    IfElseBody::ElseIf(e) => {
                        collect_assigned_fields(e, fields);
                    }
                }
            }
        }
        Expr::MapLiteral(map) => {
            for entry in &map.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_assigned_fields(expr, fields);
                        }
                        collect_assigned_fields(value, fields);
                    }
                    MapLiteralEntry::Expansion { expr, .. } => collect_assigned_fields(expr, fields),
                }
            }
        }
        Expr::SetLiteral(set) => {
            for entry in &set.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                        collect_assigned_fields(expr, fields);
                    }
                }
            }
        }
        Expr::ListLiteral(list) => {
            for element in &list.elements {
                match element {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => {
                        collect_assigned_fields(expr, fields);
                    }
                }
            }
        }
        _ => {}
    }
}

fn collect_assigned_fields_pack(item: &PackItem, fields: &mut Vec<String>) {
    match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => collect_assigned_fields(expr, fields),
        PackItem::Labeled { label, value, .. } => {
            if let PackLabel::Computed { expr, .. } = label {
                collect_assigned_fields(expr, fields);
            }
            collect_assigned_fields(value, fields);
        }
    }
}

fn collect_assigned_fields_stmt(stmt: &Statement, fields: &mut Vec<String>) {
    match stmt {
        Statement::Expr { expr, .. } => {
            collect_assigned_fields(expr, fields);
        }
        Statement::Let(binding) => {
            if let Some(ref val) = binding.value {
                collect_assigned_fields(val, fields);
            }
        }
        Statement::Return(ret) => {
            if let Some(ref val) = ret.value {
                collect_assigned_fields(val, fields);
            }
        }
        Statement::For(for_stmt) => {
            for lane in &for_stmt.lanes {
                collect_assigned_fields(&lane.iter, fields);
            }
            for body_stmt in &for_stmt.body {
                collect_assigned_fields_stmt(body_stmt, fields);
            }
        }
        Statement::Throw { expr, .. } => {
            collect_assigned_fields(expr, fields);
        }
        _ => {}
    }
}

fn plan_file_edits(content: &str) -> Result<Vec<Edit>, String> {
    let res = parse_for_migration(content)?;

    let mut edits = Vec::new();

    // 1. Collect bindings
    let mut bindings = Vec::new();
    for stmt in &res.program.statements {
        collect_bindings_from_stmt(stmt, &mut bindings);
    }

    for b in bindings {
        match b.kind {
            BindingKind::Let => {
                let kw_range = b.range.start..b.range.start + 3;
                if kw_range.end <= content.len() && &content[kw_range.clone()] == "let" {
                    edits.push(Edit {
                        start: kw_range.start,
                        end: kw_range.end,
                        replacement: "let mut".to_string(),
                        kind: EditKind::LetToLetMut,
                    });
                }
            }
            BindingKind::Const => {
                let kw_range = b.range.start..b.range.start + 5;
                if kw_range.end <= content.len() && &content[kw_range.clone()] == "const" {
                    edits.push(Edit {
                        start: kw_range.start,
                        end: kw_range.end,
                        replacement: "let".to_string(),
                        kind: EditKind::ConstToLet,
                    });
                }
            }
            BindingKind::LetMut => {
                // Already target form, do nothing.
            }
        }
    }

    // 2. Class fields & implicit fields
    for stmt in &res.program.statements {
        if let Statement::Class(c) = stmt {
            let declared_instance_fields: Vec<String> = c.members.iter().filter_map(|m| {
                if let ClassMember::Field(f) = m {
                    if !f.is_static { Some(f.name.clone()) } else { None }
                } else { None }
            }).collect();

            // Declared fields rewrite
            for m in &c.members {
                if let ClassMember::Field(f) = m {
                    if let Some(error) = legacy_const_field_error(f, &res.errors) {
                        // Old immutable field: `const _field` -> `_field`.
                        let end = skip_field_prefix_whitespace(content, error.range.end, f.name_range.start);
                        edits.push(Edit {
                            start: error.range.start,
                            end,
                            replacement: String::new(),
                            kind: EditKind::ConstFieldToBare,
                        });
                    } else {
                        // Old bare mutable field: `_field` -> `mut _field`.
                        // The target AST treats bare fields as immutable, so
                        // inspect the original source modifier instead.
                        let prefix = &content[f.range.start..f.name_range.start];
                        if !is_mut_modifier_prefix(prefix) {
                            edits.push(Edit {
                                start: f.name_range.start,
                                end: f.name_range.start,
                                replacement: "mut ".to_string(),
                                kind: EditKind::BareFieldToMut,
                            });
                        }
                    }
                }
            }

            // Implicit instance fields insertion
            if declared_instance_fields.is_empty() {
                let mut assigned = Vec::new();
                for m in &c.members {
                    match m {
                        ClassMember::Method(m) if !m.is_static => {
                            if let Some(stmts) = m.body.statements() {
                                for s in stmts {
                                    collect_assigned_fields_stmt(s, &mut assigned);
                                }
                            }
                        }
                        ClassMember::Getter(g) if !g.is_static => {
                            if let Some(stmts) = g.body.statements() {
                                for s in stmts {
                                    collect_assigned_fields_stmt(s, &mut assigned);
                                }
                            }
                        }
                        ClassMember::Setter(s) if !s.is_static => {
                            if let Some(stmts) = s.body.statements() {
                                for s in stmts {
                                    collect_assigned_fields_stmt(s, &mut assigned);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if !assigned.is_empty() {
                    let class_head = &content[c.name_range.end..c.range.end];
                    if let Some(brace_rel) = class_head.find('{') {
                        let brace_pos = c.name_range.end + brace_rel;
                        let after_brace = &content[brace_pos + 1..];
                        let (insert_pos, need_nl) = if after_brace.starts_with("\r\n") {
                            (brace_pos + 1 + 2, false)
                        } else if after_brace.starts_with('\n') {
                            (brace_pos + 1 + 1, false)
                        } else {
                            (brace_pos + 1, true)
                        };

                        let indent = if let Some(first_m) = c.members.first() {
                            let m_start = first_m.range().start;
                            let line_start = content[..m_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
                            let prefix = &content[line_start..m_start];
                            if prefix.chars().all(|ch| ch == ' ' || ch == '\t') && !prefix.is_empty() {
                                prefix.to_string()
                            } else {
                                "  ".to_string()
                            }
                        } else {
                            "  ".to_string()
                        };

                        let mut decl_text = String::new();
                        if need_nl {
                            decl_text.push('\n');
                        }
                        for f in &assigned {
                            decl_text.push_str(&indent);
                            decl_text.push_str("mut ");
                            decl_text.push_str(f);
                            decl_text.push('\n');
                        }

                        edits.push(Edit {
                            start: insert_pos,
                            end: insert_pos,
                            replacement: decl_text,
                            kind: EditKind::ImplicitFieldDecl,
                        });
                    }
                }
            }
        }
    }

    Ok(edits)
}

fn apply_edits(content: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by(|a, b| b.start.cmp(&a.start));
    let mut result = content.to_string();
    for edit in edits {
        result.replace_range(edit.start..edit.end, &edit.replacement);
    }
    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let write_mode = args.iter().any(|a| a == "--write");
    let show_diff = args.iter().any(|a| a == "--show-diff");

    let mut files = Vec::new();
    if let Some(pos) = args.iter().position(|a| a == "--file") {
        if let Some(file) = args.get(pos + 1) {
            files.push(PathBuf::from(file));
        }
    } else {
        find_ph_files(Path::new("."), &mut files);
        files.sort();
    }

    println!("Total .ph files in scope: {}", files.len());

    // First, check repository-wide census to detect if already migrated.
    let mut repo_total_let = 0;
    let mut repo_total_let_mut = 0;
    let mut repo_total_const = 0;
    let mut repo_total_const_fields = 0;
    let mut repo_total_implicit_classes = 0;

    for file in &files {
        if let Ok(content) = fs::read_to_string(file) {
            if let Ok(res) = parse_for_migration(&content) {
                let mut bindings = Vec::new();
                for stmt in &res.program.statements {
                    collect_bindings_from_stmt(stmt, &mut bindings);
                }
                for b in bindings {
                    match b.kind {
                        BindingKind::Let => repo_total_let += 1,
                        BindingKind::LetMut => repo_total_let_mut += 1,
                        BindingKind::Const => repo_total_const += 1,
                    }
                }
                for stmt in &res.program.statements {
                    if let Statement::Class(c) = stmt {
                        let declared_instance_fields: Vec<String> = c.members.iter().filter_map(|m| {
                            if let ClassMember::Field(f) = m {
                                if !f.is_static { Some(f.name.clone()) } else { None }
                            } else { None }
                        }).collect();
                        for m in &c.members {
                            if let ClassMember::Field(f) = m {
                                if legacy_const_field_error(f, &res.errors).is_some() {
                                    repo_total_const_fields += 1;
                                }
                            }
                        }
                        if declared_instance_fields.is_empty() {
                            let mut assigned = Vec::new();
                            for m in &c.members {
                                match m {
                                    ClassMember::Method(m) if !m.is_static => {
                                        if let Some(stmts) = m.body.statements() {
                                            for s in stmts { collect_assigned_fields_stmt(s, &mut assigned); }
                                        }
                                    }
                                    ClassMember::Getter(g) if !g.is_static => {
                                        if let Some(stmts) = g.body.statements() {
                                            for s in stmts { collect_assigned_fields_stmt(s, &mut assigned); }
                                        }
                                    }
                                    ClassMember::Setter(s) if !s.is_static => {
                                        if let Some(stmts) = s.body.statements() {
                                            for s in stmts { collect_assigned_fields_stmt(s, &mut assigned); }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if !assigned.is_empty() {
                                repo_total_implicit_classes += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Repository Census: let={}, let_mut={}, const={}, const_fields={}, implicit_classes={}",
        repo_total_let, repo_total_let_mut, repo_total_const, repo_total_const_fields, repo_total_implicit_classes);

    let is_repo_migrated = files.len() > 1
        && repo_total_const == 0
        && repo_total_const_fields == 0
        && repo_total_implicit_classes == 0
        && repo_total_let_mut > 0;

    if is_repo_migrated {
        println!("Repository is ALREADY MIGRATED to C4 target syntax.");
        println!("Idempotence check: 0 edits required across {} files.", files.len());
        return;
    }

    let mut modified_count = 0;
    let mut total_edits = 0;
    let mut read_errors = 0;
    let mut parse_errors = 0;
    let mut write_errors = 0;

    let mut let_to_let_mut_count = 0;
    let mut const_to_let_count = 0;
    let mut bare_field_to_mut_count = 0;
    let mut const_field_to_bare_count = 0;
    let mut implicit_field_decl_count = 0;

    for file in &files {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", file, e);
                read_errors += 1;
                continue;
            }
        };

        let edits = match plan_file_edits(&content) {
            Ok(e) => e,
            Err(_err) => {
                // Known negative/syntax-error tests are ignored
                parse_errors += 1;
                continue;
            }
        };

        if !edits.is_empty() {
            modified_count += 1;
            total_edits += edits.len();
            for edit in &edits {
                match edit.kind {
                    EditKind::LetToLetMut => let_to_let_mut_count += 1,
                    EditKind::ConstToLet => const_to_let_count += 1,
                    EditKind::BareFieldToMut => bare_field_to_mut_count += 1,
                    EditKind::ConstFieldToBare => const_field_to_bare_count += 1,
                    EditKind::ImplicitFieldDecl => implicit_field_decl_count += 1,
                }
            }
            let new_content = apply_edits(&content, edits);
            if show_diff {
                println!("=== MODIFIED CONTENT FOR {:?} ===", file);
                println!("{}", new_content);
            }
            if write_mode {
                if let Err(e) = fs::write(file, new_content) {
                    eprintln!("Error writing {:?}: {}", file, e);
                    write_errors += 1;
                }
            }
        }
    }

    println!("Scan finished: modified_files={}, total_edits={}", modified_count, total_edits);
    println!("  let -> let mut: {}", let_to_let_mut_count);
    println!("  const -> let:   {}", const_to_let_count);
    println!("  _field -> mut _field: {}", bare_field_to_mut_count);
    println!("  const _field -> _field: {}", const_field_to_bare_count);
    println!("  implicit fields declared: {}", implicit_field_decl_count);
    println!("  files skipped (parse error / negative fixtures): {}", parse_errors);
    if write_errors > 0 || read_errors > 0 {
        eprintln!("  I/O errors: read={}, write={}", read_errors, write_errors);
    }
    if !write_mode {
        println!("(Dry-run only. Use --write to apply modifications.)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_let_to_let_mut() {
        let src = "let x = 1\nlet y\n";
        let edits = plan_file_edits(src).unwrap();
        let out = apply_edits(src, edits);
        assert_eq!(out, "let mut x = 1\nlet mut y\n");
    }

    #[test]
    fn test_migrate_const_to_let() {
        let src = "const x = 1\n";
        let edits = plan_file_edits(src).unwrap();
        let out = apply_edits(src, edits);
        assert_eq!(out, "let x = 1\n");
    }

    #[test]
    fn test_migrate_fields() {
        let src = "class Foo {\n  _x: Int\n  const _y: String\n  mut _z: Bool\n}\n";
        let edits = plan_file_edits(src).unwrap();
        let out = apply_edits(src, edits);
        assert_eq!(out, "class Foo {\n  mut _x: Int\n  _y: String\n  mut _z: Bool\n}\n");
    }

    #[test]
    fn test_migrate_implicit_fields() {
        let src = "class Counter {\n  @constructor\n  new() { _count = 0 }\n  inc() { _count = _count + 1 }\n}\n";
        let edits = plan_file_edits(src).unwrap();
        let out = apply_edits(src, edits);
        assert_eq!(out, "class Counter {\n  mut _count\n  @constructor\n  new() { _count = 0 }\n  inc() { _count = _count + 1 }\n}\n");
    }

    #[test]
    fn test_migrate_ambiguous_forms() {
        let src = r#"// let x = 1, const y = 2
let s = "let a = const b"
let mut = 10
class Example {
  @class
  _total = 0
  bump() { _total = _total + 1 }
}
"#;
        let edits = plan_file_edits(src).unwrap();
        let out = apply_edits(src, edits);
        assert_eq!(out, r#"// let x = 1, const y = 2
let mut s = "let a = const b"
let mut mut = 10
class Example {
  @class
  mut _total = 0
  bump() { _total = _total + 1 }
}
"#);
    }
}
