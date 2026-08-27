from pathlib import Path

p = Path("phalcom-semantic/src/session.rs")
text = p.read_text()
old = '''        let mut current = Some(owner.clone());
        let mut exact = Vec::new();
        let mut rest_candidates = Vec::new();
        while let Some(declaration) = current {
            if let Some(surface) = dispatch.get_surface(&declaration) {
                let members = surface.surface(side);
                let mut selectors = members.callable_signatures.keys().collect::<Vec<_>>();
                selectors.sort();
                for selector in selectors {
                    if !pattern.matches(selector) {
                        continue;
                    }
                    let Some(callable) = members.callables_by_selector.get(selector).cloned() else {
                        continue;
                    };
                    let signature = &members.callable_signatures[selector];
                    if signature.parameters.iter().any(|parameter| parameter.rest) {
                        rest_candidates.push(callable);
                    } else {
                        exact.push((selector.clone(), callable));
                    }
                }
            }
            current = hierarchy.superclass(&declaration).cloned();
        }
'''
new = '''        let mut exact = Vec::new();
        let mut rest_candidates = Vec::new();
        for dispatch_owner in dispatch.dispatch_owners(hierarchy, owner, side) {
            if let Some(surface) = dispatch.get_surface(&dispatch_owner.declaration) {
                let members = surface.surface(dispatch_owner.side);
                let mut selectors = members.callable_signatures.keys().collect::<Vec<_>>();
                selectors.sort();
                for selector in selectors {
                    if !pattern.matches(selector) {
                        continue;
                    }
                    let Some(callable) = members.callables_by_selector.get(selector).cloned() else {
                        continue;
                    };
                    let signature = &members.callable_signatures[selector];
                    if signature.parameters.iter().any(|parameter| parameter.rest) {
                        rest_candidates.push(callable);
                    } else {
                        exact.push((selector.clone(), callable));
                    }
                }
            }
        }
'''
if old not in text:
    raise SystemExit("method-family hierarchy loop anchor missing")
p.write_text(text.replace(old, new, 1))
print("method-family canonical dispatch-owner traversal applied")
