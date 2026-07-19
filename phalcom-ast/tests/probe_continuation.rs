//! Probe: which error kinds does truncated input actually produce?
//!
//! **This asserts nothing yet.** It prints a classification table; run with
//! `--nocapture` to read it. It exists as the recorded evidence behind U-REPL
//! D7 (`docs/forge/units/U-REPL/plan.md`), which establishes that the parser
//! never emits `UnrecognizedEof` — every truncation surfaces as
//! `UnrecognizedToken { token: "", .. }`, EOF being modelled as a zero-length
//! token.
//!
//! U-REPL stage 5 promotes this into real assertions over that table
//! (complete / incomplete / error per input), once EOF is routed to
//! `UnrecognizedEof`. Until then it is a probe, not a guard — do not read a
//! green result here as coverage.

use phalcom_ast::parser::parse;

#[test]
fn probe() {
    let cases = [
        ("complete: let x = 1", "let x = 1"),
        ("complete: expr", "1 + 1"),
        ("open brace", "class Foo {"),
        ("open brace + member", "class Foo {\n  bar() { 1 }"),
        ("trailing operator", "let x = 1 +"),
        ("trailing equals", "let x ="),
        ("open paren", "foo(1,"),
        ("open bracket", "[1, 2,"),
        ("unterminated string", "let s = \"abc"),
        ("genuine error", "let x = )"),
        ("genuine error 2", "1 +* 2"),
        ("empty", ""),
        ("block open", "if (x) {"),
        ("string across newline", "let s = \"abc\ndef\""),
        ("string open across newline", "let s = \"abc\ndef"),
    ];

    for (label, src) in cases {
        let p = parse(src, 0);
        let kinds: Vec<String> = p.errors.iter().map(|e| format!("{:?}", e.kind).chars().take(60).collect()).collect();
        println!("--- {label:24} | src={src:?}");
        println!("    stmts={} errors={}", p.program.statements.len(), p.errors.len());
        for k in &kinds {
            println!("      {k}");
        }
    }
}
