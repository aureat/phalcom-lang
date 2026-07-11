mod completer;
mod editor;
mod helper;
mod highlighter;
mod repl;
mod rustyline;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::{Config, Helper};
use unicode_segmentation::UnicodeSegmentation;

use crate::completer::PhalcomCompleter;
use crate::editor::build_repl_editor;
use crate::helper::is_incomplete_block;
use crate::highlighter::PhalcomHighlighter;
use crate::repl::ReplSession;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::validate::Validator;

pub fn read_user_input() -> std::io::Result<String> {
    use std::io::{self, Write};
    print!("phalcom> ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let mut session = ReplSession::start(cwd.clone());

    let config = Config::builder().auto_add_history(true).build();

    let completer = PhalcomCompleter::new(cwd);
    let highlighter = PhalcomHighlighter::default();
    let helper = helper::PhalcomHelper { completer, highlighter };
    let mut rl = build_repl_editor(helper).unwrap();

    let mut buf = String::new();
    let mut cont = false;

    loop {
        let p = if cont { "..> " } else { ">>> " };

        match rl.readline(p) {
            Ok(line) => {
                if line.trim() == ":quit" {
                    break;
                }

                buf.push_str(&line);
                if is_incomplete_block(&buf) {
                    buf.push('\n');
                    print!("..> ");
                    cont = true;
                    continue;
                }

                let id = session.eval(&buf);
                println!("{id}: {buf}");
                buf.clear();
                cont = false;
            }
            Err(ReadlineError::Interrupted) => {
                buf.clear();
                cont = false;
                println!();
            }
            Err(ReadlineError::Eof) => break,
            Err(_) => break,
        };
    }
}
