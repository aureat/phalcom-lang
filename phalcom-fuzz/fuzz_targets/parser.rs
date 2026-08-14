#![no_main]

use libfuzzer_sys::fuzz_target;

// Parsing arbitrary input must never panic — `parse_source` should always
// return `Ok(Program)` or `Err(SyntaxError)`. A panic here (e.g. an out-of-
// bounds source slice or an unhandled grammar state) is a bug to fix.
fuzz_target!(|data: &str| {
    let _ = phalcom_ast::parse_source(data, 0);
});
