from pathlib import Path

scope = Path("phalcom-semantic/src/source_index/scope.rs")
text = scope.read_text()
old = '''    /// Full declaration range.
    pub declaration_range: SourceRange,
'''
new = '''    /// Full declaration range, including attached member attributes.
    pub declaration_range: SourceRange,
'''
if old in text:
    text = text.replace(old, new, 1)
scope.write_text(text)

builder = Path("phalcom-semantic/src/source_index/builder.rs")
text = builder.read_text()
needle = '''    fn visit_member(&mut self, parent: SourceScopeId, declaration: &DeclarationId, member: &ClassMember) {
        let member_side = crate::checker::declaration::member_side(member);
'''
replacement = '''    fn visit_member(&mut self, parent: SourceScopeId, declaration: &DeclarationId, member: &ClassMember) {
        let member_side = crate::checker::declaration::member_side(member);
        let member_range = member.range();
        let declaration_range = member.attributes().first().map_or(member_range, |attribute| {
            SourceRange::new(attribute.range.start, member_range.end)
        });
'''
if needle not in text:
    raise SystemExit("visit_member header not found")
text = text.replace(needle, replacement, 1)

# Add declaration_range after each callable member's body range argument.
for old, new in [
    ('''                    method.name_range,
                    method.range,
                    &method.params,''', '''                    method.name_range,
                    declaration_range,
                    method.range,
                    &method.params,'''),
    ('''                    getter.name_range,
                    getter.range,
                    &[],''', '''                    getter.name_range,
                    declaration_range,
                    getter.range,
                    &[],'''),
    ('''                    setter.name_range,
                    setter.range,
                    std::slice::from_ref(&setter.param),''', '''                    setter.name_range,
                    declaration_range,
                    setter.range,
                    std::slice::from_ref(&setter.param),'''),
    ('''                    index.name_range,
                    index.range,
                    &parameters,''', '''                    index.name_range,
                    declaration_range,
                    index.range,
                    &parameters,'''),
]:
    if old not in text:
        raise SystemExit(f"call-site pattern not found: {old!r}")
    text = text.replace(old, new, 1)

old = '''        callable: CallableId,
        name_range: SourceRange,
        body_range: SourceRange,
        parameters: &[phalcom_ast::ast::ParameterDef],
'''
new = '''        callable: CallableId,
        name_range: SourceRange,
        declaration_range: SourceRange,
        body_range: SourceRange,
        parameters: &[phalcom_ast::ast::ParameterDef],
'''
if old not in text:
    raise SystemExit("visit_callable signature pattern not found")
text = text.replace(old, new, 1)
old = '''                name_range,
                declaration_range: body_range,
                parameter_name_ranges:'''
new = '''                name_range,
                declaration_range,
                parameter_name_ranges:'''
if old not in text:
    raise SystemExit("CallableSourceInfo range assignment not found")
text = text.replace(old, new, 1)
builder.write_text(text)
print("callable declaration attribute range applied")
