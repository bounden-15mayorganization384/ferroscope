use crate::events::AppEvent;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub current_demo: usize,
    pub demo_count: usize,
    pub show_help: bool,
    pub show_explanation: bool,
    pub tick_count: u64,
    pub speed: u8,
    pub paused: bool,
}

impl App {
    pub fn new(demo_count: usize) -> Self {
        Self {
            running: true,
            current_demo: 0,
            demo_count,
            show_help: false,
            show_explanation: false,
            tick_count: 0,
            speed: 1,
            paused: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.running = false,
            AppEvent::SelectDemo(idx) => self.select_demo(idx),
            AppEvent::NextDemo => self.next_demo(),
            AppEvent::PrevDemo => self.prev_demo(),
            AppEvent::TogglePause => self.toggle_pause(),
            AppEvent::Reset => { /* handled by registry externally */ }
            AppEvent::SpeedUp => self.set_speed(self.speed.saturating_add(1)),
            AppEvent::SpeedDown => self.set_speed(self.speed.saturating_sub(1)),
            AppEvent::ToggleHelp => self.show_help = !self.show_help,
            AppEvent::ToggleExplanation => self.show_explanation = !self.show_explanation,
            AppEvent::Screenshot => { /* handled externally */ }
            AppEvent::Tick => self.tick(),
        }
    }

    pub fn next_demo(&mut self) {
        if self.demo_count == 0 {
            return;
        }
        self.current_demo = (self.current_demo + 1) % self.demo_count;
    }

    pub fn prev_demo(&mut self) {
        if self.demo_count == 0 {
            return;
        }
        if self.current_demo == 0 {
            self.current_demo = self.demo_count - 1;
        } else {
            self.current_demo -= 1;
        }
    }

    pub fn select_demo(&mut self, idx: usize) {
        if idx < self.demo_count {
            self.current_demo = idx;
        }
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn set_speed(&mut self, s: u8) {
        self.speed = s.clamp(1, 10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let app = App::new(12);
        assert!(app.running);
        assert_eq!(app.current_demo, 0);
        assert_eq!(app.demo_count, 12);
        assert!(!app.show_help);
        assert!(!app.show_explanation);
        assert_eq!(app.tick_count, 0);
        assert_eq!(app.speed, 1);
        assert!(!app.paused);
    }

    #[test]
    fn test_next_demo_wraps() {
        let mut app = App::new(3);
        app.current_demo = 2;
        app.next_demo();
        assert_eq!(app.current_demo, 0);
    }

    #[test]
    fn test_next_demo_normal() {
        let mut app = App::new(3);
        app.next_demo();
        assert_eq!(app.current_demo, 1);
    }

    #[test]
    fn test_next_demo_empty() {
        let mut app = App::new(0);
        app.next_demo(); // should not panic
        assert_eq!(app.current_demo, 0);
    }

    #[test]
    fn test_prev_demo_wraps() {
        let mut app = App::new(3);
        app.current_demo = 0;
        app.prev_demo();
        assert_eq!(app.current_demo, 2);
    }

    #[test]
    fn test_prev_demo_normal() {
        let mut app = App::new(3);
        app.current_demo = 2;
        app.prev_demo();
        assert_eq!(app.current_demo, 1);
    }

    #[test]
    fn test_prev_demo_empty() {
        let mut app = App::new(0);
        app.prev_demo(); // should not panic
        assert_eq!(app.current_demo, 0);
    }

    #[test]
    fn test_select_demo_in_bounds() {
        let mut app = App::new(5);
        app.select_demo(3);
        assert_eq!(app.current_demo, 3);
    }

    #[test]
    fn test_select_demo_out_of_bounds() {
        let mut app = App::new(3);
        app.select_demo(10);
        assert_eq!(app.current_demo, 0); // unchanged
    }

    #[test]
    fn test_tick_increments() {
        let mut app = App::new(1);
        app.tick();
        assert_eq!(app.tick_count, 1);
        app.tick();
        assert_eq!(app.tick_count, 2);
    }

    #[test]
    fn test_tick_wrapping() {
        let mut app = App::new(1);
        app.tick_count = u64::MAX;
        app.tick();
        assert_eq!(app.tick_count, 0); // wrapping add
    }

    #[test]
    fn test_toggle_pause() {
        let mut app = App::new(1);
        assert!(!app.paused);
        app.toggle_pause();
        assert!(app.paused);
        app.toggle_pause();
        assert!(!app.paused);
    }

    #[test]
    fn test_set_speed_normal() {
        let mut app = App::new(1);
        app.set_speed(5);
        assert_eq!(app.speed, 5);
    }

    #[test]
    fn test_set_speed_clamp_min() {
        let mut app = App::new(1);
        app.set_speed(0);
        assert_eq!(app.speed, 1);
    }

    #[test]
    fn test_set_speed_clamp_max() {
        let mut app = App::new(1);
        app.set_speed(255);
        assert_eq!(app.speed, 10);
    }

    #[test]
    fn test_handle_event_quit() {
        let mut app = App::new(1);
        app.handle_event(AppEvent::Quit);
        assert!(!app.running);
    }

    #[test]
    fn test_handle_event_select_demo() {
        let mut app = App::new(5);
        app.handle_event(AppEvent::SelectDemo(3));
        assert_eq!(app.current_demo, 3);
    }

    #[test]
    fn test_handle_event_next_prev() {
        let mut app = App::new(3);
        app.handle_event(AppEvent::NextDemo);
        assert_eq!(app.current_demo, 1);
        app.handle_event(AppEvent::PrevDemo);
        assert_eq!(app.current_demo, 0);
    }

    #[test]
    fn test_handle_event_speed_up_down() {
        let mut app = App::new(1);
        app.handle_event(AppEvent::SpeedUp);
        assert_eq!(app.speed, 2);
        app.handle_event(AppEvent::SpeedDown);
        assert_eq!(app.speed, 1);
    }

    #[test]
    fn test_handle_event_speed_down_at_min() {
        let mut app = App::new(1);
        app.speed = 1;
        app.handle_event(AppEvent::SpeedDown);
        assert_eq!(app.speed, 1); // clamped at 1
    }

    #[test]
    fn test_handle_event_toggles() {
        let mut app = App::new(1);
        app.handle_event(AppEvent::ToggleHelp);
        assert!(app.show_help);
        app.handle_event(AppEvent::ToggleHelp);
        assert!(!app.show_help);
        app.handle_event(AppEvent::ToggleExplanation);
        assert!(app.show_explanation);
        app.handle_event(AppEvent::ToggleExplanation);
        assert!(!app.show_explanation);
    }

    #[test]
    fn test_handle_event_passthrough() {
        // These events are handled externally, App should not crash
        let mut app = App::new(1);
        let before_running = app.running;
        app.handle_event(AppEvent::Reset);
        app.handle_event(AppEvent::Screenshot);
        app.handle_event(AppEvent::Tick);
        assert_eq!(app.running, before_running);
        assert_eq!(app.tick_count, 1);
    }

    #[test]
    fn test_handle_event_toggle_pause() {
        let mut app = App::new(1);
        app.handle_event(AppEvent::TogglePause);
        assert!(app.paused);
    }
}
