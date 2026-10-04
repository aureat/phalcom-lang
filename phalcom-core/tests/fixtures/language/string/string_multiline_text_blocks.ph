// area: string
// spec: docs/pdr/0034-multiline-string-text-blocks.md
// status: PASS

let mut basic = """
    hello
    world
    """

let mut interpolation = """
    sum: \(1 + 2)
    end
    """

let mut escapes = """
    \"""
    \\(escaped)
    """

let mut blank_lines = """
    first

    third
    """

System.print(basic == "hello\nworld")
System.print(interpolation == "sum: 3\nend")
System.print(escapes == "\"\"\"\n\\(escaped)")
System.print(blank_lines == "first\n\nthird")
System.print(basic)
System.print(interpolation)
