//! [`rustyline`] editor construction for the experimental REPL backend.
//!
//! [`build_repl_editor`] creates a fully-configured [`rustyline::Editor`]
//! with Emacs key-bindings, persistent history, and Phalcom-specific key
//! sequences wired up.
//!
//! > **Note:** this module is part of the *experimental* rustyline REPL stack
//! > (`src/rustyline/`).  The active REPL uses `reedline`; see `src/main.rs`.

use crate::helper::PhalcomHelper;
use rustyline::config::Configurer;
use rustyline::history::DefaultHistory;
use rustyline::{At, Cmd, ColorMode, CompletionType, Config, EditMode, Editor, KeyCode, KeyEvent, Modifiers, Movement, Word};

/// Builds a [`rustyline::Editor`] pre-configured for the Phalcom REPL.
///
/// The editor is set up with:
/// - Emacs edit mode with colour enabled
/// - Auto-history with duplicate suppression
/// - List-style tab completion with a 4-space tab stop
/// - Phalcom-specific key bindings (Shift+Enter, Alt+Backspace, Ctrl+Left/Right)
///
/// # Errors
///
/// Returns a [`rustyline::Result`] error if the editor or history could not
/// be initialised (e.g. unsupported terminal or invalid config).
pub fn build_repl_editor(helper: PhalcomHelper) -> rustyline::Result<Editor<PhalcomHelper, DefaultHistory>> {
    let mut builder = Config::builder()
        .edit_mode(EditMode::Emacs) // sane defaults
        .color_mode(ColorMode::Enabled)
        .auto_add_history(true)
        .history_ignore_dups(true)?
        .completion_type(CompletionType::List)
        .tab_stop(4);

    builder.set_max_history_size(10_000)?;

    let config = builder.build();

    let mut rl: Editor<PhalcomHelper, DefaultHistory> = Editor::with_config(config)?;

    rl.set_helper(Some(helper));

    rl.set_auto_add_history(true);

    // Shift + Enter → insert newline (if terminal passes Shift modifier)
    rl.bind_sequence(KeyEvent(KeyCode::Enter, Modifiers::SHIFT), Cmd::Insert(1, "\n".into()));

    // Option/Alt + Delete → delete previous word
    rl.bind_sequence(KeyEvent(KeyCode::Backspace, Modifiers::ALT), Cmd::Kill(Movement::BackwardWord(1, Word::Emacs)));

    // Ctrl + Left → move to beginning of line
    rl.bind_sequence(KeyEvent(KeyCode::Left, Modifiers::CTRL), Cmd::Move(Movement::BeginningOfLine));

    // Ctrl + Right → move to end of line
    rl.bind_sequence(KeyEvent(KeyCode::Right, Modifiers::CTRL), Cmd::Move(Movement::EndOfLine));

    // Cmd + Delete → delete entire line (requires terminal to emit something!)
    // If you make iTerm2 send ESC[3;9~ for Cmd+Backspace, you can bind its bytes:
    rl.bind_sequence(KeyEvent(KeyCode::Delete, Modifiers::CTRL), Cmd::Kill(Movement::WholeLine));

    // Portable fallback: Ctrl+U kills to BOL; Ctrl+K kills to EOL:
    rl.bind_sequence(KeyEvent::ctrl('u'), Cmd::Kill(Movement::BeginningOfLine));
    rl.bind_sequence(KeyEvent::ctrl('k'), Cmd::Kill(Movement::EndOfLine));

    // Nice extras:
    // Alt+f/b jump by word
    rl.bind_sequence(KeyEvent::alt('f'), Cmd::Move(Movement::ForwardWord(1, At::Start, Word::Big)));
    rl.bind_sequence(KeyEvent::alt('b'), Cmd::Move(Movement::BackwardWord(1, Word::Big)));

    Ok(rl)
}
