#!/usr/bin/env python3
from pathlib import Path
from collections import Counter
import re, sys

root = Path(__file__).resolve().parents[1] / 'type'
files = [p for p in root.glob('*.md') if p.name != 'README.md']
ids, names = [], []
for f in files:
    s = f.read_text(encoding='utf-8')
    ids.extend(re.findall(r'@id\((T\d{4})\)', s))
    names.extend(re.findall(r'@name\(([^)]+)\)', s))
idset = set(ids)
errors = []

for label, values in [('ID', ids), ('name', names)]:
    for value, count in Counter(values).items():
        if count > 1:
            errors.append(f'duplicate {label}: {value} ({count})')

for name in names:
    if not re.fullmatch(r'[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)+', name):
        errors.append(f'invalid semantic path: {name}')

for f in root.glob('*.md'):
    s = f.read_text(encoding='utf-8')
    for ref in set(re.findall(r'T\d{4}', s)):
        if ref not in idset:
            errors.append(f'{f.name}: dangling reference {ref}')
    if s.count('```') % 2:
        errors.append(f'{f.name}: unbalanced markdown code fences')

for f in files:
    s = f.read_text(encoding='utf-8')
    fids = re.findall(r'@id\((T\d{4})\)', s)
    nums = [int(x[1:]) for x in fids]
    if nums != sorted(nums):
        errors.append(f'{f.name}: law IDs are not ascending')
    m = re.search(r'\*\*Identifier range used here:\*\* `?(T\d{4})[–-](T\d{4})`?', s)
    if not m or (m.group(1), m.group(2)) != (fids[0], fids[-1]):
        errors.append(f'{f.name}: identifier range header does not match contained laws')


def expand_refs(text):
    refs = set(re.findall(r'T\d{4}', text))
    for a, b in re.findall(r'(T\d{4})[–-](T\d{4})', text):
        lo, hi = int(a[1:]), int(b[1:])
        refs |= {x for x in idset if lo <= int(x[1:]) <= hi}
    return refs

for f in files:
    current = []
    for lineno, line in enumerate(f.read_text(encoding='utf-8').splitlines(), 1):
        if line.startswith('## ') or line.startswith('### '):
            current = re.findall(r'T\d{4}', line)
        if line.startswith('**Depends on:**') and current:
            current_floor = min(int(x[1:]) for x in current)
            for dep in expand_refs(line):
                if dep in idset and int(dep[1:]) >= current_floor:
                    errors.append(
                        f'{f.name}:{lineno}: dependency {dep} is not lower than {current}'
                    )

if errors:
    print('REGISTRY VALIDATION: FAILED')
    for e in errors:
        print(f'- {e}')
    sys.exit(1)

print('REGISTRY VALIDATION: PASSED')
print(f'specification files: {len(files)}')
print(f'laws: {len(ids)}')
print(f'unique IDs: {len(set(ids))}')
print(f'unique semantic paths: {len(set(names))}')
print('dangling references: 0')
print('forward/self explicit dependencies: 0')
