use winit::keyboard::{Key, ModifiersState, NamedKey};

/// Editor actions that can be triggered by keybindings or a future command palette.
///
/// Designed as a flat enum so a command palette can enumerate variants, display
/// their names, and dispatch through the same `handle_action` path.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    InsertChar(char),
    Backspace,
    Delete,
    Enter,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
    Save,
}

/// Maps a key press and active modifiers to an editor action.
pub fn resolve_action(key: &Key, modifiers: ModifiersState) -> Option<Action> {
    let has_command = modifiers.super_key() || modifiers.control_key();

    if has_command {
        if let Key::Character(ch) = key {
            if ch.as_str().eq_ignore_ascii_case("s") {
                return Some(Action::Save);
            }
        }
        return None;
    }

    match key {
        Key::Named(NamedKey::Backspace) => Some(Action::Backspace),
        Key::Named(NamedKey::Delete) => Some(Action::Delete),
        Key::Named(NamedKey::Enter) => Some(Action::Enter),
        Key::Named(NamedKey::ArrowLeft) => Some(Action::MoveLeft),
        Key::Named(NamedKey::ArrowRight) => Some(Action::MoveRight),
        Key::Named(NamedKey::ArrowUp) => Some(Action::MoveUp),
        Key::Named(NamedKey::ArrowDown) => Some(Action::MoveDown),
        Key::Named(NamedKey::Home) => Some(Action::MoveHome),
        Key::Named(NamedKey::End) => Some(Action::MoveEnd),
        Key::Character(ch) => {
            let mut chars = ch.chars();
            let first = chars.next()?;
            if chars.next().is_none() {
                Some(Action::InsertChar(first))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_character_key() {
        let action = resolve_action(&Key::Character("a".into()), ModifiersState::empty());
        assert_eq!(action, Some(Action::InsertChar('a')));
    }

    #[test]
    fn resolve_backspace() {
        let action = resolve_action(&Key::Named(NamedKey::Backspace), ModifiersState::empty());
        assert_eq!(action, Some(Action::Backspace));
    }

    #[test]
    fn resolve_delete() {
        let action = resolve_action(&Key::Named(NamedKey::Delete), ModifiersState::empty());
        assert_eq!(action, Some(Action::Delete));
    }

    #[test]
    fn resolve_enter() {
        let action = resolve_action(&Key::Named(NamedKey::Enter), ModifiersState::empty());
        assert_eq!(action, Some(Action::Enter));
    }

    #[test]
    fn resolve_arrows() {
        assert_eq!(
            resolve_action(&Key::Named(NamedKey::ArrowLeft), ModifiersState::empty()),
            Some(Action::MoveLeft)
        );
        assert_eq!(
            resolve_action(&Key::Named(NamedKey::ArrowRight), ModifiersState::empty()),
            Some(Action::MoveRight)
        );
        assert_eq!(
            resolve_action(&Key::Named(NamedKey::ArrowUp), ModifiersState::empty()),
            Some(Action::MoveUp)
        );
        assert_eq!(
            resolve_action(&Key::Named(NamedKey::ArrowDown), ModifiersState::empty()),
            Some(Action::MoveDown)
        );
    }

    #[test]
    fn resolve_home_end() {
        assert_eq!(
            resolve_action(&Key::Named(NamedKey::Home), ModifiersState::empty()),
            Some(Action::MoveHome)
        );
        assert_eq!(
            resolve_action(&Key::Named(NamedKey::End), ModifiersState::empty()),
            Some(Action::MoveEnd)
        );
    }

    #[test]
    fn cmd_s_resolves_to_save() {
        let action = resolve_action(&Key::Character("s".into()), ModifiersState::SUPER);
        assert_eq!(action, Some(Action::Save));
    }

    #[test]
    fn ctrl_s_resolves_to_save() {
        let action = resolve_action(&Key::Character("s".into()), ModifiersState::CONTROL);
        assert_eq!(action, Some(Action::Save));
    }

    #[test]
    fn cmd_other_key_returns_none() {
        let action = resolve_action(&Key::Character("a".into()), ModifiersState::SUPER);
        assert_eq!(action, None);
    }

    #[test]
    fn multi_char_key_returns_none() {
        let action = resolve_action(&Key::Character("ab".into()), ModifiersState::empty());
        assert_eq!(action, None);
    }
}
