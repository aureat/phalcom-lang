//! Entry point and prompt implementation for the Phalcom REPL.
//!
//! Assembles the [`reedline`]-backed interactive REPL: configures the editor
//! with tab-completion, syntax highlighting, persistent file history, and a
//! two-state prompt (`ph>` / `...`) driven by §D7 parser classification.

use phalcom_repl::completer::PhalcomCompleter;
use phalcom_repl::highlighter::PhalcomHighlighter;
use phalcom_repl::oracle::ReplOracle;
use phalcom_repl::repl::{CellOutcome, ReplSession, ValueExt};
use phalcom_repl::snapshot::ReplSnapshot;
use phalcom_repl::validator::{classify, explicit_continuation, PhalcomValidator, Verdict};

use reedline::{
    default_emacs_keybindings, Color, EditCommand, Emacs, FileBackedHistory, IdeMenu, KeyCode,
    KeyModifiers, Keybindings, MenuBuilder, PromptEditMode, PromptHistorySearch, Reedline,
    ReedlineError, ReedlineEvent, ReedlineMenu, Signal,
};

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A two-state reedline prompt that switches between the primary (`ph>`) and
/// continuation (`...`) prefix once an incomplete block is detected.
struct PhPrompt {
    primary: String,
    cont: String,
    is_cont: bool,
}

impl PhPrompt {
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

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed(" ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.cont)
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Borrowed("(search)")
    }
}

/// Launches the Phalcom interactive REPL.
fn main() -> Result<(), ReedlineError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut session = ReplSession::start(cwd.clone());

    let snapshot_cell = Arc::new(Mutex::new(ReplSnapshot::new()));
    let oracle = Arc::new(ReplOracle::new(snapshot_cell.clone()));

    let completer = Box::new(PhalcomCompleter::new(cwd.clone(), oracle.clone()));
    let highlighter = Box::new(PhalcomHighlighter::new(oracle.clone()));
    let validator = Box::new(PhalcomValidator);

    let mut keybindings: Keybindings = default_emacs_keybindings();

    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".into()),
            ReedlineEvent::MenuNext,
        ]),
    );

    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Backspace,
        ReedlineEvent::Edit(vec![EditCommand::BackspaceWord]),
    );
    keybindings.add_binding(
        KeyModifiers::SUPER,
        KeyCode::Backspace,
        ReedlineEvent::Edit(vec![EditCommand::KillLine]),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));

    let mut rl = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_validator(validator)
        .with_completer(completer)
        .with_highlighter(highlighter)
        .with_quick_completions(false)
        .with_partial_completions(true);

    let completion_menu = IdeMenu::default().with_name("completion_menu");
    rl = rl.with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));

    let hist_path = cwd.join(".phalcom_history");
    let history = FileBackedHistory::with_file(10_000, hist_path)?;
    rl = rl.with_history(Box::new(history));

    let mut prompt = PhPrompt::new("ph>", "...");
    let mut buf = String::new();

    loop {
        prompt.is_cont = !buf.is_empty();

        match rl.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let trimmed = line.trim();
                if buf.is_empty() && trimmed.starts_with(':') {
                    match trimmed {
                        ":quit" => {
                            println!("Exiting REPL.");
                            break;
                        }
                        ":reload" => {
                            // `reload` halts at the first cell that fails and says which
                            // one (U-REPL §07 §5). Discarding that answer left the user
                            // unable to tell a full reload from a partial one.
                            let complete = session.reload();
                            if !complete {
                                eprintln!(
                                    "Reload stopped early; the session is at the state reached before that cell."
                                );
                            }
                            if let Ok(mut snap) = snapshot_cell.lock() {
                                *snap = ReplSnapshot::capture(&session.vm, session.module);
                            }
                        }
                        ":reset" => {
                            println!("Command ':reset' is not yet implemented.");
                        }
                        ":help" => {
                            println!("Command ':help' is not yet implemented.");
                        }
                        cmd => {
                            println!("Unknown command '{cmd}'. Available commands: :reload, :reset, :help");
                        }
                    }
                    buf.clear();
                    continue;
                }

                let (has_explicit_cont, effective_line) = match explicit_continuation(&line) {
                    Some(joined) => (true, joined),
                    None => (false, line.clone()),
                };

                buf.push_str(&effective_line);

                if has_explicit_cont {
                    buf.push('\n');
                    continue;
                }

                let is_blank_submit = !buf.trim().is_empty() && trimmed.is_empty();

                if !is_blank_submit && classify(&buf) == Verdict::Incomplete {
                    buf.push('\n');
                    continue;
                }

                if buf.trim().is_empty() {
                    buf.clear();
                    continue;
                }

                match session.eval(&buf) {
                    CellOutcome::Value(val) => {
                        // Echo runs *after* `eval` has settled the outcome, which is what
                        // guarantees a raising `toString` cannot turn a successful cell
                        // into a failed one (§S4, PDR-0008 §4). Do not move it inside `eval`.
                        let display = val.to_string_guarded(&mut session.vm);
                        println!("// => {display}");
                    }
                    // `eval` has already printed the diagnostic for `Failed`.
                    CellOutcome::Unit | CellOutcome::Failed => {}
                }

                if let Ok(mut snap) = snapshot_cell.lock() {
                    *snap = ReplSnapshot::capture(&session.vm, session.module);
                }

                buf.clear();
            }
            Ok(Signal::CtrlC) => {
                buf.clear();
                prompt.is_cont = false;
            }
            Ok(Signal::CtrlD) => break,
            // A read error is not transient: on a closed or non-tty stdin every
            // subsequent read fails identically, so continuing spins forever
            // printing the same line. Report once and leave.
            Err(err) => {
                eprintln!("Input closed: {err}");
                break;
            }
        }
    }

    Ok(())
}
