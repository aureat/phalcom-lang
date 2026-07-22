## Allowing ordinary `"..."` strings to span lines

### Arguments for

**Minimal syntax.** There is only one interpreted-string form to learn:

```phalcom
message = "Hello,
world!"
```

Escapes, interpolation, type inference, and runtime representation remain identical.

**Natural composability.** A string can grow from one line to several without changing delimiters. This is convenient for generated text, SQL, templates, and error messages.

**No arbitrary distinction between one-line and multiline text.** A newline is simply another character, much like `\n`.

**Easy copy and paste.** Users can paste text directly between quotes without converting delimiters or adding escape sequences.

**Potentially smaller language surface.** You avoid a second delimiter and questions such as whether triple-quoted strings are raw, interpolated, dedented, or semantically different.

### Arguments against

**A missing quote becomes much more destructive.**

With single-line strings:

```phalcom
name = "Altun
System.print(name)
```

The lexer can report an unterminated string at the end of the first line.

With multiline ordinary strings, the lexer must continue looking:

```phalcom
name = "Altun
System.print(name)
doSomething()
description = "
```

A distant quote may accidentally terminate the string. The parser then reports bizarre downstream errors rather than the actual missing quote.

This is especially important for Phalcom because interpolation already requires nested lexical awareness. Allowing unmarked multiline strings expands the recovery problem further.

**Code indentation silently becomes data.**

```phalcom
message = "first line
           second line
           third line"
```

What is the runtime value?

```text
first line
           second line
           third line
```

Or should indentation be removed? If it is removed, according to which rule?

Possible rules include:

- Remove indentation equal to the closing quote.
- Remove the smallest common indentation.
- Ignore indentation on blank lines.
- Preserve tabs exactly.
- Convert tabs to columns before dedenting.

Once ordinary strings are multiline, these questions affect the basic string syntax rather than a specialized construct.

**Formatters cannot safely reindent code.**

Moving this:

```phalcom
message = "alpha
beta"
```

into a nested block may change its value:

```phalcom
if condition {
    message = "alpha
    beta"
}
```

A formatter must either alter runtime content, leave visually broken indentation, or understand a language-defined dedentation algorithm.

**Multiline intent is invisible.** The opening delimiter does not tell the reader whether the string ends later on the same line or fifty lines below. A dedicated delimiter acts as a visual warning that a block of text follows.

**Syntax-highlighting recovery becomes worse.** Editors commonly recover from an unterminated ordinary string at a newline. When newlines are legal, one missing quote can cause the remainder of a file to be highlighted as a string.

**Accidental newlines become valid.** A line accidentally split by editing, wrapping, or conflict resolution changes the string value instead of producing an error.

**Leading and trailing newline behavior becomes awkward.**

```phalcom
text = "
hello
"
```

Does the value begin and end with a newline? Literal semantics say yes, but users frequently expect those structural newlines to be omitted.

## Using a distinct multiline syntax

For example:

```phalcom
message = """
    Hello,
    world!
    """
```

### Arguments for

**Intent is explicit.** The opening delimiter immediately tells readers, editors, and formatters that this is a text block.

**Ordinary strings get strong error recovery.** An unescaped newline inside `"..."` can immediately produce an unterminated-string diagnostic.

**Multiline-specific semantics can be designed deliberately.** The language can define:

- Whether the opening structural newline is included.
- Whether the closing structural newline is included.
- How indentation is stripped.
- Whether escapes are interpreted.
- Whether interpolation is enabled.
- How source line endings are normalized.
- Whether the closing delimiter must appear on its own line.

Trying to attach all these policies to ordinary strings makes the simplest string form surprisingly complicated.

**Formatting can remain semantics-preserving.** If indentation is derived from the closing delimiter, an entire block can be moved safely:

```phalcom
message = """
    first
    second
    """
```

```phalcom
if condition {
    message = """
        first
        second
        """
}
```

Both can evaluate to:

```text
first
second
```

**Better editor behavior.** Triple quotes are a strong synchronization point for syntax highlighting and lexical recovery.

**Room for future specialization.** Phalcom could later add a raw multiline form without changing ordinary-string behavior.

### Arguments against

**More language surface.** Users must understand two string forms and when each is appropriate.

**The feature matrix can multiply.** You may eventually face combinations such as:

- Ordinary interpreted string
- Ordinary raw string
- Multiline interpreted string
- Multiline raw string
- Interpolated versus non-interpolated forms

Without restraint, string syntax becomes a miniature language.

**Delimiter collision exists.** Text containing `"""` needs escaping or a delimiter-extension mechanism.

For documentation or generated source, this can be common.

**Small multiline strings become slightly ceremonial.**

```phalcom
value = """
    first
    second
    """
```

is heavier than simply allowing:

```phalcom
value = "first
second"
```

**Triple quotes do not solve indentation by themselves.** Python demonstrates this clearly:

```python
value = """
    first
    second
"""
```

The indentation is part of the value unless something such as `textwrap.dedent` is used. A separate delimiter without carefully specified block semantics only makes intent explicit; it does not make the content ergonomic.

**Delimiter meaning may be less obvious than expected.** Does `"""` mean “multiline,” “raw,” “dedented,” or merely “a string terminated by three quotes”? Phalcom must state this precisely.

## The real design decision

The main choice is not actually:

> Can a string contain a newline?

It is:

> Should an accidental newline inside the most common string form be accepted as data or rejected as a likely mistake?

For a general-purpose programming language, rejecting it is usually the stronger default. Explicit multiline syntax makes uncommon complexity visible and preserves sharp diagnostics for common code.

## Recommended Phalcom design

Use `"..."` for single-line interpreted strings:

```phalcom
name = "Phalcom"
message = "Hello, \(name)"
newline = "first\nsecond"
```

A literal source newline before the closing quote should be an error.

Use `"""..."""` as an interpreted multiline text block:

```phalcom
message = """
    Hello, \(name).

    Welcome to Phalcom.
    """
```

I would give it these semantics:

1. The opening delimiter must be followed only by optional horizontal whitespace and a newline.
2. The closing delimiter appears on its own line, apart from indentation.
3. The opening structural newline is excluded.
4. The newline immediately preceding the closing delimiter is excluded.
5. The indentation preceding the closing delimiter defines the block margin.
6. Every nonblank content line must contain at least that margin.
7. That margin is removed from every line.
8. Additional indentation is preserved.
9. Line endings are normalized to `\n`.
10. Escapes and interpolation behave exactly as in ordinary strings.
11. A line violating the required margin produces an error rather than receiving heuristic dedentation.

Thus:

```phalcom
message = """
    first
        second
    third
    """
```

evaluates to:

```text
first
    second
third
```

This closing-delimiter rule is more predictable than “remove the smallest common indentation.” The author explicitly controls the margin, formatters can move the whole construct safely, and a mistakenly under-indented line is diagnosed.

## One useful compromise

Phalcom could technically allow physical line continuation in ordinary strings while still rejecting literal newlines:

```phalcom
message = "This is a long \
           logical line"
```

The backslash-newline pair and structural indentation would be omitted. This supports source wrapping without turning ordinary strings into text blocks.

I would only add this if long literal lines become a demonstrated problem. Implicit concatenation is another option, but it introduces its own parsing surprises.

## Bottom line

Multiline ordinary quotes optimize for fewer delimiters. Dedicated multiline syntax optimizes for error detection, readable intent, stable formatting, and explicit whitespace semantics.

For Phalcom, the second set of properties is more valuable. Use single-line `"..."` strings plus a carefully specified multiline block form. Avoid copying Python’s triple-quote behavior exactly; Python supplies a delimiter but leaves indentation management largely to libraries. Phalcom can make indentation-safe multiline strings a proper language feature.