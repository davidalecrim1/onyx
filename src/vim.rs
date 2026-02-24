#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Escape,
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferCommand {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBack,
    MoveWordEnd,
    MoveLineStart,
    MoveLineEnd,
    MoveFirstLine,
    MoveLastLine,
    MoveParagraphForward,
    MoveParagraphBack,
    Insert(char),
    InsertNewline,
    DeleteBefore,
    DeleteCharAtCursor,
    DeleteLine,
    Yank,
    Delete,
    Change,
    /// Carries the text to insert so the engine's register never needs to escape into the editor.
    Paste(String),
    Undo,
    Redo,
    StartVisual,
    StartVisualLine,
    ClearSelection,
}

pub struct VimEngine {
    mode: Mode,
    /// The first key of an in-progress multi-key sequence; cleared once the sequence resolves or is cancelled.
    pending: Option<char>,
    /// Stores the last yanked or deleted text; filled by the editor after buffer operations.
    register: String,
}

impl VimEngine {
    /// Creates a new engine in Normal mode with no pending state and an empty register.
    pub fn new() -> Self {
        VimEngine { mode: Mode::Normal, pending: None, register: String::new() }
    }

    /// Returns the current modal state.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Stores yanked or deleted text; called by the editor after it performs the buffer operation.
    pub fn set_register(&mut self, text: String) {
        self.register = text;
    }

    /// Dispatches a key to the handler for the current mode and returns any resulting command.
    pub fn handle_key(&mut self, key: Key) -> Option<BufferCommand> {
        match self.mode {
            Mode::Normal => self.handle_normal(key),
            Mode::Insert => self.handle_insert(key),
            Mode::Visual => self.handle_visual(key),
        }
    }

    fn handle_normal(&mut self, key: Key) -> Option<BufferCommand> {
        if let Some(pending) = self.pending.take() {
            return match (pending, &key) {
                ('g', Key::Char('g')) => Some(BufferCommand::MoveFirstLine),
                ('d', Key::Char('d')) => Some(BufferCommand::DeleteLine),
                ('y', Key::Char('y')) => Some(BufferCommand::Yank),
                _ => None,
            };
        }

        match key {
            Key::Char('h') | Key::Left  => Some(BufferCommand::MoveLeft),
            Key::Char('l') | Key::Right => Some(BufferCommand::MoveRight),
            Key::Char('k') | Key::Up    => Some(BufferCommand::MoveUp),
            Key::Char('j') | Key::Down  => Some(BufferCommand::MoveDown),
            Key::Char('w') => Some(BufferCommand::MoveWordForward),
            Key::Char('b') => Some(BufferCommand::MoveWordBack),
            Key::Char('e') => Some(BufferCommand::MoveWordEnd),
            Key::Char('0') => Some(BufferCommand::MoveLineStart),
            Key::Char('$') => Some(BufferCommand::MoveLineEnd),
            Key::Char('G') => Some(BufferCommand::MoveLastLine),
            Key::Char('{') => Some(BufferCommand::MoveParagraphBack),
            Key::Char('}') => Some(BufferCommand::MoveParagraphForward),
            Key::Char('i') => { self.mode = Mode::Insert; None }
            Key::Char('a') => { self.mode = Mode::Insert; Some(BufferCommand::MoveRight) }
            Key::Char('A') => { self.mode = Mode::Insert; Some(BufferCommand::MoveLineEnd) }
            Key::Char('o') => { self.mode = Mode::Insert; Some(BufferCommand::InsertNewline) }
            Key::Char('v') => { self.mode = Mode::Visual; Some(BufferCommand::StartVisual) }
            Key::Char('V') => { self.mode = Mode::Visual; Some(BufferCommand::StartVisualLine) }
            Key::Char('x') => Some(BufferCommand::DeleteCharAtCursor),
            Key::Char(c @ ('g' | 'd' | 'c' | 'y')) => { self.pending = Some(c); None }
            Key::Char('p') => Some(BufferCommand::Paste(self.register.clone())),
            Key::Char('u') => Some(BufferCommand::Undo),
            Key::Char('\x12') => Some(BufferCommand::Redo), // Ctrl-R
            _ => None,
        }
    }

    fn handle_insert(&mut self, key: Key) -> Option<BufferCommand> {
        match key {
            Key::Escape    => { self.mode = Mode::Normal; None }
            Key::Backspace => Some(BufferCommand::DeleteBefore),
            Key::Enter     => Some(BufferCommand::InsertNewline),
            Key::Char(c)   => Some(BufferCommand::Insert(c)),
            _              => None,
        }
    }

    fn handle_visual(&mut self, key: Key) -> Option<BufferCommand> {
        match key {
            Key::Escape => { self.mode = Mode::Normal; Some(BufferCommand::ClearSelection) }
            Key::Char('h') | Key::Left  => Some(BufferCommand::MoveLeft),
            Key::Char('l') | Key::Right => Some(BufferCommand::MoveRight),
            Key::Char('k') | Key::Up    => Some(BufferCommand::MoveUp),
            Key::Char('j') | Key::Down  => Some(BufferCommand::MoveDown),
            Key::Char('d') | Key::Char('x') => {
                self.mode = Mode::Normal;
                Some(BufferCommand::Delete)
            }
            Key::Char('y') => { self.mode = Mode::Normal; Some(BufferCommand::Yank) }
            Key::Char('c') => { self.mode = Mode::Insert; Some(BufferCommand::Change) }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> VimEngine {
        VimEngine::new()
    }

    #[test]
    fn normal_h_emits_move_left() {
        let mut vm = engine();
        let cmd = vm.handle_key(Key::Char('h'));
        assert_eq!(cmd, Some(BufferCommand::MoveLeft));
        assert_eq!(vm.mode(), Mode::Normal);
    }

    #[test]
    fn normal_i_enters_insert() {
        let mut vm = engine();
        let cmd = vm.handle_key(Key::Char('i'));
        assert_eq!(cmd, None);
        assert_eq!(vm.mode(), Mode::Insert);
    }

    #[test]
    fn insert_escape_returns_normal() {
        let mut vm = engine();
        vm.handle_key(Key::Char('i'));
        let cmd = vm.handle_key(Key::Escape);
        assert_eq!(cmd, None);
        assert_eq!(vm.mode(), Mode::Normal);
    }

    #[test]
    fn normal_v_enters_visual() {
        let mut vm = engine();
        vm.handle_key(Key::Char('v'));
        assert_eq!(vm.mode(), Mode::Visual);
    }

    #[test]
    fn insert_char_emits_insert_char() {
        let mut vm = engine();
        vm.handle_key(Key::Char('i'));
        let cmd = vm.handle_key(Key::Char('a'));
        assert_eq!(cmd, Some(BufferCommand::Insert('a')));
    }

    #[test]
    fn paste_carries_register_contents() {
        let mut vm = engine();
        vm.set_register("hello".to_string());
        let cmd = vm.handle_key(Key::Char('p'));
        assert_eq!(cmd, Some(BufferCommand::Paste("hello".to_string())));
    }
}
