#![no_main]

use libfuzzer_sys::fuzz_target;
use phalcom_ast::lexer::Lexer;

// The lexer must never panic on arbitrary input — it should only ever yield
// tokens or `LexicalError`. We drain the whole iterator to exercise every
// branch (including the injected EOF).
fuzz_target!(|data: &str| {
    for token in Lexer::new(data) {
        let _ = token;
    }
});
