use std::time::Duration;

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme;

// Re-export demo modules (commented out until implemented)
pub mod d01_ownership;
pub mod d02_memory;
pub mod d03_zero_cost;
pub mod d04_concurrency;
pub mod d05_async;
pub mod d06_performance;
pub mod d07_type_system;
pub mod d08_error_handling;
pub mod d09_lifetimes;
pub mod d10_unsafe;
pub mod d11_wasm;
pub mod d12_system_metrics;
pub mod d13_compile_time;
pub mod d14_cargo_ecosystem;
pub mod d15_no_std;

pub trait Demo: Send + Sync {
    fn tick(&mut self, dt: Duration);
    fn render(&self, frame: &mut Frame, area: Rect);
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn explanation(&self) -> &'static str;
    fn reset(&mut self);
    fn toggle_pause(&mut self);
    fn is_paused(&self) -> bool;
    fn set_speed(&mut self, speed: u8);
    fn speed(&self) -> u8;
    /// Toggle an optional split-view comparison mode (e.g. Rust vs C++).
    /// Default implementation is a no-op.
    fn toggle_vsmode(&mut self) {}
}

// ─── Placeholder Demo ────────────────────────────────────────────────────────

#[derive(Debug)]
struct PlaceholderDemo {
    name: &'static str,
    description: &'static str,
    explanation: &'static str,
    paused: bool,
    speed: u8,
    tick_count: u64,
}

impl PlaceholderDemo {
    fn new(name: &'static str, description: &'static str, explanation: &'static str) -> Self {
        Self {
            name,
            description,
            explanation,
            paused: false,
            speed: 1,
            tick_count: 0,
        }
    }
}

impl Demo for PlaceholderDemo {
    fn tick(&mut self, _dt: Duration) {
        if !self.paused {
            self.tick_count = self.tick_count.wrapping_add(1);
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(Span::styled(
                format!("🦀  {}", self.name),
                Style::default()
                    .fg(theme::RUST_ORANGE)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(self.description, theme::dim_style())),
            Line::from(""),
            Line::from(Span::styled(
                format!("  tick: {}", self.tick_count),
                theme::dim_style(),
            )),
        ];

        let para = Paragraph::new(lines).block(
            Block::default()
                .title(self.name)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::RUST_ORANGE)),
        );
        frame.render_widget(para, area);
    }

    fn name(&self) -> &'static str {
        self.name
    }
    fn description(&self) -> &'static str {
        self.description
    }
    fn explanation(&self) -> &'static str {
        self.explanation
    }
    fn reset(&mut self) {
        self.tick_count = 0;
        self.paused = false;
    }
    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    fn is_paused(&self) -> bool {
        self.paused
    }
    fn set_speed(&mut self, speed: u8) {
        self.speed = speed.clamp(1, 10);
    }
    fn speed(&self) -> u8 {
        self.speed
    }
}

// ─── DemoRegistry ─────────────────────────────────────────────────────────────

pub struct DemoRegistry {
    demos: Vec<Box<dyn Demo>>,
}

impl DemoRegistry {
    pub fn new() -> Self {
        let demos: Vec<Box<dyn Demo>> = vec![
            Box::new(d01_ownership::OwnershipDemo::new()),
            Box::new(d02_memory::MemoryDemo::new()),
            Box::new(d03_zero_cost::ZeroCostDemo::new()),
            Box::new(d04_concurrency::ConcurrencyDemo::new()),
            Box::new(d05_async::AsyncDemo::new()),
            Box::new(d06_performance::PerformanceDemo::new()),
            Box::new(d07_type_system::TypeSystemDemo::new()),
            Box::new(d08_error_handling::ErrorHandlingDemo::new()),
            Box::new(d09_lifetimes::LifetimesDemo::new()),
            Box::new(d10_unsafe::UnsafeDemo::new()),
            Box::new(d11_wasm::WasmDemo::new()),
            Box::new(d12_system_metrics::SystemMetricsDemo::new()),
            Box::new(d13_compile_time::CompileTimeDemo::new()),
            Box::new(d14_cargo_ecosystem::CargoDemo::new()),
            Box::new(d15_no_std::NoStdDemo::new()),
        ];
        Self { demos }
    }

    pub fn get(&self, idx: usize) -> Option<&dyn Demo> {
        self.demos.get(idx).map(|d| d.as_ref())
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut dyn Demo> {
        match self.demos.get_mut(idx) {
            Some(d) => Some(d.as_mut()),
            None => None,
        }
    }

    pub fn len(&self) -> usize {
        self.demos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.demos.is_empty()
    }

    pub fn name(&self, idx: usize) -> Option<&'static str> {
        self.demos.get(idx).map(|d| d.name())
    }

    pub fn description(&self, idx: usize) -> Option<&'static str> {
        self.demos.get(idx).map(|d| d.description())
    }

    pub fn explanation(&self, idx: usize) -> Option<&'static str> {
        self.demos.get(idx).map(|d| d.explanation())
    }

    pub fn tick_current(&mut self, idx: usize, dt: Duration) {
        if let Some(demo) = self.demos.get_mut(idx) {
            demo.tick(dt);
        }
    }

    pub fn render_current(&self, idx: usize, frame: &mut Frame, area: Rect) {
        if let Some(demo) = self.demos.get(idx) {
            demo.render(frame, area);
        }
    }

    pub fn reset_current(&mut self, idx: usize) {
        if let Some(demo) = self.demos.get_mut(idx) {
            demo.reset();
        }
    }

    pub fn toggle_vsmode_current(&mut self, idx: usize) {
        if let Some(demo) = self.demos.get_mut(idx) {
            demo.toggle_vsmode();
        }
    }
}

impl Default for DemoRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_registry_new_len() {
        let reg = DemoRegistry::new();
        assert_eq!(reg.len(), 15);
        assert!(!reg.is_empty());
    }

    #[test]
    fn test_registry_get_in_bounds() {
        let reg = DemoRegistry::new();
        let demo = reg.get(0);
        assert!(demo.is_some());
        assert!(!demo.unwrap().name().is_empty());
    }

    #[test]
    fn test_registry_get_out_of_bounds() {
        let reg = DemoRegistry::new();
        assert!(reg.get(100).is_none());
    }

    #[test]
    fn test_registry_get_mut() {
        let mut reg = DemoRegistry::new();
        let demo = reg.get_mut(0);
        assert!(demo.is_some());
    }

    #[test]
    fn test_registry_get_mut_out_of_bounds() {
        let mut reg = DemoRegistry::new();
        assert!(reg.get_mut(100).is_none());
    }

    #[test]
    fn test_registry_name() {
        let reg = DemoRegistry::new();
        assert!(reg.name(0).is_some());
        assert!(reg.name(100).is_none());
    }

    #[test]
    fn test_registry_description() {
        let reg = DemoRegistry::new();
        assert!(reg.description(0).is_some());
        assert!(reg.description(100).is_none());
    }

    #[test]
    fn test_registry_explanation() {
        let reg = DemoRegistry::new();
        assert!(reg.explanation(0).is_some());
        assert!(reg.explanation(100).is_none());
    }

    #[test]
    fn test_registry_tick_current() {
        let mut reg = DemoRegistry::new();
        reg.tick_current(0, Duration::from_millis(50));
        reg.tick_current(100, Duration::from_millis(50)); // out of bounds, no panic
    }

    #[test]
    fn test_registry_render_current() {
        let reg = DemoRegistry::new();
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| reg.render_current(0, f, f.area()))
            .unwrap();
    }

    #[test]
    fn test_registry_render_current_out_of_bounds() {
        let reg = DemoRegistry::new();
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| reg.render_current(100, f, f.area()))
            .unwrap(); // no panic
    }

    #[test]
    fn test_registry_reset_current() {
        let mut reg = DemoRegistry::new();
        reg.reset_current(0);
        reg.reset_current(100); // out of bounds, no panic
    }

    #[test]
    fn test_registry_default() {
        let reg = DemoRegistry::default();
        assert_eq!(reg.len(), 15);
    }

    #[test]
    fn test_placeholder_demo_trait_methods() {
        let mut reg = DemoRegistry::new();
        // is_paused initially false
        assert!(!reg.get(0).unwrap().is_paused());
        // toggle_pause
        reg.get_mut(0).unwrap().toggle_pause();
        assert!(reg.get(0).unwrap().is_paused());
        reg.get_mut(0).unwrap().toggle_pause();
        assert!(!reg.get(0).unwrap().is_paused());
        // set_speed / speed
        reg.get_mut(0).unwrap().set_speed(7);
        assert_eq!(reg.get(0).unwrap().speed(), 7);
        // reset
        reg.get_mut(0).unwrap().reset();
    }

    #[test]
    fn test_placeholder_demo_tick_when_not_paused() {
        let mut demo = PlaceholderDemo::new("T", "D", "E");
        demo.tick(Duration::from_millis(50));
        assert_eq!(demo.tick_count, 1);
    }

    #[test]
    fn test_placeholder_demo_tick_when_paused() {
        let mut demo = PlaceholderDemo::new("T", "D", "E");
        demo.paused = true;
        demo.tick(Duration::from_millis(50));
        assert_eq!(demo.tick_count, 0);
    }

    #[test]
    fn test_placeholder_demo_speed_clamp() {
        let mut demo = PlaceholderDemo::new("T", "D", "E");
        demo.set_speed(0);
        assert_eq!(demo.speed(), 1);
        demo.set_speed(255);
        assert_eq!(demo.speed(), 10);
    }

    #[test]
    fn test_all_demos_render_without_panic() {
        let reg = DemoRegistry::new();
        for i in 0..15 {
            let backend = TestBackend::new(80, 20);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| reg.render_current(i, f, f.area()))
                .unwrap();
        }
    }
}
