use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    Quit,
    SelectDemo(usize),
    NextDemo,
    PrevDemo,
    TogglePause,
    Reset,
    SpeedUp,
    SpeedDown,
    ToggleHelp,
    ToggleExplanation,
    Screenshot,
    ToggleVsMode,
    Tick,
    ScrollUp,
    ScrollDown,
}

pub fn key_event_to_app_event(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(AppEvent::Quit),
        KeyCode::Char('1') => Some(AppEvent::SelectDemo(0)),
        KeyCode::Char('2') => Some(AppEvent::SelectDemo(1)),
        KeyCode::Char('3') => Some(AppEvent::SelectDemo(2)),
        KeyCode::Char('4') => Some(AppEvent::SelectDemo(3)),
        KeyCode::Char('5') => Some(AppEvent::SelectDemo(4)),
        KeyCode::Char('6') => Some(AppEvent::SelectDemo(5)),
        KeyCode::Char('7') => Some(AppEvent::SelectDemo(6)),
        KeyCode::Char('8') => Some(AppEvent::SelectDemo(7)),
        KeyCode::Char('9') => Some(AppEvent::SelectDemo(8)),
        KeyCode::Char('0') => Some(AppEvent::SelectDemo(9)),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(AppEvent::SelectDemo(10)),
        KeyCode::Char('b') | KeyCode::Char('B') => Some(AppEvent::SelectDemo(11)),
        KeyCode::Char('c') | KeyCode::Char('C') => Some(AppEvent::SelectDemo(12)),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(AppEvent::SelectDemo(13)),
        KeyCode::Char('f') | KeyCode::Char('F') => Some(AppEvent::SelectDemo(14)),
        KeyCode::Left | KeyCode::Char('h') => Some(AppEvent::PrevDemo),
        KeyCode::Right | KeyCode::Char('l') => Some(AppEvent::NextDemo),
        KeyCode::Char(' ') => Some(AppEvent::TogglePause),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(AppEvent::Reset),
        KeyCode::Char('+') => Some(AppEvent::SpeedUp),
        KeyCode::Char('-') => Some(AppEvent::SpeedDown),
        KeyCode::Char('?') => Some(AppEvent::ToggleHelp),
        KeyCode::Char('e') | KeyCode::Char('E') => Some(AppEvent::ToggleExplanation),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(AppEvent::Screenshot),
        KeyCode::Char('v') | KeyCode::Char('V') => Some(AppEvent::ToggleVsMode),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Some(AppEvent::ScrollUp),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Some(AppEvent::ScrollDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_quit_keys() {
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('q'))), Some(AppEvent::Quit));
        assert_eq!(key_event_to_app_event(key(KeyCode::Esc)), Some(AppEvent::Quit));
    }

    #[test]
    fn test_demo_select_keys() {
        for (ch, idx) in [('1',0),('2',1),('3',2),('4',3),('5',4),('6',5),('7',6),('8',7),('9',8)] {
            assert_eq!(key_event_to_app_event(key(KeyCode::Char(ch))), Some(AppEvent::SelectDemo(idx)));
        }
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('0'))), Some(AppEvent::SelectDemo(9)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('a'))), Some(AppEvent::SelectDemo(10)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('A'))), Some(AppEvent::SelectDemo(10)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('b'))), Some(AppEvent::SelectDemo(11)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('B'))), Some(AppEvent::SelectDemo(11)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('c'))), Some(AppEvent::SelectDemo(12)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('C'))), Some(AppEvent::SelectDemo(12)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('d'))), Some(AppEvent::SelectDemo(13)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('D'))), Some(AppEvent::SelectDemo(13)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('f'))), Some(AppEvent::SelectDemo(14)));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('F'))), Some(AppEvent::SelectDemo(14)));
    }

    #[test]
    fn test_nav_keys() {
        assert_eq!(key_event_to_app_event(key(KeyCode::Left)), Some(AppEvent::PrevDemo));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('h'))), Some(AppEvent::PrevDemo));
        assert_eq!(key_event_to_app_event(key(KeyCode::Right)), Some(AppEvent::NextDemo));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('l'))), Some(AppEvent::NextDemo));
    }

    #[test]
    fn test_action_keys() {
        assert_eq!(key_event_to_app_event(key(KeyCode::Char(' '))), Some(AppEvent::TogglePause));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('r'))), Some(AppEvent::Reset));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('R'))), Some(AppEvent::Reset));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('+'))), Some(AppEvent::SpeedUp));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('-'))), Some(AppEvent::SpeedDown));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('?'))), Some(AppEvent::ToggleHelp));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('e'))), Some(AppEvent::ToggleExplanation));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('E'))), Some(AppEvent::ToggleExplanation));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('s'))), Some(AppEvent::Screenshot));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('S'))), Some(AppEvent::Screenshot));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('v'))), Some(AppEvent::ToggleVsMode));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('V'))), Some(AppEvent::ToggleVsMode));
    }

    #[test]
    fn test_unknown_key_returns_none() {
        assert_eq!(key_event_to_app_event(key(KeyCode::F(1))), None);
        assert_eq!(key_event_to_app_event(key(KeyCode::Enter)), None);
        assert_eq!(key_event_to_app_event(key(KeyCode::Tab)), None);
        assert_eq!(key_event_to_app_event(key(KeyCode::Backspace)), None);
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('z'))), None);
    }

    #[test]
    fn test_scroll_up_keys() {
        assert_eq!(key_event_to_app_event(key(KeyCode::Up)), Some(AppEvent::ScrollUp));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('k'))), Some(AppEvent::ScrollUp));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('K'))), Some(AppEvent::ScrollUp));
    }

    #[test]
    fn test_scroll_down_keys() {
        assert_eq!(key_event_to_app_event(key(KeyCode::Down)), Some(AppEvent::ScrollDown));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('j'))), Some(AppEvent::ScrollDown));
        assert_eq!(key_event_to_app_event(key(KeyCode::Char('J'))), Some(AppEvent::ScrollDown));
    }
}
