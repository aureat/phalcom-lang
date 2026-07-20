//! Entry point and prompt implementation for the Phalcom REPL.
//!
//! Assembles the [`reedline`]-backed interactive REPL: configures the editor
//! with tab-completion, syntax highlighting, persistent file history, and a
//! two-state prompt (`ph>` / `..>`) that detects incomplete blocks before
//! evaluating input.

mod common;
mod completer;
mod highlighter;
mod repl;

use completer::PhalcomCompleter;
use highlighter::PhalcomHighlighter;
use repl::{CellOutcome, ReplSession};


use reedline::{
    default_emacs_keybindings, Color, EditCommand, Emacs, FileBackedHistory, IdeMenu, KeyCode,
    KeyModifiers, Keybindings, MenuBuilder, PromptEditMode, PromptHistorySearch, Reedline,
    ReedlineError, ReedlineEvent, ReedlineMenu, Signal,
};

use std::borrow::Cow;
use std::path::PathBuf;

fn is_incomplete_block(src: &str) -> bool {
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    let mut in_str = false;
    let mut str_delim = '\0';
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut iter = src.chars().peekable();
    let mut prev = '\0';

    while let Some(c) = iter.next() {
        let next = *iter.peek().unwrap_or(&'\0');

        if !in_str {
            if !in_block_comment && c == '/' && next == '/' {
                in_line_comment = true;
            }
            if !in_line_comment && c == '/' && next == '*' {
                in_block_comment = true;
            }
            if in_block_comment && prev == '*' && c == '/' {
                in_block_comment = false;
            }
        }

        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            prev = c;
            continue;
        }
        if in_block_comment {
            prev = c;
            continue;
        }

        if !in_str {
            if c == '"' || c == '\'' {
                in_str = true;
                str_delim = c;
                prev = c;
                continue;
            }
        } else {
            if c == str_delim && prev != '\\' {
                in_str = false;
                str_delim = '\0';
                prev = c;
                continue;
            }
            prev = c;
            continue;
        }

        match c {
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            _ => {}
        }
        prev = c;
    }

    in_str || in_block_comment || paren > 0 || brace > 0 || bracket > 0
}

/// A two-state reedline prompt that switches between the primary (`ph>`) and
/// continuation (`..>`) prefix once an incomplete block is detected.
struct PhPrompt {
    /// The normal first-line prompt string, e.g. `"ph>".
    primary: String,
    /// The continuation-line prompt string shown when a block is incomplete.
    cont: String,
    /// `true` while the current input buffer contains an incomplete block.
    is_cont: bool,
}

impl PhPrompt {
    /// Creates a new [`PhPrompt`] with the given primary and continuation strings.
    fn new(primary: &str, cont: &str) -> Self {
        Self {
            primary: primary.into(),
            cont: cont.into(),
            is_cont: false,
        }
    }
}

impl reedline::Prompt for PhPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        if self.is_cont {
            Cow::Borrowed(&self.cont)
        } else {
            Cow::Borrowed(&self.primary)
        }
    }

    fn get_prompt_color(&self) -> Color {
        Color::Grey
    }

    // nothing on the right side of the prompt
    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    // extra indicator appended by reedline for edit mode; we keep it minimal
    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        // return a single space to avoid cramped prompts
        Cow::Borrowed(" ")
    }

    // indicator shown for wrapped/multiline prompts
    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.cont)
    }

    // indicator shown during history search; short and unobtrusive
    fn render_prompt_history_search_indicator(&self, _history_search: PromptHistorySearch) -> Cow<'_, str> {
        Cow::Borrowed("(search)")
    }
}

/// Launches the Phalcom interactive REPL.
///
/// Builds a [`Reedline`] editor with tab-completion, syntax highlighting, Emacs
/// key-bindings, and a persistent history file (`.phalcom_history` in the
/// working directory), then enters the read-eval-print loop until the user
/// types `:quit` or sends `Ctrl-D`.
///
/// # Errors
///
/// Returns a [`ReedlineError`] if the history file cannot be opened or if
/// reedline reports an unrecoverable I/O error.
fn main() -> Result<(), ReedlineError> {
    // session + cwd (unchanged placeholder session)
    let cwd = std::env::current_dir().unwrap_or(PathBuf::from("."));
    let mut session = ReplSession::start(cwd.clone());

    // Build completer + highlighter
    let completer = Box::new(PhalcomCompleter::new(cwd.clone()));
    let highlighter = Box::new(PhalcomHighlighter);

    let mut keybindings: Keybindings = default_emacs_keybindings();

    // TAB opens the completion menu; TAB again moves selection (Fish-like)
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![ReedlineEvent::Menu("completion_menu".into()), ReedlineEvent::MenuNext]),
    );

    // Shift+Enter (and Alt+Enter) insert a newline
    keybindings.add_binding(KeyModifiers::SHIFT, KeyCode::Enter, ReedlineEvent::Edit(vec![EditCommand::InsertNewline]));
    keybindings.add_binding(KeyModifiers::ALT, KeyCode::Enter, ReedlineEvent::Edit(vec![EditCommand::InsertNewline]));

    // Alt+Backspace → delete previous word
    keybindings.add_binding(KeyModifiers::ALT, KeyCode::Backspace, ReedlineEvent::Edit(vec![EditCommand::BackspaceWord]));

    // Cmd/Super+Backspace → kill entire line (if terminal sends SUPER)
    keybindings.add_binding(KeyModifiers::SUPER, KeyCode::Backspace, ReedlineEvent::Edit(vec![EditCommand::KillLine]));

    let edit_mode = Box::new(Emacs::new(keybindings));

    // --- Reedline editor with menu, completer, history ---
    let mut rl = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_completer(completer)
        .with_highlighter(highlighter)
        .with_quick_completions(true) // optional: auto-accept when single candidate
        .with_partial_completions(true); // optional: common-prefix fill

    // IDE-style popup menu (this is the one you asked for)
    // Name must match the one used in the TAB keybinding.
    let completion_menu = IdeMenu::default().with_name("completion_menu");
    rl = rl.with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));

    // persistent history file (unchanged behavior)
    let hist_path = cwd.join(".phalcom_history");
    let history = FileBackedHistory::with_file(10_000, hist_path)?;
    rl = rl.with_history(Box::new(history));

    // Prompt + continuation buffer
    let mut prompt = PhPrompt::new("ph>", "..>");
    let mut buf = String::new();

    loop {
        prompt.is_cont = is_incomplete_block(&buf);

        match rl.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                buf.push_str(&line);

                if line.trim() == ":quit" {
                    println!("Exiting REPL.");
                    break;
                }

                if line.ends_with("\\ ") {
                    buf.pop();
                    buf.push(' ');
                    continue;
                }

                if line.ends_with('\\') {
                    // Line continuation: remove the backslash and keep reading
                    buf.pop();
                    buf.push('\n');
                    continue;
                }

                if prompt.is_cont && line.trim().is_empty() {
                    buf.push('\n');
                    continue;
                }

                if line.trim().is_empty() {
                    buf.clear();
                    continue;
                }

                if is_incomplete_block(&buf) {
                    buf.push('\n');
                    continue;
                }

                match session.eval(&buf) {
                    CellOutcome::Value(val) => {
                        println!("// => {}", val.to_string(&session.vm));
                    }
                    CellOutcome::Unit | CellOutcome::Failed => {}
                }
                buf.clear();

            }
            Ok(Signal::CtrlC) => {
                // println!();
                buf.clear();
            }
            Ok(Signal::CtrlD) => break,
            x => {
                // Other events (resize etc.)
                eprintln!("Event: {x:?}");
            }
        }
    }

    Ok(())
}
