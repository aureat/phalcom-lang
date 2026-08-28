from pathlib import Path

path = Path("phalcom-lsp/tests/module_navigation.rs")
text = path.read_text()
old = '    let shapes_path = workspace.write("shapes.ph", "class Circle {\\n  area() { 3.14 }\\n}\\n");\n'
new = '    let shapes_path = workspace.write("shapes.ph", "class Circle {\\n  area() { 3.14 }\\n}\\nexport Circle\\n");\n'
if old not in text:
    raise SystemExit("missing module-navigation fixture anchor")
path.write_text(text.replace(old, new, 1))
