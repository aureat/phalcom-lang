# PDR-0030 — Replace `extends` keyword with `is` for Class Inheritance

- Status: Accepted
- Date: 2026-07-22
- Related: [statements and declarations specification](../spec/current/syntax/statements-and-declarations.md), [grammar specification](../spec/current/syntax/grammar.md), [classes specification](../spec/current/classes.md)

## Context

Phalcom previously used `extends` as a contextual keyword in class header declarations (`class Sub is Super`). The keyword `extends` was only recognized immediately after a class identifier, remaining an ordinary identifier elsewhere.

The language already uses `is` as a reserved keyword (`Token::Is`) for type-test expression queries (`x is Type`). Replacing `extends` with `is` simplifies the keyword set, unifies type/class relationships under `is`, and removes `extends` from the language entirely.

## Decision

1. **Keyword Replacement**:
   - `extends` is removed as a keyword (both reserved and contextual).
   - `is` is used for class superclass inheritance declarations:
     `class Sub is Super { ... }`
   - `class Sub is Super` is no longer valid syntax.

2. **Grammar Update**:
   - `class_decl := "class" IDENT [ "is" IDENT ] "{" { member } "}"`

3. **Dual Role for `is`**:
   - In class header position (`class IDENT is IDENT`), `is` specifies single superclass inheritance.
   - In expression position (`expr is Type`), `is` performs runtime type/class testing.
