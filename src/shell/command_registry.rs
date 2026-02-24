use std::collections::HashMap;

type CommandFn = Box<dyn FnMut() + Send>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    FileSave,
    PaneFileTreeToggle,
    PaneTerminalToggle,
    PaneTerminalFocus,
    TerminalNewTab,
    TerminalCloseTab,
    CommandPaletteOpen,
}

impl Command {
    /// Converts the string key used in keybindings JSON to a typed Command variant.
    pub fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "file.save"               => Ok(Self::FileSave),
            "pane.file_tree.toggle"   => Ok(Self::PaneFileTreeToggle),
            "pane.terminal.toggle"    => Ok(Self::PaneTerminalToggle),
            "pane.terminal.focus"     => Ok(Self::PaneTerminalFocus),
            "terminal.new_tab"        => Ok(Self::TerminalNewTab),
            "terminal.close_tab"      => Ok(Self::TerminalCloseTab),
            "command_palette.open"    => Ok(Self::CommandPaletteOpen),
            _                         => Err(()),
        }
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, CommandFn>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry { commands: HashMap::new() }
    }

    /// Registers a named command, replacing any existing binding for that name.
    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: FnMut() + Send + 'static,
    {
        self.commands.insert(name.to_string(), Box::new(f));
    }

    /// Executes a named command if registered; silently ignores unknown names.
    pub fn execute(&mut self, name: &str) {
        if let Some(cmd) = self.commands.get_mut(name) {
            cmd();
        }
    }

    /// Returns all registered command names in unspecified order.
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_command_is_callable() {
        let mut reg = CommandRegistry::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called2 = called.clone();
        reg.register("test.command", move || {
            called2.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        reg.execute("test.command");
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn unknown_command_does_not_panic() {
        let mut reg = CommandRegistry::new();
        reg.execute("does.not.exist"); // must not panic
    }

    #[test]
    fn list_commands_returns_all_registered() {
        let mut reg = CommandRegistry::new();
        reg.register("a.command", || {});
        reg.register("b.command", || {});
        let names = reg.command_names();
        assert!(names.contains(&"a.command"));
        assert!(names.contains(&"b.command"));
    }

    #[test]
    fn command_from_str_round_trips() {
        assert_eq!(Command::from_str("file.save").unwrap(), Command::FileSave);
        assert_eq!(Command::from_str("pane.file_tree.toggle").unwrap(), Command::PaneFileTreeToggle);
        assert_eq!(Command::from_str("pane.terminal.toggle").unwrap(), Command::PaneTerminalToggle);
        assert_eq!(Command::from_str("pane.terminal.focus").unwrap(), Command::PaneTerminalFocus);
        assert_eq!(Command::from_str("terminal.new_tab").unwrap(), Command::TerminalNewTab);
        assert_eq!(Command::from_str("terminal.close_tab").unwrap(), Command::TerminalCloseTab);
        assert_eq!(Command::from_str("command_palette.open").unwrap(), Command::CommandPaletteOpen);
        assert!(Command::from_str("does.not.exist").is_err());
    }
}
