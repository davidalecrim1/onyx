use std::collections::HashMap;

type CommandFn = Box<dyn FnMut() + Send>;

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
}
