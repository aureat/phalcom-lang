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


# The registry is a formal-law view, not an alternate semantic authority. These
# assertions keep the high-risk structural rules aligned with their owning and
# companion specifications. They intentionally use stable normative markers,
# rather than attempting to parse the full prose grammar.
specs_root = root.parents[1]


def read_spec(relative):
    return (specs_root / relative).read_text(encoding='utf-8')


def require_match(invariant, relative, pattern):
    if not re.search(pattern, read_spec(relative), re.MULTILINE | re.DOTALL):
        errors.append(
            f'cross-spec invariant {invariant!r} missing from {relative}: {pattern!r}'
        )


def require_absent(invariant, relative, pattern):
    if re.search(pattern, read_spec(relative), re.IGNORECASE):
        errors.append(
            f'cross-spec invariant {invariant!r} violated by {relative}: {pattern!r}'
        )


tuple_registry = 'typing/type/type.tuple.md'
record_registry = 'typing/type/type.record-row.md'
tuple_companion = 'tuple-and-record-typing.md'
structural_products = 'structural-products.md'
arguments = 'callables/arguments.md'
closures = 'callables/closures.md'

require_match('Tuple<> == Unit', tuple_registry, r'Tuple<>\s*≡\s*Unit\s*≡\s*\(\)')
require_match('Tuple<> == Unit', tuple_companion, r'Tuple<>\s*≡\s*Unit\s*≡\s*\(\)')
require_match(
    'negative Tuple indexing',
    tuple_registry,
    r'I\s*>=\s*0\s*->\s*I\s*\nI\s*<\s*0\s*->\s*N\s*\+\s*I',
)
require_match('negative Tuple indexing', structural_products, r'I\s*>=\s*0.*N\s*\+\s*I')
require_match('closed Tuple no width subtyping', tuple_registry, r'\(A, B\) </: \(A,\)')
require_match('closed Tuple no width subtyping', tuple_companion, r'does not apply implicit width subtyping to closed tuple')
require_match(
    'open Tuple remainder subtyping',
    tuple_registry,
    r'PositionalTupleRemainderSubtype.*LabeledTupleRemainderSubtype.*CompleteTupleRemainderIsLaneWise',
)
require_match('open Tuple remainder subtyping', tuple_companion, r'If the source positional lane is open, the target positional lane must also be open')
require_match('no Tuple optionality', tuple_registry, r'Fixed positional and\s+labeled entries are always required')
require_absent('no Tuple optionality', tuple_registry, r'\boptional(?:ity)?\b')
require_match('closed Record no implicit width subtyping', record_registry, r'\{ x: A, y: B \} </: \{ x: A \}')
require_match('closed Record no implicit width subtyping', tuple_companion, r'does not apply implicit width subtyping to closed tuple or record')
require_match('open Record width compatibility', record_registry, r'\{ x: A, y: B \} <: \{ x: A, \.\.\. \}')
require_match('open Record width compatibility', tuple_companion, r'Width subtyping is enabled by an explicit open remainder in the target')
require_match('Record optional presence', record_registry, r'OptionalTargetFieldCompatibility')
require_match('Record optional presence', tuple_companion, r'### 10\.3 Optional Record fields')
require_match('FieldKey domain', record_registry, r'Symbol \| String \| Int \| Bool')
require_match('FieldKey domain', structural_products, r'Record keys belong to `FieldKey` \(`Symbol \| String \| Int \| Bool`\)')
require_match('Pack-native rest capture', arguments, r'Closure \*rest -> positional Pack')
require_match('Pack-native rest capture', closures, r'Closure \*rest -> positional Pack')
require_match('Pack() != Unit', arguments, r'Pack\(\) != Unit')
require_match('Pack() != Unit', closures, r'Pack\(\) != Unit')

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
print('cross-spec canonical assertions: 11')
