use std::path::Path;

use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    history::FileHistory,
    validate::Validator,
    CompletionType, Config, Editor, Helper,
};

struct RjshEditorHelper(FilenameCompleter);

impl Completer for RjshEditorHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.0.complete(line, pos, ctx)
    }

    fn update(
        &self,
        line: &mut rustyline::line_buffer::LineBuffer,
        start: usize,
        elected: &str,
        cl: &mut rustyline::Changeset,
    ) {
        self.0.update(line, start, elected, cl);
    }
}
impl Hinter for RjshEditorHelper {
    type Hint = String;
    //TODO: Do something with this
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Highlighter for RjshEditorHelper {}

impl Validator for RjshEditorHelper {}

impl Helper for RjshEditorHelper {}

pub struct RjshEditor {
    internal: Editor<RjshEditorHelper, FileHistory>,
}

impl RjshEditor {
    pub fn new() -> Result<Self, ReadlineError> {
        let config = Config::builder()
            .max_history_size(10_000)?
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .build();

        let mut internal = rustyline::Editor::with_config(config)?;
        internal.set_helper(Some(RjshEditorHelper(FilenameCompleter::new())));

        Ok(Self { internal })
    }

    pub fn readline(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.internal.readline(prompt)
    }

    pub fn load_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<(), ReadlineError> {
        self.internal.load_history(path)
    }
    pub fn add_history_entry<S: AsRef<str> + Into<String>>(
        &mut self,
        line: S,
    ) -> Result<bool, ReadlineError> {
        self.internal.add_history_entry(line)
    }

    pub fn save_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<(), ReadlineError> {
        self.internal.save_history(path)
    }
}
