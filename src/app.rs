use crossterm::event::KeyCode;

use crate::events::AppEvent;

// Konami sequence: Up Up Down Down Left Right Left Right b a
const KONAMI_SEQUENCE: &[KeyCode] = &[
    KeyCode::Up,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Char('b'),
    KeyCode::Char('a'),
];

/// Ticks that Konami mode stays active (180 ticks ≈ 6s at 30fps)
const KONAMI_ACTIVE_TICKS: u64 = 180;

/// Ticks an achievement flash stays visible (60 ticks ≈ 2s at 30fps)
const ACHIEVEMENT_FLASH_TICKS: u64 = 60;

/// Animated Ferris crab frames (cycling via tick_count).
const CRAB_FRAMES: &[&str] = &["( •_•)  ", "( •_•)> ", "(>•_•)> ", "( •_•)  "];

/// Achievement bit indices and names.
pub const ACHIEVEMENT_EXPLORER: u32 = 1 << 0; // visit 5 demos
pub const ACHIEVEMENT_COMPLETIONIST: u32 = 1 << 1; // visit all 15 demos
pub const ACHIEVEMENT_CONNOISSEUR: u32 = 1 << 2; // visit all 3 Advanced demos (9,10,14)
pub const ACHIEVEMENT_SPEEDRUNNER: u32 = 1 << 3; // complete all 15 in one session (same session = same as completionist)
pub const ACHIEVEMENT_EVANGELIST: u32 = 1 << 4; // awarded externally when tour mode completes

pub const ACHIEVEMENT_NAMES: &[(&str, &str)] = &[
    ("Explorer", "Visited 5 demos"),
    ("Completionist", "Visited all 15 demos"),
    ("Connoisseur", "Played every Advanced demo"),
    ("Speedrunner", "Blazed through all demos"),
    ("Rust Evangelist", "Completed the guided tour"),
];

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

    // Exploration tracking
    pub visited_demos: u32, // bitfield: bit N set when demo N visited

    // Animation helpers
    pub crab_frame: u8, // advances every 8 ticks

    // Rust Facts ticker
    pub fact_tick: u64, // independent counter; increments on Tick

    // Gamification
    pub achievements_unlocked: u32,
    pub achievement_flash: Option<(&'static str, u64)>, // (name, flash_until_tick)

    // Konami easter egg
    konami_buffer: Vec<KeyCode>,
    pub konami_active: bool,
    pub konami_countdown: u64,

    // Explanation panel
    pub explanation_scroll: u16,
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
            visited_demos: 0,
            crab_frame: 0,
            fact_tick: 0,
            achievements_unlocked: 0,
            achievement_flash: None,
            konami_buffer: Vec::with_capacity(10),
            konami_active: false,
            konami_countdown: 0,
            explanation_scroll: 0,
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
            AppEvent::ToggleExplanation => {
                self.show_explanation = !self.show_explanation;
                if !self.show_explanation {
                    self.explanation_scroll = 0;
                }
            }
            AppEvent::Screenshot => { /* handled externally */ }
            AppEvent::ToggleVsMode => { /* handled externally by demo registry */ }
            AppEvent::Tick => self.tick(),
            AppEvent::ScrollUp => {
                if self.show_explanation {
                    self.explanation_scroll = self.explanation_scroll.saturating_sub(1);
                }
            }
            AppEvent::ScrollDown => {
                if self.show_explanation {
                    self.explanation_scroll = self.explanation_scroll.saturating_add(1);
                }
            }
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
        self.fact_tick = self.fact_tick.wrapping_add(1);
        // Advance crab frame every 8 ticks
        if self.tick_count % 8 == 0 {
            self.crab_frame = (self.crab_frame + 1) % CRAB_FRAMES.len() as u8;
        }
        // Decrement konami countdown
        if self.konami_active {
            if self.konami_countdown > 0 {
                self.konami_countdown -= 1;
            }
            if self.konami_countdown == 0 {
                self.konami_active = false;
            }
        }
        // Auto-visit current demo on every tick
        self.visit(self.current_demo);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn set_speed(&mut self, s: u8) {
        self.speed = s.clamp(1, 10);
    }

    pub fn scroll_explanation_up(&mut self) {
        self.explanation_scroll = self.explanation_scroll.saturating_sub(1);
    }

    pub fn scroll_explanation_down(&mut self) {
        self.explanation_scroll = self.explanation_scroll.saturating_add(1);
    }

    pub fn reset_explanation_scroll(&mut self) {
        self.explanation_scroll = 0;
    }

    // ── Exploration tracking ──────────────────────────────────────────────────

    /// Mark demo `idx` as visited and check for new achievements.
    pub fn visit(&mut self, idx: usize) {
        if idx >= 32 {
            return;
        }
        self.visited_demos |= 1u32 << idx;
        if let Some(name) = self.check_achievements() {
            self.achievement_flash = Some((name, self.tick_count + ACHIEVEMENT_FLASH_TICKS));
        }
    }

    /// Number of demos visited so far.
    pub fn visited_count(&self) -> usize {
        self.visited_demos.count_ones() as usize
    }

    // ── Animated Ferris crab ──────────────────────────────────────────────────

    /// Returns the current crab animation frame string.
    pub fn crab_frame_str(frame: u8) -> &'static str {
        CRAB_FRAMES[(frame as usize) % CRAB_FRAMES.len()]
    }

    // ── Achievements ──────────────────────────────────────────────────────────

    /// Check achievement conditions; returns the name of the first newly unlocked
    /// achievement, or `None` if nothing changed.
    pub fn check_achievements(&mut self) -> Option<&'static str> {
        // Explorer: 5 demos visited
        if self.visited_count() >= 5 && (self.achievements_unlocked & ACHIEVEMENT_EXPLORER) == 0 {
            self.achievements_unlocked |= ACHIEVEMENT_EXPLORER;
            return Some("Explorer");
        }
        // Completionist: all demos visited
        if self.visited_count() >= self.demo_count
            && self.demo_count > 0
            && (self.achievements_unlocked & ACHIEVEMENT_COMPLETIONIST) == 0
        {
            self.achievements_unlocked |= ACHIEVEMENT_COMPLETIONIST;
            return Some("Completionist");
        }
        // Connoisseur: Advanced demos 9, 10, 14 visited
        let advanced_mask: u32 = (1 << 9) | (1 << 10) | (1 << 14);
        if (self.visited_demos & advanced_mask) == advanced_mask
            && (self.achievements_unlocked & ACHIEVEMENT_CONNOISSEUR) == 0
        {
            self.achievements_unlocked |= ACHIEVEMENT_CONNOISSEUR;
            return Some("Connoisseur");
        }
        // Speedrunner: same as completionist (unlocked simultaneously when all visited)
        if self.visited_count() >= self.demo_count
            && self.demo_count > 0
            && (self.achievements_unlocked & ACHIEVEMENT_SPEEDRUNNER) == 0
        {
            self.achievements_unlocked |= ACHIEVEMENT_SPEEDRUNNER;
            return Some("Speedrunner");
        }
        None
    }

    /// Unlock the Rust Evangelist achievement (called from tour mode completion).
    pub fn unlock_evangelist(&mut self) {
        if (self.achievements_unlocked & ACHIEVEMENT_EVANGELIST) == 0 {
            self.achievements_unlocked |= ACHIEVEMENT_EVANGELIST;
            self.achievement_flash =
                Some(("Rust Evangelist", self.tick_count + ACHIEVEMENT_FLASH_TICKS));
        }
    }

    /// Returns true if there is an achievement flash to display right now.
    pub fn has_achievement_flash(&self) -> bool {
        self.achievement_flash
            .map(|(_, until)| self.tick_count <= until)
            .unwrap_or(false)
    }

    // ── Konami code ───────────────────────────────────────────────────────────

    /// Returns the expected Konami sequence.
    pub fn konami_sequence() -> &'static [KeyCode] {
        KONAMI_SEQUENCE
    }

    /// Push a key press into the buffer and check for the Konami sequence.
    /// Returns `true` if the sequence was just completed.
    pub fn check_konami(&mut self, key: KeyCode) -> bool {
        self.konami_buffer.push(key);
        if self.konami_buffer.len() > KONAMI_SEQUENCE.len() {
            self.konami_buffer.remove(0);
        }
        if self.konami_buffer == KONAMI_SEQUENCE {
            self.konami_active = true;
            self.konami_countdown = KONAMI_ACTIVE_TICKS;
            self.konami_buffer.clear();
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let app = App::new(15);
        assert!(app.running);
        assert_eq!(app.current_demo, 0);
        assert_eq!(app.demo_count, 15);
        assert!(!app.show_help);
        assert!(!app.show_explanation);
        assert_eq!(app.tick_count, 0);
        assert_eq!(app.speed, 1);
        assert!(!app.paused);
        assert_eq!(app.visited_demos, 0);
        assert_eq!(app.crab_frame, 0);
        assert_eq!(app.fact_tick, 0);
        assert_eq!(app.achievements_unlocked, 0);
        assert!(!app.konami_active);
        assert_eq!(app.explanation_scroll, 0);
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
        assert_eq!(app.fact_tick, 1);
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
    fn test_crab_frame_advances_every_8_ticks() {
        let mut app = App::new(1);
        for _ in 0..7 {
            app.tick();
        }
        assert_eq!(app.crab_frame, 0);
        app.tick(); // 8th tick
        assert_eq!(app.crab_frame, 1);
    }

    #[test]
    fn test_crab_frame_str() {
        for i in 0..4u8 {
            let s = App::crab_frame_str(i);
            assert!(!s.is_empty());
        }
        // Wraps correctly
        assert_eq!(App::crab_frame_str(0), App::crab_frame_str(4));
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
    fn test_explanation_scroll_starts_at_zero() {
        let app = App::new(1);
        assert_eq!(app.explanation_scroll, 0);
    }

    #[test]
    fn test_scroll_explanation_down_increments() {
        let mut app = App::new(1);
        app.scroll_explanation_down();
        assert_eq!(app.explanation_scroll, 1);
        app.scroll_explanation_down();
        assert_eq!(app.explanation_scroll, 2);
    }

    #[test]
    fn test_scroll_explanation_up_decrements() {
        let mut app = App::new(1);
        app.explanation_scroll = 5;
        app.scroll_explanation_up();
        assert_eq!(app.explanation_scroll, 4);
    }

    #[test]
    fn test_scroll_explanation_up_saturates_at_zero() {
        let mut app = App::new(1);
        app.explanation_scroll = 0;
        app.scroll_explanation_up();
        assert_eq!(app.explanation_scroll, 0); // saturating_sub, no underflow
    }

    #[test]
    fn test_reset_explanation_scroll() {
        let mut app = App::new(1);
        app.explanation_scroll = 42;
        app.reset_explanation_scroll();
        assert_eq!(app.explanation_scroll, 0);
    }

    #[test]
    fn test_scroll_down_saturates_at_max() {
        let mut app = App::new(1);
        app.explanation_scroll = u16::MAX;
        app.scroll_explanation_down(); // saturating_add stays at MAX
        assert_eq!(app.explanation_scroll, u16::MAX);
    }

    #[test]
    fn test_handle_event_scroll_down_when_shown() {
        let mut app = App::new(1);
        app.show_explanation = true;
        app.handle_event(AppEvent::ScrollDown);
        assert_eq!(app.explanation_scroll, 1);
    }

    #[test]
    fn test_handle_event_scroll_up_when_shown() {
        let mut app = App::new(1);
        app.show_explanation = true;
        app.explanation_scroll = 3;
        app.handle_event(AppEvent::ScrollUp);
        assert_eq!(app.explanation_scroll, 2);
    }

    #[test]
    fn test_handle_event_scroll_down_when_hidden_noop() {
        let mut app = App::new(1);
        app.show_explanation = false;
        app.handle_event(AppEvent::ScrollDown);
        assert_eq!(app.explanation_scroll, 0); // not shown → no change
    }

    #[test]
    fn test_handle_event_scroll_up_when_hidden_noop() {
        let mut app = App::new(1);
        app.show_explanation = false;
        app.explanation_scroll = 5;
        app.handle_event(AppEvent::ScrollUp);
        assert_eq!(app.explanation_scroll, 5); // not shown → no change
    }

    #[test]
    fn test_toggle_explanation_off_resets_scroll() {
        let mut app = App::new(1);
        app.show_explanation = true;
        app.explanation_scroll = 7;
        app.handle_event(AppEvent::ToggleExplanation); // turns off
        assert!(!app.show_explanation);
        assert_eq!(app.explanation_scroll, 0);
    }

    #[test]
    fn test_toggle_explanation_on_preserves_zero_scroll() {
        let mut app = App::new(1);
        app.show_explanation = false;
        app.explanation_scroll = 0;
        app.handle_event(AppEvent::ToggleExplanation); // turns on
        assert!(app.show_explanation);
        assert_eq!(app.explanation_scroll, 0);
    }

    #[test]
    fn test_handle_event_passthrough() {
        let mut app = App::new(1);
        let before_running = app.running;
        app.handle_event(AppEvent::Reset);
        app.handle_event(AppEvent::Screenshot);
        app.handle_event(AppEvent::ToggleVsMode);
        assert_eq!(app.running, before_running);
        app.handle_event(AppEvent::Tick);
        assert_eq!(app.tick_count, 1);
    }

    #[test]
    fn test_handle_event_toggle_pause() {
        let mut app = App::new(1);
        app.handle_event(AppEvent::TogglePause);
        assert!(app.paused);
    }

    // ── Visit / achievement tests ─────────────────────────────────────────────

    #[test]
    fn test_visit_sets_bit() {
        let mut app = App::new(15);
        app.visit(3);
        assert_eq!(app.visited_demos & (1 << 3), 1 << 3);
    }

    #[test]
    fn test_visited_count() {
        let mut app = App::new(15);
        assert_eq!(app.visited_count(), 0);
        app.visit(0);
        app.visit(1);
        app.visit(2);
        assert_eq!(app.visited_count(), 3);
    }

    #[test]
    fn test_visit_out_of_bounds_no_panic() {
        let mut app = App::new(15);
        app.visit(99); // idx >= 32, ignored silently
    }

    #[test]
    fn test_achievement_explorer_unlocks_at_5() {
        let mut app = App::new(15);
        for i in 0..4 {
            app.visit(i);
        }
        assert_eq!(app.achievements_unlocked & ACHIEVEMENT_EXPLORER, 0);
        app.visit(4);
        assert_ne!(app.achievements_unlocked & ACHIEVEMENT_EXPLORER, 0);
    }

    #[test]
    fn test_achievement_completionist_unlocks_all() {
        let mut app = App::new(3);
        app.visit(0);
        app.visit(1);
        assert_eq!(app.achievements_unlocked & ACHIEVEMENT_COMPLETIONIST, 0);
        app.visit(2);
        assert_ne!(app.achievements_unlocked & ACHIEVEMENT_COMPLETIONIST, 0);
    }

    #[test]
    fn test_achievement_connoisseur() {
        let mut app = App::new(15);
        app.visit(9);
        app.visit(10);
        assert_eq!(app.achievements_unlocked & ACHIEVEMENT_CONNOISSEUR, 0);
        app.visit(14);
        assert_ne!(app.achievements_unlocked & ACHIEVEMENT_CONNOISSEUR, 0);
    }

    #[test]
    fn test_achievement_flash_active() {
        let mut app = App::new(15);
        // Force an achievement flash
        app.achievement_flash = Some(("Test", app.tick_count + 60));
        assert!(app.has_achievement_flash());
    }

    #[test]
    fn test_achievement_flash_expires() {
        let mut app = App::new(15);
        app.tick_count = 100;
        app.achievement_flash = Some(("Test", 99)); // already expired
        assert!(!app.has_achievement_flash());
    }

    #[test]
    fn test_unlock_evangelist() {
        let mut app = App::new(15);
        app.unlock_evangelist();
        assert_ne!(app.achievements_unlocked & ACHIEVEMENT_EVANGELIST, 0);
        assert!(app.achievement_flash.is_some());
    }

    // ── Konami tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_konami_sequence_correct_length() {
        assert_eq!(App::konami_sequence().len(), 10);
    }

    #[test]
    fn test_konami_partial_not_active() {
        let mut app = App::new(15);
        for key in &KONAMI_SEQUENCE[..9] {
            app.check_konami(*key);
        }
        assert!(!app.konami_active);
    }

    #[test]
    fn test_konami_full_sequence_activates() {
        let mut app = App::new(15);
        for &key in KONAMI_SEQUENCE {
            app.check_konami(key);
        }
        assert!(app.konami_active);
        assert_eq!(app.konami_countdown, KONAMI_ACTIVE_TICKS);
    }

    #[test]
    fn test_konami_wrong_sequence_no_activate() {
        let mut app = App::new(15);
        let wrong = [KeyCode::Char('z'); 10];
        for key in wrong {
            app.check_konami(key);
        }
        assert!(!app.konami_active);
    }

    #[test]
    fn test_konami_countdown_decrements() {
        let mut app = App::new(1);
        // Activate konami manually
        app.konami_active = true;
        app.konami_countdown = 5;
        app.tick();
        assert_eq!(app.konami_countdown, 4);
    }

    #[test]
    fn test_konami_expires_after_countdown() {
        let mut app = App::new(1);
        app.konami_active = true;
        app.konami_countdown = 1;
        app.tick(); // countdown hits 0
        assert!(!app.konami_active);
    }
}
