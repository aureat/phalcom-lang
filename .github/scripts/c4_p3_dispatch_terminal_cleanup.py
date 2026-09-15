from pathlib import Path

call = Path('phalcom-semantic/src/checker/call.rs')
text = call.read_text()
old = '''    TraitTerminal {
        receiver: TypeId,
        terminal: crate::trait_dispatch::TraitDispatchTerminal,
    },
'''
new = '''    TraitTerminal {
        terminal: crate::trait_dispatch::TraitDispatchTerminal,
    },
'''
if text.count(old) != 1:
    raise SystemExit(f'union terminal receiver field count: {text.count(old)}')
text = text.replace(old, new, 1)
old = '''            UnionCallArm::TraitTerminal { receiver: _, terminal } => {
'''
new = '''            UnionCallArm::TraitTerminal { terminal } => {
'''
if text.count(old) != 1:
    raise SystemExit(f'union terminal consume arm count: {text.count(old)}')
call.write_text(text.replace(old, new, 1))

expr = Path('phalcom-semantic/src/checker/expression.rs')
text = expr.read_text()
old = '''                    ResolvedDispatchResult::TraitTerminal(terminal) => UnionCallArm::TraitTerminal { receiver, terminal },
'''
new = '''                    ResolvedDispatchResult::TraitTerminal(terminal) => UnionCallArm::TraitTerminal { terminal },
'''
if text.count(old) != 1:
    raise SystemExit(f'union terminal produce arm count: {text.count(old)}')
expr.write_text(text.replace(old, new, 1))
