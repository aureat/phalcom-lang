from pathlib import Path

path = Path("phalcom-lsp/src/backend.rs")
text = path.read_text()
old = '''        let uri = self.compiler_uri_for_module(compiler, module)?;
        let range = self.with_source_snapshot(&uri, |_, _, line_index| line_index.range(source.range.start..source.range.end))?;
        Some(Location { uri, range })
'''
new = '''        let uri = self.compiler_uri_for_module(compiler, module)?;
        let text = compiler
            .sources
            .get(module)
            .map(|published| published.text.as_ref())
            .or_else(|| compiler.presentation_source(module))?;
        let line_index = LineIndex::new(text);
        let range = line_index.range(source.range.start..source.range.end);
        Some(Location { uri, range })
'''
if old not in text:
    raise SystemExit("compiler_site_location source conversion seam not found")
path.write_text(text.replace(old, new, 1))
